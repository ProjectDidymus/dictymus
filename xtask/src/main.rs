//! Workspace automation: build Windows release binaries and package the
//! per-architecture Inno Setup installers.
//!
//! Usage:
//!   cargo xtask build-windows [--target <triple>]...
//!   cargo xtask dist [--staging <dir>] [--only-native] [--webview2 <path>]

use std::path::{Path, PathBuf};
use std::process::{Command as Proc, ExitCode};

const BIN: &str = "dictymus.exe";

/// Windows targets shipped by default (x64 + arm64).
const WINDOWS_TARGETS: [&str; 2] = ["x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"];

/// Microsoft's permalink for the WebView2 Evergreen bootstrapper.
const WEBVIEW2_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
const WEBVIEW2_EXE: &str = "MicrosoftEdgeWebView2Setup.exe";

/// The parsed subcommand.
#[derive(Debug, PartialEq, Eq)]
enum Command {
	/// Build the app in release for each target.
	BuildWindows { targets: Vec<String> },
	/// Assemble the per-arch installers from a staging directory via ISCC.
	Dist { staging: Option<PathBuf>, only_native: bool, webview2: Option<PathBuf> },
}

fn usage() -> String {
	"usage: cargo xtask <build-windows [--target <triple>]... | dist [--staging <dir>] [--only-native] [--webview2 <path>]>"
		.to_string()
}

fn parse(args: &[String]) -> Result<Command, String> {
	let (cmd, rest) = args.split_first().ok_or_else(usage)?;
	match cmd.as_str() {
		"build-windows" => parse_build_windows(rest),
		"dist" => parse_dist(rest),
		other => Err(format!("unknown command: {other}\n{}", usage())),
	}
}

fn parse_build_windows(rest: &[String]) -> Result<Command, String> {
	let mut targets = Vec::new();
	let mut it = rest.iter();
	while let Some(arg) = it.next() {
		match arg.as_str() {
			"--target" => {
				let t = it.next().ok_or_else(|| "--target requires a value".to_string())?;
				targets.push(t.clone());
			}
			other => return Err(format!("unknown build-windows option: {other}")),
		}
	}
	if targets.is_empty() {
		targets = WINDOWS_TARGETS.iter().map(|s| s.to_string()).collect();
	}
	Ok(Command::BuildWindows { targets })
}

fn parse_dist(rest: &[String]) -> Result<Command, String> {
	let mut staging = None;
	let mut only_native = false;
	let mut webview2 = None;
	let mut it = rest.iter();
	while let Some(arg) = it.next() {
		match arg.as_str() {
			"--staging" => {
				let p = it.next().ok_or_else(|| "--staging requires a path".to_string())?;
				staging = Some(PathBuf::from(p));
			}
			"--only-native" => only_native = true,
			"--webview2" => {
				let p = it.next().ok_or_else(|| "--webview2 requires a path".to_string())?;
				webview2 = Some(PathBuf::from(p));
			}
			other => return Err(format!("unknown dist option: {other}")),
		}
	}
	Ok(Command::Dist { staging, only_native, webview2 })
}

/// The app version, read from crates/dictymus/Cargo.toml at run time.
fn app_version(root: &Path) -> Result<String, String> {
	let manifest = root.join("crates").join("dictymus").join("Cargo.toml");
	let text = std::fs::read_to_string(&manifest)
		.map_err(|e| format!("read {}: {e}", manifest.display()))?;
	text.lines()
		.find_map(|l| l.trim().strip_prefix("version = \"")?.strip_suffix('"').map(str::to_string))
		.ok_or_else(|| format!("no version in {}", manifest.display()))
}

/// VersionInfoVersion accepts only numeric x.y.z.w: strip prerelease/build
/// metadata and pad ("0.2.0-rc.1" -> "0.2.0.0").
fn numeric_version(version: &str) -> String {
	let core = version.split(['-', '+']).next().unwrap_or(version);
	let mut parts: Vec<u64> = core.split('.').map_while(|p| p.parse().ok()).collect();
	parts.resize(3, 0);
	format!("{}.{}.{}.0", parts[0], parts[1], parts[2])
}

/// The CPU architecture segment of a target triple ("x86_64", "aarch64").
fn arch_of(triple: &str) -> &str {
	triple.split('-').next().unwrap_or(triple)
}

/// The Windows marketing name for a Rust CPU architecture, used in installer
/// file names and Inno Setup architecture directives.
fn win_arch(rust_arch: &str) -> &'static str {
	if rust_arch == "aarch64" { "arm64" } else { "x64" }
}

fn native_arch() -> &'static str {
	if cfg!(target_arch = "aarch64") { "aarch64" } else { "x86_64" }
}

/// Where a target's exe is staged for the installer, grouped by CPU arch.
fn staged_exe(staging: &Path, triple: &str) -> PathBuf {
	staging.join("windows").join(arch_of(triple)).join(BIN)
}

/// Copy `src` to `dst`, creating parent directories as needed.
fn stage_copy(src: &Path, dst: &Path) -> Result<(), String> {
	if let Some(parent) = dst.parent() {
		std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
	}
	std::fs::copy(src, dst)
		.map_err(|e| format!("copy {} -> {}: {e}", src.display(), dst.display()))?;
	Ok(())
}

/// The repository root, one level above this crate's manifest.
fn repo_root() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.expect("xtask manifest dir has a parent")
		.to_path_buf()
}

/// Locate the Inno Setup compiler: `$ISCC`, then `PATH`, then well-known
/// install directories on Windows (per-machine and per-user winget installs).
fn find_iscc() -> Option<PathBuf> {
	if let Some(p) = std::env::var_os("ISCC") {
		return Some(PathBuf::from(p));
	}
	let exe = if cfg!(windows) { "ISCC.exe" } else { "iscc" };
	if let Some(paths) = std::env::var_os("PATH") {
		for dir in std::env::split_paths(&paths) {
			let candidate = dir.join(exe);
			if candidate.is_file() {
				return Some(candidate);
			}
		}
	}
	if cfg!(windows) {
		for base in ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"] {
			if let Some(root) = std::env::var_os(base) {
				let mut candidate = PathBuf::from(root);
				if base == "LOCALAPPDATA" {
					candidate.push("Programs");
				}
				candidate.push("Inno Setup 6");
				candidate.push("ISCC.exe");
				if candidate.is_file() {
					return Some(candidate);
				}
			}
		}
	}
	None
}

fn run_build_windows(targets: &[String]) -> Result<(), String> {
	for target in targets {
		eprintln!("xtask: building dictymus (release) for {target}");
		let status = Proc::new("cargo")
			.args(["build", "--release", "--target", target, "-p", "dictymus"])
			.status()
			.map_err(|e| format!("failed to spawn cargo: {e}"))?;
		if !status.success() {
			return Err(format!("cargo build failed for {target}"));
		}
	}
	Ok(())
}

fn stage_webview2(staging: &Path, override_path: Option<&Path>) -> Result<(), String> {
	let dst = staging.join(WEBVIEW2_EXE);
	if let Some(src) = override_path {
		return stage_copy(src, &dst);
	}
	if dst.is_file() {
		return Ok(());
	}
	std::fs::create_dir_all(staging).map_err(|e| format!("create {}: {e}", staging.display()))?;
	eprintln!("xtask: downloading the WebView2 Evergreen bootstrapper");
	let status = Proc::new("curl.exe")
		.args(["-fSL", "--retry", "3", "-o"])
		.arg(&dst)
		.arg(WEBVIEW2_URL)
		.status()
		.map_err(|e| format!("failed to spawn curl: {e}"))?;
	if !status.success() {
		return Err("WebView2 bootstrapper download failed".to_string());
	}
	Ok(())
}

fn run_dist(
	staging: Option<&Path>,
	only_native: bool,
	webview2: Option<&Path>,
) -> Result<(), String> {
	let root = repo_root();
	let script = root.join("dist").join("windows").join("installer.iss");
	if !script.exists() {
		return Err(format!("installer script not found at {}", script.display()));
	}
	let staging =
		staging.map(Path::to_path_buf).unwrap_or_else(|| root.join("target").join("dist"));
	// ISCC resolves relative Source paths against the script directory, so
	// the staging path must reach it absolute and with native separators.
	let staging: PathBuf = std::path::absolute(&staging)
		.map_err(|e| format!("absolutize {}: {e}", staging.display()))?
		.components()
		.collect();

	// A file already present in the staging layout wins; CI downloads the
	// built artifacts straight into it.
	let targets: Vec<&str> = if only_native {
		WINDOWS_TARGETS.iter().copied().filter(|t| arch_of(t) == native_arch()).collect()
	} else {
		WINDOWS_TARGETS.to_vec()
	};
	for target in &targets {
		let dst = staged_exe(&staging, target);
		if dst.is_file() {
			continue;
		}
		let src = root.join("target").join(target).join("release").join(BIN);
		if !src.is_file() {
			return Err(format!(
				"missing release binary for {target}: {} (run: cargo xtask build-windows --target {target})",
				src.display()
			));
		}
		stage_copy(&src, &dst)?;
	}

	stage_webview2(&staging, webview2)?;

	let version = app_version(&root)?;
	let outdir = root.join("target");
	let iscc =
		find_iscc().ok_or_else(|| "ISCC not found; install Inno Setup or set ISCC".to_string())?;

	for target in &targets {
		let arch = win_arch(arch_of(target));
		let defines = iscc_defines(
			&version,
			&staging,
			&outdir,
			arch,
			&root.join("assets").join("icon").join("dictymus.ico"),
		);
		eprintln!("xtask: {} {} {}", iscc.display(), defines.join(" "), script.display());
		let status = Proc::new(&iscc)
			.arg("/Q")
			.args(&defines)
			.arg(&script)
			.status()
			.map_err(|e| format!("failed to spawn ISCC: {e}"))?;
		if !status.success() {
			return Err(format!("ISCC failed for {arch}"));
		}
		eprintln!("xtask: wrote {}", outdir.join(format!("dictymus_setup-{arch}.exe")).display());
	}
	Ok(())
}

/// The `/D` defines passed to ISCC, one argv element each so paths with
/// spaces survive. Values must not end in a backslash.
fn iscc_defines(
	version: &str,
	staging: &Path,
	outdir: &Path,
	arch: &str,
	icon: &Path,
) -> Vec<String> {
	vec![
		format!("/DVERSION={version}"),
		format!("/DVERSIONNUM={}", numeric_version(version)),
		format!("/DSTAGING={}", staging.display()),
		format!("/DOUTDIR={}", outdir.display()),
		format!("/DARCH={arch}"),
		format!("/DICONFILE={}", icon.display()),
	]
}

fn main() -> ExitCode {
	let args: Vec<String> = std::env::args().skip(1).collect();
	let cmd = match parse(&args) {
		Ok(c) => c,
		Err(e) => {
			eprintln!("{e}");
			return ExitCode::FAILURE;
		}
	};
	let result = match cmd {
		Command::BuildWindows { targets } => run_build_windows(&targets),
		Command::Dist { staging, only_native, webview2 } => {
			run_dist(staging.as_deref(), only_native, webview2.as_deref())
		}
	};
	match result {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			eprintln!("xtask: {e}");
			ExitCode::FAILURE
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn v(args: &[&str]) -> Vec<String> {
		args.iter().map(|s| s.to_string()).collect()
	}

	#[test]
	fn build_windows_defaults_to_both_arches() {
		assert_eq!(
			parse(&v(&["build-windows"])).unwrap(),
			Command::BuildWindows {
				targets: vec![
					"x86_64-pc-windows-msvc".to_string(),
					"aarch64-pc-windows-msvc".to_string(),
				],
			},
		);
	}

	#[test]
	fn dist_options_parse() {
		assert_eq!(
			parse(&v(&["dist"])).unwrap(),
			Command::Dist { staging: None, only_native: false, webview2: None },
		);
		assert_eq!(
			parse(&v(&["dist", "--staging", "out", "--only-native", "--webview2", "wv.exe"]))
				.unwrap(),
			Command::Dist {
				staging: Some(PathBuf::from("out")),
				only_native: true,
				webview2: Some(PathBuf::from("wv.exe")),
			},
		);
	}

	#[test]
	fn unknown_or_missing_command_errors() {
		assert!(parse(&v(&["frobnicate"])).is_err());
		assert!(parse(&v(&[])).is_err());
		assert!(parse(&v(&["dist", "--bogus"])).is_err());
	}

	#[test]
	fn numeric_version_strips_prerelease_and_pads() {
		assert_eq!(numeric_version("0.1.0"), "0.1.0.0");
		assert_eq!(numeric_version("0.2.0-rc.1"), "0.2.0.0");
		assert_eq!(numeric_version("1.2.3+build.5"), "1.2.3.0");
		assert_eq!(numeric_version("1.2"), "1.2.0.0");
	}

	#[test]
	fn arch_of_extracts_cpu() {
		assert_eq!(arch_of("x86_64-pc-windows-msvc"), "x86_64");
		assert_eq!(arch_of("aarch64-pc-windows-msvc"), "aarch64");
	}

	#[test]
	fn staged_exe_groups_by_arch() {
		assert_eq!(
			staged_exe(Path::new("stage"), "aarch64-pc-windows-msvc"),
			Path::new("stage").join("windows").join("aarch64").join("dictymus.exe"),
		);
	}

	#[test]
	fn win_arch_maps_rust_arches() {
		assert_eq!(win_arch("x86_64"), "x64");
		assert_eq!(win_arch("aarch64"), "arm64");
	}

	#[test]
	fn iscc_defines_carry_version_arch_and_paths() {
		assert_eq!(
			iscc_defines(
				"1.2.3-rc.1",
				Path::new("stage"),
				Path::new("out"),
				"arm64",
				Path::new("app.ico")
			),
			vec![
				"/DVERSION=1.2.3-rc.1".to_string(),
				"/DVERSIONNUM=1.2.3.0".to_string(),
				"/DSTAGING=stage".to_string(),
				"/DOUTDIR=out".to_string(),
				"/DARCH=arm64".to_string(),
				"/DICONFILE=app.ico".to_string(),
			],
		);
	}
}
