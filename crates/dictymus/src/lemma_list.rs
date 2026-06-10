use crate::tabs::DictionaryTab;
use wxdragon::prelude::*;

pub fn repopulate(tab: &DictionaryTab) {
	let count = tab.filtered.borrow().len();
	tab.list.set_item_count(count as i64);
	if count > 0 {
		tab.list.refresh_items(0, count as i64 - 1);
	}
	select_first(tab);
}

/// Select and focus the first visible item if list is non-empty.
pub fn select_first(tab: &DictionaryTab) {
	if tab.list.get_item_count() > 0 {
		let sel_focused = ListItemState::Selected | ListItemState::Focused;
		tab.list.set_item_state(0, sel_focused, sel_focused);
		tab.list.ensure_visible(0);
	}
}
