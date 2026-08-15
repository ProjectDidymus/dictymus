use std::{env, sync::Arc};

use dictymus_core::config::UpdateChannel;
use ship_shape::UpdaterConfig;
use wxdragon::prelude::*;

const GITHUB_REPO: &str = "ProjectDidymus/dictymus";
/// Base64 minisign public key; downloaded release assets are verified against
/// it before anything is executed.
const MINISIGN_PUBLIC_KEY: &str = "RWSIEq1WkvZZ4ZTn4dM16OvD6A/FjX9J0c5FTETolqvixOstZMQ3CK3e";
const COMMIT_HASH: &str = env!("DICTYMUS_COMMIT_HASH");

/// Suffix of the per-platform installer and portable zip assets published
/// by CI. macOS needs its own suffix: its arm64 zip would otherwise clash
/// with the Windows `dictymus-arm64.zip`.
#[cfg(all(windows, target_arch = "x86_64"))]
const ASSET_SUFFIX: &str = "-x64";
#[cfg(all(windows, target_arch = "aarch64"))]
const ASSET_SUFFIX: &str = "-arm64";
#[cfg(target_os = "macos")]
const ASSET_SUFFIX: &str = "-macos";

/// The channel this build tracks when the config does not pin one: release
/// builds (HEAD on a tag) follow stable, development builds follow dev.
pub fn default_channel() -> UpdateChannel {
	if env!("DICTYMUS_IS_DEV") == "true" { UpdateChannel::Dev } else { UpdateChannel::Stable }
}

fn user_agent() -> String {
	format!("dictymus/{}", env!("CARGO_PKG_VERSION"))
}

/// Installed copies have the Inno Setup uninstaller next to the exe; portable
/// unzips do not, and get an in-place zip swap instead of the installer.
fn is_installer_distribution() -> bool {
	env::current_exe()
		.ok()
		.and_then(|p| p.parent().map(|d| d.join("unins000.exe").exists()))
		.unwrap_or(false)
}

/// Spawn ship-shape's background update flow. Safe to call from both the
/// silent startup check and the Help menu; concurrent calls are a no-op.
pub fn run_update_check(frame: &Frame, channel: UpdateChannel, silent: bool) {
	tracing::info!(%channel, silent, "checking for updates");
	let config = Arc::new(
		UpdaterConfig::new(GITHUB_REPO, "dictymus", "Dictymus", MINISIGN_PUBLIC_KEY, user_agent())
			.with_asset_suffix(ASSET_SUFFIX),
	);
	let ship_channel = match channel {
		UpdateChannel::Stable => ship_shape::UpdateChannel::Stable,
		UpdateChannel::Dev => ship_shape::UpdateChannel::Dev,
	};
	ship_shape::ui::run_update_check(
		config,
		frame.handle_ptr() as usize,
		env!("CARGO_PKG_VERSION"),
		COMMIT_HASH,
		is_installer_distribution(),
		ship_channel,
		silent,
	);
}
