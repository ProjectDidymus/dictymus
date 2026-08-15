use crate::crypto::{self, KEY_LEN, NONCE_LEN, WRAPPED_KEY_LEN};
use crate::wire::{Reader, put_str16, put_u16};
use crate::{Error, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

const MAGIC: &[u8; 8] = b"DICTYKEY";
const VERSION: u16 = 1;

struct Grant {
	scope_id: String,
	nonce: [u8; NONCE_LEN],
	wrapped_key: Vec<u8>,
}

/// A parsed and signature-verified `.dictykey` license.
pub struct License {
	licensee: String,
	issued: String,
	grants: Vec<Grant>,
}

/// Issue a signed license granting `grants` (scope id → scope key) to
/// `licensee`.
pub fn issue(
	licensee: &str,
	issued: &str,
	grants: &[(String, [u8; 32])],
	signing: &SigningKey,
) -> Vec<u8> {
	let mut out = Vec::new();
	out.extend_from_slice(MAGIC);
	put_u16(&mut out, VERSION);
	put_str16(&mut out, licensee);
	put_str16(&mut out, issued);
	put_u16(&mut out, grants.len().try_into().expect("too many grants"));
	for (scope_id, scope_key) in grants {
		put_str16(&mut out, scope_id);
		let kek = crypto::grant_kek(scope_id, licensee);
		let (nonce, wrapped) = crypto::wrap_key(&kek, scope_key);
		out.extend_from_slice(&nonce);
		out.extend_from_slice(&wrapped);
	}
	let signature: Signature = signing.sign(&out);
	out.extend_from_slice(&signature.to_bytes());
	out
}

impl License {
	/// Parse `bytes` and verify the publisher signature.
	pub fn verify(bytes: &[u8], publisher: &VerifyingKey) -> Result<Self> {
		Self::parse(bytes, Some(publisher))
	}

	/// Parse without checking the signature — display purposes only,
	/// never for unlocking anything.
	pub(crate) fn parse_without_verifying(bytes: &[u8]) -> Result<Self> {
		Self::parse(bytes, None)
	}

	fn parse(bytes: &[u8], publisher: Option<&VerifyingKey>) -> Result<Self> {
		let mut r = Reader::new(bytes);
		if r.take(MAGIC.len(), "magic").map_err(|_| Error::NotALicense)? != MAGIC {
			return Err(Error::NotALicense);
		}
		let version = r.u16("version")?;
		if version != VERSION {
			return Err(Error::UnsupportedVersion(version));
		}
		let licensee = r.str16("licensee")?;
		let issued = r.str16("issue date")?;
		let count = r.u16("grant count")?;
		let mut grants = Vec::with_capacity(count as usize);
		for _ in 0..count {
			let scope_id = r.str16("grant scope")?;
			let nonce = r.take(NONCE_LEN, "grant nonce")?.try_into().unwrap();
			let wrapped_key = r.take(WRAPPED_KEY_LEN, "grant key")?.to_vec();
			grants.push(Grant { scope_id, nonce, wrapped_key });
		}
		let signed = r.consumed();
		let signature = r.take(64, "signature")?;
		if r.remaining() != 0 {
			return Err(Error::Malformed("trailing data after signature"));
		}
		if let Some(publisher) = publisher {
			let signature = Signature::from_bytes(signature.try_into().unwrap());
			publisher.verify(signed, &signature).map_err(|_| Error::LicenseInvalidSignature)?;
		}
		Ok(Self { licensee, issued, grants })
	}

	pub fn licensee(&self) -> &str {
		&self.licensee
	}

	pub fn issued(&self) -> &str {
		&self.issued
	}

	pub fn scope_ids(&self) -> Vec<&str> {
		self.grants.iter().map(|g| g.scope_id.as_str()).collect()
	}

	/// Unwrap the scope key of the grant for `scope_id`, if present.
	pub fn scope_key(&self, scope_id: &str) -> Option<Result<[u8; KEY_LEN]>> {
		let grant = self.grants.iter().find(|g| g.scope_id == scope_id)?;
		let kek = crypto::grant_kek(&grant.scope_id, &self.licensee);
		Some(crypto::unwrap_key(&kek, &grant.nonce, &grant.wrapped_key))
	}
}
