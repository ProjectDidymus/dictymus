// crates/dictymus-core/tests/pipeline.rs
use dictymus_core::{normalize::normalize_for_search, transliterate::transliterate_char};

#[test]
fn typed_query_normalizes_to_match_pointed_lemma() {
	// user types "dbr" on the Hebrew layout -> דבר
	let typed: String = "dbr".chars().map(|c| transliterate_char(c, "he")).collect();
	assert_eq!(typed, "דבר");
	// a pointed lemma normalizes to the same key
	assert_eq!(normalize_for_search("דָּבָר"), normalize_for_search(&typed));
}

#[test]
fn greek_transliterate_and_normalize() {
	let typed: String = "logos".chars().map(|c| transliterate_char(c, "el")).collect();
	assert_eq!(normalize_for_search("λόγος"), normalize_for_search(&typed));
}
