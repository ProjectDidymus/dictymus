use crate::language;
use crate::normalize::normalize_for_search;
use opendict::Dictionary;
use opendict::stardict::StarDictDictionary;
use std::path::{Path, PathBuf};

pub struct DictHandle {
	dict: StarDictDictionary,
	words: Vec<String>,
	normalized_words: Vec<String>,
	language: &'static str,
	title: String,
	path: PathBuf,
}

impl DictHandle {
	pub fn open(path: &Path) -> Result<Self, opendict::Error> {
		let dir =
			path.parent().ok_or_else(|| opendict::Error::InvalidFormat("no parent dir".into()))?;
		let name = path
			.file_stem()
			.and_then(|s| s.to_str())
			.ok_or_else(|| opendict::Error::InvalidFormat("invalid stem".into()))?;
		let dict = StarDictDictionary::open(dir, name)?;
		let words: Vec<String> = dict.word_list().iter().map(|s| s.to_string()).collect();
		let normalized_words = words.iter().map(|w| normalize_for_search(w)).collect();
		let language = language::detect(&words);
		let title = dict.info().name.clone();
		Ok(Self { dict, words, normalized_words, language, title, path: path.to_path_buf() })
	}

	pub fn title(&self) -> &str {
		&self.title
	}

	pub fn language(&self) -> &'static str {
		self.language
	}

	pub fn word_count(&self) -> usize {
		self.words.len()
	}

	pub fn words(&self) -> &[String] {
		&self.words
	}

	pub fn normalized_words(&self) -> &[String] {
		&self.normalized_words
	}

	pub fn path(&self) -> &Path {
		&self.path
	}

	/// HTML article for the lemma at `index` (concatenates multiple entries).
	pub fn article_html(&self, index: usize) -> Option<String> {
		let entries = self.dict.lookup_by_index(index).ok().flatten()?;
		let mut out = String::new();
		for e in &entries {
			out.push_str(&String::from_utf8_lossy(&e.data));
		}
		Some(out)
	}
}
