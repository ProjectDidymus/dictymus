//! Logos Greek keyboard Shift plane against the real GUI: Shift enters a
//! capital, and the search folds case.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`
//!
#![cfg(target_os = "windows")]

mod common;

use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn greek_shift_gives_capitals_and_the_search_folds_case() {
	let app = common::launch("greek_capitals");
	common::wait_for_search_focus(app.pid);
	let search = common::search_field(app.pid);
	let list = common::find_widget(app.pid, ControlType::List, "Lemmas");

	common::type_query("Logos");
	common::wait_for_value(&search, "Λογοσ");
	common::wait_for_selection(&list, &["λόγος"]);
}
