//! The Hebrew ASCII braille view against the real GUI: with the mode enabled
//! in the config, the search takes ASCII braille input and the lemma list
//! shows ASCII braille cells.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`
//!
#![cfg(target_os = "windows")]

mod common;

use std::time::{Duration, Instant};
use uiautomation::controls::ControlType;
use uiautomation::patterns::UISelectionPattern;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn braille_mode_searches_and_lists_in_ascii_braille() {
	let app = common::launch_with(
		"braille",
		dictymus_core::testing::write_hebrew,
		"language = \"en\"\nbraille_languages = [\"he\"]\n",
	);

	// In braille mode the query is ASCII braille and matches the folded
	// braille forms: "dvr" leaves only דָּבָר.
	let search = common::find_widget(app.pid, ControlType::Edit, "Search");
	common::set_value(&search, "dvr");
	assert_eq!(common::value(&search), "dvr");

	// The remaining row is selected by the filter and shows the lemma as
	// IHBC ASCII braille. Poll the selection: the filter runs on the UI
	// thread after the value change lands.
	let list = common::find_widget(app.pid, ControlType::List, "Lemmas");
	let deadline = Instant::now() + Duration::from_secs(10);
	let mut names: Vec<String>;
	loop {
		let selection: UISelectionPattern = list.get_pattern().expect("SelectionPattern");
		names = selection
			.get_selection()
			.expect("selection")
			.iter()
			.map(|item| item.get_name().expect("item name"))
			.collect();
		if names == ["\"d<v<r"] {
			break;
		}
		assert!(Instant::now() < deadline, "selection stayed at {names:?}");
		std::thread::sleep(Duration::from_millis(500));
	}
}
