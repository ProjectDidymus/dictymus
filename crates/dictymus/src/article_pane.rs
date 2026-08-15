use crate::tabs::DictionaryTab;
use dictymus_core::normalize::normalize_for_search;
use patois::t;
use wxdragon::prelude::*;

/// Render the article for the lemma at filtered-row `row` into the WebView.
pub fn render_row(tab: &DictionaryTab, row: usize) {
	let filtered = tab.filtered.borrow();
	let Some(&word_idx) = filtered.get(row) else { return };
	// TRANSLATORS: Fallback page title when the entry has no headword
	let title = tab.dict.words().get(word_idx).cloned().unwrap_or_else(|| t("Article"));
	drop(filtered);
	// Render a readable message rather than a blank page — real page content,
	// so a screen reader arrowing into the article reads it.
	let html = tab
		.dict
		.article_html(word_idx)
		// TRANSLATORS: Shown in the article pane when the entry has no content
		.unwrap_or_else(|| format!("<p>{}</p>", t("Article unavailable.")));
	let page = wrap_html(&html, &title);
	*tab.article_html.borrow_mut() = page;
	tab.article.load_url("assets:///article");
}

/// Navigate within the tab to the given word (used for bword:// cross-refs).
pub fn navigate_to(tab: &DictionaryTab, word: &str) {
	let key = normalize_for_search(word);

	// Match against the precomputed normalized list — re-normalizing every
	// word here made each cross-ref click O(n · normalize).
	let normalized = tab.dict.normalized_words();
	let Some(word_idx) = normalized.iter().position(|w| w.starts_with(&key)) else {
		crate::accessibility::announce_status(
			tab.frame,
			tab.status_bar,
			// TRANSLATORS: Announced when a cross-reference target is not in the dictionary; the placeholder is the word
			&t("Not found: {}").replace("{}", word),
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

	// Mark the row selected+focused in the list (so it tracks the article), but
	// do NOT pull widget focus to the list — the user clicked a link in the
	// article and should keep reading there.
	let sel_focused = ListItemState::Selected | ListItemState::Focused;
	tab.list.set_item_state(row as i64, sel_focused, sel_focused);
	tab.list.ensure_visible(row as i64);
	render_row(tab, row);
}

pub const ARTICLE_CSS: &str = "\
@font-face { font-family:'SBL BibLit'; src:url('SBL_BLit.ttf') format('truetype'); }
body { font-family:'SBL BibLit',serif; font-size:14pt; margin:8px; }
a { color:#2a6bbf; text-decoration:underline; cursor:pointer; }
ol { padding-left:1.4em; } li { margin:.2em 0; }
b, strong { font-weight:bold; } i, em { font-style:italic; }";

/// JS injected into every article page.
///
/// Besides the bword cross-ref click handler, this forwards menu-accelerator
/// keystrokes (Ctrl+O / Ctrl+F4 / Ctrl+L / Ctrl+Q) to the host. The WebView2 control
/// swallows keyboard input when it has focus, so wx's accelerator table never
/// sees these chords — we catch them in JS and post the target menu id back,
/// where `process_menu_command` runs the same handler the menu does.
///
/// Everything goes over the single `bword` channel: the wxWidgets Edge backend
/// only honours the first registered script-message handler, so a second
/// `window.*` object never exists. Menu commands are tagged with a `menu:`
/// prefix; bare strings are cross-ref words.
pub fn article_js() -> String {
	use crate::menu::ids;
	format!(
		"\
document.addEventListener('click',function(e){{
  var a=e.target.closest('[data-ref-word]');
  if(a){{ e.preventDefault(); window.bword.postMessage(a.getAttribute('data-ref-word')); }}
}});
document.addEventListener('keydown',function(e){{
  if(!e.ctrlKey||e.altKey||e.shiftKey||e.metaKey) return;
  var id=0, k=e.key.toLowerCase();
  if(e.key==='F4') id={close};
  else if(k==='o') id={open};
  else if(k==='l') id={licenses};
  else if(k==='q') id={exit};
  if(id){{ e.preventDefault(); window.bword.postMessage('menu:'+id); }}
}});
// The SBL face finishes loading ~15ms after the first paint, but on a
// re-navigation WebView2 doesn't repaint already-laid-out text, so the glyphs
// stay in the serif fallback even though the face is loaded and the computed
// font-family is correct. Only tearing the body out of the render tree and
// rebuilding it forces WebView2 to re-shape the text against the now-loaded
// face — sub-property nudges (font-family, letter-spacing) just get repainted
// with the cached fallback glyph run. So toggle display off/on, forcing a
// synchronous relayout in between via offsetHeight. Because paint happens on
// the next frame (after we have already restored display), the hidden state is
// computed but never painted, so there is no visible flicker.
if(document.fonts && document.fonts.ready){{
  document.fonts.ready.then(function(){{
    var b=document.body; if(!b) return;
    b.style.display='none';
    void b.offsetHeight;
    b.style.display='';
  }});
}}",
		close = ids::CLOSE,
		open = ids::OPEN,
		licenses = ids::LICENSES,
		exit = ids::EXIT,
	)
}

pub fn wrap_html(body: &str, title: &str) -> String {
	let title = title.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
	format!(
		r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>{title}</title><link rel="stylesheet" href="style.css"></head><body>{body}<script src="script.js"></script></body></html>"#
	)
}
