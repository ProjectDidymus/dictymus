//! Search filtering and lemma selection against the real GUI.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`
//!
#![cfg(target_os = "windows")]

mod common;

use std::time::Duration;
use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn typing_filters_and_list_click_selects() {
	let app = common::launch("smoke");
	common::wait_for_focus(app.pid, Duration::from_secs(30), common::is_search_field);

	let search = common::find_widget(app.pid, ControlType::Edit, "Search");
	common::set_value(&search, "λ");
	assert_eq!(common::value(&search), "λ");

	let list = common::find_widget(app.pid, ControlType::List, "Lemmas");
	common::click(&list);

	// The λ filter leaves only λόγος, so clicking selects it.
	common::wait_for_focus(app.pid, Duration::from_secs(10), |control_type, name| {
		control_type == ControlType::ListItem && name == "λόγος"
	});
}
