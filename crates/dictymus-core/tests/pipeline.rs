// crates/dictymus-core/tests/pipeline.rs
use dictymus_core::normalize::SearchKey;
use dictymus_core::transliterate::transliterate;

/// The field text after typing `keys` in a tab of `language`.
fn typed(keys: &str, language: &str) -> String {
	keys.chars()
		.map(|c| transliterate(c, language).map_or_else(|| c.to_string(), str::to_string))
		.collect()
}

fn matches(lemma: &str, keys: &str, language: &str) -> bool {
	SearchKey::new(lemma).starts_with(&SearchKey::new(&typed(keys, language)))
}

const DBR: [&str; 4] = ["דָּבָר", "דִּבֵּר", "דֶּבֶר", "דֹּבֶר"];

#[test]
fn consonants_only_match_every_pointing() {
	assert_eq!(typed("dbr", "he"), "דבר");
	for lemma in DBR {
		assert!(matches(lemma, "dbr", "he"), "{lemma}");
	}
}

#[test]
fn typed_segol_narrows_to_lemmas_with_that_vowel() {
	assert_eq!(typed("dber", "he"), "דב\u{5b6}ר");
	assert!(matches("דֶּבֶר", "dber", "he"));
	assert!(matches("דֹּבֶר", "dber", "he"));
	assert!(!matches("דָּבָר", "dber", "he"));
	assert!(!matches("דִּבֵּר", "dber", "he"));
}

#[test]
fn shift_o_is_holam_and_plain_o_is_qamats() {
	assert!(matches("דֹּבֶר", "dOber", "he"));
	assert!(!matches("דֶּבֶר", "dOber", "he"));
	assert!(matches("דָּבָר", "dobor", "he"));
	assert!(!matches("דֹּבֶר", "dobor", "he"));
}

#[test]
fn vowel_after_first_letter_narrows() {
	assert!(matches("יַיִן", "ya", "he"));
	assert!(!matches("יוֹם", "ya", "he"));
}

#[test]
fn greek_case_folds_and_typed_accents_constrain() {
	assert_eq!(typed("Logos", "grc"), "Λογοσ");
	assert!(matches("λόγος", "Logos", "grc"));
	assert!(matches("λόγος", "lo/gos", "grc"));
	assert!(!matches("λόγος", "lo\\gos", "grc"));
}
