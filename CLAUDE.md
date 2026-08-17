# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Dictymus — native Rust wxdragon desktop dictionary for biblical languages (Hebrew, Greek).
Cargo workspace with three crates: `dictymus-core` (pure logic), `dictymus-container`
(`.dicty` container + `.dictykey` license formats and publisher CLI), and `dictymus`
(wxdragon UI).
Python converter for source HTML → StarDict in `tools/converter/`.

First-class accessibility for blind or visually impaired users.

## Commands

**All `cargo` commands must run inside a VS Developer Command Prompt (or
`vcvars64.bat` environment) — wxdragon-sys requires MSVC + Ninja via CMake.**

Build and run:

- `cargo run -p dictymus` — run the app (optional `-- <path.ifo>` to load a dictionary)
- `cargo run -p dictymus -- <path.ifo>` — run with a dictionary file
- `cargo build` — build workspace
- `cargo build -p dictymus` — build GUI only

Tests:

- `cargo test` — all unit tests
- `cargo test -p dictymus-core` — core logic tests only
- `cargo test -p dictymus -- --ignored` — UI tests in `crates/dictymus/tests/`
  (drive the real GUI via the `uiautomation` crate; need an interactive
  desktop; run in CI; shared harness in `tests/common/`)

Test fixtures: `dictymus_core::testing` generates tiny StarDict sets
(Greek/Hebrew/Latin, public-domain words) — no real dictionaries needed.

Translations (need `xgettext`, `msgmerge` and `msgfmt` on `PATH` — MSYS2 UCRT64
`gettext` + `mingw-w64-ucrt-x86_64-gettext-tools` on Windows):

- `cargo gen-pot` — regenerate `po/dictymus.pot` from every crate tagged
  `[package.metadata.patois] translatable = true`, registry dependencies such as
  `ship-shape` included
- `cargo xtask translate` — regenerate the pot, then `msgmerge` it into every
  `po/*.po`
- `cargo build` only compiles `po/*.po` into the embedded
  `crates/dictymus/locale/<lang>/LC_MESSAGES/dictymus.mo` catalogs; it does not
  touch the pot
- retiring a msgid needs a manual delete from `po/dictymus.pot`:
  `patois-build` re-appends any entry the fresh scan misses, so that
  dependency strings survive regeneration
- translatable literals must stay on one source line; `xgettext` reads the
  sources as C, so `xtask/src/sanitize_rust.rs` blanks lifetimes, raw strings
  and multi-line literals before they reach it, and `gen-pot` fails if a
  blanked literal belonged to a `t(`/`nt(` call

Releasing:

- `cargo release <level> --execute` — lockstep version bump, `CHANGELOG.md`
  stamp, commit, tag `<version>` (no `v` prefix), push; the tag triggers the
  CI release (see README "Releasing")

## Architecture

**Workspace:** `crates/opendict-rs` (vendored, unchanged) + `crates/dictymus-core` + `crates/dictymus`.

**dictymus-core** — pure logic, no GUI:

- `dictionary.rs` — `DictHandle` wrapping `opendict::stardict::StarDictDictionary`
- `language.rs` — `detect()` scanning word list via `unicode-script`
- `normalize.rs` — `normalize_for_search()`: NFD→strip marks→NFC→lowercase→fold final sigma
- `transliterate.rs` — `transliterate_char()`: Logos Biblical keyboard maps for Hebrew/Greek
- `config.rs` — `AppConfig` (open dictionary paths, update settings, persisted via TOML in OS app-data dir) + `UpdateChannel`

**dictymus** — wxdragon UI:

- `app.rs` — `App` struct, startup (CLI arg / reopen config), menu wiring
- `menu.rs` — menu IDs + `create_menu_bar()`
- `tabs.rs` — `TabManager` + `DictionaryTab` (panel, search, list, article WebView)
- `search_field.rs` — char-level transliteration + live list filtering
- `lemma_list.rs` — `repopulate()` for virtual ListCtrl (`set_item_count` + `refresh_items`)
- `article_pane.rs` — `render_row()` + `wrap_html()` (WebView HTML injection) + `navigate_to()` + `percent_decode()`
- `dialogs.rs` — File Open dialog, About dialog
- `fonts.rs` — SBL BibLit font loading
- `update.rs` (Windows only) — auto-update glue over the `ship-shape` crate
  (GitHub Releases + minisign + silent Inno Setup handoff); channel defaults
  follow the build type via `DICTYMUS_IS_DEV`, `DICTYMUS_NO_UPDATE_CHECK` skips
  the startup check; config keys `check_for_updates_on_startup` / `update_channel`

**Article rendering:** `DictHandle::article_html()` returns raw HTML from
StarDict payload. `article_pane::wrap_html()` adds a CSS wrapper (SBL BibLit via
@font-face, direction, link colors) and injects into `WebView::set_page()`.
Cross-refs use `bword://WORD` scheme; intercepted by `on_navigating` handler.

**Font:** SBL BibLit (`assets/fonts/SBL_BLit.ttf`) — one face for Hebrew, Greek,
Latin. Loaded via `Font::add_private_font` for native widgets; via CSS
`@font-face file://` URL for WebView.

**Normalization:** `normalize_for_search` strips Hebrew points and Greek diacritics so unpointed queries match pointed lemmas.

**Transliteration:** `transliterate_char` maps Logos Biblical keyboard layout chars to Hebrew/Greek glyphs on keypress.

## Conventions

- Tabs for indentation in Rust source
- wxdragon `WxWidget` trait via `wxdragon::prelude::*`
- `WebViewEvents` must be imported explicitly: `use wxdragon::event::WebViewEvents;`
- Definition HTML is trusted (converter output); rendered via WebView `set_page`
- `Font::add_private_font` / `Font::new_with_details` (no `Font::builder`)
- WebView `.url()` builder method is `.with_url(Some("...".to_string()))`
- Menu events: `on_menu_selected` (not `on_menu`)
- Text change events: `on_text_updated` (not `on_text`)
- User-visible strings go through `patois::t`, counts through
  `patois::nt(singular, plural, n)`; placeholders are `{}` filled in with
  `.replace()`. Each call takes a `// TRANSLATORS:` comment on the line
  immediately above it — `tools/check_translators.py` (prek hook) fails the
  commit when rustfmt wrapping separates the two

## Converter contract

See `tools/converter/CONSTRAINED_HTML.md` for the HTML subset contract between converter and frontend.
