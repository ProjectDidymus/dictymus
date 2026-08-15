//! Modal Licenses dialog: a report-view list of the installed licenses
//! with Import and Remove actions.

use patois::t;
use std::cell::RefCell;
use std::rc::Rc;
use wxdragon::id::{ID_CANCEL, ID_YES};
use wxdragon::prelude::*;

use crate::licensing::{self, InstalledLicense};

const DIALOG_PADDING: i32 = 5;

pub fn show_license_manager(parent: &Frame) {
	// TRANSLATORS: Title of the Licenses dialog
	let dialog = Dialog::builder(parent, &t("Licenses")).build();

	// TRANSLATORS: Label of the list of installed licenses
	let list_label_text = t("Installed &licenses:");
	let list_label = StaticText::builder(&dialog).with_label(&list_label_text).build();
	let list = ListCtrl::builder(&dialog)
		.with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
		.with_size(Size::new(480, 200))
		.build();
	// TRANSLATORS: Column header in the Licenses dialog: who the license was issued to
	list.insert_column(0, &t("Licensee"), ListColumnFormat::Left, 180);
	// TRANSLATORS: Column header in the Licenses dialog: license issue date
	list.insert_column(1, &t("Issued"), ListColumnFormat::Left, 100);
	// TRANSLATORS: Column header in the Licenses dialog: the dictionaries the license unlocks
	list.insert_column(2, &t("Dictionaries"), ListColumnFormat::Left, 180);
	#[cfg(target_os = "macos")]
	list.set_accessibility_label(list_label_text.replace('&', "").trim_end_matches(':').trim());

	// TRANSLATORS: Button in the Licenses dialog importing a license file
	let import_button = Button::builder(&dialog).with_label(&t("&Import...")).build();
	// TRANSLATORS: Button in the Licenses dialog removing the selected license
	let remove_button = Button::builder(&dialog).with_label(&t("&Remove")).build();
	// TRANSLATORS: Label of the button closing the Licenses dialog
	let close_button = Button::builder(&dialog).with_id(ID_CANCEL).with_label(&t("Close")).build();
	dialog.set_escape_id(ID_CANCEL);
	// Enter must close the dialog, never trigger an import or a removal.
	close_button.set_default();

	let rows: Rc<RefCell<Vec<InstalledLicense>>> = Rc::new(RefCell::new(Vec::new()));
	let refresh: Rc<dyn Fn()> = {
		let rows = Rc::clone(&rows);
		Rc::new(move || {
			let licenses = licensing::list_licenses(&licensing::license_pubkey());
			list.delete_all_items();
			for (i, license) in licenses.iter().enumerate() {
				let index = i as i64;
				list.insert_item(index, &license.licensee, None);
				list.set_item_text_by_column(index, 1, &license.issued);
				list.set_item_text_by_column(index, 2, &license.scope_ids.join(", "));
			}
			let has_rows = !licenses.is_empty();
			rows.replace(licenses);
			if has_rows {
				let sel_focused = ListItemState::Selected | ListItemState::Focused;
				list.set_item_state(0, sel_focused, sel_focused);
				list.ensure_visible(0);
			}
			remove_button.enable(has_rows);
		})
	};
	refresh();

	list.on_item_selected(move |_| remove_button.enable(true));
	list.on_item_deselected(move |_| remove_button.enable(list.get_first_selected_item() >= 0));

	{
		let rows = Rc::clone(&rows);
		let refresh = Rc::clone(&refresh);
		import_button.on_click(move |_| {
			let Some(picked) = crate::dialogs::pick_license(&dialog) else { return };
			match licensing::install_license(
				std::path::Path::new(&picked),
				&licensing::license_pubkey(),
			) {
				Ok(installed) => {
					refresh();
					let index =
						rows.borrow().iter().position(|l| l.path == installed).unwrap_or(0) as i64;
					let sel_focused = ListItemState::Selected | ListItemState::Focused;
					list.set_item_state(index, sel_focused, sel_focused);
					list.ensure_visible(index);
					// Focus the list so the new row is announced.
					list.set_focus();
				}
				Err(e) => {
					tracing::warn!("import license failed: {e}");
					crate::dialogs::show_error(&dialog, &e);
				}
			}
		});
	}

	{
		let rows = Rc::clone(&rows);
		let refresh = Rc::clone(&refresh);
		remove_button.on_click(move |_| {
			let sel = list.get_first_selected_item();
			if sel < 0 {
				return;
			}
			// Clone out of the RefCell so no borrow is held across the modals.
			let Some((path, licensee)) =
				rows.borrow().get(sel as usize).map(|l| (l.path.clone(), l.licensee.clone()))
			else {
				return;
			};
			// TRANSLATORS: Yes/No confirmation before deleting a license; the placeholder is the licensee name
			let msg = t("Remove the license for {}? Dictionaries it unlocks will no longer open.")
				.replace("{}", &licensee);
			// TRANSLATORS: Title of the remove-license confirmation dialog
			let title = t("Remove license");
			let confirmed = MessageDialog::builder(&dialog, &msg, &title)
				.with_style(
					MessageDialogStyle::YesNo
						| MessageDialogStyle::IconQuestion
						| MessageDialogStyle::Centre,
				)
				.build()
				.show_modal()
				== ID_YES;
			if !confirmed {
				return;
			}
			if let Err(e) = licensing::remove_license(&path) {
				tracing::warn!("remove license failed: {e}");
				crate::dialogs::show_error(&dialog, &e);
			}
			refresh();
			// Never leave focus on the Remove button once it is disabled.
			if list.get_item_count() == 0 {
				import_button.set_focus();
			}
		});
	}

	let content_sizer = BoxSizer::builder(Orientation::Vertical).build();
	content_sizer.add(&list_label, 0, SizerFlag::All, DIALOG_PADDING);
	content_sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, DIALOG_PADDING);
	let button_sizer = BoxSizer::builder(Orientation::Horizontal).build();
	button_sizer.add_stretch_spacer(1);
	button_sizer.add(&import_button, 0, SizerFlag::All, DIALOG_PADDING);
	button_sizer.add(&remove_button, 0, SizerFlag::All, DIALOG_PADDING);
	button_sizer.add(&close_button, 0, SizerFlag::All, DIALOG_PADDING);
	content_sizer.add_sizer(&button_sizer, 0, SizerFlag::Expand, 0);
	dialog.set_sizer_and_fit(content_sizer, true);
	dialog.centre();

	// An empty list is a dead end; land on the only useful action instead.
	if list.get_item_count() > 0 {
		list.set_focus();
	} else {
		import_button.set_focus();
	}
	dialog.show_modal();
}
