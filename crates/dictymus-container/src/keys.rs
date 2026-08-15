//! Publisher-side keyfile handling for the CLI: hex-encoded 32-byte
//! scope keys (`.dek`) and Ed25519 signing keys.

use crate::{Error, Result, crypto};
use ed25519_dalek::SigningKey;
use std::path::Path;

fn to_hex(bytes: &[u8]) -> String {
	bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn parse_hex32(s: &str) -> Result<[u8; 32]> {
	let s = s.trim();
	if s.len() != 64 || !s.is_ascii() {
		return Err(Error::Malformed("keyfile must hold 64 hex digits"));
	}
	let mut out = [0; 32];
	for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
		let chunk = std::str::from_utf8(chunk).unwrap();
		out[i] =
			u8::from_str_radix(chunk, 16).map_err(|_| Error::Malformed("keyfile hex digit"))?;
	}
	Ok(out)
}

fn write_key(path: &Path, key: &[u8; 32]) -> Result<()> {
	Ok(std::fs::write(path, to_hex(key) + "\n")?)
}

fn read_key(path: &Path) -> Result<[u8; 32]> {
	parse_hex32(&std::fs::read_to_string(path)?)
}

/// Read the scope keyfile at `path`, creating it with a fresh random
/// key first if it does not exist.
pub fn load_or_create_scope_key(path: &Path) -> Result<[u8; 32]> {
	if path.exists() {
		read_key(path)
	} else {
		let key = crypto::random_key();
		write_key(path, &key)?;
		Ok(key)
	}
}

pub fn write_signing_key(path: &Path, key: &SigningKey) -> Result<()> {
	write_key(path, &key.to_bytes())
}

pub fn load_signing_key(path: &Path) -> Result<SigningKey> {
	Ok(SigningKey::from_bytes(&read_key(path)?))
}
