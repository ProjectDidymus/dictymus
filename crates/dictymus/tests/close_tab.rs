//! Ctrl+F4 must close the tab, keep the app responsive, and free the tab.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`

#![cfg(target_os = "windows")]

mod common;

use std::time::Duration;
use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn ctrl_f4_closes_tab_and_frees_it() {
	let mut app = common::launch("close-tab");
	common::wait_for_focus(app.pid, Duration::from_secs(30), common::is_search_field);

	common::send_keys("{ctrl}{F4}");

	// With no tabs left, focus lands on the empty notebook.
	common::wait_for_focus(app.pid, Duration::from_secs(10), |control_type, _| {
		control_type == ControlType::Tab
	});
	assert!(app.child.try_wait().expect("try_wait").is_none(), "app exited after Ctrl+F4");

	common::send_keys("{alt}{F4}");
	let status = app.wait_exit(Duration::from_secs(10));
	assert!(status.success(), "app exited with {status:?}");

	let logs = app.logs();
	assert!(logs.contains("DictionaryTab dropped"), "tab leaked; logs:\n{logs}");
}
