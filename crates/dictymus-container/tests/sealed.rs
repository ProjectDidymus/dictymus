use dictymus_container::Error;
use dictymus_container::container::{Container, seal};
use dictymus_container::license::{License, issue};
use ed25519_dalek::SigningKey;

fn sample_files() -> Vec<(String, Vec<u8>)> {
	vec![
		("dict.ifo".into(), b"StarDict's dict ifo file\nversion=3.0.0\n".to_vec()),
		("dict.idx".into(), vec![0, 1, 2, 255, 254]),
		("dict.dict.dz".into(), vec![0x1f, 0x8b, 0x08, 0x00]),
	]
}

const SCOPE_KEY: [u8; 32] = [42; 32];
const LICENSEE: &str = "Jane Doe <jane@example.com>";

fn signing_key() -> SigningKey {
	SigningKey::from_bytes(&[7; 32])
}

#[test]
fn sealed_container_opens_with_matching_license() {
	let files = sample_files();
	let bytes = seal("bdag", "BDAG Greek-English Lexicon", &files, &[("bdag".into(), SCOPE_KEY)]);
	let container = Container::parse(&bytes).unwrap();
	assert_eq!(container.dict_id(), "bdag");
	assert_eq!(container.name(), "BDAG Greek-English Lexicon");
	assert!(container.is_sealed());
	assert_eq!(container.slot_scopes(), ["bdag"]);

	let signing = signing_key();
	let license_bytes = issue(LICENSEE, "2026-08-15", &[("bdag".into(), SCOPE_KEY)], &signing);
	let license = License::verify(&license_bytes, &signing.verifying_key()).unwrap();
	assert_eq!(license.licensee(), LICENSEE);
	assert_eq!(license.issued(), "2026-08-15");

	assert_eq!(container.open_sealed(&license).unwrap(), files);
}

#[test]
fn sealed_container_refuses_open_unsealed() {
	let bytes = seal("bdag", "BDAG", &sample_files(), &[("bdag".into(), SCOPE_KEY)]);
	let container = Container::parse(&bytes).unwrap();
	assert!(matches!(container.open_unsealed(), Err(Error::LicenseMissing)));
}
