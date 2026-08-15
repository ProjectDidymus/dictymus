//! File → Licenses... (Ctrl+L) must open the Licenses dialog with Import,
//! Remove, and Close buttons; Escape must close it and restore focus.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`

#![cfg(target_os = "windows")]

mod common;

use std::time::Duration;
use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn ctrl_l_opens_licenses_dialog() {
	let mut app = common::launch("licenses-dialog");
	common::wait_for_focus(app.pid, Duration::from_secs(30), common::is_search_field);
	common::wait_for_webview(app.pid);

	common::send_keys("{ctrl}l");

	let dialog = common::find_window(app.pid, "Licenses");
	let remove = common::find_widget_in(&dialog, ControlType::Button, "Remove");
	common::find_widget_in(&dialog, ControlType::Button, "Close");

	// The fixture has no licenses: focus starts on Import, Remove is disabled.
	common::wait_for_focus(app.pid, Duration::from_secs(10), |control_type, name| {
		control_type == ControlType::Button && name == "Import..."
	});
	assert!(!remove.is_enabled().expect("is_enabled"), "Remove enabled with no licenses");

	common::send_keys("{esc}");
	common::wait_for_focus(app.pid, Duration::from_secs(10), common::is_search_field);
	assert!(app.child.try_wait().expect("try_wait").is_none(), "app exited after Escape");
}
