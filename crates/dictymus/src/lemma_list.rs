use crate::tabs::DictionaryTab;
use wxdragon::prelude::*;

pub fn repopulate(tab: &DictionaryTab) {
	repopulate_at(tab, 0);
}

/// Reload the list from `tab.filtered` and select `row`.
pub fn repopulate_at(tab: &DictionaryTab, row: usize) {
	let count = tab.filtered.borrow().len();
	tab.list.set_item_count(count as i64);
	if count > 0 {
		tab.list.refresh_items(0, count as i64 - 1);
		select_row(tab, row);
	}
}

/// Select and focus `row` if the list has it.
///
/// Programmatic `set_item_state` does NOT fire `EVT_LIST_ITEM_SELECTED`, so the
/// article is rendered explicitly here — otherwise the row highlights but the
/// lemma never loads.
pub fn select_row(tab: &DictionaryTab, row: usize) {
	if row < usize::try_from(tab.list.get_item_count()).unwrap_or(0) {
		let sel_focused = ListItemState::Selected | ListItemState::Focused;
		tab.list.set_item_state(row as i64, sel_focused, sel_focused);
		tab.list.ensure_visible(row as i64);
		crate::article_pane::render_row(tab, row);
	}
}
