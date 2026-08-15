use dictymus_container::Error;
use dictymus_container::container::{Container, seal};
use dictymus_container::license::{License, issue};
use ed25519_dalek::SigningKey;

fn sample_files() -> Vec<(String, Vec<u8>)> {
	vec![("dict.ifo".into(), b"version=3.0.0\n".to_vec()), ("dict.idx".into(), vec![1, 2, 3])]
}

const BDAG_KEY: [u8; 32] = [1; 32];
const SUITE_KEY: [u8; 32] = [2; 32];

fn signing_key() -> SigningKey {
	SigningKey::from_bytes(&[7; 32])
}

fn suite_license() -> Vec<u8> {
	issue(
		"Jane Doe <jane@example.com>",
		"2026-08-15",
		&[("suite".into(), SUITE_KEY)],
		&signing_key(),
	)
}

#[test]
fn future_dictionary_opens_with_existing_license() {
	// License issued first; a container sealed later with the same
	// scope key must open without touching the license.
	let license_bytes = suite_license();
	let files = sample_files();
	let container_bytes =
		seal("muraoka", "A Later Lexicon", &files, &[("suite".into(), SUITE_KEY)]);
	let container = Container::parse(&container_bytes).unwrap();
	let license = License::verify(&license_bytes, &signing_key().verifying_key()).unwrap();
	assert_eq!(container.open_sealed(&license).unwrap(), files);
}

#[test]
fn multi_slot_container_opens_via_any_matching_grant() {
	let files = sample_files();
	let bytes =
		seal("bdag", "BDAG", &files, &[("bdag".into(), BDAG_KEY), ("suite".into(), SUITE_KEY)]);
	let container = Container::parse(&bytes).unwrap();
	// The license grants only "suite", not the container's first slot.
	let license = License::verify(&suite_license(), &signing_key().verifying_key()).unwrap();
	assert_eq!(container.open_sealed(&license).unwrap(), files);
}

#[test]
fn license_without_matching_grant_is_rejected() {
	let bytes = seal("bdag", "BDAG", &sample_files(), &[("bdag".into(), BDAG_KEY)]);
	let container = Container::parse(&bytes).unwrap();
	let license = License::verify(&suite_license(), &signing_key().verifying_key()).unwrap();
	assert!(matches!(container.open_sealed(&license), Err(Error::NoMatchingGrant)));
}

#[test]
fn tampered_license_fails_signature_check() {
	let mut bytes = suite_license();
	// Flip a byte inside the licensee name.
	let pos = bytes.windows(4).position(|w| w == b"Jane").unwrap();
	bytes[pos] ^= 0x01;
	assert!(matches!(
		License::verify(&bytes, &signing_key().verifying_key()),
		Err(Error::LicenseInvalidSignature)
	));
}

#[test]
fn license_from_other_publisher_is_rejected() {
	let other = SigningKey::from_bytes(&[9; 32]);
	assert!(matches!(
		License::verify(&suite_license(), &other.verifying_key()),
		Err(Error::LicenseInvalidSignature)
	));
}

#[test]
fn tampered_container_header_fails_decryption() {
	let mut bytes =
		seal("bdag", "BDAG Greek-English", &sample_files(), &[("bdag".into(), BDAG_KEY)]);
	// Flip a byte of the display name: parsing still succeeds, but the
	// header is bound as AAD so decryption must fail.
	let pos = bytes.windows(5).position(|w| w == b"Greek").unwrap();
	bytes[pos] ^= 0x01;
	let container = Container::parse(&bytes).unwrap();
	let license = issue("Jane", "2026-08-15", &[("bdag".into(), BDAG_KEY)], &signing_key());
	let license = License::verify(&license, &signing_key().verifying_key()).unwrap();
	assert!(matches!(container.open_sealed(&license), Err(Error::DecryptFailed)));
}

#[test]
fn tampered_payload_fails_decryption() {
	let mut bytes = seal("bdag", "BDAG", &sample_files(), &[("bdag".into(), BDAG_KEY)]);
	*bytes.last_mut().unwrap() ^= 0x01;
	let container = Container::parse(&bytes).unwrap();
	let license = issue("Jane", "2026-08-15", &[("bdag".into(), BDAG_KEY)], &signing_key());
	let license = License::verify(&license, &signing_key().verifying_key()).unwrap();
	assert!(matches!(container.open_sealed(&license), Err(Error::DecryptFailed)));
}

#[test]
fn garbage_is_neither_container_nor_license() {
	assert!(matches!(Container::parse(b"hello world"), Err(Error::NotAContainer)));
	assert!(matches!(
		License::verify(b"hello world", &signing_key().verifying_key()),
		Err(Error::NotALicense)
	));
}

#[test]
fn future_container_version_is_rejected() {
	let mut bytes = seal("bdag", "BDAG", &sample_files(), &[("bdag".into(), BDAG_KEY)]);
	// The version field sits right after the 8-byte magic.
	bytes[8..10].copy_from_slice(&99u16.to_le_bytes());
	assert!(matches!(Container::parse(&bytes), Err(Error::UnsupportedVersion(99))));
}
