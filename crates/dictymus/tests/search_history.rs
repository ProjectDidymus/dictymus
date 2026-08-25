//! The search field's per-tab history against the real GUI: Enter commits
//! the shown lemma to the combo box, Down and Up step back and forward
//! through the visited lemmas, and a query without matches commits nothing.
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
fn enter_commits_and_arrows_step_through_the_history() {
	let app = common::launch("search-history");
	common::wait_for_search_focus(app.pid);
	let search = common::search_field(app.pid);
	let list = common::find_widget(app.pid, ControlType::List, "Lemmas");

	// Enter commits the lemma the filter selected and shows it in full
	// ("l" transliterates to λ, "q" to θ).
	common::type_query("l");
	common::wait_for_selection(&list, &["λόγος"]);
	common::send_keys("{enter}");
	common::wait_for_value(&search, "λόγος");

	common::type_query("q");
	common::wait_for_selection(&list, &["θεός"]);
	common::send_keys("{enter}");
	common::wait_for_value(&search, "θεός");

	// Down steps back to the previously committed lemma and re-runs the
	// filter on it; Up steps forward again.
	common::arrow_down();
	common::wait_for_value(&search, "λόγος");
	common::wait_for_selection(&list, &["λόγος"]);
	common::arrow_up();
	common::wait_for_value(&search, "θεός");
	common::wait_for_selection(&list, &["θεός"]);

	// A query without matches commits nothing: the field keeps the query.
	common::type_query("ll");
	common::wait_for_selection(&list, &[]);
	common::send_keys("{enter}");
	std::thread::sleep(Duration::from_secs(1));
	assert_eq!(common::value(&search), "λλ");
}
