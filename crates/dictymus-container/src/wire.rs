//! Little-endian length-prefixed primitives shared by the container and
//! license formats.

use crate::{Error, Result};

pub struct Reader<'a> {
	buf: &'a [u8],
	pos: usize,
}

impl<'a> Reader<'a> {
	pub fn new(buf: &'a [u8]) -> Self {
		Self { buf, pos: 0 }
	}

	pub fn take(&mut self, n: usize, what: &'static str) -> Result<&'a [u8]> {
		let end = self.pos.checked_add(n).filter(|&e| e <= self.buf.len());
		let end = end.ok_or(Error::Malformed(what))?;
		let slice = &self.buf[self.pos..end];
		self.pos = end;
		Ok(slice)
	}

	pub fn u16(&mut self, what: &'static str) -> Result<u16> {
		Ok(u16::from_le_bytes(self.take(2, what)?.try_into().unwrap()))
	}

	pub fn u32(&mut self, what: &'static str) -> Result<u32> {
		Ok(u32::from_le_bytes(self.take(4, what)?.try_into().unwrap()))
	}

	pub fn u64(&mut self, what: &'static str) -> Result<u64> {
		Ok(u64::from_le_bytes(self.take(8, what)?.try_into().unwrap()))
	}

	/// u16 length-prefixed UTF-8 string.
	pub fn str16(&mut self, what: &'static str) -> Result<String> {
		let len = self.u16(what)? as usize;
		let bytes = self.take(len, what)?;
		String::from_utf8(bytes.to_vec()).map_err(|_| Error::Malformed(what))
	}

	/// u64 length-prefixed byte block.
	pub fn bytes64(&mut self, what: &'static str) -> Result<&'a [u8]> {
		let len = self.u64(what)?;
		let len = usize::try_from(len).map_err(|_| Error::Malformed(what))?;
		self.take(len, what)
	}

	pub fn remaining(&self) -> usize {
		self.buf.len() - self.pos
	}

	/// Bytes consumed so far, as a slice from the start of the buffer.
	pub fn consumed(&self) -> &'a [u8] {
		&self.buf[..self.pos]
	}
}

pub fn put_u16(out: &mut Vec<u8>, v: u16) {
	out.extend_from_slice(&v.to_le_bytes());
}

pub fn put_u32(out: &mut Vec<u8>, v: u32) {
	out.extend_from_slice(&v.to_le_bytes());
}

pub fn put_u64(out: &mut Vec<u8>, v: u64) {
	out.extend_from_slice(&v.to_le_bytes());
}

pub fn put_str16(out: &mut Vec<u8>, s: &str) {
	put_u16(out, s.len().try_into().expect("string too long for u16 length"));
	out.extend_from_slice(s.as_bytes());
}

pub fn put_bytes64(out: &mut Vec<u8>, bytes: &[u8]) {
	put_u64(out, bytes.len() as u64);
	out.extend_from_slice(bytes);
}
