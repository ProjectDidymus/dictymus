use crate::t;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_log_level() -> String {
	"warn".into()
}

/// Which release stream the auto-updater tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdateChannel {
	/// Tagged GitHub releases.
	#[default]
	Stable,
	/// The rolling `latest` prerelease rebuilt on every push to master.
	Dev,
}

impl std::fmt::Display for UpdateChannel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::Stable => "stable",
			Self::Dev => "dev",
		})
	}
}

impl std::str::FromStr for UpdateChannel {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_ascii_lowercase().as_str() {
			"stable" => Ok(Self::Stable),
			"dev" => Ok(Self::Dev),
			other => Err(format!("unknown update channel: {other}")),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
	/// Dictionaries to reopen on startup, in tab order.
	pub open_dictionaries: Vec<PathBuf>,
	/// Default tracing level when `RUST_LOG` is unset
	/// (`trace`|`debug`|`info`|`warn`|`error`).
	pub log_level: String,
	/// Check for updates when the app starts.
	pub check_for_updates_on_startup: bool,
	/// Auto-update stream: `"stable"`, `"dev"`, or empty to follow the build
	/// type (release builds track stable, development builds track dev).
	pub update_channel: String,
	/// UI language code (e.g. `"en"`, `"nl"`), or empty to follow the system
	/// language.
	pub language: String,
}

impl Default for AppConfig {
	fn default() -> Self {
		Self {
			open_dictionaries: Vec::new(),
			log_level: default_log_level(),
			check_for_updates_on_startup: true,
			update_channel: String::new(),
			language: String::new(),
		}
	}
}

impl AppConfig {
	pub fn to_toml(&self) -> String {
		toml::to_string_pretty(self).unwrap_or_default()
	}

	/// The update channel to use: the configured value if it parses, otherwise `default`.
	pub fn effective_update_channel(&self, default: UpdateChannel) -> UpdateChannel {
		self.update_channel.parse().unwrap_or(default)
	}

	pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
		if s.trim().is_empty() {
			return Ok(Self::default());
		}
		toml::from_str(s)
	}

	/// App-data root: `DICTYMUS_DATA_DIR` when set, else the OS app-data dir.
	fn data_dir() -> Option<PathBuf> {
		match std::env::var_os("DICTYMUS_DATA_DIR") {
			Some(dir) => Some(PathBuf::from(dir)),
			None => Some(dirs::data_dir()?.join("dictymus")),
		}
	}

	fn path() -> Option<PathBuf> {
		Some(Self::data_dir()?.join("config.toml"))
	}

	/// Directory where installed `.dictykey` licenses live.
	pub fn licenses_dir() -> Option<PathBuf> {
		Some(Self::data_dir()?.join("licenses"))
	}

	/// Directory for rotated log files, alongside the config in the app-data
	/// dir. `None` when no app-data dir is available.
	pub fn log_dir() -> Option<PathBuf> {
		Some(Self::data_dir()?.join("logs"))
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
				// TRANSLATORS: Startup warning; first placeholder is the settings file path, second the OS error.
				let msg = t("Could not read settings ({}): {}")
					.replacen("{}", &p.display().to_string(), 1)
					.replacen("{}", &e.to_string(), 1);
				return (Self::default(), Some(msg));
			}
		};
		match Self::from_toml(&text) {
			Ok(cfg) => (cfg, None),
			Err(e) => {
				// TRANSLATORS: Startup warning; first placeholder is the settings file path, second the parse error.
				let msg = t("Settings file is invalid ({}): {}")
					.replacen("{}", &p.display().to_string(), 1)
					.replacen("{}", &e.to_string(), 1);
				(Self::default(), Some(msg))
			}
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
	use super::{AppConfig, UpdateChannel};
	use std::path::PathBuf;

	#[test]
	fn data_dir_env_override_redirects_paths() {
		let dir = std::env::temp_dir().join(format!("dictymus-cfg-test-{}", std::process::id()));
		unsafe { std::env::set_var("DICTYMUS_DATA_DIR", &dir) };
		assert_eq!(AppConfig::log_dir().unwrap(), dir.join("logs"));
		let cfg =
			AppConfig { open_dictionaries: vec![PathBuf::from("/a/x.ifo")], ..Default::default() };
		cfg.save().unwrap();
		assert!(dir.join("config.toml").is_file());
		let (loaded, warning) = AppConfig::load();
		assert!(warning.is_none());
		assert_eq!(loaded.open_dictionaries, cfg.open_dictionaries);
		unsafe { std::env::remove_var("DICTYMUS_DATA_DIR") };
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn round_trips_open_paths() {
		let cfg = AppConfig {
			open_dictionaries: vec![PathBuf::from("/a/x.ifo"), PathBuf::from("/b/y.ifo")],
			..Default::default()
		};
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

	#[test]
	fn default_log_level_is_warn() {
		assert_eq!(AppConfig::default().log_level, "warn");
	}

	#[test]
	fn old_config_without_log_level_loads_with_default() {
		// Configs written before the field existed must still deserialize.
		let cfg = AppConfig::from_toml("open_dictionaries = [\"/a/x.ifo\"]").unwrap();
		assert_eq!(cfg.open_dictionaries, vec![PathBuf::from("/a/x.ifo")]);
		assert_eq!(cfg.log_level, "warn");
	}

	#[test]
	fn log_level_round_trips() {
		let cfg = AppConfig { log_level: "debug".into(), ..Default::default() };
		let back = AppConfig::from_toml(&cfg.to_toml()).unwrap();
		assert_eq!(back.log_level, "debug");
	}

	#[test]
	fn old_config_without_update_fields_loads_with_defaults() {
		// Configs written before the update fields existed must still deserialize.
		let cfg = AppConfig::from_toml("open_dictionaries = [\"/a/x.ifo\"]").unwrap();
		assert!(cfg.check_for_updates_on_startup);
		assert_eq!(cfg.update_channel, "");
	}

	#[test]
	fn update_fields_round_trip() {
		let cfg = AppConfig {
			check_for_updates_on_startup: false,
			update_channel: "dev".into(),
			..Default::default()
		};
		let back = AppConfig::from_toml(&cfg.to_toml()).unwrap();
		assert!(!back.check_for_updates_on_startup);
		assert_eq!(back.update_channel, "dev");
	}

	#[test]
	fn effective_update_channel_parses_explicit_values() {
		let cfg = AppConfig { update_channel: "stable".into(), ..Default::default() };
		assert_eq!(cfg.effective_update_channel(UpdateChannel::Dev), UpdateChannel::Stable);
		let cfg = AppConfig { update_channel: "dev".into(), ..Default::default() };
		assert_eq!(cfg.effective_update_channel(UpdateChannel::Stable), UpdateChannel::Dev);
		let cfg = AppConfig { update_channel: "DEV".into(), ..Default::default() };
		assert_eq!(cfg.effective_update_channel(UpdateChannel::Stable), UpdateChannel::Dev);
	}

	#[test]
	fn effective_update_channel_falls_back_to_passed_default() {
		let cfg = AppConfig::default();
		assert_eq!(cfg.effective_update_channel(UpdateChannel::Stable), UpdateChannel::Stable);
		assert_eq!(cfg.effective_update_channel(UpdateChannel::Dev), UpdateChannel::Dev);
		let cfg = AppConfig { update_channel: "nightly".into(), ..Default::default() };
		assert_eq!(cfg.effective_update_channel(UpdateChannel::Stable), UpdateChannel::Stable);
	}

	#[test]
	fn old_config_without_language_loads_with_default() {
		// Configs written before the language field existed must still deserialize.
		let cfg = AppConfig::from_toml("open_dictionaries = [\"/a/x.ifo\"]").unwrap();
		assert_eq!(cfg.language, "");
	}

	#[test]
	fn language_round_trips() {
		let cfg = AppConfig { language: "nl".into(), ..Default::default() };
		let back = AppConfig::from_toml(&cfg.to_toml()).unwrap();
		assert_eq!(back.language, "nl");
	}

	#[test]
	fn update_channel_display_and_from_str_round_trip() {
		assert_eq!(UpdateChannel::Stable.to_string(), "stable");
		assert_eq!(UpdateChannel::Dev.to_string(), "dev");
		assert_eq!("stable".parse::<UpdateChannel>(), Ok(UpdateChannel::Stable));
		assert_eq!("Dev".parse::<UpdateChannel>(), Ok(UpdateChannel::Dev));
		assert!("nightly".parse::<UpdateChannel>().is_err());
	}
}
