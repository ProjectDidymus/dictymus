use crate::lemma_list;
use crate::tabs::DictionaryTab;
use dictymus_core::normalize::normalize_for_search;
use dictymus_core::transliterate::transliterate_char;
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
				if let Some(code) = kbd.get_unicode_key() {
					if let Some(ch) = char::from_u32(code as u32) {
						if ch.is_ascii() && !ch.is_control() {
							let mapped = transliterate_char(ch, lang);
							if mapped != ch {
								search_for_char.write_text(&mapped.to_string());
								kbd.event.skip(false);
								return;
							}
						}
					}
				}
			}
			event.skip(true);
		});
	}

	// Filtering: recompute on every text change
	let tab_for_text = Rc::clone(tab);
	let last_count: Cell<usize> = Cell::new(usize::MAX);
	tab.search.on_text_updated(move |_event| {
		let query = normalize_for_search(&tab_for_text.search.get_value());
		let words = tab_for_text.dict.normalized_words();
		let mut filtered = Vec::new();
		for (i, w) in words.iter().enumerate() {
			if query.is_empty() || w.starts_with(&query) {
				filtered.push(i);
			}
		}
		let count = filtered.len();
		*tab_for_text.filtered.borrow_mut() = filtered;
		lemma_list::repopulate(&tab_for_text);
		if count != last_count.get() {
			last_count.set(count);
			let msg = if count == 1 { "1 result".to_string() } else { format!("{count} results") };
			crate::accessibility::announce_status(
				tab_for_text.frame,
				tab_for_text.status_bar,
				&msg,
			);
		}
	});
}
