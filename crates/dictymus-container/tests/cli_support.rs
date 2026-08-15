use dictymus_container::container::{collect_stardict_files, pack, seal};
use dictymus_container::keys::{load_or_create_scope_key, load_signing_key, write_signing_key};
use dictymus_container::license::issue;
use dictymus_container::{Error, inspect};
use ed25519_dalek::SigningKey;

#[test]
fn scope_keyfile_is_created_once_and_reused() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("suite.dek");
	let first = load_or_create_scope_key(&path).unwrap();
	let second = load_or_create_scope_key(&path).unwrap();
	assert_eq!(first, second);
	assert!(path.exists());
}

#[test]
fn signing_key_roundtrips_through_keyfile() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join("publisher.key");
	let key = SigningKey::from_bytes(&[7; 32]);
	write_signing_key(&path, &key).unwrap();
	let loaded = load_signing_key(&path).unwrap();
	assert_eq!(loaded.to_bytes(), key.to_bytes());
}

#[test]
fn collects_stardict_fileset_next_to_ifo() {
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(dir.path().join("bdag.ifo"), b"ifo").unwrap();
	std::fs::write(dir.path().join("bdag.idx"), b"idx").unwrap();
	std::fs::write(dir.path().join("bdag.dict.dz"), b"dz").unwrap();
	std::fs::write(dir.path().join("unrelated.txt"), b"no").unwrap();
	let files = collect_stardict_files(&dir.path().join("bdag.ifo")).unwrap();
	let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
	assert_eq!(names, ["bdag.ifo", "bdag.idx", "bdag.dict.dz"]);
}

#[test]
fn collecting_without_dict_data_fails() {
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(dir.path().join("bdag.ifo"), b"ifo").unwrap();
	std::fs::write(dir.path().join("bdag.idx"), b"idx").unwrap();
	assert!(matches!(collect_stardict_files(&dir.path().join("bdag.ifo")), Err(Error::Io(_))));
}

#[test]
fn bookname_is_read_from_ifo_contents() {
	let ifo = b"StarDict's dict ifo file\nversion=3.0.0\nbookname=BDAG Greek-English Lexicon\nwordcount=8110\n";
	assert_eq!(
		dictymus_container::container::ifo_bookname(ifo),
		Some("BDAG Greek-English Lexicon".to_string())
	);
	assert_eq!(dictymus_container::container::ifo_bookname(b"version=3.0.0\n"), None);
}

#[test]
fn inspect_describes_containers_and_licenses() {
	let files = vec![("a.ifo".to_string(), b"x".to_vec())];
	let unsealed = pack("harting", "Harting", &files);
	let text = inspect(&unsealed).unwrap();
	assert!(text.contains("harting"), "{text}");
	assert!(text.contains("unsealed"), "{text}");

	let sealed = seal("bdag", "BDAG", &files, &[("suite".into(), [2; 32])]);
	let text = inspect(&sealed).unwrap();
	assert!(text.contains("sealed"), "{text}");
	assert!(text.contains("suite"), "{text}");

	let license = issue(
		"Jane",
		"2026-08-15",
		&[("suite".into(), [2; 32])],
		&SigningKey::from_bytes(&[7; 32]),
	);
	let text = inspect(&license).unwrap();
	assert!(text.contains("Jane"), "{text}");
	assert!(text.contains("suite"), "{text}");
	assert!(text.contains("2026-08-15"), "{text}");
}
