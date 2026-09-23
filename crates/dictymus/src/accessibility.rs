use live_region::Priority;
use wxdragon::prelude::*;

/// The empty zero-size label that screen reader announcements are raised
/// from; a child of `parent`, outside any sizer.
pub fn create_announcer(parent: &impl WxWidget) -> StaticText {
	let announcer = StaticText::builder(parent).with_label("").with_size(Size::new(0, 0)).build();
	live_region::set_live_region(&announcer);
	announcer
}

/// Show `msg` in the status bar and announce it through `announcer`, queued
/// behind the speech in progress.
pub fn announce_status(frame: Frame, announcer: StaticText, msg: &str) {
	frame.set_status_text(msg, 0);
	live_region::announce_with_priority(announcer, msg, Priority::Medium);
}
