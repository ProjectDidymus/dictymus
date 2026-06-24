# Dictymus

Dictymus is a fast, native, fully accessible StarDict  dictionary reader.
Its primary goal is to support dictionaries for the biblical languages — Hebrew and Greek.
It is built from the ground up for blind and visually impaired users, with first-class screen reader support.

## Features

- Native desktop UI via [wxDragon](https://github.com/AllenDang/wxDragon) (wxWidgets), with full screen reader accessibility
- Reads StarDict dictionaries through [opendict-rs](https://github.com/callum-gander/opendict-rs)
- Tabbed interface for working with multiple dictionaries at once
- Live, incremental lemma search with on-the-fly transliteration: type with the Logos Biblical keyboard layout and get Hebrew or Greek glyphs
- Diacritic-insensitive matching: unpointed Hebrew and unaccented Greek queries match pointed/accented lemmas
- Automatic language detection per dictionary (Hebrew vs. Greek)
- Articles rendered in an embedded WebView with cross-reference links (`bword://` scheme)
- Bundled SBL BibLit font covering Hebrew, Greek, and Latin in a single face
- Remembers open dictionaries between sessions

## Workspace layout

This is a Cargo workspace. The main crates are:

| Crate | Description |
|---|---|
| `dictymus-core` | Pure logic: dictionary loading, language detection, normalization, transliteration, config (library) |
| `dictymus` | The GUI application (wxWidgets via wxDragon) |

## Requirements

- Rust (stable, edition 2024). Install via [rustup](https://rustup.rs).
- **MSVC toolchain** plus **CMake** and **Ninja** — required to compile wxWidgets via wxDragon.

> **All `cargo` commands must run inside a Visual Studio Developer Command Prompt** (or a shell where `vcvars64.bat` has been sourced), because `wxdragon-sys` builds wxWidgets with MSVC and Ninja through CMake.

## Building

```
cargo build --release
```

This produces the binary in `target/release/`. For a debug build of just the GUI:

```
cargo build -p dictymus
```

## Running

```
cargo run -p dictymus
```

Pass a dictionary's `.ifo` file to open it on startup:

```
cargo run -p dictymus -- <path/to/dictionary.ifo>
```

## Testing

```
cargo test                      # all unit tests
cargo test -p dictymus-core     # core logic only
```

UI smoke test (requires the winapp CLI):

```
pwsh winapp-tests/smoke.ps1 -Binary target/debug/dictymus.exe -Fixture <path/to/dictionary.ifo>
```

## Pre-commit hooks

This project uses [prek](https://github.com/j178/prek), a Rust-based pre-commit hook runner. Hooks are configured in `prek.toml`.

Install prek and set up the hooks:

```
cargo install prek
prek install
```

## License

Dictymus is licensed under the [MIT License](LICENSE).

The bundled SBL BibLit font is **not** covered by that license. It is distributed by the [Society of Biblical Literature](https://www.sbl-site.org/resources/fonts/) under the [SBL Font End User License Agreement](https://www.sbl-site.org/wp-content/uploads/2024/05/SBL_Font_End_User_License_Agreement.pdf), which permits free non-commercial use and unmodified redistribution. Commercial use requires a separate license from SBL.
