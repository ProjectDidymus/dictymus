use crate::tabs::DictionaryTab;
use dictymus_core::normalize::normalize_for_search;
use wxdragon::prelude::*;

/// Render the article for the lemma at filtered-row `row` into the WebView.
pub fn render_row(tab: &DictionaryTab, row: usize) {
	let filtered = tab.filtered.borrow();
	let Some(&word_idx) = filtered.get(row) else { return };
	let title = tab.dict.words().get(word_idx).cloned().unwrap_or_else(|| "Article".to_string());
	drop(filtered);
	let html = tab.dict.article_html(word_idx).unwrap_or_default();
	let page = wrap_html(&html, &title);
	*tab.article_html.borrow_mut() = page;
	tab.article.load_url("assets:///article");
}

/// Navigate within the tab to the given word (used for bword:// cross-refs).
pub fn navigate_to(tab: &DictionaryTab, word: &str) {
	let key = normalize_for_search(word);
	let words = tab.dict.words();

	let Some(word_idx) = words.iter().position(|w| normalize_for_search(w).starts_with(&key)) else {
		crate::accessibility::announce_status(
			tab.frame,
			tab.status_bar,
			&format!("Not found: {word}"),
		);
		return;
	};

	let row = tab.filtered.borrow().iter().position(|&i| i == word_idx);

	let row = if let Some(r) = row {
		r
	} else {
		tab.search.change_value("");
		*tab.filtered.borrow_mut() = (0..tab.dict.word_count()).collect();
		crate::lemma_list::repopulate(tab);
		tab.filtered.borrow().iter().position(|&i| i == word_idx).unwrap_or(0)
	};

	tab.list.set_item_state(row as i64, ListItemState::Selected, ListItemState::Selected);
	tab.list.ensure_visible(row as i64);
	tab.list.set_focus();
	render_row(tab, row);
}

pub const ARTICLE_CSS: &str = "\
@font-face { font-family:'SBL BibLit'; src:url('SBL_BLit.ttf') format('truetype'); }
body { font-family:'SBL BibLit',serif; font-size:14pt; margin:8px; }
a { color:#2a6bbf; text-decoration:underline; cursor:pointer; }
ol { padding-left:1.4em; } li { margin:.2em 0; }
b, strong { font-weight:bold; } i, em { font-style:italic; }";

pub const ARTICLE_JS: &str = "\
document.addEventListener('click',function(e){
  var a=e.target.closest('[data-ref-word]');
  if(a){ e.preventDefault(); window.bword.postMessage(a.getAttribute('data-ref-word')); }
});";

pub fn wrap_html(body: &str, title: &str) -> String {
	let title = title.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
	format!(
		r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>{title}</title><link rel="stylesheet" href="style.css"></head><body>{body}<script src="script.js"></script></body></html>"#
	)
}
