//! The Help menu must open on View Help in Browser, the item F1 runs; Escape
//! must close the menu again. The item itself isn't activated, as that would
//! open a browser.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`

#![cfg(target_os = "windows")]

mod common;

use std::time::Duration;
use uiautomation::UIAutomation;
use uiautomation::controls::ControlType;
use uiautomation::core::UIElement;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn help_menu_starts_with_view_help_in_browser() {
	// A Greek tab: Alt+H must reach the menu bar, not the transliteration.
	let app = common::launch("help-menu");
	common::wait_for_search_focus(app.pid);

	common::send_keys("{alt}h");
	// A popup menu doesn't take keyboard focus, so look the menu up instead.
	let automation = UIAutomation::new().expect("UI Automation unavailable");
	let pid = app.pid;
	let menu = automation
		.create_matcher()
		.from(automation.get_root_element().expect("desktop root"))
		.depth(3)
		.filter_fn(Box::new(move |e: &UIElement| Ok(e.get_process_id()? == pid)))
		.control_type(ControlType::Menu)
		.timeout(10_000)
		.find_first()
		.expect("Help menu popup");
	let first = automation
		.get_control_view_walker()
		.and_then(|walker| walker.get_first_child(&menu))
		.expect("first Help menu item");
	assert_eq!(first.get_name().expect("item name"), "View Help in Browser\tF1");

	common::send_keys("{esc}{esc}");
	common::wait_for_focus(app.pid, Duration::from_secs(10), common::is_search_field);
}
