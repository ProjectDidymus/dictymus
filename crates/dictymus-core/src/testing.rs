//! StarDict fixture generation for tests: tiny dictionaries with a couple
//! of public-domain words per script.

use dictymus_container::{SigningKey, VerifyingKey};
use std::path::{Path, PathBuf};

/// The StarDict fileset for a tiny dictionary, as (file name, bytes).
pub fn stardict_buffers(
	name: &str,
	bookname: &str,
	entries: &[(&str, &str)],
) -> Vec<(String, Vec<u8>)> {
	let mut entries: Vec<(&str, &str)> = entries.to_vec();
	entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

	let mut idx: Vec<u8> = Vec::new();
	let mut dict: Vec<u8> = Vec::new();
	for (word, article) in &entries {
		idx.extend_from_slice(word.as_bytes());
		idx.push(0);
		idx.extend_from_slice(&(dict.len() as u32).to_be_bytes());
		idx.extend_from_slice(&(article.len() as u32).to_be_bytes());
		dict.extend_from_slice(article.as_bytes());
	}
	let ifo = format!(
		"StarDict's dict ifo file\nversion=3.0.0\nbookname={bookname}\nwordcount={}\nidxfilesize={}\nsametypesequence=h\n",
		entries.len(),
		idx.len(),
	);
	vec![
		(format!("{name}.ifo"), ifo.into_bytes()),
		(format!("{name}.idx"), idx),
		(format!("{name}.dict"), dict),
	]
}

/// Write a StarDict set (.ifo/.idx/.dict) into `dir`; returns the .ifo path.
pub fn write_stardict(dir: &Path, name: &str, bookname: &str, entries: &[(&str, &str)]) -> PathBuf {
	std::fs::create_dir_all(dir).unwrap();
	for (file_name, data) in stardict_buffers(name, bookname, entries) {
		std::fs::write(dir.join(file_name), data).unwrap();
	}
	dir.join(format!("{name}.ifo"))
}

/// Write a `.dicty` container into `dir`; returns its path. `seal` is
/// `Some((scope_id, scope_key))` for a sealed container, `None` for an
/// unsealed one.
pub fn write_dicty(
	dir: &Path,
	name: &str,
	bookname: &str,
	entries: &[(&str, &str)],
	seal: Option<(&str, [u8; 32])>,
) -> PathBuf {
	let files = stardict_buffers(name, bookname, entries);
	let bytes = match seal {
		Some((scope_id, scope_key)) => dictymus_container::container::seal(
			name,
			bookname,
			&files,
			&[(scope_id.to_string(), scope_key)],
		),
		None => dictymus_container::container::pack(name, bookname, &files),
	};
	std::fs::create_dir_all(dir).unwrap();
	let path = dir.join(format!("{name}.dicty"));
	std::fs::write(&path, bytes).unwrap();
	path
}

/// Fixed scope key for sealed test containers.
pub fn test_scope_key() -> [u8; 32] {
	[42; 32]
}

/// Fixed publisher signing key for test licenses.
pub fn test_signing_key() -> SigningKey {
	SigningKey::from_bytes(&[7; 32])
}

/// The verifying half of [`test_signing_key`].
pub fn test_license_pubkey() -> VerifyingKey {
	test_signing_key().verifying_key()
}

/// Write a `.dictykey` license signed with the test publisher key.
pub fn write_test_license(path: &Path, licensee: &str, grants: &[(String, [u8; 32])]) {
	let bytes =
		dictymus_container::license::issue(licensee, "2026-08-15", grants, &test_signing_key());
	std::fs::write(path, bytes).unwrap();
}

pub fn write_greek(dir: &Path) -> PathBuf {
	write_stardict(
		dir,
		"greek",
		"Greek Fixture",
		&[("λόγος", "<b>λόγος</b> word, speech"), ("θεός", "<b>θεός</b> God")],
	)
}

pub fn write_hebrew(dir: &Path) -> PathBuf {
	write_stardict(
		dir,
		"hebrew",
		"Hebrew Fixture",
		&[("דָּבָר", "<b>דָּבָר</b> word, matter"), ("אֱלֹהִים", "<b>אֱלֹהִים</b> God")],
	)
}

pub fn write_latin(dir: &Path) -> PathBuf {
	write_stardict(
		dir,
		"latin",
		"Latin Fixture",
		&[("verbum", "<b>verbum</b> word"), ("lex", "<b>lex</b> law")],
	)
}
