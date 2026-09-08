//! The Hebrew ASCII braille view against the real GUI: with the mode enabled
//! in the config, the search takes ASCII braille input and the lemma list
//! shows ASCII braille cells.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`
//!
#![cfg(target_os = "windows")]

mod common;

use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn braille_mode_searches_and_lists_in_ascii_braille() {
	let app = common::launch_with(
		"braille",
		dictymus_core::testing::write_hebrew,
		"language = \"en\"\nbraille_languages = [\"he\"]\n",
	);

	// In braille mode the query is ASCII braille: consonant cells alone
	// match every pointing of דבר.
	common::wait_for_search_focus(app.pid);
	let search = common::search_field(app.pid);
	let list = common::find_widget(app.pid, ControlType::List, "Lemmas");
	common::type_query("dvr");
	common::wait_for_value(&search, "dvr");
	common::wait_for_item_count(&list, 4);

	// A vowel cell narrows: dalet-segol leaves only דֶּבֶר, selected and shown
	// as IHBC ASCII braille.
	common::type_query("dev");
	common::wait_for_item_count(&list, 1);
	common::wait_for_selection(&list, &["\"dever"]);
}
