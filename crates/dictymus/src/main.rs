#![cfg_attr(not(test), windows_subsystem = "windows")]
mod accessibility;
mod app;
mod article_pane;
mod dialogs;
mod fonts;
mod lemma_list;
mod menu;
mod search_field;
mod tabs;

fn main() {
	// If wx itself failed to init there is no toolkit to show a dialog with;
	// the nonzero exit code is what scripts and smoke tests need.
	if let Err(e) = wxdragon::main(|_| {
		let app = app::App::new();
		app.show();
		Box::leak(Box::new(app));
	}) {
		eprintln!("dictymus: {e}");
		std::process::exit(1);
	}
}
