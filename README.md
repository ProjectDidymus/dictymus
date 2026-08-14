# Dictymus

Dictymus is a fast, native, fully accessible StarDict and MDict dictionary reader.
Its primary goal is to support dictionaries for the biblical languages — Hebrew and Greek.
It is built from the ground up for blind and visually impaired users, with first-class screen reader support.

## Features

- Native desktop UI via [wxDragon](https://github.com/AllenDang/wxDragon) (wxWidgets), with full screen reader accessibility
- Reads StarDict and MDict dictionaries through [opendict-rs](https://github.com/callum-gander/opendict-rs);
  `.dict.dz` is read via random access (no full decompression, no side-by-side `.dict` file)
- Tabbed interface for working with multiple dictionaries at once
- Live, incremental lemma search with on-the-fly transliteration: type with the
  Logos Biblical keyboard layout and get Hebrew or Greek glyphs
- Diacritic-insensitive matching: unpointed Hebrew and unaccented Greek queries match pointed/accented lemmas
- Automatic language detection per dictionary (Hebrew vs. Greek)
- Articles rendered in an embedded WebView with cross-reference links (`bword://` scheme)
- Bundled SBL BibLit font covering Hebrew, Greek, and Latin in a single face
- Remembers open dictionaries between sessions
- Automatic updates on Windows, with minisign-verified downloads

## Workspace layout

This is a Cargo workspace. The main crates are:

| Crate | Description |
| --- | --- |
| `dictymus-core` | Pure logic: dictionary loading, language detection, normalization, transliteration, config |
| `dictymus` | The GUI application (wxWidgets via wxDragon) |

## Requirements

- Rust (stable, edition 2024). Install via [rustup](https://rustup.rs).
- **MSVC toolchain** plus **CMake** and **Ninja** — required to compile wxWidgets via wxDragon.

> **All `cargo` commands must run inside a Visual Studio Developer Command
> Prompt** (or a shell where `vcvars64.bat` has been sourced), because
> `wxdragon-sys` builds wxWidgets with MSVC and Ninja through CMake.

## Building

```sh
cargo build --release
```

This produces the binary in `target/release/`. For a debug build of just the GUI:

```sh
cargo build -p dictymus
```

## Running

```sh
cargo run -p dictymus
```

Pass a dictionary's `.ifo` (StarDict) or `.mdx` (MDict) file to open it on startup:

```sh
cargo run -p dictymus -- <path/to/dictionary.ifo>
cargo run -p dictymus -- <path/to/dictionary.mdx>
```

## Testing

```sh
cargo test                      # all unit tests
cargo test -p dictymus-core     # core logic only
```

UI smoke test (requires the winapp CLI):

```sh
pwsh winapp-tests/smoke.ps1 -Binary target/debug/dictymus.exe -Fixture <path/to/dictionary.ifo>
```

## Automatic updates

On Windows, Dictymus checks for updates silently at startup and on demand via
**Help → Check for Updates**. Downloads are verified with a
[minisign](https://jedisct1.github.io/minisign/) signature before anything is
executed. Installed copies (via the setup program) update through a silent
reinstall; portable copies swap the executable in place, which requires the
executable to live in a user-writable folder. macOS builds do not check for
updates.

Two release channels exist:

- **stable** — tagged releases (the default for installer/release builds)
- **dev** — a rolling prerelease rebuilt on every push to master (the default
  for development builds)

Both behaviors are configurable in `%APPDATA%\dictymus\config.toml`:

```toml
check_for_updates_on_startup = true
update_channel = ""   # "" = follow the build type, or pin "stable" / "dev"
```

Setting the `DICTYMUS_NO_UPDATE_CHECK` environment variable suppresses the
startup check (used by the UI tests).

## Pre-commit hooks

This project uses [prek](https://github.com/j178/prek), a Rust-based pre-commit hook runner. Hooks are configured in `prek.toml`.

Install prek and set up the hooks:

```sh
cargo install prek
prek install
```

## Releasing

Releases are driven by [cargo-release](https://github.com/crate-ci/cargo-release)
(`cargo install cargo-release`). Nothing is published to crates.io; the tag is
what CI builds and publishes.

1. Record notable changes under `## [Unreleased]` in [CHANGELOG.md](CHANGELOG.md).
2. From a clean `master`, preview (dry run is the default), then execute:

   ```sh
   cargo release patch             # or minor | major | <x.y.z>
   cargo release patch --execute
   ```

   cargo-release runs fmt, clippy, and the core/xtask tests (the GUI crate is
   built by CI, so no VS Developer Prompt is needed), bumps all three crate
   versions in lockstep, updates `Cargo.lock`, stamps `CHANGELOG.md`, commits,
   tags `vX.Y.Z`, and pushes.
3. The pushed tag triggers CI, which builds every target, signs the Windows
   updater assets, and publishes the GitHub release with the changelog section
   as its body.

## License

Dictymus is licensed under the [MIT License](LICENSE).

The bundled SBL BibLit font is **not** covered by that license. It is
distributed by the [Society of Biblical Literature](https://www.sbl-site.org/resources/fonts/)
under the [SBL Font End User License Agreement](https://www.sbl-site.org/wp-content/uploads/2024/05/SBL_Font_End_User_License_Agreement.pdf),
which permits free non-commercial use and unmodified redistribution.
Commercial use requires a separate license from SBL.
