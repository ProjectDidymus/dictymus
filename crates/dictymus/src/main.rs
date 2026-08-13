#![cfg_attr(not(test), windows_subsystem = "windows")]
mod accessibility;
mod app;
mod article_pane;
mod dialogs;
mod fonts;
mod ipc;
mod lemma_list;
mod logging;
mod menu;
mod search_field;
mod tabs;
#[cfg(windows)]
mod update;

fn main() {
	// Load config first so logging can honour its level, then init logging
	// before any UI work. The guard stays in this frame and outlives the
	// leaked App, so buffered log lines flush on exit.
	let (config, config_warning) = dictymus_core::config::AppConfig::load();
	let _guard = logging::init(&config.log_level);
	tracing::info!("dictymus starting");

	// If wx itself failed to init there is no toolkit to show a dialog with;
	// the nonzero exit code is what scripts and smoke tests need.
	if let Err(e) = wxdragon::main(move |_| {
		let app = app::App::new(config.clone(), config_warning.clone());
		app.show();
		let app: &'static app::App = Box::leak(Box::new(app));
		app::store_app(app);
		if !ipc::start_server() {
			dialogs::show_error(
				&app.frame,
				"Could not start the single-instance service. Opening dictionary files from Explorer will start a separate window.",
			);
		}
		#[cfg(windows)]
		if config.check_for_updates_on_startup
			&& std::env::var_os("DICTYMUS_NO_UPDATE_CHECK").is_none()
		{
			let channel = config.effective_update_channel(update::default_channel());
			update::run_update_check(&app.frame, channel, true);
		}
	}) {
		tracing::error!("wxdragon init failed: {e}");
		eprintln!("dictymus: {e}"); // echo: visible if launched from a console
		std::process::exit(1);
	}
}
