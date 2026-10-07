//! The macOS side of packaging: the `Dictymus.app` bundle `cargo xtask release` copies the
//! freshly built binary into.

use std::{env, fs};

use shipfitter::macos::MacApp;

use crate::paths::{target_profile_dir, workspace_dir};

/// Lays out `Dictymus.app/Contents` next to the executable, with its `Info.plist` and icon.
pub fn generate_app_bundle() {
	let Some(target_dir) = target_profile_dir() else {
		return;
	};
	let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
	let version = version.split(['-', '+']).next().unwrap_or(&version);
	let contents = target_dir.join("Dictymus.app").join("Contents");
	fs::create_dir_all(contents.join("MacOS")).expect("unable to create Dictymus.app");
	fs::create_dir_all(contents.join("Resources")).expect("unable to create Dictymus.app");
	let plist = MacApp {
		name: "Dictymus",
		identifier: "com.projectdidymus.dictymus",
		executable: "dictymus",
		version,
		icon: Some("dictymus"),
		// Must stay in sync with MACOSX_DEPLOYMENT_TARGET in the CI macOS job.
		extra: "\t<key>LSMinimumSystemVersion</key>\n\t<string>11.0</string>\n",
	}
	.info_plist();
	fs::write(contents.join("Info.plist"), plist).expect("unable to write Info.plist");
	let icns = workspace_dir().join("assets").join("icon").join("dictymus.icns");
	fs::copy(&icns, contents.join("Resources").join("dictymus.icns"))
		.expect("unable to copy dictymus.icns");
}
