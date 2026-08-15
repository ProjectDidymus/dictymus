use crate::language;
use crate::normalize::normalize_for_search;
use dictymus_container::VerifyingKey;
use opendict::Dictionary;
use opendict::mdict::MdictDictionary;
use opendict::stardict::StarDictDictionary;
use std::path::{Path, PathBuf};

/// Why a dictionary failed to open. The UI matches on this to phrase
/// its error dialogs; `LicenseMissing` carries the display name read
/// from the container's public header.
#[derive(Debug)]
pub enum OpenError {
	Dict(opendict::Error),
	Container(dictymus_container::Error),
	/// A sealed container with no installed license that unlocks it.
	LicenseMissing {
		dict_name: String,
	},
}

impl std::fmt::Display for OpenError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			OpenError::Dict(e) => e.fmt(f),
			OpenError::Container(e) => e.fmt(f),
			OpenError::LicenseMissing { dict_name } => {
				write!(f, "no license found for {dict_name}")
			}
		}
	}
}

impl std::error::Error for OpenError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			OpenError::Dict(e) => Some(e),
			OpenError::Container(e) => Some(e),
			OpenError::LicenseMissing { .. } => None,
		}
	}
}

impl From<opendict::Error> for OpenError {
	fn from(e: opendict::Error) -> Self {
		OpenError::Dict(e)
	}
}

impl From<dictymus_container::Error> for OpenError {
	fn from(e: dictymus_container::Error) -> Self {
		OpenError::Container(e)
	}
}

/// Open a `.dicty` container, decrypting sealed payloads with the
/// first installed license that unlocks them.
fn open_container(
	path: &Path,
	license_pubkey: &VerifyingKey,
) -> Result<StarDictDictionary, OpenError> {
	use dictymus_container::container::Container;
	use dictymus_container::license::License;

	let bytes = std::fs::read(path).map_err(dictymus_container::Error::Io)?;
	let container = Container::parse(&bytes)?;
	let files = if container.is_sealed() {
		let mut unlocked = None;
		for candidate in license_candidates(path) {
			let Ok(data) = std::fs::read(&candidate) else { continue };
			let Ok(license) = License::verify(&data, license_pubkey) else { continue };
			match container.open_sealed(&license) {
				Ok(files) => {
					tracing::info!(
						license = %candidate.display(),
						licensee = license.licensee(),
						"sealed dictionary unlocked"
					);
					unlocked = Some(files);
					break;
				}
				Err(dictymus_container::Error::NoMatchingGrant) => continue,
				Err(e) => return Err(e.into()),
			}
		}
		unlocked
			.ok_or_else(|| OpenError::LicenseMissing { dict_name: container.name().to_string() })?
	} else {
		container.open_unsealed()?
	};
	Ok(StarDictDictionary::open_from_memory(files)?)
}

/// Places a license for `container_path` may live: a sibling
/// `.dictykey`, then every `.dictykey` in the app-data licenses dir.
fn license_candidates(container_path: &Path) -> Vec<PathBuf> {
	let mut out = vec![container_path.with_extension("dictykey")];
	if let Some(dir) = crate::config::AppConfig::licenses_dir()
		&& let Ok(entries) = std::fs::read_dir(dir)
	{
		for entry in entries.flatten() {
			let p = entry.path();
			if p.extension().and_then(|e| e.to_str()) == Some("dictykey") {
				out.push(p);
			}
		}
	}
	out
}

pub struct DictHandle {
	dict: Box<dyn Dictionary + Send + Sync>,
	words: Vec<String>,
	normalized_words: Vec<String>,
	language: &'static str,
	title: String,
	path: PathBuf,
}

impl DictHandle {
	#[tracing::instrument(skip_all, fields(path = %path.display()))]
	pub fn open(path: &Path, license_pubkey: &VerifyingKey) -> Result<Self, OpenError> {
		let dir =
			path.parent().ok_or_else(|| opendict::Error::InvalidFormat("no parent dir".into()))?;
		// Dispatch by extension: `.ifo` targets a named StarDict (so a directory
		// holding several dictionaries stays unambiguous), `.mdx` an MDict,
		// `.dicty` a Dictymus container. Anything else falls back to opendict's
		// StarDict→MDict autodetect.
		let dict: Box<dyn Dictionary + Send + Sync> =
			match path.extension().and_then(|e| e.to_str()) {
				Some("ifo") => {
					let name = path
						.file_stem()
						.and_then(|s| s.to_str())
						.ok_or_else(|| opendict::Error::InvalidFormat("invalid stem".into()))?;
					Box::new(StarDictDictionary::open(dir, name)?)
				}
				Some("mdx") => Box::new(MdictDictionary::open(dir)?),
				Some("dicty") => Box::new(open_container(path, license_pubkey)?),
				_ => opendict::open(dir)?,
			};
		let words: Vec<String> = dict.word_list().iter().map(|s| s.to_string()).collect();
		let normalized_words = words.iter().map(|w| normalize_for_search(w)).collect();
		let language = language::detect(&words);
		let title = dict.info().name.clone();
		tracing::info!(title, language, entries = words.len(), "dictionary loaded");
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
		let word = self.words.get(index).map(String::as_str).unwrap_or("");
		tracing::debug!(index, word, bytes = out.len(), "rendered article");
		Some(out)
	}
}
