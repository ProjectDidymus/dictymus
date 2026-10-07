//! The translation catalogs embedded in the binary.

/// Compiles each `po/*.po` into `locale/<lang>/LC_MESSAGES/dictymus.mo` for
/// `patois::embed_domain!` to pick up. Needs `msgfmt` on `PATH`; degrades to a
/// warning and an untranslated binary without it. Regenerating
/// `po/dictymus.pot` is a separate step: `cargo gen-pot`.
pub fn build() {
	patois_build::compile_translations("../../po", "locale");
}
