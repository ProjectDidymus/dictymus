use dictymus_container::container::{Container, pack};

fn sample_files() -> Vec<(String, Vec<u8>)> {
	vec![
		("dict.ifo".into(), b"StarDict's dict ifo file\nversion=3.0.0\n".to_vec()),
		("dict.idx".into(), vec![0, 1, 2, 255, 254]),
		("dict.dict.dz".into(), vec![0x1f, 0x8b, 0x08, 0x00]),
	]
}

#[test]
fn unsealed_container_roundtrips() {
	let files = sample_files();
	let bytes = pack("harting", "Harting Greek-Dutch", &files);
	let container = Container::parse(&bytes).unwrap();
	assert_eq!(container.dict_id(), "harting");
	assert_eq!(container.name(), "Harting Greek-Dutch");
	assert!(!container.is_sealed());
	assert_eq!(container.open_unsealed().unwrap(), files);
}
