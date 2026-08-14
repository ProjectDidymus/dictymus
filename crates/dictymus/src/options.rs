use dictymus_core::config::AppConfig;
use patois::t;
use std::cell::RefCell;
use std::rc::Rc;
use wxdragon::id::{ID_CANCEL, ID_OK};
use wxdragon::prelude::*;

use crate::translation_manager::TranslationManager;

const DIALOG_PADDING: i32 = 5;

/// Modal Options dialog. On OK the shared config is updated and saved
/// immediately; a language change is applied to the running process (new
/// widgets pick it up) and announced as fully effective after a restart.
pub fn show_options(parent: &Frame, config: &Rc<RefCell<AppConfig>>) {
	// TRANSLATORS: Title of the Options dialog
	let dialog = Dialog::builder(parent, &t("Options")).build();

	// TRANSLATORS: Label of the language selector in the Options dialog
	let language_label_text = t("&Language:");
	let language_label = StaticText::builder(&dialog).with_label(&language_label_text).build();
	let language_combo = Choice::builder(&dialog).build();
	// TRANSLATORS: First entry of the language selector: follow the Windows display language
	language_combo.append(&t("System default"));
	let mut language_codes = vec![String::new()];
	let languages = TranslationManager::instance().lock().unwrap().available_languages();
	language_codes.extend(patois::ui::populate_language_choice(&language_combo, &languages));
	#[cfg(target_os = "macos")]
	language_combo
		.set_accessibility_label(language_label_text.replace('&', "").trim_end_matches(':').trim());
	let current_language = config.borrow().language.clone();
	let language_index =
		language_codes.iter().position(|code| code == &current_language).unwrap_or(0);
	language_combo.set_selection(u32::try_from(language_index).unwrap_or(0));

	#[cfg(windows)]
	let updates_check = {
		let check = CheckBox::builder(&dialog)
			// TRANSLATORS: Option in the Options dialog
			.with_label(&t("Check for &updates on startup"))
			.build();
		check.set_value(config.borrow().check_for_updates_on_startup);
		check
	};
	#[cfg(windows)]
	let channel_codes = ["", "stable", "dev"];
	#[cfg(windows)]
	let (channel_label, channel_combo) = {
		// TRANSLATORS: Label of the update channel selector in the Options dialog
		let channel_label_text = t("Update &channel:");
		let label = StaticText::builder(&dialog).with_label(&channel_label_text).build();
		let combo = Choice::builder(&dialog).build();
		// TRANSLATORS: Update channel entry: stable releases for release builds, development builds otherwise
		combo.append(&t("Default for this build"));
		// TRANSLATORS: Update channel entry: tagged releases only
		combo.append(&t("Stable"));
		// TRANSLATORS: Update channel entry: rolling development builds
		combo.append(&t("Development"));
		let current = config.borrow().update_channel.clone();
		let index = channel_codes.iter().position(|code| *code == current).unwrap_or(0);
		combo.set_selection(u32::try_from(index).unwrap_or(0));
		(label, combo)
	};

	// TRANSLATORS: Label for the confirmation button
	let ok_button = Button::builder(&dialog).with_id(ID_OK).with_label(&t("OK")).build();
	// TRANSLATORS: Label for the cancellation button
	let cancel_button =
		Button::builder(&dialog).with_id(ID_CANCEL).with_label(&t("Cancel")).build();
	dialog.set_escape_id(ID_CANCEL);
	dialog.set_affirmative_id(ID_OK);
	ok_button.set_default();

	let content_sizer = BoxSizer::builder(Orientation::Vertical).build();
	let language_sizer = BoxSizer::builder(Orientation::Horizontal).build();
	language_sizer.add(
		&language_label,
		0,
		SizerFlag::AlignCenterVertical | SizerFlag::Right,
		DIALOG_PADDING,
	);
	language_sizer.add(&language_combo, 0, SizerFlag::AlignCenterVertical, 0);
	content_sizer.add_sizer(&language_sizer, 0, SizerFlag::All, DIALOG_PADDING);
	#[cfg(windows)]
	{
		content_sizer.add(&updates_check, 0, SizerFlag::All, DIALOG_PADDING);
		let channel_sizer = BoxSizer::builder(Orientation::Horizontal).build();
		channel_sizer.add(
			&channel_label,
			0,
			SizerFlag::AlignCenterVertical | SizerFlag::Right,
			DIALOG_PADDING,
		);
		channel_sizer.add(&channel_combo, 0, SizerFlag::AlignCenterVertical, 0);
		content_sizer.add_sizer(&channel_sizer, 0, SizerFlag::All, DIALOG_PADDING);
	}
	let button_sizer = BoxSizer::builder(Orientation::Horizontal).build();
	button_sizer.add_stretch_spacer(1);
	button_sizer.add(&ok_button, 0, SizerFlag::All, DIALOG_PADDING);
	button_sizer.add(&cancel_button, 0, SizerFlag::All, DIALOG_PADDING);
	content_sizer.add_sizer(&button_sizer, 0, SizerFlag::Expand, 0);
	dialog.set_sizer_and_fit(content_sizer, true);
	dialog.centre();

	if dialog.show_modal() != ID_OK {
		return;
	}

	let new_language = patois::ui::resolve_language_choice(&language_combo, &language_codes)
		.unwrap_or_else(|| current_language.clone());
	let language_changed = new_language != current_language;
	let save_result = {
		let mut cfg = config.borrow_mut();
		cfg.language = new_language.clone();
		#[cfg(windows)]
		{
			cfg.check_for_updates_on_startup = updates_check.is_checked();
			let index = channel_combo
				.get_selection()
				.and_then(|index| usize::try_from(index).ok())
				.unwrap_or(0);
			cfg.update_channel = channel_codes.get(index).copied().unwrap_or("").to_string();
		}
		cfg.save()
	};
	if let Err(e) = save_result {
		tracing::error!("settings save failed: {e}");
		crate::dialogs::show_error(
			parent,
			// TRANSLATORS: Error dialog text; the placeholder is the underlying error
			&t("Could not save settings: {}").replace("{}", &e.to_string()),
		);
	}
	if language_changed {
		apply_language(&new_language);
		// TRANSLATORS: Shown after changing the language in the Options dialog
		let message = t("The language change will take full effect after you restart Dictymus.");
		MessageDialog::builder(parent, &message, "Dictymus")
			.with_style(
				MessageDialogStyle::OK
					| MessageDialogStyle::IconInformation
					| MessageDialogStyle::Centre,
			)
			.build()
			.show_modal();
	}
}

/// Activate `code`, resolving the empty "follow the system" setting to the
/// system language (or English when that language isn't shipped).
fn apply_language(code: &str) {
	let mut translations = TranslationManager::instance().lock().unwrap();
	if code.is_empty() {
		let raw = patois::LanguageManager::system_language();
		let sys = raw.split('_').next().unwrap_or(&raw).to_string();
		let code = if translations.is_language_available(&sys) { sys } else { "en".to_string() };
		translations.set_language(&code);
	} else {
		translations.set_language(code);
	}
}
