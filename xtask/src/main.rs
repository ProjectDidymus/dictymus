//! Workspace automation: build the app in release mode and package it, and
//! regenerate the translation catalogs.
//!
//! Usage:
//!   cargo xtask release
//!   cargo xtask gen-pot
//!   cargo xtask translate

use std::path::{Path, PathBuf};
use std::process::{Command as Proc, ExitCode};

mod release;
mod sanitize_rust;

/// The parsed subcommand.
#[derive(Debug, PartialEq, Eq)]
enum Command {
	/// Build the app in release mode and package it for this platform.
	Release,
	/// Regenerate the pot from source.
	GenPot,
	/// Regenerate the pot from source and msgmerge it into every po file.
	Translate,
}

fn usage() -> String {
	"usage: cargo xtask <release | gen-pot | translate>".to_string()
}

fn parse(args: &[String]) -> Result<Command, String> {
	let (cmd, rest) = args.split_first().ok_or_else(usage)?;
	match cmd.as_str() {
		"release" => match rest {
			[] => Ok(Command::Release),
			[other, ..] => Err(format!("unknown release option: {other}")),
		},
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

/// The repository root, one level above this crate's manifest.
fn repo_root() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.expect("xtask manifest dir has a parent")
		.to_path_buf()
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
	fn unknown_or_missing_command_errors() {
		assert!(parse(&v(&["frobnicate"])).is_err());
		assert!(parse(&v(&[])).is_err());
		assert!(parse(&v(&["dist"])).is_err());
		assert!(parse(&v(&["build-windows"])).is_err());
		assert!(parse(&v(&["dist-mac"])).is_err());
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

	#[test]
	fn release_dir_follows_cargo_target_dir() {
		let root = Path::new("repo");
		assert_eq!(release::release_dir(root, None), root.join("target").join("release"));
		assert_eq!(
			release::release_dir(root, Some("out".as_ref())),
			root.join("out").join("release")
		);
		let absolute = std::env::temp_dir().join("dictymus-target");
		assert_eq!(
			release::release_dir(root, Some(absolute.as_os_str())),
			absolute.join("release")
		);
	}

	#[test]
	fn mac_assets_match_the_updater_names() {
		assert_eq!(release::MAC_DMG, "dictymus-macos.dmg");
		assert_eq!(release::MAC_ZIP, "dictymus-macos.zip");
	}
}
