//! The user guide behind Help → View Help in Browser: `doc/readme*.md`, rendered to HTML by the
//! build script and embedded, one page per language.

use std::{
	env, fs, io,
	path::{Path, PathBuf},
};

use patois::t;
use wxdragon::prelude::*;

use crate::translation_manager::TranslationManager;

mod lang_readmes {
	include!(concat!(env!("OUT_DIR"), "/lang_readmes.rs"));
}

/// Opens the guide for the interface language in the default browser.
pub fn view_help_in_browser(frame: &Frame) {
	let lang = TranslationManager::instance().lock().unwrap().current_language();
	let path = match write_readme(&env::temp_dir(), &lang) {
		Ok(path) => path,
		Err(e) => {
			tracing::warn!(error = %e, "failed to write the user guide");
			// TRANSLATORS: Error shown when the user guide could not be prepared for the web browser
			crate::dialogs::show_error(frame, &t("Could not open the user guide."));
			return;
		}
	};
	let url = format!("file://{}", path.to_string_lossy());
	if !launch_default_browser(&url, BrowserLaunchFlags::Default) {
		tracing::warn!(url = %url, "failed to launch the default browser");
		crate::dialogs::show_error(
			frame,
			// TRANSLATORS: Error shown when the web browser could not be started to show the user guide
			&t("Could not start your web browser to show the user guide."),
		);
	}
}

/// Writes the guide for `lang` into `dir` and returns its path. A language without a guide of its
/// own gets the English one.
pub fn write_readme(dir: &Path, lang: &str) -> io::Result<PathBuf> {
	let (lang, bytes) = match lang_readmes::readme_for_lang(lang) {
		Some(bytes) => (lang, bytes),
		None => (
			"en",
			lang_readmes::readme_for_lang("en").ok_or_else(|| {
				io::Error::new(io::ErrorKind::NotFound, "no English guide embedded")
			})?,
		),
	};
	let path = dir.join(format!("dictymus-readme-{lang}.html"));
	fs::write(&path, bytes)?;
	Ok(path)
}

#[cfg(test)]
mod tests {
	use super::{lang_readmes::readme_for_lang, write_readme};

	fn page(lang: &str) -> String {
		String::from_utf8_lossy(readme_for_lang(lang).expect("guide embedded")).into_owned()
	}

	#[test]
	fn writes_the_guide_in_the_interface_language() {
		let dir = tempfile::tempdir().unwrap();
		let path = write_readme(dir.path(), "nl").unwrap();
		assert_eq!(path, dir.path().join("dictymus-readme-nl.html"));
		assert_eq!(std::fs::read_to_string(&path).unwrap(), page("nl"));
	}

	#[test]
	fn falls_back_to_english_for_a_language_without_a_guide() {
		let dir = tempfile::tempdir().unwrap();
		let path = write_readme(dir.path(), "fr").unwrap();
		assert_eq!(path, dir.path().join("dictymus-readme-en.html"));
		assert_eq!(std::fs::read_to_string(&path).unwrap(), page("en"));
	}

	#[test]
	fn each_guide_is_embedded_under_its_language() {
		assert!(page("en").contains(r#"<html lang="en">"#));
		assert!(page("nl").contains(r#"<html lang="nl">"#));
	}

	#[test]
	fn a_language_without_a_guide_has_none() {
		assert!(readme_for_lang("fr").is_none());
	}
}
