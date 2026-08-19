//! Shared harness for UI Automation tests: launches the real binary against
//! a generated fixture with app data redirected to a temp dir, then drives
//! it through the `uiautomation` crate.

#![allow(dead_code)]

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use uiautomation::UIAutomation;
use uiautomation::controls::ControlType;
use uiautomation::core::UIElement;
use uiautomation::inputs::Keyboard;

pub struct App {
	pub child: Child,
	pub pid: u32,
	base: PathBuf,
}

impl App {
	/// All log lines the app wrote to its redirected data dir.
	pub fn logs(&self) -> String {
		let mut out = String::new();
		let Ok(entries) = std::fs::read_dir(self.base.join("data").join("logs")) else {
			return out;
		};
		for entry in entries.flatten() {
			out.push_str(&std::fs::read_to_string(entry.path()).unwrap_or_default());
		}
		out
	}

	/// Poll until the app exits; returns its exit status.
	pub fn wait_exit(&mut self, timeout: Duration) -> std::process::ExitStatus {
		let deadline = Instant::now() + timeout;
		loop {
			if let Some(status) = self.child.try_wait().expect("try_wait") {
				return status;
			}
			assert!(Instant::now() < deadline, "app did not exit within {timeout:?}");
			std::thread::sleep(Duration::from_millis(200));
		}
	}
}

impl Drop for App {
	fn drop(&mut self) {
		let _ = self.child.kill();
		let _ = self.child.wait();
		let _ = std::fs::remove_dir_all(&self.base);
	}
}

/// Launch the app with a generated Greek fixture; `test_name` keys the temp dir.
pub fn launch(test_name: &str) -> App {
	launch_with(test_name, dictymus_core::testing::write_greek, "language = \"en\"\n")
}

/// Launch the app against the fixture `write_fixture` produces, with the
/// given config file contents; `test_name` keys the temp dir. Configs should
/// pin `language = "en"`: the tests match widgets by their English
/// accessible names regardless of the machine's display language.
pub fn launch_with(
	test_name: &str,
	write_fixture: fn(&std::path::Path) -> PathBuf,
	config: &str,
) -> App {
	let base = std::env::temp_dir().join(format!("dictymus-ui-{test_name}-{}", std::process::id()));
	let ifo = write_fixture(&base.join("fixture"));
	let data_dir = base.join("data");
	std::fs::create_dir_all(&data_dir).expect("create data dir");
	std::fs::write(data_dir.join("config.toml"), config).expect("write config");
	let child = Command::new(env!("CARGO_BIN_EXE_dictymus"))
		.arg(&ifo)
		.env("DICTYMUS_NO_UPDATE_CHECK", "1")
		.env("DICTYMUS_DATA_DIR", &data_dir)
		.env("RUST_LOG", "dictymus=debug")
		.spawn()
		.expect("launch dictymus");
	let pid = child.id();
	App { child, pid, base }
}

fn automation() -> UIAutomation {
	UIAutomation::new().expect("UI Automation unavailable")
}

/// The app's focused element type and name, or None while another process
/// has focus or the window is not up yet.
pub fn focused(pid: u32) -> Option<(ControlType, String)> {
	let el = automation().get_focused_element().ok()?;
	if el.get_process_id().ok()? != pid {
		return None;
	}
	Some((el.get_control_type().ok()?, el.get_name().ok()?))
}

pub fn is_search_field(control_type: ControlType, name: &str) -> bool {
	control_type == ControlType::Edit && name == "Search"
}

/// Poll the focused element until `expected` matches it; panics on timeout.
pub fn wait_for_focus(
	pid: u32,
	timeout: Duration,
	expected: impl Fn(ControlType, &str) -> bool,
) -> (ControlType, String) {
	let deadline = Instant::now() + timeout;
	let mut last = focused(pid);
	loop {
		if let Some((control_type, name)) = last.as_ref()
			&& expected(*control_type, name)
		{
			return (*control_type, name.clone());
		}
		assert!(
			Instant::now() < deadline,
			"expected focus not reached within {timeout:?}; focused element: {last:?}"
		);
		std::thread::sleep(Duration::from_millis(500));
		last = focused(pid);
	}
}

/// Find a widget by control type and name, scoped to the app's window and
/// depth-limited to stay out of the WebView subtree.
pub fn find_widget(pid: u32, control_type: ControlType, name: &str) -> UIElement {
	let automation = automation();
	let root = automation.get_root_element().expect("desktop root");
	let window = automation
		.create_matcher()
		.from(root)
		.depth(2)
		.filter_fn(Box::new(move |e: &UIElement| Ok(e.get_process_id()? == pid)))
		.control_type(ControlType::Window)
		.timeout(10_000)
		.find_first()
		.expect("app window");
	automation
		.create_matcher()
		.from(window)
		.depth(7)
		.control_type(control_type)
		.name(name)
		.timeout(15_000)
		.find_first()
		.unwrap_or_else(|e| panic!("widget {control_type:?} \"{name}\" not found: {e}"))
}

/// Find a top-level window of `pid` titled `title` (e.g. a modal dialog).
/// Depth 3, not 2: a modal dialog nests under its owner window in the UIA
/// tree, one level deeper than the frame itself.
pub fn find_window(pid: u32, title: &str) -> UIElement {
	let automation = automation();
	let root = automation.get_root_element().expect("desktop root");
	automation
		.create_matcher()
		.from(root)
		.depth(3)
		.filter_fn(Box::new(move |e: &UIElement| Ok(e.get_process_id()? == pid)))
		.control_type(ControlType::Window)
		.name(title)
		.timeout(15_000)
		.find_first()
		.unwrap_or_else(|e| panic!("window \"{title}\" not found: {e}"))
}

/// Find a widget by control type and name inside an arbitrary container
/// element (e.g. a dialog found via `find_window`).
pub fn find_widget_in(container: &UIElement, control_type: ControlType, name: &str) -> UIElement {
	automation()
		.create_matcher()
		.from(container.clone())
		.depth(7)
		.control_type(control_type)
		.name(name)
		.timeout(15_000)
		.find_first()
		.unwrap_or_else(|e| panic!("widget {control_type:?} \"{name}\" not found: {e}"))
}

/// Block until the current tab's embedded WebView exists in the UIA tree
/// (its Chromium child window has appeared); panics on timeout. Closing a
/// tab before creation completes aborts the WebView mid-flight.
pub fn wait_for_webview(pid: u32) {
	let automation = automation();
	let root = automation.get_root_element().expect("desktop root");
	let window = automation
		.create_matcher()
		.from(root)
		.depth(2)
		.filter_fn(Box::new(move |e: &UIElement| Ok(e.get_process_id()? == pid)))
		.control_type(ControlType::Window)
		.timeout(10_000)
		.find_first()
		.expect("app window");
	automation
		.create_matcher()
		.from(window)
		.depth(10)
		.filter_fn(Box::new(|e: &UIElement| Ok(e.get_classname()?.starts_with("Chrome_WidgetWin"))))
		.timeout(30_000)
		.find_first()
		.expect("webview child window");
}

pub fn set_value(element: &UIElement, text: &str) {
	let pattern: uiautomation::patterns::UIValuePattern =
		element.get_pattern().expect("ValuePattern");
	pattern.set_value(text).expect("set value");
}

pub fn value(element: &UIElement) -> String {
	let pattern: uiautomation::patterns::UIValuePattern =
		element.get_pattern().expect("ValuePattern");
	pattern.get_value().expect("get value")
}

pub fn click(element: &UIElement) {
	element.click().expect("click");
}

/// Send a key chord OS-wide; the freshly spawned app owns the foreground.
pub fn send_keys(keys: &str) {
	Keyboard::new().send_keys(keys).expect("send keys");
}
