use std::{env, sync::Arc};

use dictymus_core::config::UpdateChannel;
use ship_shape::{InstallKind, UpdaterConfig, ui::CheckTrigger};
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
	let install_kind =
		if is_installer_distribution() { InstallKind::Installer } else { InstallKind::Portable };
	let config = Arc::new(
		UpdaterConfig::new(
			GITHUB_REPO,
			"dictymus",
			"Dictymus",
			MINISIGN_PUBLIC_KEY,
			env!("CARGO_PKG_VERSION"),
		)
		.with_commit(COMMIT_HASH)
		.with_install_kind(install_kind)
		.with_asset_suffix(ASSET_SUFFIX),
	);
	let ship_channel = match channel {
		UpdateChannel::Stable => ship_shape::UpdateChannel::Stable,
		UpdateChannel::Dev => ship_shape::UpdateChannel::Dev,
	};
	let trigger = if silent { CheckTrigger::Automatic } else { CheckTrigger::Manual };
	ship_shape::ui::run_update_check(config, frame, ship_channel, trigger);
}
