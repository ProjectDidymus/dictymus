use crate::crypto::{self, NONCE_LEN, WRAPPED_KEY_LEN};
use crate::license::License;
use crate::wire::{Reader, put_bytes64, put_str16, put_u16, put_u32};
use crate::{Error, Result};

const MAGIC: &[u8; 8] = b"DICTYMUS";
const VERSION: u16 = 1;
const FLAG_SEALED: u16 = 1;

/// A key slot: the container's reference to a license scope that can
/// unlock it, carrying the content key wrapped under that scope's key.
struct Slot {
	scope_id: String,
	nonce: [u8; NONCE_LEN],
	wrapped_dek: Vec<u8>,
}

/// A parsed `.dicty` container. Header fields are readable without any
/// key; the payload needs [`Container::open_unsealed`] or a license.
pub struct Container {
	dict_id: String,
	name: String,
	sealed: bool,
	slots: Vec<Slot>,
	payload_nonce: [u8; NONCE_LEN],
	/// Header bytes bound as AAD, so header tampering fails decryption.
	aad: Vec<u8>,
	payload: Vec<u8>,
}

fn pack_files(files: &[(String, Vec<u8>)]) -> Vec<u8> {
	let mut out = Vec::new();
	put_u32(&mut out, files.len().try_into().expect("too many files"));
	for (name, data) in files {
		put_str16(&mut out, name);
		put_bytes64(&mut out, data);
	}
	out
}

fn unpack_files(payload: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
	let mut r = Reader::new(payload);
	let count = r.u32("file count")?;
	let mut files = Vec::with_capacity(count as usize);
	for _ in 0..count {
		let name = r.str16("file name")?;
		let data = r.bytes64("file data")?.to_vec();
		files.push((name, data));
	}
	if r.remaining() != 0 {
		return Err(Error::Malformed("trailing data after fileset"));
	}
	Ok(files)
}

fn write_header(dict_id: &str, name: &str, flags: u16) -> Vec<u8> {
	let mut out = Vec::new();
	out.extend_from_slice(MAGIC);
	put_u16(&mut out, VERSION);
	put_u16(&mut out, flags);
	put_str16(&mut out, dict_id);
	put_str16(&mut out, name);
	out
}

/// Gather the StarDict fileset next to `ifo_path` (`.ifo`, `.idx` or
/// `.idx.gz`, `.dict.dz` or `.dict`, optional `.syn`) as name → bytes.
pub fn collect_stardict_files(ifo_path: &std::path::Path) -> Result<Vec<(String, Vec<u8>)>> {
	let dir = ifo_path.parent().unwrap_or(std::path::Path::new("."));
	let stem = ifo_path
		.file_stem()
		.and_then(|s| s.to_str())
		.ok_or(Error::Malformed("ifo path has no file stem"))?;
	let read = |name: String| -> Result<(String, Vec<u8>)> {
		let data = std::fs::read(dir.join(&name))?;
		Ok((name, data))
	};
	let read_first = |exts: &[&str]| -> Result<(String, Vec<u8>)> {
		let found = exts.iter().map(|e| format!("{stem}.{e}")).find(|n| dir.join(n).exists());
		match found {
			Some(name) => read(name),
			None => Err(std::io::Error::new(
				std::io::ErrorKind::NotFound,
				format!("no {stem}.{} next to the .ifo", exts.join(&format!(" or {stem}."))),
			)
			.into()),
		}
	};
	let mut files = vec![
		read(format!("{stem}.ifo"))?,
		read_first(&["idx", "idx.gz"])?,
		read_first(&["dict.dz", "dict"])?,
	];
	if dir.join(format!("{stem}.syn")).exists() {
		files.push(read(format!("{stem}.syn"))?);
	}
	Ok(files)
}

/// The `bookname=` value of a StarDict `.ifo` file, used as the
/// default display name when packing.
pub fn ifo_bookname(ifo: &[u8]) -> Option<String> {
	let text = String::from_utf8_lossy(ifo);
	let value = text.lines().find_map(|l| l.strip_prefix("bookname="))?;
	Some(value.trim().to_string())
}

/// Build an unsealed container holding `files` (name → contents).
pub fn pack(dict_id: &str, name: &str, files: &[(String, Vec<u8>)]) -> Vec<u8> {
	let mut out = write_header(dict_id, name, 0);
	put_bytes64(&mut out, &pack_files(files));
	out
}

/// Build a sealed container: the fileset is encrypted under a fresh
/// content key, which is wrapped once per scope in `scopes`.
pub fn seal(
	dict_id: &str,
	name: &str,
	files: &[(String, Vec<u8>)],
	scopes: &[(String, [u8; 32])],
) -> Vec<u8> {
	let dek = crypto::random_key();
	let mut out = write_header(dict_id, name, FLAG_SEALED);
	put_u16(&mut out, scopes.len().try_into().expect("too many scopes"));
	for (scope_id, scope_key) in scopes {
		put_str16(&mut out, scope_id);
		let (nonce, wrapped) = crypto::wrap_key(scope_key, &dek);
		out.extend_from_slice(&nonce);
		out.extend_from_slice(&wrapped);
	}
	let payload_nonce = crypto::random_nonce();
	out.extend_from_slice(&payload_nonce);
	let ciphertext = crypto::encrypt(&dek, &payload_nonce, &out, &pack_files(files));
	put_bytes64(&mut out, &ciphertext);
	out
}

impl Container {
	pub fn parse(bytes: &[u8]) -> Result<Self> {
		let mut r = Reader::new(bytes);
		if r.take(MAGIC.len(), "magic").map_err(|_| Error::NotAContainer)? != MAGIC {
			return Err(Error::NotAContainer);
		}
		let version = r.u16("version")?;
		if version != VERSION {
			return Err(Error::UnsupportedVersion(version));
		}
		let flags = r.u16("flags")?;
		let sealed = flags & FLAG_SEALED != 0;
		let dict_id = r.str16("dict id")?;
		let name = r.str16("name")?;
		let mut slots = Vec::new();
		let mut payload_nonce = [0; NONCE_LEN];
		if sealed {
			let count = r.u16("slot count")?;
			for _ in 0..count {
				let scope_id = r.str16("slot scope")?;
				let nonce = r.take(NONCE_LEN, "slot nonce")?.try_into().unwrap();
				let wrapped_dek = r.take(WRAPPED_KEY_LEN, "slot key")?.to_vec();
				slots.push(Slot { scope_id, nonce, wrapped_dek });
			}
			payload_nonce = r.take(NONCE_LEN, "payload nonce")?.try_into().unwrap();
		}
		let aad = r.consumed().to_vec();
		let payload = r.bytes64("payload")?.to_vec();
		if r.remaining() != 0 {
			return Err(Error::Malformed("trailing data after payload"));
		}
		Ok(Self { dict_id, name, sealed, slots, payload_nonce, aad, payload })
	}

	pub fn dict_id(&self) -> &str {
		&self.dict_id
	}

	/// Display name of the dictionary, readable without a license.
	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn is_sealed(&self) -> bool {
		self.sealed
	}

	/// Unpack the fileset of an unsealed container.
	pub fn open_unsealed(&self) -> Result<Vec<(String, Vec<u8>)>> {
		if self.sealed {
			return Err(Error::LicenseMissing);
		}
		unpack_files(&self.payload)
	}

	/// Scope ids of the key slots on a sealed container.
	pub fn slot_scopes(&self) -> Vec<&str> {
		self.slots.iter().map(|s| s.scope_id.as_str()).collect()
	}

	/// Decrypt and unpack the fileset of a sealed container using a
	/// grant from `license`.
	pub fn open_sealed(&self, license: &License) -> Result<Vec<(String, Vec<u8>)>> {
		if !self.sealed {
			return self.open_unsealed();
		}
		let (slot, scope_key) = self
			.slots
			.iter()
			.find_map(|slot| Some((slot, license.scope_key(&slot.scope_id)?)))
			.ok_or(Error::NoMatchingGrant)?;
		let dek = crypto::unwrap_key(&scope_key?, &slot.nonce, &slot.wrapped_dek)?;
		let plaintext = crypto::decrypt(&dek, &self.payload_nonce, &self.aad, &self.payload)?;
		unpack_files(&plaintext)
	}
}
