//! `cargo xtask release`: builds the app in release mode and packages `target/release` for this
//! platform.

use std::path::Path;

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

pub fn release() -> Result<()> {
	let root = repo_root();
	cargo_build_release(&root, &["dictymus"])?;
	package(&root.join("target").join("release"))
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

#[cfg(not(windows))]
fn package(_target_dir: &Path) -> Result<()> {
	Err("cargo xtask release packages on Windows; use cargo xtask dist-mac on macOS".into())
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
