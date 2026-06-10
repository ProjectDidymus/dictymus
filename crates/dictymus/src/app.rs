use crate::{menu, tabs};
use dictymus_core::config::AppConfig;
use std::cell::RefCell;
use std::rc::Rc;
use wxdragon::prelude::*;
pub struct App {
	pub frame: Frame,
	#[allow(dead_code)] // kept alive here; used only via cloned Rc handles in closures
	pub tabs: Rc<RefCell<tabs::TabManager>>,
}

impl App {
	pub fn new() -> Self {
		let frame = Frame::builder().with_title("Dictymus").with_size(Size::new(900, 650)).build();
		frame.set_name("Dictionary");
		frame.set_menu_bar(menu::create_menu_bar());
		let status_bar = frame.create_status_bar(1, 0, -1, "statusbar");
		status_bar.set_name("Status");
		frame.set_status_text("Ready", 0);
		crate::accessibility::init_status_bar_live_region(status_bar);

		let base_font = crate::fonts::load_base_font();

		let panel = Panel::builder(&frame).build();
		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		let notebook = Notebook::builder(&panel).build();
		notebook.set_name("Dictionary tabs");
		sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 0);
		panel.set_sizer(sizer, true);

		let tab_manager = tabs::TabManager::new(notebook, base_font.clone(), status_bar);
		let tabs = Rc::new(RefCell::new(tab_manager));

		// Startup: load CLI arg or reopen saved config paths
		let cli = std::env::args().nth(1);
		if let Some(path) = cli {
			if let Err(e) = tabs.borrow_mut().open_dictionary(std::path::Path::new(&path), frame) {
				eprintln!("Reopen failed: {e}");
			}
		} else {
			let cfg = AppConfig::load();
			for p in cfg.open_dictionaries.clone() {
				if let Err(e) = tabs.borrow_mut().open_dictionary(&p, frame) {
					eprintln!("Reopen failed: {e}");
				}
			}
		}

		let frame_for_menu = frame;
		let tabs_for_menu = Rc::clone(&tabs);
		frame.on_menu_selected(move |event| match event.get_id() {
			menu::ids::OPEN => {
				if let Some(path) = crate::dialogs::pick_dictionary(&frame_for_menu) {
					match tabs_for_menu
						.borrow_mut()
						.open_dictionary(std::path::Path::new(&path), frame_for_menu)
					{
						Ok(_) => {
							frame_for_menu.set_status_text("Dictionary loaded", 0);
						}
						Err(e) => {
							frame_for_menu.set_status_text(&e, 0);
						}
					}
				}
			}
			menu::ids::CLOSE => {
				let mut mgr = tabs_for_menu.borrow_mut();
				let sel = mgr.notebook.selection();
				if sel >= 0 {
					mgr.notebook.remove_page(sel as usize);
					mgr.tabs.remove(sel as usize);
				}
			}
			menu::ids::CLOSE_ALL => {
				let mut mgr = tabs_for_menu.borrow_mut();
				while !mgr.tabs.is_empty() {
					mgr.notebook.remove_page(0);
					mgr.tabs.remove(0);
				}
			}
			menu::ids::EXIT => {
				frame_for_menu.close(false);
			}
			menu::ids::ABOUT => {
				crate::dialogs::show_about(&frame_for_menu);
			}
			_ => {}
		});

		let tabs_for_close = Rc::clone(&tabs);
		frame.on_close(move |event| {
			let mgr = tabs_for_close.borrow();
			let cfg = AppConfig {
				open_dictionaries: mgr.tabs.iter().map(|t| t.dict.path().to_path_buf()).collect(),
			};
			cfg.save();
			event.skip(true);
		});

		App { frame, tabs }
	}

	pub fn show(&self) {
		self.frame.show(true);
		self.frame.centre();
	}
}
