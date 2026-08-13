fn main() {
	#[cfg(target_os = "windows")]
	{
		use embed_manifest::{
			embed_manifest,
			manifest::{ActiveCodePage, DpiAwareness, SupportedOS::*},
			new_manifest,
		};
		let manifest = new_manifest("Dictymus")
			.supported_os(Windows7..=Windows10)
			.active_code_page(ActiveCodePage::Utf8)
			.dpi_awareness(DpiAwareness::PerMonitorV2);
		embed_manifest(manifest).expect("unable to embed manifest");

		// Icon only; never call set_manifest* here.
		let mut res = winresource::WindowsResource::new();
		res.set_icon("../../assets/icon/dictymus.ico");
		res.compile().expect("unable to embed icon resource");
		println!("cargo:rerun-if-changed=../../assets/icon/dictymus.ico");
	}
}
