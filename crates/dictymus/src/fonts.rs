use wxdragon::prelude::*;

pub const FACE: &str = "SBL BibLit";

/// Font bytes embedded at compile time — no external file needed at runtime.
pub static FONT_BYTES: &[u8] = include_bytes!("../../../assets/fonts/SBL_BLit.ttf");

/// Extract the embedded font to a temp file so wxWidgets can register it.
/// Returns the path on success; falls back to exe/cwd-relative lookup on failure.
pub fn font_path() -> String {
	// Try to write embedded bytes to temp dir
	if let Some(path) = font_path_from_embedded() {
		return path;
	}
	// Fallback: exe-relative
	if let Ok(exe) = std::env::current_exe() {
		if let Some(dir) = exe.parent() {
			let p = dir.join("assets/fonts/SBL_BLit.ttf");
			if p.exists() {
				return p.to_string_lossy().into_owned();
			}
		}
	}
	// Fallback: cwd-relative (cargo run from workspace root)
	let p = std::path::PathBuf::from("assets/fonts/SBL_BLit.ttf");
	if let Ok(abs) = p.canonicalize() {
		return abs.to_string_lossy().into_owned();
	}
	p.to_string_lossy().into_owned()
}

fn font_path_from_embedded() -> Option<String> {
	let dir = std::env::temp_dir().join("dictymus");
	std::fs::create_dir_all(&dir).ok()?;
	let path = dir.join("SBL_BLit.ttf");
	if !path.exists() {
		std::fs::write(&path, FONT_BYTES).ok()?;
	}
	Some(path.to_string_lossy().into_owned())
}

/// Register the bundled font with the OS and build a base Font.
/// Used for native widgets (TextCtrl, ListCtrl). The WebView uses CSS @font-face separately.
/// Returns the font plus a warning when the bundled font is unavailable —
/// degradation worth telling the user about, not a failure.
pub fn load_base_font() -> (Font, Option<String>) {
	let path = font_path();
	let registered = Font::add_private_font(&path);
	let font = Font::new_with_details(
		14,
		FontFamily::Default as i32,
		FontStyle::Normal as i32,
		FontWeight::Normal as i32,
		false,
		FACE,
	);
	match font {
		Some(font) if registered => (font, None),
		Some(font) => {
			(font, Some("Bundled font could not be registered; using system font".to_string()))
		}
		None => (Font::default(), Some("Bundled font unavailable; using system font".to_string())),
	}
}
