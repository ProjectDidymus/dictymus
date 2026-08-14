//! StarDict fixture generation for tests: tiny dictionaries with a couple
//! of public-domain words per script.

use std::path::{Path, PathBuf};

/// Write a StarDict set (.ifo/.idx/.dict) into `dir`; returns the .ifo path.
pub fn write_stardict(dir: &Path, name: &str, bookname: &str, entries: &[(&str, &str)]) -> PathBuf {
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
	std::fs::create_dir_all(dir).unwrap();
	std::fs::write(dir.join(format!("{name}.idx")), idx).unwrap();
	std::fs::write(dir.join(format!("{name}.dict")), dict).unwrap();
	let ifo_path = dir.join(format!("{name}.ifo"));
	std::fs::write(&ifo_path, ifo).unwrap();
	ifo_path
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
