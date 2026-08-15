use crate::{ipc, menu, tabs};
use dictymus_core::config::AppConfig;
use patois::t;
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
	/// A dictionary that failed to open at startup for lack of a license;
	/// the offer to import one runs once the frame is visible.
	pending_license_prompt: Option<std::path::PathBuf>,
	#[allow(dead_code)] // holds the instance mutex for the process lifetime
	single_instance_checker: Option<SingleInstanceChecker>,
}

impl App {
	pub fn new(config: AppConfig, config_warning: Option<String>) -> Self {
		// Single source of truth for settings: the Options dialog mutates it at
		// runtime and the close handler persists it, so both must share one copy.
		let config = Rc::new(RefCell::new(config));
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
		frame.set_menu_bar(menu::create_menu_bar());
		let status_bar = frame.create_status_bar(1, 0, -1, "statusbar");
		// TRANSLATORS: Initial status bar text
		frame.set_status_text(&t("Ready"), 0);
		crate::accessibility::init_status_bar_live_region(status_bar);

		let (base_font, font_warning) = crate::fonts::load_base_font();

		let panel = Panel::builder(&frame).build();
		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		let notebook = Notebook::builder(&panel).build();
		sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 0);
		panel.set_sizer(sizer, true);

		let tab_manager = tabs::TabManager::new(notebook, base_font.clone(), status_bar, frame);
		let tabs = Rc::new(RefCell::new(tab_manager));

		// Startup: load CLI arg or reopen saved config paths
		let mut startup_errors = Vec::new();
		let mut pending_license_prompt = None;
		let cli = std::env::args().nth(1);
		if let Some(path) = cli {
			let path = ipc::normalize_cli_path(std::path::Path::new(&path));
			let result = tabs.borrow_mut().open_dictionary(&path);
			match result {
				Ok(_) => {}
				Err(tabs::OpenFailure::LicenseMissing { .. }) => {
					tracing::info!("startup: license missing for {}", path.display());
					pending_license_prompt = Some(path);
				}
				Err(e) => {
					let e = e.into_message();
					tracing::error!("startup: {e}");
					startup_errors.push(e);
				}
			}
		} else {
			if let Some(w) = config_warning {
				tracing::warn!("config: {w}");
				startup_errors.push(w);
			}
			for p in config.borrow().open_dictionaries.clone() {
				if let Err(e) = tabs.borrow_mut().open_dictionary(&p) {
					let e = e.into_message();
					tracing::warn!("reopen failed: {e}");
					// TRANSLATORS: Startup warning; the placeholder is the error that prevented reopening
					startup_errors
						.push(t("{} — the dictionary was not reopened.").replace("{}", &e));
				}
			}
		}
		if let Some(w) = font_warning {
			crate::accessibility::announce_status(frame, status_bar, &w);
		}

		let frame_for_menu = frame;
		let tabs_for_menu = Rc::clone(&tabs);
		let config_for_menu = Rc::clone(&config);
		frame.on_menu_selected(move |event| match event.get_id() {
			menu::ids::OPEN => {
				if let Some(path) = crate::dialogs::pick_dictionary(&frame_for_menu)
					&& open_with_license_prompt(
						&frame_for_menu,
						&tabs_for_menu,
						std::path::Path::new(&path),
					) {
					// TRANSLATORS: Status bar text after opening a dictionary
					frame_for_menu.set_status_text(&t("Dictionary loaded"), 0);
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
			menu::ids::LICENSES => {
				crate::license_manager::show_license_manager(&frame_for_menu);
			}
			menu::ids::OPTIONS => {
				crate::options::show_options(&frame_for_menu, &config_for_menu);
			}
			menu::ids::ABOUT => {
				crate::dialogs::show_about(&frame_for_menu);
			}
			#[cfg(windows)]
			menu::ids::CHECK_UPDATES => {
				let channel = config_for_menu
					.borrow()
					.effective_update_channel(crate::update::default_channel());
				crate::update::run_update_check(&frame_for_menu, channel, false);
			}
			_ => {}
		});

		let tabs_for_close = Rc::clone(&tabs);
		let frame_for_close = frame;
		let config_for_close = Rc::clone(&config);
		frame.on_close(move |event| {
			let mgr = tabs_for_close.borrow();
			// Persist the shared settings as they stand; only the open-tab list
			// is owned by the tab manager rather than the config.
			let mut cfg = config_for_close.borrow().clone();
			cfg.open_dictionaries = mgr.tabs.iter().map(|t| t.dict.path().to_path_buf()).collect();
			// Inform the user the session was lost, but always let the app exit.
			if let Err(e) = cfg.save() {
				tracing::error!("session save failed: {e}");
				crate::dialogs::show_error(
					&frame_for_close,
					// TRANSLATORS: Error shown while exiting; the placeholder is the underlying error
					&t("Could not save session: {}").replace("{}", &e.to_string()),
				);
			}
			event.skip(true);
		});

		App { frame, tabs, startup_errors, pending_license_prompt, single_instance_checker }
	}

	pub fn show(&self) {
		self.frame.show(true);
		self.frame.centre();
		if !self.startup_errors.is_empty() {
			crate::dialogs::show_error(&self.frame, &self.startup_errors.join("\n\n"));
		}
		if let Some(path) = &self.pending_license_prompt
			&& open_with_license_prompt(&self.frame, &self.tabs, path)
		{
			// TRANSLATORS: Status bar text after opening a dictionary
			self.frame.set_status_text(&t("Dictionary loaded"), 0);
		}
	}

	pub fn handle_ipc_command(&self, command: ipc::IpcCommand) {
		tracing::info!(command = ?command, "received IPC command");
		self.activate_from_ipc();
		if let ipc::IpcCommand::OpenFile(path) = command
			&& open_with_license_prompt(&self.frame, &self.tabs, &path)
		{
			// TRANSLATORS: Status bar text after opening a dictionary
			self.frame.set_status_text(&t("Dictionary loaded"), 0);
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

/// Open `path` in a tab; on a missing license offer to import one, install
/// it, and retry once. Shows its own dialogs; returns true if a tab opened.
fn open_with_license_prompt(
	frame: &Frame,
	tabs: &Rc<RefCell<tabs::TabManager>>,
	path: &std::path::Path,
) -> bool {
	// Bind each attempt before matching: a RefMut held across the modal
	// dialogs in the arms would panic on any re-entrant open (e.g. an IPC
	// command arriving while a dialog is up).
	let result = tabs.borrow_mut().open_dictionary(path);
	let dict_name = match result {
		Ok(_) => return true,
		Err(tabs::OpenFailure::LicenseMissing { dict_name }) => dict_name,
		Err(e) => {
			let e = e.into_message();
			tracing::warn!("open dictionary failed: {e}");
			crate::dialogs::show_error(frame, &e);
			return false;
		}
	};
	// TRANSLATORS: Yes/No question when opening a protected dictionary; the placeholder is the dictionary name
	let message = t("{} requires a license. Do you want to import a license file now?")
		.replace("{}", &dict_name);
	// TRANSLATORS: Title of the license question dialog
	let title = t("License required");
	let answer = MessageDialog::builder(frame, &message, &title)
		.with_style(
			MessageDialogStyle::YesNo
				| MessageDialogStyle::IconQuestion
				| MessageDialogStyle::Centre,
		)
		.build()
		.show_modal();
	if answer != ID_YES {
		return false;
	}
	let Some(picked) = crate::dialogs::pick_license(frame) else { return false };
	if let Err(e) = crate::licensing::install_license(
		std::path::Path::new(&picked),
		&crate::licensing::license_pubkey(),
	) {
		tracing::warn!("import license failed: {e}");
		crate::dialogs::show_error(frame, &e);
		return false;
	}
	let result = tabs.borrow_mut().open_dictionary(path);
	match result {
		Ok(_) => true,
		Err(tabs::OpenFailure::LicenseMissing { dict_name }) => {
			// TRANSLATORS: Error after importing a license that does not unlock the dictionary; the placeholder is the dictionary name
			let message = t("The imported license does not unlock {}.").replace("{}", &dict_name);
			tracing::warn!("open dictionary failed: {message}");
			crate::dialogs::show_error(frame, &message);
			false
		}
		Err(e) => {
			let e = e.into_message();
			tracing::warn!("open dictionary failed: {e}");
			crate::dialogs::show_error(frame, &e);
			false
		}
	}
}
