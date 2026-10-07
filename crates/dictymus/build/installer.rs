//! Filling in `dictymus.iss.in`, the Inno Setup script `cargo xtask release` compiles.

use std::env;

use shipfitter::build::configure_file;

use crate::paths::{target_profile_dir, workspace_dir};

/// The core `x.y.z` of `version` plus `.0`, the only form `VersionInfoVersion` takes.
fn numeric_version(version: &str) -> String {
	let core = version.split(['-', '+']).next().unwrap_or(version);
	let mut parts: Vec<u64> = core.split('.').map_while(|p| p.parse().ok()).collect();
	parts.resize(3, 0);
	format!("{}.{}.{}.0", parts[0], parts[1], parts[2])
}

/// Writes `dictymus.iss` next to the executable.
pub fn configure() {
	let Some(target_dir) = target_profile_dir() else {
		return;
	};
	let root = workspace_dir();
	let version = env::var("CARGO_PKG_VERSION").unwrap_or_default();
	let license = root.join("dist").join("windows").join("license.txt");
	let icon = root.join("assets").join("icon").join("dictymus.ico");
	let replacements = [
		("VERSION_NUM", numeric_version(&version)),
		("LICENSE_FILE", license.display().to_string()),
		("SETUP_ICON", icon.display().to_string()),
	];
	let replacements: Vec<(&str, &str)> =
		replacements.iter().map(|(name, value)| (*name, value.as_str())).collect();
	configure_file(&root.join("dictymus.iss.in"), &target_dir.join("dictymus.iss"), &replacements)
		.expect("unable to write dictymus.iss");
}
