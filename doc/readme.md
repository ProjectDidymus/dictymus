# Dictymus User Guide

Dictymus is a dictionary program, primarily meant for Biblical Hebrew and Greek, made by
[Project Didymus](https://didymus.info/).
It is built for people who use a screen reader, and you can do everything in it with the keyboard.

Dictymus comes without dictionaries of its own.
It opens dictionaries in these formats:

- StarDict: open the `.ifo` file.
  The `.idx` and `.dict` or `.dict.dz` files with the same name must be in the same folder.
- MDict: open the `.mdx` file.
- Dictymus: a single `.dicty` file.

Each dictionary opens in its own tab.
Dictymus recognizes whether a dictionary is Hebrew or Greek, and in a Hebrew or Greek dictionary you type with the
Logos keyboard layout on your normal keyboard.
For dictionaries you can use with Dictymus, see [didymus.info](https://didymus.info/software/dictymus/).

## System requirements

- Windows 10 or 11, on an x64 or arm64 processor.
- macOS 11 or later, on a Mac with Apple silicon.

## Installing

### Windows

Download Dictymus from [didymus.info](https://didymus.info/software/dictymus/).
There are two versions:

- The installer, `dictymus_setup-x64.exe` or `dictymus_setup-arm64.exe`.
  It can create shortcuts in the Start menu and on the desktop, and open `.ifo`, `.mdx` and `.dicty` files with
  Dictymus when you double-click them.
  It also installs the Microsoft Edge WebView2 Runtime, which shows the articles, if Windows doesn't have it yet.
- The zip file, `dictymus-x64.zip` or `dictymus-arm64.zip`.
  Unzip it to a folder you can write to and start `dictymus.exe`; nothing needs to be installed.
  Your settings are still stored in your Windows user account, not next to `dictymus.exe`.

### macOS

Download `dictymus-macos.dmg`, open it, and drag Dictymus to the Applications folder.

Dictymus isn't signed with an Apple Developer ID yet, so macOS blocks the first start.
After the message that Apple could not verify Dictymus, open System Settings, choose Privacy & Security, go to the
Security section, and choose Open Anyway.
If macOS instead says that Dictymus is damaged and can't be opened, run this command in Terminal:

```sh
xattr -d com.apple.quarantine /Applications/Dictymus.app
```

You only need to do this once for each copy you install from the disk image.

## Opening dictionaries

To open a dictionary, choose File → Open (Ctrl+O) and pick its `.ifo`, `.mdx` or `.dicty` file.
On Windows you can also double-click the file, if the installer associated it with Dictymus, or drop it onto
`dictymus.exe` or its shortcut.
When Dictymus is already running, a file opened this way opens in a new tab of that window.

Each dictionary has its own tab.
Press Ctrl+Tab to go to the next tab and Ctrl+Shift+Tab to go back.
To close the current tab, choose File → Close (Ctrl+F4); File → Close All closes every tab.

When you quit, Dictymus remembers which dictionaries were open and opens them again the next time you start it.
If you start Dictymus by opening a dictionary file, only that dictionary opens.

## Searching

A dictionary tab has three parts, which you move between with Tab and Shift+Tab:

- Search: the field you type in.
- Lemmas: the list of entries that start with what you typed.
- Article: the dictionary article for the entry selected in the list.

The list follows what you type, letter by letter.
The first matching entry is selected and its article is shown, and your screen reader announces how many entries match.
Use the Up and Down arrows in the list to go through the entries; the article changes along with the selection.

You don't need to type vowels or accents:

- In a Hebrew dictionary, a word without vowel points finds every pointing of it.
  `dbr` finds <span lang="he">דָּבָר</span>, <span lang="he">דִּבֵּר</span> and <span lang="he">דֶּבֶר</span>.
  Each point you do type narrows the list: `dber` finds only <span lang="he">דֶּבֶר</span> and
  <span lang="he">דֹּבֶר</span>.
  Cantillation marks, meteg and rafe are always ignored.
- In a Greek dictionary, type the letters without accents or breathings.
- Final forms count as the ordinary letter, so <span lang="he">ם</span> finds <span lang="he">מ</span> and
  <span lang="grc">ς</span> finds <span lang="grc">σ</span>, and capitals count as small letters.

### Search history

Each tab remembers the entries you visited.
An entry is added when you press Enter in the search field, press Enter on it in the list, or leave it by following a
cross-reference.
In the search field, press Down to go back to an earlier entry and Up to come forward again, or open the field's list
with Alt+Down and pick one.
The history is cleared when you close the tab.

### Cross-references

Some articles link to other entries.
Follow a link the way your screen reader follows links in a web page.
The linked entry becomes the selected entry, its article is shown, and focus stays in the article.
If the dictionary has no such entry, your screen reader says "Not found" and the word.

## Typing Hebrew

In a Hebrew dictionary, the search field turns what you type into Hebrew, following the Logos Biblical Hebrew
keyboard layout.
Type a letter first and then its points.
Shift with a letter types a final form or a long vowel; Shift with a letter that has no Shift meaning types the same
as the letter alone.
Keep Caps Lock off: with Caps Lock on, letters type their Shift meaning.

In the tables below, the vowels and the dagesh are shown on the letter bet (<span lang="he">ב</span>).

### Consonants

| Letter | Name | Key |
| --- | --- | --- |
| <span lang="he">א</span> | alef | `'` |
| <span lang="he">ב</span> | bet | `B` |
| <span lang="he">ג</span> | gimel | `G` |
| <span lang="he">ד</span> | dalet | `D` |
| <span lang="he">ה</span> | he | `H` |
| <span lang="he">ו</span> | vav | `W` or `V` |
| <span lang="he">ז</span> | zayin | `Z` |
| <span lang="he">ח</span> | het | `J` or `Shift+H` |
| <span lang="he">ט</span> | tet | `F` or `Shift+T` |
| <span lang="he">י</span> | yod | `Y` |
| <span lang="he">כ</span> | kaf | `K` |
| <span lang="he">ל</span> | lamed | `L` |
| <span lang="he">מ</span> | mem | `M` |
| <span lang="he">נ</span> | nun | `N` |
| <span lang="he">ס</span> | samekh | `S` |
| <span lang="he">ע</span> | ayin | `` ` `` or `Shift+'` |
| <span lang="he">פ</span> | pe | `P` |
| <span lang="he">צ</span> | tsade | `C` |
| <span lang="he">ק</span> | qof | `Q` |
| <span lang="he">ר</span> | resh | `R` |
| <span lang="he">ש</span> | shin or sin without its dot | `X` or `Shift+S` |
| <span lang="he">שׂ</span> | sin | `[` |
| <span lang="he">שׁ</span> | shin | `]` |
| <span lang="he">ת</span> | tav | `T` |

### Final forms

| Letter | Name | Key |
| --- | --- | --- |
| <span lang="he">ך</span> | final kaf | `Shift+K` |
| <span lang="he">ם</span> | final mem | `Shift+M` |
| <span lang="he">ן</span> | final nun | `Shift+N` |
| <span lang="he">ף</span> | final pe | `Shift+P` |
| <span lang="he">ץ</span> | final tsade | `Shift+C` |

### Vowels

| Vowel | Name | Key |
| --- | --- | --- |
| <span lang="he">בַ</span> | patah | `A` |
| <span lang="he">בָ</span> | qamats | `O` or `Shift+A` |
| <span lang="he">בֶ</span> | segol | `E` |
| <span lang="he">בֵ</span> | tsere | `Shift+E` |
| <span lang="he">בִ</span> | hiriq | `I` or `Shift+I` |
| <span lang="he">בֹ</span> | holam | `Shift+O` |
| <span lang="he">בֻ</span> | qubuts | `U` |
| <span lang="he">וּ</span> | shureq | `Shift+U` |
| <span lang="he">בְ</span> | sheva | `,` or `Shift+,` |
| <span lang="he">בֲ</span> | hataf patah | `Shift+Z` |
| <span lang="he">בֱ</span> | hataf segol | `Shift+D` |
| <span lang="he">בֳ</span> | hataf qamats | `Shift+L` |

### Dagesh

| Mark | Name | Key |
| --- | --- | --- |
| <span lang="he">בּ</span> | dagesh | `.` or `Shift+.` |

## Typing Greek

In a Greek dictionary, the search field turns what you type into Greek, following the Logos Biblical Greek keyboard
layout.
Type the letters without accents or breathings; Dictymus finds the entry without them.
Shift with a letter types the capital, but you don't need capitals to search.

| Letter | Name | Key |
| --- | --- | --- |
| <span lang="grc">α</span> | alpha | `A` |
| <span lang="grc">β</span> | beta | `B` |
| <span lang="grc">γ</span> | gamma | `G` |
| <span lang="grc">δ</span> | delta | `D` |
| <span lang="grc">ε</span> | epsilon | `E` |
| <span lang="grc">ζ</span> | zeta | `Z` |
| <span lang="grc">η</span> | eta | `H` |
| <span lang="grc">θ</span> | theta | `Q` |
| <span lang="grc">ι</span> | iota | `I` |
| <span lang="grc">κ</span> | kappa | `K` |
| <span lang="grc">λ</span> | lambda | `L` |
| <span lang="grc">μ</span> | mu | `M` |
| <span lang="grc">ν</span> | nu | `N` |
| <span lang="grc">ξ</span> | xi | `X` |
| <span lang="grc">ο</span> | omicron | `O` |
| <span lang="grc">π</span> | pi | `P` |
| <span lang="grc">ρ</span> | rho | `R` |
| <span lang="grc">σ</span> | sigma | `S` |
| <span lang="grc">ς</span> | final sigma | `V` |
| <span lang="grc">τ</span> | tau | `T` |
| <span lang="grc">υ</span> | upsilon | `U` |
| <span lang="grc">φ</span> | phi | `F` |
| <span lang="grc">χ</span> | chi | `C` |
| <span lang="grc">ψ</span> | psi | `Y` |
| <span lang="grc">ω</span> | omega | `W` |

## Hebrew in braille

Dictymus can show Hebrew as braille, in the International Hebrew Braille Code.
To turn this on, choose File → Options, and in the Braille group check "Show Hebrew as ASCII braille".
In Hebrew dictionaries:

- The Lemmas list and the articles show Hebrew as braille cells, written as lowercase Braille ASCII.
  Your braille display shows them as braille.
  Text in other scripts stays as it is.
- The search field takes braille instead of the Logos layout: type the Braille ASCII for the cells.
  As with Hebrew letters, a plain consonant cell finds every pointing, and a vowel cell or a letter with dagesh
  narrows the list.

## Protected dictionaries

Some dictionaries open only with a personal license: a `.dictykey` file you receive from the publisher.
When you open a protected dictionary that no license unlocks yet, Dictymus asks whether you want to import a license
file.
Choose Yes and pick your `.dictykey` file.

You can also manage your licenses yourself.
Choose File → Licenses (Ctrl+L) to see each installed license, who it was issued to, when, and which dictionaries it
unlocks.
Import adds a license file, and Remove deletes the selected license, after asking you to confirm.

A license also works without importing it, if it has the same name as the dictionary and is in the same folder: for
example `Lexicon.dictykey` next to `Lexicon.dicty`.

## Options

Choose File → Options to change these settings:

- Language: the language of Dictymus itself, English or Dutch.
  "System default" follows the language of your system.
  A change takes full effect after you restart Dictymus.
- Check for updates on startup: whether Dictymus looks for a new version each time it starts.
- Update channel: Stable or Development; see [Updates](#updates).
- Braille: whether Hebrew is shown as braille; see [Hebrew in braille](#hebrew-in-braille).

## Updates

On Windows and macOS, Dictymus checks for a new version when it starts, unless you turned that off in Options.
To check yourself, choose Help → Check for Updates.
When there is a new version, Dictymus shows what's new and asks whether you want to update.
Every download is checked against its signature before it is used.

- On Windows, an installed copy updates by running the new installer.
  A copy from the zip file replaces its own `dictymus.exe`, so it must be in a folder you can write to.
- On macOS, Dictymus downloads the new version and opens its disk image.
  Quit Dictymus and drag the new version to the Applications folder.

There are two update channels:

- Stable: released versions.
  This is the default.
- Development: the newest build, made from each change to Dictymus.
  It gets new features and fixes first, but has had less testing.

## Keyboard shortcuts

| Action | Shortcut |
| --- | --- |
| Open a dictionary | Ctrl+O |
| Close the current dictionary | Ctrl+F4 |
| Licenses | Ctrl+L |
| Quit | Ctrl+Q |
| View this guide | F1 |
| Next tab | Ctrl+Tab |
| Previous tab | Ctrl+Shift+Tab |
| Move between Search, Lemmas and Article | Tab, Shift+Tab |
| Add the current entry to the history | Enter, in the search field or the list |
| Go back in the history | Down, in the search field |
| Go forward in the history | Up, in the search field |
| Open the history list | Alt+Down, in the search field |

On macOS, use Command instead of Ctrl for the menu shortcuts.
On many Mac keyboards, F1 needs the Fn key.

## Help and feedback

For questions about Dictymus, contact Project Didymus through [didymus.info/contact](https://didymus.info/contact/).
To report a bug or suggest an improvement, open an issue on [GitHub](https://github.com/ProjectDidymus/dictymus/issues).

## Credits and licenses

Dictymus is made by Project Didymus and is licensed under the MIT license.

Dictymus includes the SBL BibLit font from the
[Society of Biblical Literature](https://www.sbl-site.org/resources/fonts/), which is not covered by the MIT license.
Its license allows free non-commercial use.

The braille view uses the [louis-rs](https://github.com/liblouis/louis-rs) braille translator and Hebrew braille
tables from [liblouis](https://github.com/liblouis/liblouis), both licensed under the GNU Lesser General Public
License, version 2.1 or later.
Their source code is available at those links.
