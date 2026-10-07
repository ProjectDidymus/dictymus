//! Workspace automation: build the app in release mode and package it, and
//! regenerate the translation catalogs.
//!
//! Usage:
//!   cargo xtask release
//!   cargo xtask dist-mac [--target <triple>]
//!   cargo xtask gen-pot
//!   cargo xtask translate

use std::path::{Path, PathBuf};
use std::process::{Command as Proc, ExitCode};

mod release;
mod sanitize_rust;

const MAC_BIN: &str = "dictymus";

/// The parsed subcommand.
#[derive(Debug, PartialEq, Eq)]
enum Command {
	/// Build the app in release mode and package it for this platform.
	Release,
	/// Assemble Dictymus.app and package the DMG and updater zip.
	DistMac { target: Option<String> },
	/// Regenerate the pot from source.
	GenPot,
	/// Regenerate the pot from source and msgmerge it into every po file.
	Translate,
}

fn usage() -> String {
	"usage: cargo xtask <release | dist-mac [--target <triple>] | gen-pot | translate>".to_string()
}

fn parse(args: &[String]) -> Result<Command, String> {
	let (cmd, rest) = args.split_first().ok_or_else(usage)?;
	match cmd.as_str() {
		"release" => match rest {
			[] => Ok(Command::Release),
			[other, ..] => Err(format!("unknown release option: {other}")),
		},
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
		Command::Release => release::release().map_err(|e| e.to_string()),
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
		assert!(parse(&v(&["dist"])).is_err());
		assert!(parse(&v(&["build-windows"])).is_err());
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
	fn release_parses() {
		assert_eq!(parse(&v(&["release"])).unwrap(), Command::Release);
		assert!(parse(&v(&["release", "--bogus"])).is_err());
	}

	#[test]
	fn zip_name_carries_the_arch_suffix() {
		assert_eq!(release::zip_name("x64"), "dictymus-x64.zip");
		assert_eq!(release::zip_name("arm64"), "dictymus-arm64.zip");
	}
}
