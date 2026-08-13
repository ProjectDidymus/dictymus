use crate::{ipc, menu, tabs};
use dictymus_core::config::AppConfig;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use wxdragon::prelude::*;

/// Set once from the leaked `App`; read on the main thread by the IPC
/// dispatch closure, which cannot capture the `Rc`-holding `App` directly.
static APP_PTR: AtomicUsize = AtomicUsize::new(0);

pub fn store_app(app: &'static App) {
	APP_PTR.store(app as *const App as usize, Ordering::SeqCst);
}

pub fn app_from_ptr() -> Option<&'static App> {
	let ptr = APP_PTR.load(Ordering::SeqCst);
	if ptr == 0 {
		return None;
	}
	unsafe { (ptr as *const App).as_ref() }
}

pub struct App {
	pub frame: Frame,
	#[allow(dead_code)] // kept alive here; used only via cloned Rc handles in closures
	pub tabs: Rc<RefCell<tabs::TabManager>>,
	/// Failures collected during startup, shown modally once the frame is
	/// visible (a modal without a visible parent confuses screen readers).
	startup_errors: Vec<String>,
	#[allow(dead_code)] // holds the instance mutex for the process lifetime
	single_instance_checker: Option<SingleInstanceChecker>,
}

impl App {
	pub fn new(config: AppConfig, config_warning: Option<String>) -> Self {
		let single_instance_checker = SingleInstanceChecker::new(ipc::SINGLE_INSTANCE_NAME, None);
		if let Some(checker) = single_instance_checker.as_ref()
			&& checker.is_another_running()
		{
			let cmd = ipc::command_from_cli();
			tracing::info!(command = ?cmd, "another instance is running, forwarding command and exiting");
			ipc::send_command(&cmd);
			std::process::exit(0);
		}

		let frame = Frame::builder().with_title("Dictymus").with_size(Size::new(900, 650)).build();
		frame.set_name("Dictionary");
		frame.set_menu_bar(menu::create_menu_bar());
		let status_bar = frame.create_status_bar(1, 0, -1, "statusbar");
		status_bar.set_name("Status");
		frame.set_status_text("Ready", 0);
		crate::accessibility::init_status_bar_live_region(status_bar);

		let (base_font, font_warning) = crate::fonts::load_base_font();

		let panel = Panel::builder(&frame).build();
		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		let notebook = Notebook::builder(&panel).build();
		notebook.set_name("Dictionary tabs");
		sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 0);
		panel.set_sizer(sizer, true);

		let tab_manager = tabs::TabManager::new(notebook, base_font.clone(), status_bar, frame);
		let tabs = Rc::new(RefCell::new(tab_manager));

		// Startup: load CLI arg or reopen saved config paths
		let mut startup_errors = Vec::new();
		let cli = std::env::args().nth(1);
		if let Some(path) = cli {
			let path = ipc::normalize_cli_path(std::path::Path::new(&path));
			if let Err(e) = tabs.borrow_mut().open_dictionary(&path) {
				tracing::error!("startup: {e}");
				startup_errors.push(e);
			}
		} else {
			if let Some(w) = config_warning {
				tracing::warn!("config: {w}");
				startup_errors.push(w);
			}
			for p in config.open_dictionaries.clone() {
				if let Err(e) = tabs.borrow_mut().open_dictionary(&p) {
					tracing::warn!("reopen failed: {e}");
					startup_errors.push(format!("{e} — the dictionary was not reopened."));
				}
			}
		}
		if let Some(w) = font_warning {
			crate::accessibility::announce_status(frame, status_bar, &w);
		}

		let frame_for_menu = frame;
		let tabs_for_menu = Rc::clone(&tabs);
		#[cfg(windows)]
		let update_channel = config.effective_update_channel(crate::update::default_channel());
		frame.on_menu_selected(move |event| match event.get_id() {
			menu::ids::OPEN => {
				if let Some(path) = crate::dialogs::pick_dictionary(&frame_for_menu) {
					match tabs_for_menu.borrow_mut().open_dictionary(std::path::Path::new(&path)) {
						Ok(_) => {
							frame_for_menu.set_status_text("Dictionary loaded", 0);
						}
						Err(e) => {
							tracing::warn!("open dictionary failed: {e}");
							crate::dialogs::show_error(&frame_for_menu, &e);
						}
					}
				}
			}
			menu::ids::CLOSE => {
				let mut mgr = tabs_for_menu.borrow_mut();
				let sel = mgr.notebook.selection();
				if sel >= 0 {
					mgr.close_tab(sel as usize);
				}
			}
			menu::ids::CLOSE_ALL => {
				let mut mgr = tabs_for_menu.borrow_mut();
				while !mgr.tabs.is_empty() {
					mgr.close_tab(0);
				}
			}
			menu::ids::EXIT => {
				frame_for_menu.close(false);
			}
			menu::ids::ABOUT => {
				crate::dialogs::show_about(&frame_for_menu);
			}
			#[cfg(windows)]
			menu::ids::CHECK_UPDATES => {
				crate::update::run_update_check(&frame_for_menu, update_channel, false);
			}
			_ => {}
		});

		let tabs_for_close = Rc::clone(&tabs);
		let frame_for_close = frame;
		let log_level = config.log_level.clone();
		let check_for_updates_on_startup = config.check_for_updates_on_startup;
		let update_channel_setting = config.update_channel.clone();
		frame.on_close(move |event| {
			let mgr = tabs_for_close.borrow();
			let cfg = AppConfig {
				open_dictionaries: mgr.tabs.iter().map(|t| t.dict.path().to_path_buf()).collect(),
				// Preserve the user's level setting across sessions.
				log_level: log_level.clone(),
				check_for_updates_on_startup,
				update_channel: update_channel_setting.clone(),
			};
			// Inform the user the session was lost, but always let the app exit.
			if let Err(e) = cfg.save() {
				tracing::error!("session save failed: {e}");
				crate::dialogs::show_error(
					&frame_for_close,
					&format!("Could not save session: {e}"),
				);
			}
			event.skip(true);
		});

		App { frame, tabs, startup_errors, single_instance_checker }
	}

	pub fn show(&self) {
		self.frame.show(true);
		self.frame.centre();
		if !self.startup_errors.is_empty() {
			crate::dialogs::show_error(&self.frame, &self.startup_errors.join("\n\n"));
		}
	}

	pub fn handle_ipc_command(&self, command: ipc::IpcCommand) {
		tracing::info!(command = ?command, "received IPC command");
		self.activate_from_ipc();
		if let ipc::IpcCommand::OpenFile(path) = command {
			match self.tabs.borrow_mut().open_dictionary(&path) {
				Ok(_) => self.frame.set_status_text("Dictionary loaded", 0),
				Err(e) => {
					tracing::warn!("open dictionary via IPC failed: {e}");
					crate::dialogs::show_error(&self.frame, &e);
				}
			}
		}
	}

	fn activate_from_ipc(&self) {
		self.frame.show(true);
		self.frame.iconize(false);
		self.frame.request_user_attention(UserAttentionFlag::Info);
		self.frame.raise();
		#[cfg(windows)]
		{
			use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::SetForegroundWindow};
			let handle = self.frame.get_handle();
			if !handle.is_null() {
				let _ = unsafe { SetForegroundWindow(HWND(handle)) };
			}
		}
	}
}
