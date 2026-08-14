use std::sync::{Mutex, OnceLock};

pub use patois::LanguageInfo;
use patois::ui::WxTranslationManager;

use crate::WxStdCatalogLoader;

/// Thin app-side wrapper around `patois::ui::WxTranslationManager`: owns the
/// singleton lifecycle and the dictymus-specific logging around it.
pub struct TranslationManager {
	inner: WxTranslationManager,
	initialized: bool,
}

impl TranslationManager {
	pub fn instance() -> &'static Mutex<Self> {
		static INSTANCE: OnceLock<Mutex<TranslationManager>> = OnceLock::new();
		INSTANCE.get_or_init(|| Mutex::new(Self::new()))
	}

	pub fn initialize(&mut self) -> bool {
		if self.initialized {
			return true;
		}
		let raw_sys_lang = patois::LanguageManager::system_language();
		let sys_lang = raw_sys_lang.split('_').next().unwrap_or(&raw_sys_lang).to_string();
		self.inner.initialize(WxStdCatalogLoader);
		self.initialized = true;
		if sys_lang != "en" && !self.is_language_available(&sys_lang) {
			tracing::warn!(system_lang = %raw_sys_lang, "system language not available, falling back to English");
		}
		tracing::info!(system_lang = %raw_sys_lang, selected = %self.inner.current_language(), "translations initialized");
		true
	}

	pub fn set_language(&mut self, language_code: &str) -> bool {
		if !self.initialized {
			tracing::warn!(language = %language_code, "set_language called before initialize");
			return false;
		}
		if !self.is_language_available(language_code) {
			tracing::warn!(language = %language_code, "requested language not available");
			return false;
		}
		tracing::info!(language = %language_code, "switching language");
		self.inner.set_language(language_code, WxStdCatalogLoader)
	}

	pub fn available_languages(&self) -> Vec<LanguageInfo> {
		self.inner.available_languages()
	}

	pub fn is_language_available(&self, language_code: &str) -> bool {
		self.inner.is_language_available(language_code)
	}

	fn new() -> Self {
		Self { inner: WxTranslationManager::new("dictymus"), initialized: false }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_manager_has_english_available_by_default() {
		let manager = TranslationManager::new();
		assert!(manager.is_language_available("en"));
		assert!(!manager.is_language_available("zz"));
	}

	#[test]
	fn set_language_fails_when_not_initialized() {
		let mut manager = TranslationManager::new();
		assert!(!manager.set_language("en"));
	}

	/// Confirms `patois::embed_wx_translations!()` (invoked in `main.rs`)
	/// embedded wxstd catalogs restricted to dictymus's own shipped languages.
	/// Degrades gracefully (no-ops) if wxstd catalogs weren't available at
	/// build time, e.g. a wxWidgets build without gettext.
	#[test]
	fn only_shipped_languages_have_wxstd_catalogs_embedded() {
		use wxdragon::translations::TranslationsLoader as _;
		let loader = WxStdCatalogLoader;
		let langs = loader.available_translations("wxstd-3.3");
		if langs.is_empty() {
			return;
		}
		assert!(
			langs.iter().any(|l| l.eq_ignore_ascii_case("nl")),
			"expected Dutch catalog, got {langs:?}"
		);
		assert!(
			!langs.iter().any(|l| l.eq_ignore_ascii_case("af")),
			"af is not a dictymus language: {langs:?}"
		);
	}
}
