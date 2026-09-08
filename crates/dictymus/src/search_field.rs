use crate::lemma_list;
use crate::tabs::DictionaryTab;
use dictymus_core::braille;
use dictymus_core::normalize::SearchKey;
use dictymus_core::transliterate::transliterate;
use patois::nt;
use std::rc::Rc;
use wxdragon::prelude::*;

pub fn wire(tab: &Rc<DictionaryTab>) {
	let lang = tab.language;

	// Transliteration: intercept char input for he/grc tabs. In braille mode
	// the field takes ASCII braille directly, so the mapping is skipped.
	if lang == "he" || lang == "grc" {
		let tab_for_char = Rc::downgrade(tab);
		tab.search.on_char(move |event| {
			let Some(tab) = tab_for_char.upgrade() else {
				event.skip(true);
				return;
			};
			if !tab.braille.get()
				&& let WindowEventData::Keyboard(kbd) = &event
			{
				// Use get_unicode_key for the actual typed character
				if let Some(code) = kbd.get_unicode_key()
					&& let Some(ch) = char::from_u32(code as u32)
					&& ch.is_ascii()
					&& !ch.is_control()
					&& let Some(text) = transliterate(ch, lang)
				{
					tab.search.write_text(text);
					kbd.event.skip(false);
					return;
				}
			}
			event.skip(true);
		});
	}

	// Filtering: recompute on every text change, typed or picked from the
	// history dropdown. A pick recalls the exact entry: the combo's selection
	// names a history slot, taken only when the field shows that entry's
	// display form.
	// Weak capture: bail if the tab is being closed (see TabManager::close_tab).
	let tab_for_text = Rc::downgrade(tab);
	tab.search.on_text_updated(move |_event| {
		let Some(tab) = tab_for_text.upgrade() else { return };
		let exact = tab
			.search
			.get_selection()
			.and_then(|i| tab.history.borrow().entries().get(i as usize).copied())
			.filter(|&idx| tab.display_word(idx) == tab.search.get_value());
		if apply_filter(&tab, exact) {
			let count = tab.result_count.get();
			// TRANSLATORS: Announced after a search; the placeholder is the number of matching entries
			let msg = nt("{} result", "{} results", count as u64).replace("{}", &count.to_string());
			crate::accessibility::announce_status(tab.frame, tab.status_bar, &msg);
		}
	});

	let tab_for_enter = Rc::downgrade(tab);
	tab.search.on_enter_pressed(move |event| {
		event.event.skip(false);
		let Some(tab) = tab_for_enter.upgrade() else { return };
		crate::search_history::commit(&tab);
	});
}

/// Filter the lemma list by the field's text: the query's key (read as
/// ASCII braille in braille mode) must be a prefix of the lemma's key, so
/// typed points narrow the match and untyped ones match any pointing. The
/// row of `exact` is selected when it survives the filter, else the first
/// row. Returns whether the result count changed.
pub fn apply_filter(tab: &DictionaryTab, exact: Option<usize>) -> bool {
	let text = tab.search.get_value();
	let query = if tab.braille.get() {
		braille::search_key(&text, tab.language)
	} else {
		SearchKey::new(&text)
	};
	let filtered: Vec<usize> = tab
		.dict
		.search_keys()
		.iter()
		.enumerate()
		.filter(|(_, key)| key.starts_with(&query))
		.map(|(i, _)| i)
		.collect();
	let count = filtered.len();
	tracing::debug!(query = %text, results = count, "search");
	let row = exact.and_then(|idx| filtered.iter().position(|&i| i == idx)).unwrap_or(0);
	*tab.filtered.borrow_mut() = filtered;
	lemma_list::repopulate_at(tab, row);
	let changed = count != tab.result_count.get();
	tab.result_count.set(count);
	changed
}
