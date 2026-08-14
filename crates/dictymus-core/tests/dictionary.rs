use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use dictymus_core::dictionary::DictHandle;
use dictymus_core::testing;

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

/// Per-test temp dir, unique within the process.
fn fixture_dir() -> PathBuf {
	let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
	std::env::temp_dir().join(format!("dictymus-core-test-{}-{n}", std::process::id()))
}

#[test]
fn loads_word_list_and_detects_greek() {
	let dir = fixture_dir();
	let d = DictHandle::open(&testing::write_greek(&dir)).expect("open fixture");
	assert_eq!(d.word_count(), 2);
	assert_eq!(d.language(), "el");
	assert_eq!(d.words().len(), d.word_count());
	drop(d);
	let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn detects_hebrew() {
	let dir = fixture_dir();
	let d = DictHandle::open(&testing::write_hebrew(&dir)).expect("open fixture");
	assert_eq!(d.language(), "he");
	drop(d);
	let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn latin_is_unknown_language() {
	let dir = fixture_dir();
	let d = DictHandle::open(&testing::write_latin(&dir)).expect("open fixture");
	assert_eq!(d.language(), "unknown");
	drop(d);
	let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn lookup_by_index_returns_html() {
	let dir = fixture_dir();
	let d = DictHandle::open(&testing::write_greek(&dir)).expect("open fixture");
	let html = d.article_html(0).expect("entry 0");
	assert!(html.contains("<b>"));
	drop(d);
	let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn path_matches_input() {
	let dir = fixture_dir();
	let ifo = testing::write_greek(&dir);
	let d = DictHandle::open(&ifo).expect("open");
	assert!(d.path().exists());
	drop(d);
	let _ = std::fs::remove_dir_all(&dir);
}
