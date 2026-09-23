//! Screen reader announcements against the real GUI: the search's result
//! count reaches screen readers as a UI Automation notification, queued
//! behind the speech in progress.
//!
//! Drives the real GUI through UI Automation, so it needs an interactive
//! desktop. Run explicitly: `cargo test -p dictymus -- --ignored`
//!
#![cfg(target_os = "windows")]

mod common;

use windows::Win32::UI::Accessibility::NotificationProcessing_CurrentThenMostRecent;

#[test]
#[ignore = "drives the real GUI via UI Automation; needs an interactive desktop"]
fn search_result_count_is_announced_as_a_queued_notification() {
	let app = common::launch("announcements");
	common::wait_for_search_focus(app.pid);
	let notifications = common::Notifications::listen(app.pid);

	common::type_query("l");
	notifications.wait_for("1 result", NotificationProcessing_CurrentThenMostRecent);
}
