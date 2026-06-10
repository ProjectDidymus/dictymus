use wxdragon::prelude::*;

pub mod ids {
	pub const OPEN: i32 = 5001;
	pub const CLOSE: i32 = 5002;
	pub const CLOSE_ALL: i32 = 5003;
	pub const EXIT: i32 = 5004;
	pub const ABOUT: i32 = 5005;
}

pub fn create_menu_bar() -> MenuBar {
	let file = Menu::builder()
		.append_item(ids::OPEN, "&Open...\tCtrl+O", "Open a dictionary")
		.append_item(ids::CLOSE, "&Close\tCtrl+F4", "Close current dictionary")
		.append_item(ids::CLOSE_ALL, "Close &All", "Close all dictionaries")
		.append_separator()
		.append_item(ids::EXIT, "E&xit\tCtrl+Q", "Exit")
		.build();

	let help = Menu::builder()
		.append_item(ids::ABOUT, "&About Dictymus", "About this application")
		.build();

	MenuBar::builder().append(file, "&File").append(help, "&Help").build()
}
