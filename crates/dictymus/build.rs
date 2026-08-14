fn main() {
	embed_commit_info();
	build_translations();
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

/// Regenerates `po/dictymus.pot` from every crate tagged
/// `[package.metadata.patois] translatable = true` (including registry
/// dependencies such as `ship-shape`), then compiles each `po/*.po` into
/// `locale/<lang>/LC_MESSAGES/dictymus.mo` for `patois::embed_domain!` to
/// pick up. Needs `xgettext`/`msgfmt` on `PATH`; degrades to warnings and an
/// untranslated binary without them.
fn build_translations() {
	let manifest_dir =
		std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
	if let Some(workspace_dir) = manifest_dir.ancestors().nth(2) {
		let po_dir = workspace_dir.join("po");
		if let Err(e) = patois_build::gen_pot(workspace_dir, &po_dir, "dictymus") {
			println!("cargo:warning=Failed to regenerate dictymus.pot from Rust sources: {e}");
		}
	}
	patois_build::compile_translations("../../po", "locale");
}

/// Embeds `DICTYMUS_COMMIT_HASH` (HEAD hash, or `unknown` outside a git
/// checkout) and `DICTYMUS_IS_DEV` (`false` only when HEAD sits exactly on a
/// `v*` tag, i.e. a release build).
fn embed_commit_info() {
	let hash = std::process::Command::new("git")
		.args(["rev-parse", "HEAD"])
		.output()
		.ok()
		.filter(|o| o.status.success())
		.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
		.unwrap_or_else(|| "unknown".to_string());
	let on_tag = std::process::Command::new("git")
		.args(["describe", "--tags", "--match", "v*", "--exact-match", "HEAD"])
		.output()
		.is_ok_and(|o| o.status.success());
	println!("cargo:rustc-env=DICTYMUS_COMMIT_HASH={hash}");
	println!("cargo:rustc-env=DICTYMUS_IS_DEV={}", !on_tag);
	println!("cargo:rerun-if-changed=../../.git/HEAD");
}
