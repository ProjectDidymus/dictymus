//! The `rerun-if-changed` list that decides when Cargo runs this script again.

pub fn track_packaging_inputs() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=build");
	println!("cargo:rerun-if-changed=Cargo.toml");
	println!("cargo:rerun-if-changed=../../assets/icon");
}
