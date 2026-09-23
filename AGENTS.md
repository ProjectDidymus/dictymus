# AGENTS.md

This file provides guidance to coding agents working with code in this repository.

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
  desktop; run in CI; shared harness in `tests/common/`: type into the search
  field with `type_query` (ASCII, transliterated by the app) after
  `wait_for_search_focus`, arrows via `arrow_down`/`arrow_up`, row counts
  via `wait_for_item_count`; UIA `set_value` and the crate's `{down}` do not
  reach the combo box; one test per file, since each launches the app and
  takes the foreground)

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
- `language.rs` — `detect()` scanning word list; returns `"he"`, `"grc"`
  (Ancient Greek, ISO 639-3 — never `"el"`) or `"unknown"`
- `normalize.rs` — `SearchKey`: NFD→lowercase→fold finals, split into
  clusters of base char + significant marks (Hebrew accents, meteg, rafe and
  the upper/lower dots dropped; qamats qatan→qamats, holam haser→holam);
  `starts_with()` is the search's prefix match: bases equal, query marks ⊆
  lemma marks, so typed points narrow and untyped ones match any;
  `unpointed()` clears the marks
- `transliterate.rs` — `transliterate()`: Logos Biblical keyboard maps for
  Hebrew/Greek, plain + Shift planes (Shift entry for the key as typed first,
  then the plain entry for its lowercase); `Option<&str>` because shureq and
  `[`/`]` are two code points; plain `o` is qamats, Shift+o holam
- `braille.rs` — per-language ASCII braille: embedded liblouis IHBC tables
  (`assets/braille-tables/`, include-flattened into `louis-rs`'s
  `from_table_source`, forward and backward translators), Unicode-braille↔
  lowercase-BRF mapping, `to_ascii_braille()` / `braille_html()`
  (HTML-escaping text-node transform) / `search_key()` (back-translates an
  ASCII braille query into a `SearchKey`; drops a trailing dagesh cell,
  expands the hiriq-yod/tsere-yod cells itself, translates with a sentinel
  alef so the final-form rules leave the last letter alone, and strips the
  shin dot since the shin cell is ambiguous); louis-rs is pinned to an
  upstream main rev until the multipass fix (louis-rs #21) is released
- `config.rs` — `AppConfig` (open dictionary paths, update settings, braille
  languages, persisted via TOML in OS app-data dir) + `UpdateChannel`
- `history.rs` — `History`: capped, most-recent-first list of visited word
  indices (one per tab, session only)

**dictymus** — wxdragon UI:

- `app.rs` — `App` struct, startup (CLI arg / reopen config), menu wiring
- `menu.rs` — menu IDs + `create_menu_bar()`
- `tabs.rs` — `TabManager` + `DictionaryTab` (panel, search `ComboBox`, list,
  article WebView; `display_word()` for the lemma as shown, `current` = the
  rendered article's word index)
- `search_field.rs` — char-level transliteration + live list filtering
  (`apply_filter(tab, exact)` keys the field text — via `braille::search_key`
  in braille mode — and keeps the lemmas whose `SearchKey` starts with it; a
  history pick is recalled by exact index when the combo's selection matches
  the field's text)
- `search_history.rs` — glue between `History` and the search `ComboBox`:
  `push` (rebuilds the dropdown items), `commit` (Enter / list activation:
  records the shown lemma and shows it in the field); a link follow records
  only the lemma left; Down/Up in the field arrive as text events
- `lemma_list.rs` — `repopulate()` / `repopulate_at(row)` for virtual ListCtrl (`set_item_count` + `refresh_items`)
- `article_pane.rs` — `render_row()` + `wrap_html()` (WebView HTML injection) + `navigate_to()` + `percent_decode()`
- `options.rs` — Options dialog; the Braille group toggles
  `braille_languages` and re-renders open tabs (per-tab `braille` flag +
  display cache in `tabs.rs`; the search back-translates the ASCII braille
  query when on)
- `dialogs.rs` — File Open dialog, About dialog
- `accessibility.rs` — `announce_status()`: status bar text plus a screen
  reader announcement through the `live-region` crate (UIA notification on
  Windows, `Priority::Medium` so it queues behind speech) raised from a
  zero-size `StaticText` on the frame's panel (`create_announcer()`); the UI
  test harness records these with `common::Notifications`
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

**Search matching:** `SearchKey` compares cluster by cluster; an unpointed
query matches every pointing of a lemma, and every point or accent the query
carries must be on the lemma. Cross-references (`navigate_to`) try the pointed
target first, then its letters alone.

**Transliteration:** `transliterate` maps Logos Biblical keyboard layout keys
(plain and Shift planes) to Hebrew/Greek text on keypress; the authoritative
tables were dumped from the installed layouts (`ToUnicodeEx`), the manual is
`D:\logos\LogosBiblicalHebrewKeyboard\Logos Biblical Hebrew Keyboard.pdf`.

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
