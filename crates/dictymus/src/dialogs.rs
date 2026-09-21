use patois::t;
use wxdragon::prelude::*;

/// Modal error dialog — the single error surface of the app. Modal so focus
/// moves into the dialog and screen readers read the message text.
pub fn show_error(parent: &dyn WxWidget, message: &str) {
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
	#[rustfmt::skip]
	// TRANSLATORS: Third-party license notice in the About dialog; the source code links follow on the next lines
	let braille_notice = t("The ASCII braille view uses the louis-rs braille translator and Hebrew braille tables from liblouis, both licensed under the GNU Lesser General Public License, version 2.1 or later. Their source code is available at:");
	let description = format!(
		"{}\n\n{}\nhttps://github.com/liblouis/louis-rs\nhttps://github.com/liblouis/liblouis",
		// TRANSLATORS: One-line app description in the About dialog
		t("An accessible dictionary for biblical languages"),
		braille_notice,
	);
	info.set_description(&description);
	show_about_box(&info, Some(parent));
}

pub fn pick_dictionary(parent: &Frame) -> Option<String> {
	use wxdragon::id::ID_OK;
	// TRANSLATORS: Title of the dictionary file picker dialog
	let message = t("Open dictionary");
	// Only the labels are translated; the glob patterns between the pipes are
	// part of the wx wildcard format and must stay as they are.
	let wildcard = format!(
		"{}|*.dicty;*.ifo;*.mdx|{}|*.dicty|{}|*.ifo|{}|*.mdx|{}|*.*",
		// TRANSLATORS: File picker filter label for all supported dictionary formats
		t("Dictionaries (*.dicty;*.ifo;*.mdx)"),
		// TRANSLATORS: File picker filter label for Dictymus dictionary containers
		t("Dictymus dictionaries (*.dicty)"),
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

pub fn pick_license(parent: &dyn WxWidget) -> Option<String> {
	use wxdragon::id::ID_OK;
	// TRANSLATORS: Title of the license file picker dialog
	let message = t("Import license");
	let wildcard = format!(
		"{}|*.dictykey|{}|*.*",
		// TRANSLATORS: File picker filter label for Dictymus license files
		t("Dictymus licenses (*.dictykey)"),
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
