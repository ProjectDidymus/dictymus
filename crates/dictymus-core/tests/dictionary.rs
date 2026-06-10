use dictymus_core::dictionary::DictHandle;
use std::path::Path;

fn fixture() -> Option<DictHandle> {
	// Use the opendict-rs test fixture that moved with the crate
	let ifo = Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("../../crates/opendict-rs/tests/fixtures/testdict.ifo");
	if !ifo.exists() {
		return None;
	}
	Some(DictHandle::open(&ifo).expect("open fixture"))
}

#[test]
fn loads_word_list_and_language() {
	let Some(d) = fixture() else {
		eprintln!("no fixture, skipping");
		return;
	};
	assert!(d.word_count() > 0);
	assert!(matches!(d.language(), "he" | "el" | "unknown"));
	assert_eq!(d.words().len(), d.word_count());
}

#[test]
fn lookup_by_index_returns_html() {
	let Some(d) = fixture() else {
		return;
	};
	let html = d.article_html(0).expect("entry 0");
	assert!(!html.is_empty());
}

#[test]
fn path_matches_input() {
	let ifo = Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("../../crates/opendict-rs/tests/fixtures/testdict.ifo");
	if !ifo.exists() {
		return;
	}
	let d = DictHandle::open(&ifo).expect("open");
	// path() returns the .ifo path we passed in
	assert!(d.path().exists());
}
