//! Logos keyboard transliteration and point-aware search against the real
//! GUI: vowel and Shift keys enter points, typed points narrow the lemma
//! list, untyped ones match any pointing.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`
//!
#![cfg(target_os = "windows")]

mod common;

use uiautomation::controls::ControlType;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn hebrew_vowel_keys_enter_points_that_narrow_the_search() {
	let app = common::launch_with(
		"transliteration_hebrew",
		dictymus_core::testing::write_hebrew,
		"language = \"en\"\n",
	);
	common::wait_for_search_focus(app.pid);
	let search = common::search_field(app.pid);
	let list = common::find_widget(app.pid, ControlType::List, "Lemmas");

	// Consonants only: every pointing of דבר.
	common::type_query("dbr");
	common::wait_for_value(&search, "דבר");
	common::wait_for_item_count(&list, 4);

	// `e` is a segol on the bet: only the lemmas pointed that way.
	common::type_query("dber");
	common::wait_for_value(&search, "דב\u{5b6}ר");
	common::wait_for_item_count(&list, 2);
	common::wait_for_selection(&list, &["דֶּבֶר"]);

	// Shift+o is a holam (plain `o` would be a qamats).
	common::type_query("dOber");
	common::wait_for_item_count(&list, 1);
	common::wait_for_selection(&list, &["דֹּבֶר"]);

	// A vowel on the first letter narrows too.
	common::type_query("ya");
	common::wait_for_value(&search, "י\u{5b7}");
	common::wait_for_item_count(&list, 1);
	common::wait_for_selection(&list, &["יַיִן"]);
}
