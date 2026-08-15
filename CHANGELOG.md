# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-08-15

### Added

- Dictionaries can now come as a single `.dicty` file instead of a folder
  of separate files
- Protected dictionaries, which open only with the personal `.dictykey`
  license file you receive from the publisher
- A Licenses window (File → Licenses, Ctrl+L) listing the licenses you have
  installed, with buttons to import a new one or remove one you no longer
  need
- Opening a protected dictionary without its license now offers to import
  the license there and then; a license file kept next to the dictionary is
  also found automatically
- A command-line tool for publishers, to package dictionaries and issue
  licenses for them
- The Windows installer can offer to open `.dicty` files with Dictymus when
  you double-click them
- macOS now gets a real Dictymus application on a disk image you drag to
  your Applications folder, instead of a bare program file (Apple silicon,
  macOS 11 or later)
- macOS now checks for new versions at startup and through Help → Check for
  Updates; the download is verified, but you install it yourself

## [0.2.0] - 2026-08-15

### Added

- The user interface is now available in Dutch alongside English, following
  the Windows display language by default
- An Options dialog (File menu) with the interface language, whether to
  check for updates on startup, and the update channel
- The Windows installer can now also run in Dutch

### Fixed

- Opening a dictionary now places keyboard focus in the search field instead
  of the tab bar

## [0.1.0] - 2026-08-14

### Added

- Native, fully accessible desktop UI (wxWidgets via wxDragon) with
  first-class screen reader support
- StarDict and MDict dictionary reading through opendict-rs, including
  random-access `.dict.dz` without a side-by-side `.dict` file
- Tabbed interface for working with multiple dictionaries at once
- Live, incremental lemma search with on-the-fly Biblical keyboard
  transliteration for Hebrew and Greek, based on Logos keyboard layout
- Diacritic-insensitive matching: unpointed Hebrew and unaccented Greek
  queries match pointed and accented lemmas
- Automatic per-dictionary language detection (Hebrew or Greek)
- Articles rendered in an embedded WebView with `bword://` cross-reference
  links
- Bundled SBL BibLit font covering Hebrew, Greek, and Latin in a single face
- Open dictionaries remembered between sessions
- Automatic updates on Windows (stable and dev channels) with
  minisign-verified downloads
- Installer welcome page and a license agreement page covering the MIT
  License and the SBL BibLit font license
