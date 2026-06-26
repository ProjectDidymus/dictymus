#![cfg_attr(not(test), windows_subsystem = "windows")]
mod accessibility;
mod app;
mod article_pane;
mod dialogs;
mod fonts;
mod lemma_list;
mod logging;
mod menu;
mod search_field;
mod tabs;

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
		Box::leak(Box::new(app));
	}) {
		tracing::error!("wxdragon init failed: {e}");
		eprintln!("dictymus: {e}"); // echo: visible if launched from a console
		std::process::exit(1);
	}
}
