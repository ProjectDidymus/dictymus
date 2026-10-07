//! Build script for the Dictymus desktop app. Everything it does lives in the modules under
//! `build/`; this file only decides what runs, and in what order.

#[path = "build/installer.rs"]
mod installer;
#[path = "build/macos.rs"]
mod macos;
#[path = "build/paths.rs"]
mod paths;
#[path = "build/translations.rs"]
mod translations;
#[path = "build/version.rs"]
mod version;
#[path = "build/windows.rs"]
mod windows;

use std::env;

fn main() {
	paths::track_packaging_inputs();
	translations::build();
	installer::configure();
	let commit = version::embed_commit_info();
	let target = env::var("TARGET").unwrap_or_default();
	if target.contains("apple") {
		macos::generate_app_bundle();
	}
	if target.contains("windows") {
		windows::embed_app_manifest();
		windows::embed_version_info(&commit);
	}
}
