//! XChaCha20-Poly1305 + BLAKE3 primitives shared by container sealing
//! and license key wrapping.

use crate::{Error, Result};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore;
use rand::rngs::OsRng;

pub const NONCE_LEN: usize = 24;
pub const KEY_LEN: usize = 32;
/// A wrapped 32-byte key: ciphertext + Poly1305 tag.
pub const WRAPPED_KEY_LEN: usize = KEY_LEN + 16;

const LICENSE_KDF_CONTEXT: &str = "dictymus license v1";

pub fn random_key() -> [u8; KEY_LEN] {
	let mut key = [0; KEY_LEN];
	OsRng.fill_bytes(&mut key);
	key
}

pub fn random_nonce() -> [u8; NONCE_LEN] {
	let mut nonce = [0; NONCE_LEN];
	OsRng.fill_bytes(&mut nonce);
	nonce
}

/// KEK binding a grant's scope key to the licensee identity: altering
/// either breaks the unwrap independently of the license signature.
pub fn grant_kek(scope_id: &str, licensee: &str) -> [u8; KEY_LEN] {
	let mut material = Vec::new();
	crate::wire::put_str16(&mut material, scope_id);
	crate::wire::put_str16(&mut material, licensee);
	blake3::derive_key(LICENSE_KDF_CONTEXT, &material)
}

pub fn encrypt(key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN], aad: &[u8], msg: &[u8]) -> Vec<u8> {
	let cipher = XChaCha20Poly1305::new(key.into());
	cipher
		.encrypt(XNonce::from_slice(nonce), Payload { msg, aad })
		.expect("XChaCha20-Poly1305 encryption is infallible")
}

pub fn decrypt(
	key: &[u8; KEY_LEN],
	nonce: &[u8; NONCE_LEN],
	aad: &[u8],
	ciphertext: &[u8],
) -> Result<Vec<u8>> {
	let cipher = XChaCha20Poly1305::new(key.into());
	cipher
		.decrypt(XNonce::from_slice(nonce), Payload { msg: ciphertext, aad })
		.map_err(|_| Error::DecryptFailed)
}

/// Encrypt a 32-byte key under `kek`; returns (nonce, wrapped bytes).
pub fn wrap_key(kek: &[u8; KEY_LEN], key: &[u8; KEY_LEN]) -> ([u8; NONCE_LEN], Vec<u8>) {
	let nonce = random_nonce();
	let wrapped = encrypt(kek, &nonce, &[], key);
	(nonce, wrapped)
}

pub fn unwrap_key(
	kek: &[u8; KEY_LEN],
	nonce: &[u8; NONCE_LEN],
	wrapped: &[u8],
) -> Result<[u8; KEY_LEN]> {
	let key = decrypt(kek, nonce, &[], wrapped)?;
	key.try_into().map_err(|_| Error::DecryptFailed)
}
