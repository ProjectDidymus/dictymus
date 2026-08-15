//! Trust anchor and license installation for sealed dictionaries.

use dictymus_container::VerifyingKey;
use patois::t;
use std::path::{Path, PathBuf};

/// Publisher Ed25519 public key (hex) that `.dictykey` licenses are
/// verified against; generated with `dictymus-container keygen`, the
/// private half stays with the publisher.
const LICENSE_PUBKEY_HEX: &str = "12fbbdc00b51e0a8f982939dbbf803dfd43322b0a518e99eb1ab37de59f6b9f3";

/// The key licenses are verified against: the `DICTYMUS_LICENSE_PUBKEY`
/// hex override (tests, development) or the embedded publisher key.
pub fn license_pubkey() -> VerifyingKey {
	let hex =
		std::env::var("DICTYMUS_LICENSE_PUBKEY").unwrap_or_else(|_| LICENSE_PUBKEY_HEX.to_string());
	parse_pubkey(&hex).expect("invalid license public key")
}

fn parse_pubkey(hex: &str) -> Option<VerifyingKey> {
	let hex = hex.trim();
	if hex.len() != 64 || !hex.is_ascii() {
		return None;
	}
	let mut bytes = [0; 32];
	for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
		bytes[i] = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
	}
	VerifyingKey::from_bytes(&bytes).ok()
}

/// Validate `src` as a license signed by `pubkey` and copy it into the
/// app-data licenses directory; returns the installed path.
pub fn install_license(src: &Path, pubkey: &VerifyingKey) -> Result<PathBuf, String> {
	let bytes = std::fs::read(src).map_err(|e| {
		// TRANSLATORS: Error installing a license; first placeholder is the file path, second the underlying error
		t("Cannot read {}: {}").replacen("{}", &src.display().to_string(), 1).replacen(
			"{}",
			&e.to_string(),
			1,
		)
	})?;
	dictymus_container::license::License::verify(&bytes, pubkey)
		// TRANSLATORS: Error installing a license file that is invalid or not signed by the publisher
		.map_err(|_| t("This is not a valid Dictymus license file."))?;
	let dir = dictymus_core::config::AppConfig::licenses_dir()
		// TRANSLATORS: Error installing a license when no application data directory exists
		.ok_or_else(|| t("No application data directory is available."))?;
	let file_name = src
		.file_name()
		// TRANSLATORS: Error installing a license from a path that does not end in a file name
		.ok_or_else(|| t("The license path has no file name."))?;
	let dest = dir.join(file_name);
	std::fs::create_dir_all(&dir).and_then(|()| std::fs::copy(src, &dest).map(|_| ())).map_err(
		|e| {
			// TRANSLATORS: Error installing a license; first placeholder is the destination path, second the underlying error
			t("Cannot install the license to {}: {}")
				.replacen("{}", &dest.display().to_string(), 1)
				.replacen("{}", &e.to_string(), 1)
		},
	)?;
	tracing::info!(license = %dest.display(), "license installed");
	Ok(dest)
}

/// A verified license installed in the app-data licenses directory.
pub struct InstalledLicense {
	pub path: PathBuf,
	pub licensee: String,
	pub issued: String,
	pub scope_ids: Vec<String>,
}

/// Verified licenses in the app-data licenses directory, sorted by file
/// name; unreadable or unverifiable files are skipped with a warning log.
pub fn list_licenses(pubkey: &VerifyingKey) -> Vec<InstalledLicense> {
	let Some(dir) = dictymus_core::config::AppConfig::licenses_dir() else {
		return Vec::new();
	};
	let Ok(entries) = std::fs::read_dir(&dir) else {
		return Vec::new();
	};
	let mut licenses: Vec<InstalledLicense> = entries
		.flatten()
		.map(|entry| entry.path())
		.filter(|path| path.extension().is_some_and(|ext| ext == "dictykey"))
		.filter_map(|path| {
			let license = std::fs::read(&path)
				.ok()
				.and_then(|bytes| dictymus_container::license::License::verify(&bytes, pubkey).ok())
				.or_else(|| {
					tracing::warn!(path = %path.display(), "skipping unverifiable license file");
					None
				})?;
			Some(InstalledLicense {
				licensee: license.licensee().to_string(),
				issued: license.issued().to_string(),
				scope_ids: license.scope_ids().iter().map(|s| s.to_string()).collect(),
				path,
			})
		})
		.collect();
	licenses.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
	licenses
}

/// Delete the installed license file at `path`.
pub fn remove_license(path: &Path) -> Result<(), String> {
	std::fs::remove_file(path).map_err(|e| {
		// TRANSLATORS: Error removing a license; first placeholder is the file path, second the underlying error
		t("Cannot remove {}: {}").replacen("{}", &path.display().to_string(), 1).replacen(
			"{}",
			&e.to_string(),
			1,
		)
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use dictymus_core::testing;

	// Serializes every test that mutates the process-wide DICTYMUS_DATA_DIR.
	static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

	#[test]
	fn parses_the_embedded_pubkey() {
		assert!(parse_pubkey(LICENSE_PUBKEY_HEX).is_some());
		assert!(parse_pubkey("not hex").is_none());
		assert!(parse_pubkey("abcd").is_none());
	}

	#[test]
	fn installs_a_valid_license_into_the_data_dir() {
		let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
		let data_dir = tempfile::tempdir().unwrap();
		unsafe { std::env::set_var("DICTYMUS_DATA_DIR", data_dir.path()) };
		let src_dir = tempfile::tempdir().unwrap();
		let src = src_dir.path().join("customer.dictykey");
		testing::write_test_license(&src, "Jane", &[("suite".into(), testing::test_scope_key())]);

		let installed = install_license(&src, &testing::test_license_pubkey()).unwrap();
		assert!(installed.starts_with(data_dir.path()));
		assert!(installed.exists());
		assert_eq!(installed.file_name().unwrap(), "customer.dictykey");
	}

	#[test]
	fn rejects_a_file_that_is_not_a_license() {
		let dir = tempfile::tempdir().unwrap();
		let src = dir.path().join("bogus.dictykey");
		std::fs::write(&src, b"not a license").unwrap();
		assert!(install_license(&src, &testing::test_license_pubkey()).is_err());
	}

	#[test]
	fn lists_installed_licenses_and_skips_invalid_files() {
		let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
		let data_dir = tempfile::tempdir().unwrap();
		unsafe { std::env::set_var("DICTYMUS_DATA_DIR", data_dir.path()) };
		let licenses = data_dir.path().join("licenses");
		std::fs::create_dir_all(&licenses).unwrap();
		let grants = [("suite".to_string(), testing::test_scope_key())];
		testing::write_test_license(&licenses.join("alice.dictykey"), "Alice", &grants);
		testing::write_test_license(&licenses.join("bob.dictykey"), "Bob", &grants);
		std::fs::write(licenses.join("broken.dictykey"), b"not a license").unwrap();
		std::fs::write(licenses.join("readme.txt"), b"not a license either").unwrap();

		let listed = list_licenses(&testing::test_license_pubkey());
		assert_eq!(listed.len(), 2);
		assert_eq!(listed[0].licensee, "Alice");
		assert_eq!(listed[0].issued, "2026-08-15");
		assert_eq!(listed[0].scope_ids, vec!["suite".to_string()]);
		assert_eq!(listed[0].path, licenses.join("alice.dictykey"));
		assert_eq!(listed[1].licensee, "Bob");
		assert_eq!(listed[1].path, licenses.join("bob.dictykey"));
	}

	#[test]
	fn list_returns_empty_when_licenses_dir_is_missing() {
		let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
		let data_dir = tempfile::tempdir().unwrap();
		unsafe { std::env::set_var("DICTYMUS_DATA_DIR", data_dir.path()) };
		assert!(list_licenses(&testing::test_license_pubkey()).is_empty());
	}

	#[test]
	fn removes_an_installed_license() {
		let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
		let data_dir = tempfile::tempdir().unwrap();
		unsafe { std::env::set_var("DICTYMUS_DATA_DIR", data_dir.path()) };
		let src_dir = tempfile::tempdir().unwrap();
		let src = src_dir.path().join("customer.dictykey");
		testing::write_test_license(&src, "Jane", &[("suite".into(), testing::test_scope_key())]);
		let installed = install_license(&src, &testing::test_license_pubkey()).unwrap();

		remove_license(&installed).unwrap();
		assert!(!installed.exists());

		let missing = data_dir.path().join("licenses").join("gone.dictykey");
		let err = remove_license(&missing).unwrap_err();
		assert!(err.contains(&missing.display().to_string()));
	}
}
