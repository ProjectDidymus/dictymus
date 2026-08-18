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
- Optional ASCII braille view for Hebrew: lemmas and articles are shown in
  International Hebrew Braille Code cells (Braille ASCII, lowercase), and the
  search field takes ASCII braille input directly
- Articles rendered in an embedded WebView with cross-reference links (`bword://` scheme)
- Bundled SBL BibLit font covering Hebrew, Greek, and Latin in a single face
- Remembers open dictionaries between sessions
- Automatic updates on Windows and update notifications on macOS, with
  minisign-verified downloads
- A single-file `.dicty` container format for distributing dictionaries,
  optionally sealed (encrypted) with per-user license files — see
  [Protected dictionaries](#protected-dictionaries)

## Workspace layout

This is a Cargo workspace. The main crates are:

| Crate | Description |
| --- | --- |
| `dictymus-core` | Pure logic: dictionary loading, language detection, normalization, transliteration, config |
| `dictymus-container` | The `.dicty` container and `.dictykey` license formats, plus the publisher CLI |
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

## Installing on macOS

Download `dictymus-macos.dmg` (Apple silicon, macOS 11 or later) from the
[releases page](https://github.com/ProjectDidymus/dictymus/releases), open it,
and drag **Dictymus** to the **Applications** folder.

Builds are not yet signed with an Apple Developer ID, so Gatekeeper blocks the
first launch. After the "Apple could not verify" dialog, open **System
Settings → Privacy & Security**, scroll to the Security section, and press
**Open Anyway** (macOS 15 removed the older right-click → Open shortcut). If
you instead get *"Dictymus is damaged and can't be opened"*, clear the
quarantine flag in Terminal:

```sh
xattr -d com.apple.quarantine /Applications/Dictymus.app
```

This is needed once per install from the DMG. Updates installed from the
in-app update download are not quarantined and launch without any dialog.

## Automatic updates

On Windows and macOS, Dictymus checks for updates silently at startup and on
demand via **Help → Check for Updates**. Downloads are verified with a
[minisign](https://jedisct1.github.io/minisign/) signature before anything is
executed. On Windows, installed copies (via the setup program) update through
a silent reinstall; portable copies swap the executable in place, which
requires the executable to live in a user-writable folder. On macOS, the
verified update is downloaded and Dictymus points you at it; replacing the
app in Applications is a manual step for now.

Two release channels exist:

- **stable** — tagged releases (the default for installer/release builds)
- **dev** — a rolling prerelease rebuilt on every push to master (the default
  for development builds)

Both behaviors are configurable in `%APPDATA%\dictymus\config.toml`
(Windows) or `~/Library/Application Support/dictymus/config.toml` (macOS):

```toml
check_for_updates_on_startup = true
update_channel = ""   # "" = follow the build type, or pin "stable" / "dev"
```

Setting the `DICTYMUS_NO_UPDATE_CHECK` environment variable suppresses the
startup check (used by the UI tests).

## Protected dictionaries

Dictymus can read dictionaries packaged as single-file `.dicty` containers.
A container wraps a normal StarDict fileset and is either **unsealed**
(plain, openable by anyone) or **sealed**: the payload is encrypted with
XChaCha20-Poly1305 under a per-container content key.

Sealed dictionaries are unlocked by a `.dictykey` license file. A license
names its owner, is Ed25519-signed by the publisher (verified offline
against a key embedded in the app), and carries one or more *scope* keys.
Each sealed container lists the scopes that can unlock it, so one suite
license can open several dictionaries — including dictionaries published
later under the same scope, without reissuing the license. Licenses are
installed via File → Install License, or by placing the `.dictykey` next
to the `.dicty` file.

Containers and licenses are built with the `dictymus-container` CLI
(`keygen`, `pack`, `seal`, `license`, `inspect`); run any subcommand with
`--help` for usage.

**Honest limits.** Dictymus is open source and decrypts on the user's
machine, so a determined user can extract dictionary content — this
protection cannot be stronger than the DRM on commercial e-books. What it
provides rights holders is deterrence and traceability: content is
unreadable to other tools, every license names its buyer, and altering
that name breaks both the signature and the key unwrap. Scope keys are
shared secrets by design; an extracted scope key unlocks every current and
future dictionary in that scope, so scope granularity (per-dictionary vs.
suite) bounds the blast radius.

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

   cargo-release runs fmt, clippy, and the non-GUI crates' tests (the GUI crate is
   built by CI, so no VS Developer Prompt is needed), bumps all three crate
   versions in lockstep, updates `Cargo.lock`, stamps `CHANGELOG.md`, commits,
   tags `X.Y.Z`, and pushes.
3. The pushed tag triggers CI, which builds every target, signs the updater
   assets with minisign, packages the Windows installers and the macOS app
   bundle (`dictymus-macos.dmg` for people, `dictymus-macos.zip` for the
   updater), and publishes the GitHub release with the changelog section as
   its body.

CI also carries dormant macOS code-signing and notarization steps. They
activate as soon as these repository secrets exist, with no workflow change:
`MACOS_CERTIFICATE_P12_BASE64`, `MACOS_CERTIFICATE_PASSWORD`,
`MACOS_KEYCHAIN_PASSWORD` (Developer ID Application certificate), and
`APPSTORE_API_KEY_BASE64`, `APPSTORE_API_KEY_ID`, `APPSTORE_API_ISSUER_ID`
(App Store Connect API key for `notarytool`).

## License

Dictymus is licensed under the [MIT License](LICENSE).

The bundled SBL BibLit font is **not** covered by that license. It is
distributed by the [Society of Biblical Literature](https://www.sbl-site.org/resources/fonts/)
under the [SBL Font End User License Agreement](https://www.sbl-site.org/wp-content/uploads/2024/05/SBL_Font_End_User_License_Agreement.pdf),
which permits free non-commercial use and unmodified redistribution.
Commercial use requires a separate license from SBL.

The ASCII braille view links the [louis-rs](https://github.com/liblouis/louis-rs)
braille translator and embeds Hebrew braille tables from
[liblouis](https://github.com/liblouis/liblouis)
(`crates/dictymus-core/assets/braille-tables/`). Both are licensed under the
[GNU LGPL 2.1 or later](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html);
their source is available at the linked repositories.
