use std::sync::OnceLock;

use icu_properties::{CodePointMapData, props::Script};
use louis::{Direction, Translator};

/// How `normalize_braille` treats one ASCII braille cell.
enum Fold {
	Keep,
	Drop,
	Map(char),
}

struct BrailleLanguage {
	language: &'static str,
	script: Script,
	entry_table: &'static str,
	tables: &'static [(&'static str, &'static str)],
	fold_cell: fn(char) -> Fold,
	translator: OnceLock<Option<Translator>>,
}

static HEBREW_TABLES: &[(&str, &str)] = &[
	("hbo-ihbc-rules.uti", include_str!("../assets/braille-tables/he/hbo-ihbc-rules.uti")),
	("hbo-common-rules.uti", include_str!("../assets/braille-tables/he/hbo-common-rules.uti")),
	(
		"he-common-consonants.uti",
		include_str!("../assets/braille-tables/he/he-common-consonants.uti"),
	),
	(
		"he-common-vowels-ihbc.uti",
		include_str!("../assets/braille-tables/he/he-common-vowels-ihbc.uti"),
	),
	("spaces.uti", include_str!("../assets/braille-tables/he/spaces.uti")),
];

static LANGUAGES: [BrailleLanguage; 1] = [BrailleLanguage {
	language: "he",
	script: Script::Hebrew,
	entry_table: "hbo-ihbc-rules.uti",
	tables: HEBREW_TABLES,
	fold_cell: fold_hebrew_cell,
	translator: OnceLock::new(),
}];

/// Dots 1–6 of each Unicode braille cell, indexed by its low six bits, as the
/// lowercased Braille ASCII (BRF) character set.
static BRF_LOWER: &[u8; 64] = b" a1b'k2l@cif/msp\"e3h9o6r^djg>ntq,*5<-u8v.%[$+x!&;:4\\0z7(_?w]#y)=";

/// Whether ASCII braille conversion is available for the given language code.
pub fn supported(language: &str) -> bool {
	registration(language).is_some()
}

/// Convert the script runs of the given language inside `text` to ASCII
/// braille. Other runs, and the whole text for unregistered languages, pass
/// through unchanged.
pub fn to_ascii_braille(text: &str, language: &str) -> String {
	let Some(lang) = registration(language) else {
		return text.to_string();
	};
	let Some(translator) = lang.translator() else {
		return text.to_string();
	};
	let script = CodePointMapData::<Script>::new();
	let mut out = String::with_capacity(text.len());
	let mut run = String::new();
	for ch in text.chars() {
		if script.get(ch) == lang.script {
			run.push(ch);
		} else {
			flush_run(translator, &mut run, &mut out);
			// Direction control marks (converter output carries a
			// left-to-right mark after each Hebrew span) mean nothing in the
			// left-to-right ASCII output and only disturb a braille display.
			if !is_bidi_control(ch) {
				out.push(ch);
			}
		}
	}
	flush_run(translator, &mut run, &mut out);
	out
}

fn is_bidi_control(ch: char) -> bool {
	matches!(ch, '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// Convert the text nodes of trusted article HTML to ASCII braille, leaving
/// tags, attributes and entities untouched. Replaced runs are HTML-escaped:
/// ASCII braille uses `<`, `>` and `&` as ordinary cells.
pub fn braille_html(html: &str, language: &str) -> String {
	if !supported(language) {
		return html.to_string();
	}
	let mut out = String::with_capacity(html.len());
	let mut text = String::new();
	let mut chars = html.chars().peekable();
	while let Some(ch) = chars.next() {
		match ch {
			'<' => {
				flush_text_node(&mut text, language, &mut out);
				out.push(ch);
				for tag_ch in chars.by_ref() {
					out.push(tag_ch);
					if tag_ch == '>' {
						break;
					}
				}
			}
			'&' => {
				flush_text_node(&mut text, language, &mut out);
				out.push(ch);
				while let Some(&entity_ch) = chars.peek() {
					if entity_ch == '<' {
						break;
					}
					out.push(entity_ch);
					chars.next();
					if entity_ch == ';' {
						break;
					}
				}
			}
			_ => text.push(ch),
		}
	}
	flush_text_node(&mut text, language, &mut out);
	out
}

/// Braille-space analogue of `normalize_for_search`: fold an ASCII braille
/// string so that pointed and unpointed spellings of the same word compare
/// equal. Applied to both the query and the brailled lemma.
pub fn normalize_braille(text: &str, language: &str) -> String {
	let Some(lang) = registration(language) else {
		return text.to_string();
	};
	text.chars()
		.filter_map(|cell| match (lang.fold_cell)(cell) {
			Fold::Keep => Some(cell),
			Fold::Drop => None,
			Fold::Map(to) => Some(to),
		})
		.collect()
}

fn registration(language: &str) -> Option<&'static BrailleLanguage> {
	LANGUAGES.iter().find(|lang| lang.language == language)
}

impl BrailleLanguage {
	fn translator(&self) -> Option<&Translator> {
		self.translator
			.get_or_init(|| {
				let source = flatten_table(self.tables, self.entry_table)?;
				match Translator::from_table_source(&source, Direction::Forward) {
					Ok(translator) => Some(translator),
					Err(error) => {
						tracing::warn!(language = self.language, %error, "braille table failed to load");
						None
					}
				}
			})
			.as_ref()
	}
}

/// Splice the embedded table files into one include-free source, replacing
/// each `include` line with the named file's flattened content in place.
fn flatten_table(tables: &[(&str, &str)], name: &str) -> Option<String> {
	let (_, source) = tables.iter().find(|(table, _)| *table == name)?;
	let mut out = String::new();
	for line in source.lines() {
		if let Some(included) = line.strip_prefix("include ") {
			out.push_str(&flatten_table(tables, included.trim())?);
		} else {
			out.push_str(line);
			out.push('\n');
		}
	}
	Some(out)
}

fn flush_run(translator: &Translator, run: &mut String, out: &mut String) {
	if run.is_empty() {
		return;
	}
	match translator.translate(run) {
		Ok(cells) => out.push_str(&ascii_from_unicode_braille(&cells)),
		Err(error) => {
			tracing::warn!(%error, "braille translation failed");
			out.push_str(run);
		}
	}
	run.clear();
}

fn ascii_from_unicode_braille(cells: &str) -> String {
	cells
		.chars()
		.map(|cell| match u32::from(cell).checked_sub(0x2800) {
			// Dots 7 and 8 have no Braille ASCII form; mask them off.
			Some(dots) if dots <= 0xFF => char::from(BRF_LOWER[(dots & 0x3F) as usize]),
			_ => cell,
		})
		.collect()
}

fn flush_text_node(text: &mut String, language: &str, out: &mut String) {
	if text.is_empty() {
		return;
	}
	let braille = to_ascii_braille(text, language);
	for ch in braille.chars() {
		match ch {
			'&' => out.push_str("&amp;"),
			'<' => out.push_str("&lt;"),
			'>' => out.push_str("&gt;"),
			_ => out.push(ch),
		}
	}
	text.clear();
}

fn fold_hebrew_cell(cell: char) -> Fold {
	match cell {
		// Vowel points, dagesh, mappiq and the IHBC cantillation cells,
		// all stripped as marks by normalize_for_search.
		'\'' | '5' | '3' | '>' | 'i' | '/' | 'e' | 'c' | '<' | 'o' | 'u' | '"' | '^' | '@'
		| '1' | '2' => Fold::Drop,
		// Dagesh letter forms fold to their plain letters.
		'b' => Fold::Map('v'),
		'k' => Fold::Map('*'),
		'p' => Fold::Map('f'),
		'\\' => Fold::Map('?'),
		// Vowel-consonant contractions keep their consonant.
		'[' | '+' => Fold::Map('w'),
		'9' | '#' => Fold::Map('j'),
		// Sin folds onto shin, like the stripped shin/sin dots.
		':' => Fold::Map('%'),
		_ => Fold::Keep,
	}
}

#[cfg(test)]
mod tests {
	use super::{braille_html, normalize_braille, supported, to_ascii_braille};

	#[test]
	fn hebrew_is_supported() {
		assert!(supported("he"));
		assert!(!supported("grc"));
		assert!(!supported("unknown"));
	}

	#[test]
	fn single_letter_translates() {
		assert_eq!(to_ascii_braille("א", "he"), "a");
	}

	#[test]
	fn pointed_lemma_translates_to_ihbc_ascii() {
		// dalet+dagesh, qamats, bet, qamats, resh
		assert_eq!(to_ascii_braille("דָּבָר", "he"), "\"d<v<r");
	}

	#[test]
	fn contractions_apply() {
		// alef+hataf segol, lamed+holam, he+hiriq-yod contraction, final mem
		assert_eq!(to_ascii_braille("אֱלֹהִים", "he"), "a5loh9m");
	}

	#[test]
	fn punctuation_translates() {
		assert_eq!(to_ascii_braille("׃", "he"), "4");
		assert_eq!(to_ascii_braille("־", "he"), "-");
	}

	#[test]
	fn kept_cantillation_moves_to_word_end() {
		// IHBC keeps only etnahta and zaqef qatan, transcribed at the end of
		// the word; the sheva-carried sin dot merges into the sin cell.
		assert_eq!(to_ascii_braille("יִשְׂרָאֵ֑ל", "he"), "ji:'r<a/l2");
	}

	#[test]
	fn typed_and_canonical_mark_orders_agree() {
		// Consonant-dagesh-vowel (typing order) and consonant-vowel-dagesh
		// (canonical order) both occur in real data and must braille alike.
		assert_eq!(to_ascii_braille("ד\u{5bc}\u{5b8}", "he"), "\"d<");
		assert_eq!(to_ascii_braille("ד\u{5b8}\u{5bc}", "he"), "\"d<");
		assert_eq!(to_ascii_braille("ב\u{5bc}\u{5b8}", "he"), "b<");
		assert_eq!(to_ascii_braille("ב\u{5b8}\u{5bc}", "he"), "b<");
	}

	#[test]
	fn non_hebrew_runs_pass_through() {
		assert_eq!(to_ascii_braille("zie דָּבָר s.v.", "he"), "zie \"d<v<r s.v.");
	}

	#[test]
	fn unregistered_language_passes_through() {
		assert_eq!(to_ascii_braille("δαβαρ", "grc"), "δαβαρ");
		assert_eq!(to_ascii_braille("דָּבָר", "unknown"), "דָּבָר");
	}

	#[test]
	fn html_text_nodes_are_transformed_and_escaped() {
		assert_eq!(braille_html("<p>דָּבָר</p>", "he"), "<p>\"d&lt;v&lt;r</p>");
	}

	#[test]
	fn html_attributes_are_preserved() {
		assert_eq!(
			braille_html("<a data-ref-word=\"דָּבָר\">דָּבָר</a>", "he"),
			"<a data-ref-word=\"דָּבָר\">\"d&lt;v&lt;r</a>"
		);
	}

	#[test]
	fn html_entities_are_preserved() {
		assert_eq!(braille_html("<p>&amp;דָּבָר&#x20;</p>", "he"), "<p>&amp;\"d&lt;v&lt;r&#x20;</p>");
	}

	#[test]
	fn direction_marks_are_dropped() {
		// Converter output places a raw left-to-right mark after each Hebrew
		// span. Direction marks mean nothing in the left-to-right ASCII
		// braille output and only disturb a braille display.
		assert_eq!(to_ascii_braille("\u{200E}\u{5d0}\u{200F}", "he"), "a");
		assert_eq!(
			braille_html("<p><span lang=\"he\" dir=\"rtl\">\u{5d0}</span>\u{200E}; ok</p>", "he"),
			"<p><span lang=\"he\" dir=\"rtl\">a</span>; ok</p>"
		);
	}

	#[test]
	fn html_unregistered_language_passes_through() {
		let html = "<p>δαβαρ</p>";
		assert_eq!(braille_html(html, "grc"), html);
	}

	#[test]
	fn normalize_drops_points_and_folds_dagesh_forms() {
		assert_eq!(normalize_braille("\"d<v<r", "he"), "dvr");
		assert_eq!(normalize_braille("b", "he"), "v");
		assert_eq!(normalize_braille("k", "he"), "*");
		assert_eq!(normalize_braille("p", "he"), "f");
		assert_eq!(normalize_braille("\\", "he"), "?");
		assert_eq!(normalize_braille(":", "he"), "%");
	}

	#[test]
	fn normalize_maps_contractions_to_consonant_residue() {
		assert_eq!(normalize_braille("a5loh9m", "he"), "alhjm");
		assert_eq!(normalize_braille("[", "he"), "w");
		assert_eq!(normalize_braille("+", "he"), "w");
		assert_eq!(normalize_braille("#", "he"), "j");
	}

	#[test]
	fn normalize_matches_hebrew_space_normalization() {
		// The braille of the pointed lemma folds to the braille of the
		// unpointed lemma, mirroring normalize_for_search.
		let pointed = normalize_braille(&to_ascii_braille("דָּבָר", "he"), "he");
		let unpointed = to_ascii_braille("דבר", "he");
		assert_eq!(pointed, unpointed);
		let pointed = normalize_braille(&to_ascii_braille("אֱלֹהִים", "he"), "he");
		let unpointed = to_ascii_braille("אלהים", "he");
		assert_eq!(pointed, unpointed);
	}

	#[test]
	fn normalize_unregistered_language_passes_through() {
		assert_eq!(normalize_braille("\"d<v<r", "grc"), "\"d<v<r");
	}
}
