use crate::lemma_list;
use crate::tabs::DictionaryTab;
use dictymus_core::braille;
use dictymus_core::normalize::normalize_for_search;
use dictymus_core::transliterate::transliterate_char;
use patois::nt;
use std::cell::Cell;
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
				{
					let mapped = transliterate_char(ch, lang);
					if mapped != ch {
						tab.search.write_text(&mapped.to_string());
						kbd.event.skip(false);
						return;
					}
				}
			}
			event.skip(true);
		});
	}

	// Filtering: recompute on every text change. In braille mode the query is
	// ASCII braille and matches the folded braille forms of the lemmas;
	// otherwise the script query matches the normalized lemmas.
	// Weak capture: bail if the tab is being closed (see TabManager::close_tab).
	let tab_for_text = Rc::downgrade(tab);
	let last_count: Cell<usize> = Cell::new(usize::MAX);
	tab.search.on_text_updated(move |_event| {
		let Some(tab) = tab_for_text.upgrade() else { return };
		let mut filtered = Vec::new();
		let query = if tab.braille.get() {
			let query = braille::normalize_braille(&tab.search.get_value(), tab.language);
			match tab.braille_words.borrow().as_ref() {
				Some(cache) => {
					for (i, w) in cache.normalized.iter().enumerate() {
						if query.is_empty() || w.starts_with(&query) {
							filtered.push(i);
						}
					}
				}
				None => filtered.extend(0..tab.dict.word_count()),
			}
			query
		} else {
			let query = normalize_for_search(&tab.search.get_value());
			for (i, w) in tab.dict.normalized_words().iter().enumerate() {
				if query.is_empty() || w.starts_with(&query) {
					filtered.push(i);
				}
			}
			query
		};
		let count = filtered.len();
		tracing::debug!(query = %query, results = count, "search");
		*tab.filtered.borrow_mut() = filtered;
		lemma_list::repopulate(&tab);
		if count != last_count.get() {
			last_count.set(count);
			// TRANSLATORS: Announced after a search; the placeholder is the number of matching entries
			let msg = nt("{} result", "{} results", count as u64).replace("{}", &count.to_string());
			crate::accessibility::announce_status(tab.frame, tab.status_bar, &msg);
		}
	});
}
