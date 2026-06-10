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
	}
}
