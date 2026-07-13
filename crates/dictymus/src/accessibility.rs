use wxdragon::prelude::*;
use wxdragon::widgets::statusbar::StatusBar;

/// Register the status bar's first pane (child ID 1) as a polite live region
/// via IAccPropServices. Call once at startup after creating the status bar.
#[cfg(target_os = "windows")]
pub fn init_status_bar_live_region(status_bar: StatusBar) {
	use std::mem::ManuallyDrop;
	use windows::Win32::Foundation::HWND;
	use windows::Win32::System::Com::{
		CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
	};
	use windows::Win32::System::Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4};
	use windows::Win32::UI::Accessibility::{
		CLSID_AccPropServices, IAccPropServices, LiveSetting_Property_GUID,
	};
	use windows::Win32::UI::WindowsAndMessaging::OBJID_CLIENT;
	// Child ID 1 = first pane of the status bar (the static text content).
	// Child ID 0 (CHILDID_SELF) is the status bar container itself.
	const FIRST_PANE: u32 = 1;
	const LIVE_REGION_POLITE: i32 = 1;
	let raw = status_bar.get_handle();
	if raw.is_null() {
		return;
	}
	let hwnd = HWND(raw as *mut _);
	unsafe {
		let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
		// RPC_E_CHANGED_MODE (0x80010106) is OK — already initialized
		if hr.is_err() && hr.0 != -2147417850i32 {
			return;
		}
		let Ok(svc): Result<IAccPropServices, _> =
			CoCreateInstance(&CLSID_AccPropServices, None, CLSCTX_INPROC_SERVER)
		else {
			return;
		};
		let variant = VARIANT {
			Anonymous: VARIANT_0 {
				Anonymous: ManuallyDrop::new(VARIANT_0_0 {
					vt: VT_I4,
					wReserved1: 0,
					wReserved2: 0,
					wReserved3: 0,
					Anonymous: VARIANT_0_0_0 { lVal: LIVE_REGION_POLITE },
				}),
			},
		};
		let _ = svc.SetHwndProp(
			hwnd,
			OBJID_CLIENT.0.cast_unsigned(),
			FIRST_PANE,
			LiveSetting_Property_GUID,
			&variant,
		);
	}
}

#[cfg(not(target_os = "windows"))]
pub fn init_status_bar_live_region(_status_bar: StatusBar) {}

/// Update the status bar text and fire a live-region change notification on
/// the first pane (child ID 1) so screen readers announce the new text.
pub fn announce_status(frame: Frame, status_bar: StatusBar, msg: &str) {
	frame.set_status_text(msg, 0);
	#[cfg(target_os = "windows")]
	{
		use windows::Win32::Foundation::HWND;
		use windows::Win32::UI::Accessibility::NotifyWinEvent;
		use windows::Win32::UI::WindowsAndMessaging::{
			EVENT_OBJECT_LIVEREGIONCHANGED, OBJID_CLIENT,
		};
		const FIRST_PANE: i32 = 1;
		let raw = status_bar.get_handle();
		if !raw.is_null() {
			let hwnd = HWND(raw as *mut _);
			unsafe {
				NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, hwnd, OBJID_CLIENT.0, FIRST_PANE);
			}
		}
	}
}
