use icu_normalizer::DecomposingNormalizer;
use icu_properties::{CodePointMapData, props::GeneralCategory};

// Final-form folding: no Unicode property covers this; it's script-specific
// orthography (Hebrew sofit letters, Greek final sigma).
fn fold_final(c: char) -> char {
	match c {
		'ς' => 'σ', // Greek final sigma
		'ך' => 'כ', // Hebrew final kaf
		'ם' => 'מ', // Hebrew final mem
		'ן' => 'נ', // Hebrew final nun
		'ף' => 'פ', // Hebrew final pe
		'ץ' => 'צ', // Hebrew final tsadi
		_ => c,
	}
}

/// One base character with the marks attached to it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Cluster {
	base: char,
	marks: Vec<char>,
}

/// A lemma or query as matched by the search: a sequence of clusters, each a
/// lowercased base (finals folded) with its significant marks. A query
/// matches a lemma when its clusters are a prefix of the lemma's, base for
/// base, and each query cluster's marks are a subset of the lemma cluster's.
/// Hebrew accents, meteg, rafe and the upper and lower dots are dropped on
/// both sides, as are marks before the first base.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchKey(Vec<Cluster>);

impl SearchKey {
	pub fn new(text: &str) -> Self {
		// icu 2.x: const-fn singletons over baked data; constructing per call is free.
		let nfd = DecomposingNormalizer::new_nfd();
		let gc = CodePointMapData::<GeneralCategory>::new();
		let mut clusters: Vec<Cluster> = Vec::new();
		for c in nfd.normalize(text).to_lowercase().chars() {
			if is_mark(gc.get(c)) {
				if let Some(mark) = fold_mark(c)
					&& let Some(cluster) = clusters.last_mut()
					&& !cluster.marks.contains(&mark)
				{
					cluster.marks.push(mark);
				}
			} else {
				clusters.push(Cluster { base: fold_final(c), marks: Vec::new() });
			}
		}
		for cluster in &mut clusters {
			cluster.marks.sort_unstable();
		}
		Self(clusters)
	}

	pub fn starts_with(&self, query: &Self) -> bool {
		query.0.len() <= self.0.len()
			&& query.0.iter().zip(&self.0).all(|(q, lemma)| {
				q.base == lemma.base && q.marks.iter().all(|m| lemma.marks.contains(m))
			})
	}

	/// The same bases with every mark removed.
	pub fn unpointed(&self) -> Self {
		Self(self.0.iter().map(|c| Cluster { base: c.base, marks: Vec::new() }).collect())
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

fn is_mark(category: GeneralCategory) -> bool {
	matches!(
		category,
		GeneralCategory::NonspacingMark
			| GeneralCategory::SpacingMark
			| GeneralCategory::EnclosingMark
	)
}

/// The mark as compared, or None for a mark the search ignores.
fn fold_mark(c: char) -> Option<char> {
	match c {
		'\u{0591}'..='\u{05AF}' | '\u{05BD}' | '\u{05BF}' | '\u{05C4}' | '\u{05C5}' => None,
		'\u{05C7}' => Some('\u{05B8}'), // qamats qatan → qamats
		'\u{05BA}' => Some('\u{05B9}'), // holam haser for vav → holam
		_ => Some(c),
	}
}

#[cfg(test)]
mod search_key_tests {
	use super::SearchKey;

	fn matches(lemma: &str, query: &str) -> bool {
		SearchKey::new(lemma).starts_with(&SearchKey::new(query))
	}

	#[test]
	fn unpointed_query_matches_any_pointing() {
		for lemma in ["דָּבָר", "דִּבֵּר", "דֶּבֶר", "דֹּבֶר", "דבר"] {
			assert!(matches(lemma, "דבר"), "{lemma}");
		}
	}

	#[test]
	fn typed_vowel_constrains() {
		assert!(matches("דֶּבֶר", "דבֶר"));
		assert!(matches("דֹּבֶר", "דבֶר"));
		assert!(!matches("דָּבָר", "דבֶר"));
		assert!(!matches("דִּבֵּר", "דבֶר"));
		assert!(matches("דֹּבֶר", "דֹבֶר"));
		assert!(!matches("דֶּבֶר", "דֹבֶר"));
	}

	#[test]
	fn vowel_on_first_letter_narrows() {
		assert!(matches("יַיִן", "יַ"));
		assert!(!matches("יוֹם", "יַ"));
	}

	#[test]
	fn dagesh_constrains() {
		assert!(matches("דָּבָר", "דּ"));
		assert!(!matches("דבר", "דּ"));
	}

	#[test]
	fn shin_and_sin_dots_constrain() {
		assert!(matches("שָׁלוֹם", "שׁ"));
		assert!(!matches("שָׂרָה", "שׁ"));
		assert!(matches("שָׂרָה", "שׂ"));
		assert!(matches("שָׁלוֹם", "ש"));
		assert!(matches("שָׂרָה", "ש"));
	}

	#[test]
	fn ignorable_marks_on_both_sides() {
		// etnahta in the lemma; meteg and rafe in the query
		assert!(matches("דָּבָ֑ר", "דבר"));
		assert!(matches("דָּבָר", "דֽבֿר"));
		assert!(matches("דבר", "ד֯בר"));
	}

	#[test]
	fn qamats_qatan_and_holam_haser_fold() {
		assert!(matches("כׇל", "כָל"));
		assert!(matches("כָל", "כׇל"));
		assert!(matches("עֲוֺן", "עֲוֹן"));
	}

	#[test]
	fn leading_orphan_mark_is_dropped() {
		assert_eq!(SearchKey::new("ַד"), SearchKey::new("ד"));
	}

	#[test]
	fn finals_fold() {
		assert!(matches("שלום", "שלומ"));
		assert!(matches("ךםןףץ", "כמנפצ"));
	}

	#[test]
	fn greek_accent_constrains_and_case_folds() {
		assert!(matches("λόγος", "λογος"));
		assert!(matches("λόγος", "λόγος"));
		assert!(!matches("λόγος", "λὸγος"));
		assert!(matches("Λόγος", "λογοσ"));
	}

	#[test]
	fn latin_lowercases() {
		assert!(matches("Word", "word"));
	}

	#[test]
	fn unpointed_clears_marks() {
		assert_eq!(SearchKey::new("דָּבָר").unpointed(), SearchKey::new("דבר"));
	}

	#[test]
	fn empty_query_matches_everything() {
		assert!(matches("דָּבָר", ""));
		assert!(SearchKey::new("").is_empty());
		assert!(!SearchKey::new("ד").is_empty());
	}

	#[test]
	fn longer_query_does_not_match() {
		assert!(!matches("דב", "דבר"));
	}
}
