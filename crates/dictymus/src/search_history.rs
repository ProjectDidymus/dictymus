//! Glue between a tab's `History` and its search combo box: the dropdown
//! items mirror the history, and committing pushes the lemma on screen.

use crate::tabs::DictionaryTab;

/// Record `idx` as the most recently visited lemma and refresh the dropdown.
pub fn push(tab: &DictionaryTab, idx: usize) {
	tab.history.borrow_mut().push(idx);
	sync_items(tab);
}

/// Rebuild the combo's dropdown items from the history, one item at a time;
/// afterwards no item is selected and the text entry is untouched.
pub fn sync_items(tab: &DictionaryTab) {
	for i in (0..tab.search.get_count()).rev() {
		tab.search.delete(i);
	}
	for &idx in tab.history.borrow().entries() {
		tab.search.append(&tab.display_word(idx));
	}
}

/// Commit the lemma on screen: push it, show it as the field's text and
/// narrow the list to it. No-op while the list has no rows.
pub fn commit(tab: &DictionaryTab) {
	if tab.filtered.borrow().is_empty() {
		return;
	}
	let Some(idx) = tab.current.get() else { return };
	push(tab, idx);
	tab.search.set_selection(0);
	crate::search_field::apply_filter(tab, Some(idx));
}
