use icu_normalizer::{ComposingNormalizer, DecomposingNormalizer};
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

/// Strip diacritics/points and fold case so an unpointed query matches a
/// pointed lemma.
pub fn normalize_for_search(word: &str) -> String {
	// icu 2.x: const-fn singletons over baked data; constructing per call is free.
	let nfd = DecomposingNormalizer::new_nfd();
	let nfc = ComposingNormalizer::new_nfc();
	let gc = CodePointMapData::<GeneralCategory>::new();

	let decomposed = nfd.normalize(word);
	let stripped: String = decomposed
		.chars()
		.filter(|&c| {
			!matches!(
				gc.get(c),
				GeneralCategory::NonspacingMark
					| GeneralCategory::SpacingMark
					| GeneralCategory::EnclosingMark
			)
		})
		.collect();
	nfc.normalize(&stripped).to_lowercase().chars().map(fold_final).collect()
}

#[cfg(test)]
mod tests {
	use super::normalize_for_search;

	#[test]
	fn strips_hebrew_points() {
		assert_eq!(normalize_for_search("דָּבָר"), "דבר");
	}

	#[test]
	fn strips_greek_diacritics_and_lowercases() {
		assert_eq!(normalize_for_search("Λόγος"), "λογοσ");
	}

	#[test]
	fn folds_final_sigma() {
		assert_eq!(normalize_for_search("ος"), "οσ");
	}

	#[test]
	fn passthrough_ascii() {
		assert_eq!(normalize_for_search("Word"), "word");
	}

	#[test]
	fn folds_hebrew_finals() {
		// שָׁלוֹם ends with final mem ם → מ
		assert_eq!(normalize_for_search("שלום"), "שלומ");
		// all five sofit letters
		assert_eq!(normalize_for_search("ךםןףץ"), "כמנפצ");
	}
}
