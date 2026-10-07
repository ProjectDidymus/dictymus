//! `cargo xtask release`: builds the app in release mode and packages `target/release` for this
//! platform.

use std::{
	env,
	ffi::OsStr,
	path::{Path, PathBuf},
};

use shipfitter::{Result, package::cargo_build_release};

use crate::repo_root;

/// Microsoft's permalink for the WebView2 Evergreen bootstrapper.
#[cfg(windows)]
const WEBVIEW2_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
#[cfg(windows)]
const WEBVIEW2_EXE: &str = "MicrosoftEdgeWebView2Setup.exe";

/// The portable zip the updater unpacks over a portable copy, such as `dictymus-x64.zip`.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn zip_name(arch_suffix: &str) -> String {
	format!("dictymus-{arch_suffix}.zip")
}

/// The disk image the updater downloads on macOS.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub const MAC_DMG: &str = "dictymus-macos.dmg";
/// The zip of `Dictymus.app` that versions before 0.4.0 download instead.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub const MAC_ZIP: &str = "dictymus-macos.zip";

/// Where Cargo links release binaries: `release` under `cargo_target_dir` resolved against `root`,
/// or under `root/target` when it is unset.
pub fn release_dir(root: &Path, cargo_target_dir: Option<&OsStr>) -> PathBuf {
	cargo_target_dir.map_or_else(|| root.join("target"), |dir| root.join(dir)).join("release")
}

pub fn release() -> Result<()> {
	let root = repo_root();
	cargo_build_release(&root, &["dictymus"])?;
	package(&release_dir(&root, env::var_os("CARGO_TARGET_DIR").as_deref()))
}

/// Writes `dictymus-<arch>.zip`, then compiles the `dictymus.iss` the build script wrote into
/// `dictymus_setup-<arch>.exe`.
#[cfg(windows)]
fn package(target_dir: &Path) -> Result<()> {
	use shipfitter::{
		host_arch_suffix,
		package::{Zip, inno_setup},
	};

	let exe = target_dir.join("dictymus.exe");
	if !exe.is_file() {
		return Err(format!("{} not found after the build", exe.display()).into());
	}
	let zip_path = target_dir.join(zip_name(host_arch_suffix()));
	let mut zip = Zip::create(&zip_path)?;
	zip.file(&exe, "dictymus.exe")?;
	zip.finish()?;
	eprintln!("xtask: wrote {}", zip_path.display());

	download_webview2(target_dir)?;
	let iss = target_dir.join("dictymus.iss");
	if !inno_setup(&iss)? {
		return Err(format!("no installer was built from {}", iss.display()).into());
	}
	let setup = target_dir.join(format!("dictymus_setup-{}.exe", host_arch_suffix()));
	eprintln!("xtask: wrote {}", setup.display());
	Ok(())
}

/// Copies the executable into the `Dictymus.app` the build script laid out, then writes the disk
/// image and the zip of the bundle.
#[cfg(target_os = "macos")]
fn package(target_dir: &Path) -> Result<()> {
	use std::{fs, process::Command};

	let exe = target_dir.join("dictymus");
	if !exe.is_file() {
		return Err(format!("{} not found after the build", exe.display()).into());
	}
	let bundle = target_dir.join("Dictymus.app");
	let macos_dir = bundle.join("Contents/MacOS");
	fs::create_dir_all(&macos_dir)?;
	fs::copy(&exe, macos_dir.join("dictymus"))?;

	let dmg = target_dir.join(MAC_DMG);
	shipfitter::macos::dmg(&bundle, &dmg)?;
	eprintln!("xtask: wrote {}", dmg.display());

	let zip = target_dir.join(MAC_ZIP);
	let _ = fs::remove_file(&zip);
	let status =
		Command::new("ditto").args(["-c", "-k", "--keepParent"]).arg(&bundle).arg(&zip).status()?;
	if !status.success() {
		return Err("ditto failed to zip Dictymus.app".into());
	}
	eprintln!("xtask: wrote {}", zip.display());
	Ok(())
}

#[cfg(not(any(windows, target_os = "macos")))]
fn package(_target_dir: &Path) -> Result<()> {
	Err("cargo xtask release packages on Windows and macOS only".into())
}

/// Downloads the WebView2 Evergreen bootstrapper the installer carries into `dir`, unless it is
/// already there.
#[cfg(windows)]
fn download_webview2(dir: &Path) -> Result<()> {
	let dst = dir.join(WEBVIEW2_EXE);
	if dst.is_file() {
		return Ok(());
	}
	eprintln!("xtask: downloading the WebView2 Evergreen bootstrapper");
	let status = std::process::Command::new("curl.exe")
		.args(["-fSL", "--retry", "3", "-o"])
		.arg(&dst)
		.arg(WEBVIEW2_URL)
		.status()?;
	if !status.success() {
		return Err("WebView2 bootstrapper download failed".into());
	}
	Ok(())
}
