use dictymus_core::dictionary::{DictHandle, OpenError};
use dictymus_core::testing;

const LICENSEE: &str = "Jane Doe <jane@example.com>";

fn entries() -> Vec<(&'static str, &'static str)> {
	vec![("λόγος", "<b>λόγος</b> word, speech"), ("θεός", "<b>θεός</b> God")]
}

#[test]
fn opens_unsealed_container() {
	let dir = tempfile::tempdir().unwrap();
	let path = testing::write_dicty(dir.path(), "greek", "Greek Container", &entries(), None);
	let dict = DictHandle::open(&path, &testing::test_license_pubkey()).unwrap();
	assert_eq!(dict.title(), "Greek Container");
	assert_eq!(dict.word_count(), 2);
	let logos = dict.words().iter().position(|w| w == "λόγος").unwrap();
	assert!(dict.article_html(logos).unwrap().contains("word, speech"));
}

#[test]
fn opens_sealed_container_with_sibling_license() {
	let dir = tempfile::tempdir().unwrap();
	let path = testing::write_dicty(
		dir.path(),
		"greek",
		"Greek Container",
		&entries(),
		Some(("suite", testing::test_scope_key())),
	);
	testing::write_test_license(
		&dir.path().join("greek.dictykey"),
		LICENSEE,
		&[("suite".into(), testing::test_scope_key())],
	);
	let dict = DictHandle::open(&path, &testing::test_license_pubkey()).unwrap();
	assert_eq!(dict.title(), "Greek Container");
	assert!(dict.article_html(1).is_some());
}

#[test]
fn sealed_container_without_license_names_the_dictionary() {
	let dir = tempfile::tempdir().unwrap();
	// A scope no other test ever grants: tests share the process, and a
	// license installed into the data dir by another test must not
	// accidentally unlock this container.
	let path = testing::write_dicty(
		dir.path(),
		"greek",
		"Greek Container",
		&entries(),
		Some(("unlicensed-scope", [21; 32])),
	);
	let err = match DictHandle::open(&path, &testing::test_license_pubkey()) {
		Ok(_) => panic!("expected an error for a license-less sealed container"),
		Err(e) => e,
	};
	match err {
		OpenError::LicenseMissing { dict_name } => assert_eq!(dict_name, "Greek Container"),
		other => panic!("expected LicenseMissing, got {other:?}"),
	}
}

#[test]
fn license_for_other_scope_does_not_unlock() {
	let dir = tempfile::tempdir().unwrap();
	// Unique scope for the same reason as above: no cross-test unlocks.
	let path = testing::write_dicty(
		dir.path(),
		"greek",
		"Greek Container",
		&entries(),
		Some(("mismatched-scope", [22; 32])),
	);
	testing::write_test_license(
		&dir.path().join("greek.dictykey"),
		LICENSEE,
		&[("other".into(), [9; 32])],
	);
	assert!(matches!(
		DictHandle::open(&path, &testing::test_license_pubkey()),
		Err(OpenError::LicenseMissing { .. })
	));
}

#[test]
fn finds_license_in_data_dir() {
	// DICTYMUS_DATA_DIR is process-global; this is the only test that
	// sets it, and the other tests use scopes this license does not
	// grant, so the leak cannot unlock their containers.
	let data_dir = tempfile::tempdir().unwrap();
	unsafe { std::env::set_var("DICTYMUS_DATA_DIR", data_dir.path()) };
	let licenses = data_dir.path().join("licenses");
	std::fs::create_dir_all(&licenses).unwrap();
	testing::write_test_license(
		&licenses.join("customer.dictykey"),
		LICENSEE,
		&[("suite".into(), testing::test_scope_key())],
	);

	let dict_dir = tempfile::tempdir().unwrap();
	let path = testing::write_dicty(
		dict_dir.path(),
		"greek",
		"Greek Container",
		&entries(),
		Some(("suite", testing::test_scope_key())),
	);
	let dict = DictHandle::open(&path, &testing::test_license_pubkey()).unwrap();
	assert_eq!(dict.word_count(), 2);
}
