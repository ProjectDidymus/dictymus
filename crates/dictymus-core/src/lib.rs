pub mod config;
pub mod dictionary;
pub mod language;
pub mod normalize;
pub mod testing;
pub mod transliterate;

/// Translates library-internal user-visible strings via the default patois
/// domain registered by the host app.
pub(crate) fn t(s: &str) -> String {
	patois::t(s)
}
