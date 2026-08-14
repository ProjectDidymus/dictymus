//! Opening a dictionary must put keyboard focus in the search field.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`

#![cfg(target_os = "windows")]

mod common;

use std::time::Duration;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn search_field_focused_after_open() {
	let app = common::launch("focus-on-open");
	common::wait_for_focus(app.pid, Duration::from_secs(30), common::is_search_field);
}
