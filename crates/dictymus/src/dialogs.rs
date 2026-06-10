use wxdragon::prelude::*;

/// Modal error dialog — the single error surface of the app. Modal so focus
/// moves into the dialog and screen readers read the message text.
pub fn show_error(parent: &Frame, message: &str) {
	MessageDialog::builder(parent, message, "Dictymus - Error")
		.with_style(MessageDialogStyle::OK | MessageDialogStyle::IconError | MessageDialogStyle::Centre)
		.build()
		.show_modal();
}

pub fn show_about(parent: &Frame) {
	let mut info = AboutDialogInfo::new();
	info.set_name("Dictymus");
	info.set_version(env!("CARGO_PKG_VERSION"));
	info.set_description("An accessible dictionary for biblical languages");
	show_about_box(&info, Some(parent));
}

pub fn pick_dictionary(parent: &Frame) -> Option<String> {
	use wxdragon::id::ID_OK;
	let dialog = FileDialog::builder(parent)
		.with_message("Open dictionary")
		.with_wildcard("StarDict info (*.ifo)|*.ifo|All files (*.*)|*.*")
		.with_style(FileDialogStyle::Open | FileDialogStyle::FileMustExist)
		.build();
	if dialog.show_modal() == ID_OK { dialog.get_path() } else { None }
}
