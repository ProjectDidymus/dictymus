//! The commit hash and dev/release flag baked into the binary for the updater.

use shipfitter::build::CommitInfo;

/// Sets `DICTYMUS_COMMIT_HASH`, `DICTYMUS_SHORT_HASH` and `DICTYMUS_IS_DEV` (`1` unless HEAD
/// sits exactly on a tag).
pub fn embed_commit_info() -> CommitInfo {
	shipfitter::build::embed_commit_info("DICTYMUS")
}
