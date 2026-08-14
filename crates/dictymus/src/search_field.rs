use crate::lemma_list;
use crate::tabs::DictionaryTab;
use dictymus_core::normalize::normalize_for_search;
use dictymus_core::transliterate::transliterate_char;
use patois::t;
use std::cell::Cell;
use std::rc::Rc;
use wxdragon::prelude::*;

pub fn wire(tab: &Rc<DictionaryTab>) {
	let lang = tab.language;

	// Transliteration: intercept char input for he/el tabs
	if lang == "he" || lang == "el" {
		let search_for_char = tab.search;
		tab.search.on_char(move |event| {
			if let WindowEventData::Keyboard(kbd) = &event {
				// Use get_unicode_key for the actual typed character
				if let Some(code) = kbd.get_unicode_key()
					&& let Some(ch) = char::from_u32(code as u32)
					&& ch.is_ascii()
					&& !ch.is_control()
				{
					let mapped = transliterate_char(ch, lang);
					if mapped != ch {
						search_for_char.write_text(&mapped.to_string());
						kbd.event.skip(false);
						return;
					}
				}
			}
			event.skip(true);
		});
	}

	// Filtering: recompute on every text change.
	// Weak capture: bail if the tab is being closed (see TabManager::close_tab).
	let tab_for_text = Rc::downgrade(tab);
	let last_count: Cell<usize> = Cell::new(usize::MAX);
	tab.search.on_text_updated(move |_event| {
		let Some(tab) = tab_for_text.upgrade() else { return };
		let query = normalize_for_search(&tab.search.get_value());
		let words = tab.dict.normalized_words();
		let mut filtered = Vec::new();
		for (i, w) in words.iter().enumerate() {
			if query.is_empty() || w.starts_with(&query) {
				filtered.push(i);
			}
		}
		let count = filtered.len();
		tracing::debug!(query = %query, results = count, "search");
		*tab.filtered.borrow_mut() = filtered;
		lemma_list::repopulate(&tab);
		if count != last_count.get() {
			last_count.set(count);
			let msg = if count == 1 {
				// TRANSLATORS: Announced when a search matches exactly one entry
				t("1 result")
			} else {
				// TRANSLATORS: Announced after a search; the placeholder is the number of matching entries
				t("{} results").replace("{}", &count.to_string())
			};
			crate::accessibility::announce_status(tab.frame, tab.status_bar, &msg);
		}
	});
}
