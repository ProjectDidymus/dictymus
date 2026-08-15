//! The `.dicty` dictionary container and `.dictykey` license formats.
//!
//! A container wraps a StarDict fileset either unsealed (plaintext) or
//! sealed (XChaCha20-Poly1305 under a per-container content key). Sealed
//! containers carry key slots naming the license scopes that can unlock
//! them; a license grants scope keys to a named licensee and is Ed25519
//! signed by the publisher.

mod crypto;
mod error;
mod wire;

pub mod container;
pub mod keys;
pub mod license;

pub use ed25519_dalek::{SigningKey, VerifyingKey};
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Human-readable summary of a container or license file's public
/// fields, for the CLI `inspect` command. The license signature is NOT
/// verified here.
pub fn inspect(bytes: &[u8]) -> Result<String> {
	match container::Container::parse(bytes) {
		Ok(c) => {
			let sealing = if c.is_sealed() {
				format!("sealed (scopes: {})", c.slot_scopes().join(", "))
			} else {
				"unsealed".to_string()
			};
			Ok(format!(
				"Dictymus dictionary container\n  id:      {}\n  name:    {}\n  sealing: {sealing}\n",
				c.dict_id(),
				c.name(),
			))
		}
		Err(Error::NotAContainer) => {
			let l = license::License::parse_without_verifying(bytes)?;
			Ok(format!(
				"Dictymus license (signature not checked)\n  licensee: {}\n  issued:   {}\n  scopes:   {}\n",
				l.licensee(),
				l.issued(),
				l.scope_ids().join(", "),
			))
		}
		Err(e) => Err(e),
	}
}
