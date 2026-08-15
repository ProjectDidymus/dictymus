use patois::t;
use wxdragon::prelude::*;

pub mod ids {
	pub const OPEN: i32 = 5001;
	pub const CLOSE: i32 = 5002;
	pub const CLOSE_ALL: i32 = 5003;
	pub const EXIT: i32 = 5004;
	pub const ABOUT: i32 = 5005;
	#[cfg(windows)]
	pub const CHECK_UPDATES: i32 = 5006;
	pub const OPTIONS: i32 = 5007;
	pub const LICENSES: i32 = 5008;
}

pub fn create_menu_bar() -> MenuBar {
	// TRANSLATORS: File menu item; the text after the tab is the keyboard shortcut
	let open_label = t("&Open...\tCtrl+O");
	// TRANSLATORS: Status bar help for the Open menu item
	let open_help = t("Open a dictionary");
	// TRANSLATORS: File menu item; the text after the tab is the keyboard shortcut
	let close_label = t("&Close\tCtrl+F4");
	// TRANSLATORS: Status bar help for the Close menu item
	let close_help = t("Close current dictionary");
	// TRANSLATORS: File menu item
	let close_all_label = t("Close &All");
	// TRANSLATORS: Status bar help for the Close All menu item
	let close_all_help = t("Close all dictionaries");
	// TRANSLATORS: File menu item opening the Licenses dialog; the text after the tab is the keyboard shortcut
	let licenses_label = t("&Licenses...\tCtrl+L");
	// TRANSLATORS: Status bar help for the Licenses menu item
	let licenses_help = t("View, import, or remove dictionary licenses");
	// TRANSLATORS: File menu item opening the Options dialog
	let options_label = t("&Options...");
	// TRANSLATORS: Status bar help for the Options menu item
	let options_help = t("Change settings");
	// TRANSLATORS: File menu item; the text after the tab is the keyboard shortcut
	let exit_label = t("E&xit\tCtrl+Q");
	// TRANSLATORS: Status bar help for the Exit menu item
	let exit_help = t("Exit");
	let file = Menu::builder()
		.append_item(ids::OPEN, &open_label, &open_help)
		.append_item(ids::CLOSE, &close_label, &close_help)
		.append_item(ids::CLOSE_ALL, &close_all_label, &close_all_help)
		.append_separator()
		.append_item(ids::LICENSES, &licenses_label, &licenses_help)
		.append_item(ids::OPTIONS, &options_label, &options_help)
		.append_separator()
		.append_item(ids::EXIT, &exit_label, &exit_help)
		.build();

	let help = Menu::builder();
	#[cfg(any(windows, target_os = "macos"))]
	let help = {
		// TRANSLATORS: Help menu item
		let check_updates_label = t("Check for &Updates...");
		// TRANSLATORS: Status bar help for the Check for Updates menu item
		let check_updates_help = t("Check for a new version of Dictymus");
		help.append_item(ids::CHECK_UPDATES, &check_updates_label, &check_updates_help)
	};
	// TRANSLATORS: Help menu item
	let about_label = t("&About Dictymus");
	// TRANSLATORS: Status bar help for the About menu item
	let about_help = t("About this application");
	let help = help.append_item(ids::ABOUT, &about_label, &about_help).build();

	// TRANSLATORS: Menu bar title
	let file_title = t("&File");
	// TRANSLATORS: Menu bar title
	let help_title = t("&Help");
	MenuBar::builder().append(file, &file_title).append(help, &help_title).build()
}
