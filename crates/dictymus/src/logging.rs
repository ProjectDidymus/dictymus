//! Tracing subscriber setup.
//!
//! The GUI binary has no console (`windows_subsystem = "windows"`), so all
//! diagnostics go to a rotated log file in the app-data dir
//! (`%APPDATA%\dictymus\logs\` on Windows). Level defaults to the config
//! `log_level` (`warn`) but `RUST_LOG` overrides it for remote diagnosis.

use dictymus_core::config::AppConfig;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::EnvFilter;

/// Install the global subscriber and a panic hook. Returns the appender's
/// `WorkerGuard`; the caller MUST keep it alive for the whole program so
/// buffered log lines are flushed on exit. `None` means logging could not be
/// set up (no app-data dir, or the file appender failed); the app still runs.
pub fn init(config_level: &str) -> Option<WorkerGuard> {
	let dir = AppConfig::log_dir()?;
	if let Err(e) = std::fs::create_dir_all(&dir) {
		eprintln!("dictymus: cannot create log dir {}: {e}", dir.display());
		return None;
	}

	// Daily rotation, keep the last 7 days.
	let appender = match Builder::new()
		.rotation(Rotation::DAILY)
		.filename_prefix("dictymus")
		.filename_suffix("log")
		.max_log_files(7)
		.build(&dir)
	{
		Ok(a) => a,
		Err(e) => {
			eprintln!("dictymus: cannot create log appender: {e}");
			return None;
		}
	};
	let (writer, guard) = tracing_appender::non_blocking(appender);

	// RUST_LOG wins; otherwise apply the config level to our own crates.
	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
		EnvFilter::new(format!("dictymus={lvl},dictymus_core={lvl}", lvl = config_level))
	});

	tracing_subscriber::fmt()
		.compact()
		.with_ansi(false)
		.with_writer(writer)
		.with_env_filter(filter)
		.init();

	// Capture panics. Release builds use `panic = "abort"`, but the hook still
	// runs before the abort, so the backtrace lands in the log.
	let default_hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |info| {
		tracing::error!("panic: {info}");
		default_hook(info);
	}));

	Some(guard)
}
