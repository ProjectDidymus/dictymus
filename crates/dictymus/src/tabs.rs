use crate::accessibility::{AccProps, AccessibleExt, ROLE_PROPERTYPAGE};
use dictymus_core::dictionary::DictHandle;
use std::cell::RefCell;
use std::path::Path;
use std::rc::{Rc, Weak};
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::webview::WebView;

pub struct DictionaryTab {
	pub panel: Panel,
	pub search: TextCtrl,
	pub list: ListCtrl,
	pub article: WebView,
	pub dict: Rc<DictHandle>,
	pub language: &'static str,
	pub filtered: RefCell<Vec<usize>>,
	pub frame: Frame,
	pub status_bar: wxdragon::widgets::statusbar::StatusBar,
	pub article_html: RefCell<String>,
}

// Proves tabs are actually freed after close; guards against reintroducing
// the Rc-cycle leak that kept closed tabs (and their WebViews) alive.
#[cfg(debug_assertions)]
impl Drop for DictionaryTab {
	fn drop(&mut self) {
		eprintln!("DictionaryTab dropped: {}", self.dict.title());
	}
}

pub struct TabManager {
	pub notebook: Notebook,
	pub tabs: Vec<Rc<DictionaryTab>>,
	pub base_font: Font,
	pub status_bar: wxdragon::widgets::statusbar::StatusBar,
	pub frame: Frame,
}

impl TabManager {
	pub fn new(
		notebook: Notebook,
		base_font: Font,
		status_bar: wxdragon::widgets::statusbar::StatusBar,
		frame: Frame,
	) -> Self {
		Self { notebook, tabs: Vec::new(), base_font, status_bar, frame }
	}

	pub fn build_tab_panel(&self, dict: Rc<DictHandle>) -> Rc<DictionaryTab> {
		let language = dict.language();
		let (panel, search, list, article) = self.build_layout();
		panel.set_accessible_props(AccProps {
			name: Some(dict.title().to_string()),
			role: Some(ROLE_PROPERTYPAGE),
		});
		let filtered: Vec<usize> = (0..dict.word_count()).collect();
		let rc = Rc::new(DictionaryTab {
			panel,
			search,
			list,
			article,
			dict,
			language,
			frame: self.frame,
			status_bar: self.status_bar,
			filtered: RefCell::new(filtered),
			article_html: RefCell::new(String::new()),
		});
		Self::register_asset_handler(&rc);
		Self::wire_events(&rc);
		rc
	}

	/// Create the tab's widgets and sizers. Widget creation order is part of
	/// the contract: the smoke test targets wx's sequentially auto-assigned
	/// control IDs.
	fn build_layout(&self) -> (Panel, TextCtrl, ListCtrl, WebView) {
		let panel = Panel::builder(&self.notebook).build();

		let search_label = StaticText::builder(&panel).with_label("Search").build();
		let search = TextCtrl::builder(&panel).with_style(TextCtrlStyle::ProcessEnter).build();
		search.set_font(&self.base_font);

		let splitter = SplitterWindow::builder(&panel).build();

		let list_panel = Panel::builder(&splitter).build();
		let list_label = StaticText::builder(&list_panel).with_label("Lemmas").build();
		let list = ListCtrl::builder(&list_panel)
			.with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::Virtual)
			.build();
		list.insert_column(0, "Lemma", ListColumnFormat::Left, -1);
		list.set_font(&self.base_font);
		let list_sizer = BoxSizer::builder(Orientation::Vertical).build();
		list_sizer.add(
			&list_label,
			0,
			SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			4,
		);
		list_sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);
		list_panel.set_sizer(list_sizer, true);

		let article_panel = Panel::builder(&splitter).build();
		let article_sizer =
			StaticBoxSizerBuilder::new_with_label(Orientation::Vertical, &article_panel, "Article")
				.build();
		// Fall back to the plain panel as parent if the sizer has no StaticBox
		// — degrades the group label rather than crashing.
		let article = match article_sizer.get_static_box() {
			Some(article_box) => {
				WebView::builder(&article_box).with_url(Some("about:blank".to_string())).build()
			}
			None => {
				WebView::builder(&article_panel).with_url(Some("about:blank".to_string())).build()
			}
		};
		article_sizer.add(&article, 1, SizerFlag::Expand | SizerFlag::All, 4);
		article_panel.set_sizer(article_sizer, true);

		splitter.split_vertically(&list_panel, &article_panel, 300);

		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		sizer.add(
			&search_label,
			0,
			SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			4,
		);
		sizer.add(
			&search,
			0,
			SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom,
			4,
		);
		sizer.add(&splitter, 1, SizerFlag::Expand | SizerFlag::All, 0);
		panel.set_sizer(sizer, true);

		(panel, search, list, article)
	}

	// All callbacks capture Weak: TabManager::tabs is the single strong
	// owner, so a callback firing during the deferred-destroy window after
	// close_tab() upgrades to None and bails instead of touching a dying
	// widget.

	/// Serve the WebView's `assets` scheme: bundled font, CSS, JS, and the
	/// current article body.
	fn register_asset_handler(rc: &Rc<DictionaryTab>) {
		let tab_for_assets: Weak<DictionaryTab> = Rc::downgrade(rc);
		rc.article.register_handler("assets", move |uri| {
			use wxdragon::widgets::webview::WebViewHandlerResponse;
			if uri.ends_with("/SBL_BLit.ttf") {
				Some(WebViewHandlerResponse {
					data: crate::fonts::FONT_BYTES.to_vec(),
					mime_type: Some("font/truetype".to_string()),
				})
			} else if uri.ends_with("/style.css") {
				Some(WebViewHandlerResponse {
					data: crate::article_pane::ARTICLE_CSS.as_bytes().to_vec(),
					mime_type: Some("text/css".to_string()),
				})
			} else if uri.ends_with("/script.js") {
				Some(WebViewHandlerResponse {
					data: crate::article_pane::ARTICLE_JS.as_bytes().to_vec(),
					mime_type: Some("text/javascript".to_string()),
				})
			} else if uri.ends_with("/article") {
				let tab = tab_for_assets.upgrade()?;
				let data = tab.article_html.borrow().as_bytes().to_vec();
				Some(WebViewHandlerResponse {
					data,
					mime_type: Some("text/html; charset=utf-8".to_string()),
				})
			} else {
				None
			}
		});
	}

	/// Wire search filtering, list selection, and bword cross-ref handling.
	fn wire_events(rc: &Rc<DictionaryTab>) {
		crate::search_field::wire(rc);
		let tab_for_virt = Rc::downgrade(rc);
		rc.list.set_virtual_text_callback(move |row, _col| {
			let Some(tab) = tab_for_virt.upgrade() else { return String::new() };
			let filtered = tab.filtered.borrow();
			let words = tab.dict.words();
			filtered.get(row as usize).and_then(|&wi| words.get(wi)).cloned().unwrap_or_default()
		});
		let tab_for_sel = Rc::downgrade(rc);
		rc.list.on_item_selected(move |event| {
			let Some(tab) = tab_for_sel.upgrade() else { return };
			let row = event.get_item_index();
			if row >= 0 {
				crate::article_pane::render_row(&tab, row as usize);
			}
		});
		rc.article.add_script_message_handler("bword");
		let tab_for_msg = Rc::downgrade(rc);
		rc.article.on_script_message_received(move |event| {
			let Some(tab) = tab_for_msg.upgrade() else { return };
			if let Some(word) = event.get_string() {
				crate::article_pane::navigate_to(&tab, &word);
			}
		});
	}

	pub fn open_dictionary(&mut self, path: &Path) -> Result<Rc<DictionaryTab>, String> {
		let dict = Rc::new(
			DictHandle::open(path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?,
		);
		let title = dict.title().to_string();
		let tab = self.build_tab_panel(dict);
		self.notebook.add_page(&tab.panel, &title, true, None);
		crate::lemma_list::repopulate(&tab);
		self.tabs.push(Rc::clone(&tab));
		Ok(tab)
	}

	/// Close the tab at `index`: detach the page, drop the strong Rc, then
	/// destroy the panel (deferred by wx). Ordering matters: the strong Rc
	/// must be gone before destruction runs so Weak callbacks bail out.
	pub fn close_tab(&mut self, index: usize) {
		let Some(tab) = self.tabs.get(index) else { return };
		let title = tab.dict.title().to_string();
		let panel = tab.panel;
		self.notebook.remove_page(index);
		self.tabs.remove(index);
		panel.destroy();
		// Never leave focus on a destroyed window: land in the new current
		// tab's search field (immediate focus event for screen readers), or
		// the notebook when no tabs remain.
		let sel = self.notebook.selection();
		if let Some(next) = (sel >= 0).then(|| self.tabs.get(sel as usize)).flatten() {
			next.search.set_focus();
		} else {
			self.notebook.set_focus();
		}
		crate::accessibility::announce_status(
			self.frame,
			self.status_bar,
			&format!("Closed {title}"),
		);
	}
}
