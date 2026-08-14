use patois::t;
use wxdragon::prelude::*;

/// Modal error dialog — the single error surface of the app. Modal so focus
/// moves into the dialog and screen readers read the message text.
pub fn show_error(parent: &Frame, message: &str) {
	// TRANSLATORS: Title of the error dialog
	let title = t("Dictymus - Error");
	MessageDialog::builder(parent, message, &title)
		.with_style(
			MessageDialogStyle::OK | MessageDialogStyle::IconError | MessageDialogStyle::Centre,
		)
		.build()
		.show_modal();
}

pub fn show_about(parent: &Frame) {
	let mut info = AboutDialogInfo::new();
	info.set_name("Dictymus");
	info.set_version(env!("CARGO_PKG_VERSION"));
	// TRANSLATORS: One-line app description in the About dialog
	info.set_description(&t("An accessible dictionary for biblical languages"));
	show_about_box(&info, Some(parent));
}

pub fn pick_dictionary(parent: &Frame) -> Option<String> {
	use wxdragon::id::ID_OK;
	// TRANSLATORS: Title of the dictionary file picker dialog
	let message = t("Open dictionary");
	// Only the labels are translated; the glob patterns between the pipes are
	// part of the wx wildcard format and must stay as they are.
	let wildcard = format!(
		"{}|*.ifo;*.mdx|{}|*.ifo|{}|*.mdx|{}|*.*",
		// TRANSLATORS: File picker filter label for all supported dictionary formats
		t("Dictionaries (*.ifo;*.mdx)"),
		// TRANSLATORS: File picker filter label for StarDict dictionaries
		t("StarDict info (*.ifo)"),
		// TRANSLATORS: File picker filter label for MDict dictionaries
		t("MDict (*.mdx)"),
		// TRANSLATORS: File picker filter label for all files
		t("All files (*.*)"),
	);
	let dialog = FileDialog::builder(parent)
		.with_message(&message)
		.with_wildcard(&wildcard)
		.with_style(FileDialogStyle::Open | FileDialogStyle::FileMustExist)
		.build();
	if dialog.show_modal() == ID_OK { dialog.get_path() } else { None }
}
