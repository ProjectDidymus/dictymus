//! Windows-only resources: the application manifest, the icon and the version block Explorer
//! shows in the file properties dialog.

use std::env;

use shipfitter::{build::CommitInfo, windows};

/// Embeds the application manifest that asks Windows for version 6 of the common controls,
/// UTF-8, the segment heap, per-monitor DPI awareness and long path support.
pub fn embed_app_manifest() {
	windows::embed_manifest("Dictymus").expect("unable to embed manifest");
}

/// Embeds the icon and the version block; development builds show the short commit hash after
/// the version.
pub fn embed_version_info(commit: &CommitInfo) {
	let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
	let product_version =
		if commit.is_dev { format!("{version} ({})", commit.short_hash) } else { version };
	windows::VersionInfo {
		product_name: "Dictymus",
		company: "Project Didymus",
		copyright: "Copyright © 2026 Project Didymus",
		original_filename: "dictymus.exe",
		product_version: Some(&product_version),
		icon: Some("../../assets/icon/dictymus.ico"),
	}
	.embed()
	.expect("unable to embed the version resource");
}
