#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	NotAContainer,
	UnsupportedVersion(u16),
	Malformed(&'static str),
	LicenseMissing,
	NotALicense,
	LicenseInvalidSignature,
	NoMatchingGrant,
	DecryptFailed,
	Io(std::io::Error),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::NotAContainer => write!(f, "not a Dictymus dictionary container"),
			Error::UnsupportedVersion(v) => write!(f, "unsupported container version {v}"),
			Error::Malformed(what) => write!(f, "malformed data: {what}"),
			Error::LicenseMissing => write!(f, "a license is required to open this dictionary"),
			Error::NotALicense => write!(f, "not a Dictymus license file"),
			Error::LicenseInvalidSignature => write!(f, "license signature is invalid"),
			Error::NoMatchingGrant => write!(f, "license does not cover this dictionary"),
			Error::DecryptFailed => write!(f, "decryption failed"),
			Error::Io(e) => write!(f, "I/O error: {e}"),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::Io(e) => Some(e),
			_ => None,
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self {
		Error::Io(e)
	}
}
