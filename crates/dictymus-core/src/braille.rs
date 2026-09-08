use std::sync::OnceLock;

use icu_properties::{CodePointMapData, props::Script};
use louis::{Direction, Translator};

use crate::normalize::SearchKey;

struct BrailleLanguage {
	language: &'static str,
	script: Script,
	entry_table: &'static str,
	tables: &'static [(&'static str, &'static str)],
	translator: OnceLock<Option<Translator>>,
	back_translator: OnceLock<Option<Translator>>,
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
	translator: OnceLock::new(),
	back_translator: OnceLock::new(),
}];

/// Dots 1–6 of each Unicode braille cell, indexed by its low six bits, as the
/// lowercased Braille ASCII (BRF) character set.
static BRF_LOWER: &[u8; 64] = b" a1b'k2l@cif/msp\"e3h9o6r^djg>ntq,*5<-u8v.%[$+x!&;:4\\0z7(_?w]#y)=";

/// Whether ASCII braille conversion is available for the given language code.
pub fn supported(language: &str) -> bool {
	registration(language).is_some_and(|lang| lang.translator().is_some())
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

/// The search key of an ASCII braille query: the cells are back-translated
/// to script text and keyed like a typed query. A trailing dagesh cell (the
/// IHBC writes it before its letter) is left out, and the shin dot is not a
/// constraint; the sin dot is. Unregistered languages and untranslatable
/// input key the text as typed.
pub fn search_key(ascii: &str, language: &str) -> SearchKey {
	let Some(translator) = registration(language).and_then(|lang| lang.back_translator()) else {
		return SearchKey::new(ascii);
	};
	let cells = ascii.strip_suffix('"').unwrap_or(ascii);
	// Hiriq-yod and tsere-yod cells become vowel cell + yod cell; an alef
	// cell is appended for the translation and stripped off again.
	let cells = cells.replace('9', "ij").replace('#', "/j") + "a";
	match translator.translate(&unicode_braille_from_ascii(&cells)) {
		Ok(text) => {
			let text = text.strip_suffix('א').unwrap_or(&text);
			SearchKey::new(&text.replace('\u{05C1}', ""))
		}
		Err(error) => {
			tracing::warn!(%error, "braille back-translation failed");
			SearchKey::new(ascii)
		}
	}
}

fn registration(language: &str) -> Option<&'static BrailleLanguage> {
	LANGUAGES.iter().find(|lang| lang.language == language)
}

impl BrailleLanguage {
	fn translator(&self) -> Option<&Translator> {
		self.translator.get_or_init(|| self.load(Direction::Forward)).as_ref()
	}

	fn back_translator(&self) -> Option<&Translator> {
		self.back_translator.get_or_init(|| self.load(Direction::Backward)).as_ref()
	}

	fn load(&self, direction: Direction) -> Option<Translator> {
		let source = flatten_table(self.tables, self.entry_table)?;
		match Translator::from_table_source(&source, direction) {
			Ok(translator) => Some(translator),
			Err(error) => {
				tracing::warn!(language = self.language, %error, "braille table failed to load");
				None
			}
		}
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

fn unicode_braille_from_ascii(cells: &str) -> String {
	cells
		.chars()
		.map(|cell| match BRF_LOWER.iter().position(|&brf| char::from(brf) == cell) {
			Some(dots) => char::from_u32(0x2800 + dots as u32).expect("braille block"),
			None => cell,
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

#[cfg(test)]
mod search_key_tests {
	use super::search_key;
	use crate::normalize::SearchKey;

	fn matches(lemma: &str, ascii: &str) -> bool {
		SearchKey::new(lemma).starts_with(&search_key(ascii, "he"))
	}

	#[test]
	fn pointed_braille_equals_the_pointed_key() {
		assert_eq!(search_key("\"d<v<r", "he"), SearchKey::new("דָּבָר"));
	}

	#[test]
	fn consonant_cells_match_any_pointing() {
		for lemma in ["דָּבָר", "דִּבֵּר", "דֶּבֶר", "דֹּבֶר", "דבר"] {
			assert!(matches(lemma, "dvr"), "{lemma}");
		}
	}

	#[test]
	fn vowel_cell_constrains() {
		assert!(matches("דֶּבֶר", "dev"));
		assert!(matches("דֹּבֶר", "dver"));
		assert!(!matches("דָּבָר", "dev"));
	}

	#[test]
	fn dagesh_letter_form_constrains_plain_form_does_not() {
		assert!(matches("בּ", "b"));
		assert!(!matches("ב", "b"));
		assert!(matches("בּ", "v"));
		assert!(matches("ב", "v"));
	}

	#[test]
	fn shin_cell_matches_shin_sin_and_dotless_but_sin_cell_only_sin() {
		assert!(matches("שָׁלוֹם", "%"));
		assert!(matches("שָׂרָה", "%"));
		assert!(matches("ש", "%"));
		assert!(matches("שָׂרָה", ":"));
		assert!(!matches("שָׁלוֹם", ":"));
	}

	#[test]
	fn trailing_dagesh_cell_waits_for_its_letter() {
		assert_eq!(search_key("d\"", "he"), search_key("d", "he"));
	}

	#[test]
	fn contractions_expand() {
		assert_eq!(search_key("a5loh9m", "he"), SearchKey::new("אֱלֹהִים"));
		assert_eq!(search_key("h9", "he"), SearchKey::new("הִי"));
		assert_eq!(search_key("l#", "he"), SearchKey::new("לֵי"));
		assert_eq!(search_key("[", "he"), SearchKey::new("וֹ"));
		assert_eq!(search_key("+", "he"), SearchKey::new("וּ"));
		assert_eq!(search_key("^h", "he"), SearchKey::new("הּ"));
	}

	#[test]
	fn unregistered_language_passes_through() {
		assert_eq!(search_key("dvr", "grc"), SearchKey::new("dvr"));
	}

	#[test]
	fn letter_before_a_final_form_survives() {
		assert_eq!(search_key("jm", "he"), SearchKey::new("ים"));
		assert_eq!(search_key("ljm", "he"), SearchKey::new("לים"));
		assert_eq!(search_key("m", "he"), SearchKey::new("מ"));
	}
}

#[cfg(test)]
mod tests {
	use super::{braille_html, supported, to_ascii_braille};

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
}
