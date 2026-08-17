//! Workspace automation: build Windows release binaries, package the
//! per-architecture Inno Setup installers, and package the macOS app
//! bundle, DMG, and updater zip.
//!
//! Usage:
//!   cargo xtask build-windows [--target <triple>]...
//!   cargo xtask dist [--staging <dir>] [--only-native] [--webview2 <path>]
//!   cargo xtask dist-mac [--target <triple>]
//!   cargo xtask gen-pot
//!   cargo xtask translate

use std::path::{Path, PathBuf};
use std::process::{Command as Proc, ExitCode};

mod sanitize_rust;

const BIN: &str = "dictymus.exe";
const MAC_BIN: &str = "dictymus";

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
	/// Assemble Dictymus.app and package the DMG and updater zip.
	DistMac { target: Option<String> },
	/// Regenerate the pot from source.
	GenPot,
	/// Regenerate the pot from source and msgmerge it into every po file.
	Translate,
}

fn usage() -> String {
	"usage: cargo xtask <build-windows [--target <triple>]... | dist [--staging <dir>] [--only-native] [--webview2 <path>] | dist-mac [--target <triple>] | gen-pot | translate>"
		.to_string()
}

fn parse(args: &[String]) -> Result<Command, String> {
	let (cmd, rest) = args.split_first().ok_or_else(usage)?;
	match cmd.as_str() {
		"build-windows" => parse_build_windows(rest),
		"dist" => parse_dist(rest),
		"dist-mac" => parse_dist_mac(rest),
		"gen-pot" => match rest {
			[] => Ok(Command::GenPot),
			[other, ..] => Err(format!("unknown gen-pot option: {other}")),
		},
		"translate" => match rest {
			[] => Ok(Command::Translate),
			[other, ..] => Err(format!("unknown translate option: {other}")),
		},
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

fn parse_dist_mac(rest: &[String]) -> Result<Command, String> {
	let mut target = None;
	let mut it = rest.iter();
	while let Some(arg) = it.next() {
		match arg.as_str() {
			"--target" => {
				let t = it.next().ok_or_else(|| "--target requires a value".to_string())?;
				target = Some(t.clone());
			}
			other => return Err(format!("unknown dist-mac option: {other}")),
		}
	}
	Ok(Command::DistMac { target })
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

/// The bundle version for the Info.plist keys: the core x.y.z with any
/// prerelease/build metadata stripped ("0.2.0-rc.1" -> "0.2.0").
fn bundle_version(version: &str) -> &str {
	version.split(['-', '+']).next().unwrap_or(version)
}

/// The Dictymus.app Info.plist. LSMinimumSystemVersion must stay in sync
/// with MACOSX_DEPLOYMENT_TARGET in the CI macOS job.
fn info_plist(version: &str) -> String {
	let version = bundle_version(version);
	format!(
		r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>Dictymus</string>
	<key>CFBundleDisplayName</key>
	<string>Dictymus</string>
	<key>CFBundleIdentifier</key>
	<string>com.projectdidymus.dictymus</string>
	<key>CFBundleExecutable</key>
	<string>dictymus</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleVersion</key>
	<string>{version}</string>
	<key>CFBundleShortVersionString</key>
	<string>{version}</string>
	<key>CFBundleIconFile</key>
	<string>dictymus</string>
	<key>LSMinimumSystemVersion</key>
	<string>11.0</string>
	<key>NSHighResolutionCapable</key>
	<true/>
</dict>
</plist>
"#
	)
}

/// Run an external tool, failing with its name on spawn errors or a
/// nonzero exit.
fn run_tool(mut cmd: Proc) -> Result<(), String> {
	let program = cmd.get_program().to_string_lossy().into_owned();
	let status = cmd.status().map_err(|e| format!("failed to spawn {program}: {e}"))?;
	if !status.success() {
		return Err(format!("{program} failed"));
	}
	Ok(())
}

fn codesign(path: &Path, identity: &str) -> Result<(), String> {
	let mut cmd = Proc::new("codesign");
	cmd.args(["--force", "--timestamp", "--options", "runtime", "--sign", identity]).arg(path);
	run_tool(cmd)
}

/// Sign the executable, then the bundle as a whole (deepest first), with
/// the Developer ID Application identity named by `MACOS_SIGN_IDENTITY`.
/// A no-op when that variable is unset.
fn sign_mac_bundle(bundle: &Path, macos_dir: &Path) -> Result<(), String> {
	let Ok(identity) = std::env::var("MACOS_SIGN_IDENTITY") else {
		eprintln!("xtask: MACOS_SIGN_IDENTITY not set; skipping code signing");
		return Ok(());
	};
	codesign(&macos_dir.join(MAC_BIN), &identity)?;
	codesign(bundle, &identity)?;
	eprintln!("xtask: signed {}", bundle.display());
	Ok(())
}

/// Assemble `Dictymus.app` from the release binary, sign it when an
/// identity is configured, and package `target/dictymus-macos.dmg`
/// (drag-to-Applications layout) plus `target/dictymus-macos.zip` for the
/// updater.
fn run_dist_mac(target: Option<&str>) -> Result<(), String> {
	if !cfg!(target_os = "macos") {
		return Err("dist-mac requires macOS (codesign, hdiutil and ditto)".to_string());
	}
	let root = repo_root();
	let release_dir = match target {
		Some(t) => root.join("target").join(t).join("release"),
		None => root.join("target").join("release"),
	};
	let exe = release_dir.join(MAC_BIN);
	if !exe.is_file() {
		return Err(format!(
			"missing release binary: {} (run: cargo build --release -p dictymus{})",
			exe.display(),
			target.map(|t| format!(" --target {t}")).unwrap_or_default(),
		));
	}

	let stage = root.join("target").join("dist-mac");
	let _ = std::fs::remove_dir_all(&stage);
	let bundle = stage.join("Dictymus.app");
	let macos_dir = bundle.join("Contents").join("MacOS");
	let resources_dir = bundle.join("Contents").join("Resources");

	stage_copy(&exe, &macos_dir.join(MAC_BIN))?;
	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt;
		std::fs::set_permissions(macos_dir.join(MAC_BIN), std::fs::Permissions::from_mode(0o755))
			.map_err(|e| format!("chmod {}: {e}", macos_dir.join(MAC_BIN).display()))?;
	}
	stage_copy(
		&root.join("assets").join("icon").join("dictymus.icns"),
		&resources_dir.join("dictymus.icns"),
	)?;
	let plist = bundle.join("Contents").join("Info.plist");
	std::fs::write(&plist, info_plist(&app_version(&root)?))
		.map_err(|e| format!("write {}: {e}", plist.display()))?;

	sign_mac_bundle(&bundle, &macos_dir)?;

	// The DMG holds the bundle plus an /Applications symlink so Finder
	// shows the standard drag-to-install layout.
	let dmg_staging = stage.join("dmg-staging");
	std::fs::create_dir_all(&dmg_staging)
		.map_err(|e| format!("create {}: {e}", dmg_staging.display()))?;
	let mut copy = Proc::new("ditto");
	copy.arg(&bundle).arg(dmg_staging.join("Dictymus.app"));
	run_tool(copy)?;
	#[cfg(unix)]
	std::os::unix::fs::symlink("/Applications", dmg_staging.join("Applications"))
		.map_err(|e| format!("symlink /Applications: {e}"))?;

	let dmg = root.join("target").join("dictymus-macos.dmg");
	let mut hdiutil = Proc::new("hdiutil");
	hdiutil
		.args(["create", "-volname", "Dictymus", "-srcfolder"])
		.arg(&dmg_staging)
		.args(["-ov", "-format", "UDZO"])
		.arg(&dmg);
	run_tool(hdiutil)?;
	eprintln!("xtask: wrote {}", dmg.display());

	// ditto keeps the executable bit and xattrs a plain zip writer drops.
	let zip = root.join("target").join("dictymus-macos.zip");
	let _ = std::fs::remove_file(&zip);
	let mut ditto = Proc::new("ditto");
	ditto.args(["-c", "-k", "--keepParent"]).arg(&bundle).arg(&zip);
	run_tool(ditto)?;
	eprintln!("xtask: wrote {}", zip.display());
	Ok(())
}

/// Regenerate `po/dictymus.pot` from every crate tagged
/// `[package.metadata.patois] translatable = true`, registry dependencies such
/// as `ship-shape` included. Requires `xgettext` and `cargo` on `PATH`.
///
/// `xgettext` reads the sources as C and mis-tokenizes Rust lifetimes and raw
/// strings, so each package's `src` is copied through
/// `sanitize_rust::sanitize_for_xgettext` into `target/gen-pot-sanitized` and
/// the copies are scanned instead.
fn run_gen_pot() -> Result<(), String> {
	let root = repo_root();
	let po_dir = root.join("po");
	let (packages, version) = translatable_packages(&root)?;
	if packages.is_empty() {
		return Err("no translatable crates found: check [package.metadata.patois]".to_string());
	}
	let sanitized_root = root.join("target").join("gen-pot-sanitized");
	let _ = std::fs::remove_dir_all(&sanitized_root);
	let mut sanitized_dirs = Vec::new();
	for (name, src) in &packages {
		let dest = sanitized_root.join(name).join("src");
		sanitize_dir_into(src, &dest)?;
		sanitized_dirs.push(dest);
	}
	let generated = patois_build::gen_pot_from_dirs(&sanitized_dirs, &po_dir, "dictymus", &version)
		.map_err(|e| format!("gen_pot: {e}"));
	let _ = std::fs::remove_dir_all(&sanitized_root);
	generated
}

/// The name and `src` directory of every package `cargo metadata` reports as
/// translatable, sorted by name, plus dictymus's own version for the pot header.
///
/// Sorting by name rather than path keeps the file order — and so the pot's
/// entry order — the same on machines whose registry caches live elsewhere.
fn translatable_packages(root: &Path) -> Result<(Vec<(String, PathBuf)>, String), String> {
	let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
	let output = Proc::new(&cargo)
		.args(["metadata", "--format-version", "1"])
		.current_dir(root)
		.output()
		.map_err(|e| format!("failed to spawn cargo metadata: {e}"))?;
	if !output.status.success() {
		return Err("cargo metadata failed".to_string());
	}
	let meta: serde_json::Value =
		serde_json::from_slice(&output.stdout).map_err(|e| format!("cargo metadata: {e}"))?;
	let mut packages: Vec<&serde_json::Value> = meta["packages"]
		.as_array()
		.ok_or_else(|| "cargo metadata: missing packages".to_string())?
		.iter()
		.collect();
	packages.sort_by_key(|pkg| pkg["name"].as_str().unwrap_or_default().to_string());
	let version = packages
		.iter()
		.find(|pkg| pkg["name"] == "dictymus")
		.and_then(|pkg| pkg["version"].as_str())
		.ok_or_else(|| "cargo metadata: dictymus not found".to_string())?
		.to_string();
	let mut translatable = Vec::new();
	for pkg in &packages {
		if pkg["metadata"]["patois"]["translatable"] != true {
			continue;
		}
		let name = pkg["name"].as_str().unwrap_or_default().to_string();
		let manifest = pkg["manifest_path"]
			.as_str()
			.ok_or_else(|| format!("cargo metadata: {name} has no manifest_path"))?;
		let src = Path::new(manifest)
			.parent()
			.ok_or_else(|| format!("cargo metadata: bad manifest_path for {name}"))?
			.join("src");
		translatable.push((name, src));
	}
	Ok((translatable, version))
}

/// Copy every `.rs` file under `src` to the same relative path under `dest`,
/// sanitized for `xgettext`.
///
/// Fails when sanitizing would blank a literal that a `t(`/`nt(` call passes,
/// since that string would then vanish from the pot unnoticed.
fn sanitize_dir_into(src: &Path, dest: &Path) -> Result<(), String> {
	let mut files = Vec::new();
	collect_rust_files(src, &mut files)?;
	for path in files {
		let rel = path.strip_prefix(src).map_err(|e| e.to_string())?;
		let out_path = dest.join(rel);
		if let Some(parent) = out_path.parent() {
			std::fs::create_dir_all(parent)
				.map_err(|e| format!("create {}: {e}", parent.display()))?;
		}
		let content =
			std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
		let sanitized = sanitize_rust::sanitize_for_xgettext(&content);
		if let Some(line) = sanitized.blanked_call_literals.first() {
			return Err(format!(
				"{}:{line}: a translatable string spans several source lines; xgettext cannot read it, so keep it on one line",
				path.display()
			));
		}
		std::fs::write(&out_path, sanitized.text)
			.map_err(|e| format!("write {}: {e}", out_path.display()))?;
	}
	Ok(())
}

/// Append every `.rs` file under `dir`, recursively.
fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
	let Ok(entries) = std::fs::read_dir(dir) else {
		return Ok(());
	};
	for entry in entries {
		let path = entry.map_err(|e| e.to_string())?.path();
		if path.is_dir() {
			collect_rust_files(&path, files)?;
		} else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
			files.push(path);
		}
	}
	Ok(())
}

/// Regenerate the pot, then update each `po/*.po` against it via `msgmerge` so
/// new and changed strings show up for translation. Requires `xgettext`,
/// `cargo` and `msgmerge` on `PATH`.
fn run_translate() -> Result<(), String> {
	run_gen_pot()?;
	let po_dir = repo_root().join("po");
	let pot = po_dir.join("dictymus.pot");
	let entries =
		std::fs::read_dir(&po_dir).map_err(|e| format!("read {}: {e}", po_dir.display()))?;
	for entry in entries {
		let path = entry.map_err(|e| e.to_string())?.path();
		if path.extension().and_then(|e| e.to_str()) != Some("po") {
			continue;
		}
		eprintln!("xtask: merging {}", path.display());
		let status = Proc::new("msgmerge")
			.args(["--update", "--backup=off", "--no-wrap"])
			.arg(&path)
			.arg(&pot)
			.status()
			.map_err(|e| format!("failed to spawn msgmerge: {e}"))?;
		if !status.success() {
			return Err(format!("msgmerge failed for {}", path.display()));
		}
	}
	Ok(())
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
		Command::DistMac { target } => run_dist_mac(target.as_deref()),
		Command::GenPot => run_gen_pot(),
		Command::Translate => run_translate(),
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
	fn translate_parses() {
		assert_eq!(parse(&v(&["translate"])).unwrap(), Command::Translate);
		assert!(parse(&v(&["translate", "--bogus"])).is_err());
	}

	#[test]
	fn gen_pot_parses() {
		assert_eq!(parse(&v(&["gen-pot"])).unwrap(), Command::GenPot);
		assert!(parse(&v(&["gen-pot", "--bogus"])).is_err());
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
	fn dist_mac_options_parse() {
		assert_eq!(parse(&v(&["dist-mac"])).unwrap(), Command::DistMac { target: None });
		assert_eq!(
			parse(&v(&["dist-mac", "--target", "aarch64-apple-darwin"])).unwrap(),
			Command::DistMac { target: Some("aarch64-apple-darwin".to_string()) },
		);
	}

	#[test]
	fn unknown_or_missing_command_errors() {
		assert!(parse(&v(&["frobnicate"])).is_err());
		assert!(parse(&v(&[])).is_err());
		assert!(parse(&v(&["dist", "--bogus"])).is_err());
		assert!(parse(&v(&["dist-mac", "--bogus"])).is_err());
	}

	#[test]
	fn bundle_version_strips_prerelease() {
		assert_eq!(bundle_version("0.2.0"), "0.2.0");
		assert_eq!(bundle_version("0.2.0-rc.1"), "0.2.0");
		assert_eq!(bundle_version("1.2.3+build.5"), "1.2.3");
	}

	#[test]
	fn info_plist_carries_identity_and_version() {
		let plist = info_plist("0.2.0-rc.1");
		assert!(plist.contains(
			"<key>CFBundleIdentifier</key>\n\t<string>com.projectdidymus.dictymus</string>"
		));
		assert!(plist.contains("<key>CFBundleExecutable</key>\n\t<string>dictymus</string>"));
		assert!(plist.contains("<key>CFBundleVersion</key>\n\t<string>0.2.0</string>"));
		assert!(plist.contains("<key>CFBundleShortVersionString</key>\n\t<string>0.2.0</string>"));
		assert!(plist.contains("<key>LSMinimumSystemVersion</key>\n\t<string>11.0</string>"));
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
