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
	let _ = wxdragon::main(|_| {
		let app = app::App::new();
		app.show();
		Box::leak(Box::new(app));
	});
}
