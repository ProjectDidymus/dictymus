use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
	/// Dictionaries to reopen on startup, in tab order.
	pub open_dictionaries: Vec<PathBuf>,
}

impl AppConfig {
	pub fn to_toml(&self) -> String {
		toml::to_string_pretty(self).unwrap_or_default()
	}

	pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
		if s.trim().is_empty() {
			return Ok(Self::default());
		}
		toml::from_str(s)
	}

	fn path() -> Option<PathBuf> {
		let dir = dirs::data_dir()?.join("dictymus");
		Some(dir.join("config.toml"))
	}

	/// Load from the OS app-data dir. A missing file is a normal first run
	/// → `(default, None)`. Any other IO error or corrupt TOML → defaults
	/// plus a warning the caller should surface; never aborts.
	pub fn load() -> (Self, Option<String>) {
		let Some(p) = Self::path() else { return (Self::default(), None) };
		let text = match std::fs::read_to_string(&p) {
			Ok(text) => text,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				return (Self::default(), None);
			}
			Err(e) => {
				return (
					Self::default(),
					Some(format!("Could not read settings ({}): {e}", p.display())),
				);
			}
		};
		match Self::from_toml(&text) {
			Ok(cfg) => (cfg, None),
			Err(e) => (
				Self::default(),
				Some(format!("Settings file is invalid ({}): {e}", p.display())),
			),
		}
	}

	/// Save to the OS app-data dir, creating parent dirs.
	pub fn save(&self) -> Result<(), std::io::Error> {
		let Some(p) = Self::path() else {
			return Err(std::io::Error::new(
				std::io::ErrorKind::NotFound,
				"no application data directory available",
			));
		};
		if let Some(parent) = p.parent() {
			std::fs::create_dir_all(parent)?;
		}
		std::fs::write(&p, self.to_toml())
	}
}

#[cfg(test)]
mod tests {
	use super::AppConfig;
	use std::path::PathBuf;

	#[test]
	fn round_trips_open_paths() {
		let mut cfg = AppConfig::default();
		cfg.open_dictionaries = vec![PathBuf::from("/a/x.ifo"), PathBuf::from("/b/y.ifo")];
		let toml = cfg.to_toml();
		let back = AppConfig::from_toml(&toml).unwrap();
		assert_eq!(back.open_dictionaries, cfg.open_dictionaries);
	}

	#[test]
	fn empty_toml_is_default() {
		let cfg = AppConfig::from_toml("").unwrap();
		assert!(cfg.open_dictionaries.is_empty());
	}

	#[test]
	fn corrupt_toml_is_an_error() {
		assert!(AppConfig::from_toml("open_dictionaries = \"not a list").is_err());
	}
}
