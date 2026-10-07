//! Locating the directories the other build steps read from and write to, plus the
//! `rerun-if-changed` list that decides when Cargo runs this script again.

use std::{
	env,
	path::{Path, PathBuf},
};

pub fn track_packaging_inputs() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=build");
	println!("cargo:rerun-if-changed=Cargo.toml");
	println!("cargo:rerun-if-changed=../../assets/icon");
}

/// The workspace root, two levels above this crate.
pub fn workspace_dir() -> PathBuf {
	let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
	manifest_dir
		.parent()
		.and_then(Path::parent)
		.expect("crate lives two levels below the workspace root")
		.to_path_buf()
}

/// The profile directory the executable is linked into, such as `target/release`: three levels
/// above the always absolute `OUT_DIR`.
pub fn target_profile_dir() -> Option<PathBuf> {
	let out_dir = PathBuf::from(env::var_os("OUT_DIR")?);
	out_dir.ancestors().nth(3).map(Path::to_path_buf)
}
