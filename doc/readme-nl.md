# Dictymus-handleiding

Dictymus is een woordenboekprogramma, vooral bedoeld voor Bijbels Hebreeuws en Grieks, gemaakt door
[Project Didymus](https://didymus.info/).
Het is gemaakt voor mensen die een schermlezer gebruiken, en je kunt er alles met het toetsenbord in doen.

Dictymus bevat zelf geen woordenboeken.
Het opent woordenboeken in deze formaten:

- StarDict: open het `.ifo`-bestand.
  De `.idx`- en `.dict`- of `.dict.dz`-bestanden met dezelfde naam moeten in dezelfde map staan.
- MDict: open het `.mdx`-bestand.
- Dictymus: één `.dicty`-bestand.

Elk woordenboek opent in een eigen tabblad.
Dictymus herkent of een woordenboek Hebreeuws of Grieks is, en in een Hebreeuws of Grieks woordenboek typ je met de
Logos-toetsenbordindeling op je gewone toetsenbord.
Voor woordenboeken die je met Dictymus kunt gebruiken, zie [didymus.info](https://didymus.info/software/dictymus/).

## Systeemvereisten

- Windows 10 of 11, op een x64- of arm64-processor.
- macOS 11 of later, op een Mac met Apple silicon.

## Installeren

### Windows

Download Dictymus van [didymus.info](https://didymus.info/software/dictymus/).
Er zijn twee versies:

- Het installatieprogramma, `dictymus_setup-x64.exe` of `dictymus_setup-arm64.exe`.
  Het kan snelkoppelingen maken in het menu Start en op het bureaublad, en `.ifo`-, `.mdx`- en `.dicty`-bestanden met
  Dictymus laten openen als je erop dubbelklikt.
  Het installeert ook de Microsoft Edge WebView2 Runtime, die de artikelen toont, als Windows die nog niet heeft.
- Het ZIP-bestand, `dictymus-x64.zip` of `dictymus-arm64.zip`.
  Pak het uit in een map waarin je mag schrijven en start `dictymus.exe`; er hoeft niets te worden geïnstalleerd.
  Je instellingen worden wel in je Windows-gebruikersaccount opgeslagen, niet naast `dictymus.exe`.

### macOS

Download `dictymus-macos.dmg`, open het en sleep Dictymus naar de map Programma's.

Dictymus is nog niet ondertekend met een Apple Developer ID, daarom blokkeert macOS de eerste keer opstarten.
Na de melding dat Apple Dictymus niet kon verifiëren, open je Systeeminstellingen, kies je Privacy en beveiliging, ga
je naar het gedeelte Beveiliging en kies je Open toch.
Als macOS in plaats daarvan meldt dat Dictymus beschadigd is en niet kan worden geopend, voer je dit commando uit in
Terminal:

```sh
xattr -d com.apple.quarantine /Applications/Dictymus.app
```

Dit hoef je maar één keer te doen voor elke kopie die je vanaf de schijfkopie installeert.

## Woordenboeken openen

Om een woordenboek te openen, kies je Bestand → Openen (Ctrl+O) en kies je het `.ifo`-, `.mdx`- of `.dicty`-bestand.
Op Windows kun je ook op het bestand dubbelklikken, als het installatieprogramma het aan Dictymus heeft gekoppeld, of
het op `dictymus.exe` of de snelkoppeling ervan slepen.
Als Dictymus al draait, opent een bestand dat je zo opent in een nieuw tabblad van dat venster.

Elk woordenboek heeft een eigen tabblad.
Druk op Ctrl+Tab om naar het volgende tabblad te gaan en op Ctrl+Shift+Tab om terug te gaan.
Om het huidige tabblad te sluiten, kies je Bestand → Sluiten (Ctrl+F4); Bestand → Alles sluiten sluit alle tabbladen.

Als je Dictymus afsluit, onthoudt het welke woordenboeken open waren en opent het die de volgende keer weer.
Als je Dictymus start door een woordenboekbestand te openen, opent alleen dat woordenboek.

## Zoeken

Een woordenboektabblad heeft drie delen, waartussen je met Tab en Shift+Tab wisselt:

- Zoeken: het veld waarin je typt.
- Lemma's: de lijst met lemma's die beginnen met wat je hebt getypt.
- Artikel: het woordenboekartikel bij het lemma dat in de lijst is geselecteerd.

De lijst volgt wat je typt, letter voor letter.
Het eerste passende lemma wordt geselecteerd en het artikel ervan wordt getoond, en je schermlezer meldt hoeveel
lemma's passen.
Ga met Omhoog en Omlaag door de lemma's in de lijst; het artikel verandert mee met de selectie.

Je hoeft geen klinkers of accenten te typen:

- In een Hebreeuws woordenboek vindt een woord zonder klinkertekens elke vocalisatie ervan.
  `dbr` vindt <span lang="he">דָּבָר</span>, <span lang="he">דִּבֵּר</span> en <span lang="he">דֶּבֶר</span>.
  Elk klinkerteken of dageesj dat je wel typt, maakt de lijst kleiner: `dber` vindt alleen
  <span lang="he">דֶּבֶר</span> en <span lang="he">דֹּבֶר</span>.
  Accenten, meteg en rafee tellen nooit mee.
- In een Grieks woordenboek typ je de letters zonder accenten en spiritus.
- Sluitletters tellen als de gewone letter, dus <span lang="he">ם</span> vindt <span lang="he">מ</span> en
  <span lang="grc">ς</span> vindt <span lang="grc">σ</span>, en hoofdletters tellen als kleine letters.

### Zoekgeschiedenis

Elk tabblad onthoudt de lemma's die je hebt bekeken.
Een lemma wordt toegevoegd als je in het zoekveld op Enter drukt, er in de lijst op Enter drukt, of het verlaat door
een kruisverwijzing te volgen.
Druk in het zoekveld op Omlaag om terug te gaan naar een eerder lemma en op Omhoog om weer vooruit te gaan, of open de
lijst van het veld met Alt+Omlaag en kies er een.
De geschiedenis wordt gewist als je het tabblad sluit.

### Kruisverwijzingen

Sommige artikelen bevatten links naar andere lemma's.
Volg een link zoals je schermlezer links op een webpagina volgt.
Het gelinkte lemma wordt het geselecteerde lemma, het artikel ervan wordt getoond en de focus blijft in het artikel.
Als het woordenboek zo'n lemma niet heeft, zegt je schermlezer "Niet gevonden" en het woord.

## Hebreeuws typen

In een Hebreeuws woordenboek zet het zoekveld wat je typt om in Hebreeuws, volgens de toetsenbordindeling Logos
Biblical Hebrew.
Typ eerst een letter en daarna de klinkertekens en de dageesj.
Shift met een letter typt een sluitletter of een lange klinker; Shift met een letter die geen Shift-betekenis heeft,
typt hetzelfde als de letter alleen.
Laat Caps Lock uit: als Caps Lock aan staat, typen letters hun Shift-betekenis.

In de tabellen hieronder staan de klinkers en de dageesj op de letter beet (<span lang="he">ב</span>).

### Medeklinkers

| Letter | Naam | Toets |
| --- | --- | --- |
| <span lang="he">א</span> | Alef | `'` |
| <span lang="he">ב</span> | Beet | `B` |
| <span lang="he">ג</span> | Gimel | `G` |
| <span lang="he">ד</span> | Dalet | `D` |
| <span lang="he">ה</span> | Hee | `H` |
| <span lang="he">ו</span> | Wav | `W` of `V` |
| <span lang="he">ז</span> | Zajin | `Z` |
| <span lang="he">ח</span> | Chet | `J` of `Shift+H` |
| <span lang="he">ט</span> | Tet | `F` of `Shift+T` |
| <span lang="he">י</span> | Jod | `Y` |
| <span lang="he">כ</span> | Kaf | `K` |
| <span lang="he">ל</span> | Lamed | `L` |
| <span lang="he">מ</span> | Mem | `M` |
| <span lang="he">נ</span> | Noen | `N` |
| <span lang="he">ס</span> | Samech | `S` |
| <span lang="he">ע</span> | Ajin | `` ` `` of `Shift+'` |
| <span lang="he">פ</span> | Pee | `P` |
| <span lang="he">צ</span> | Tsadee | `C` |
| <span lang="he">ק</span> | Kof | `Q` |
| <span lang="he">ר</span> | Reesj | `R` |
| <span lang="he">ש</span> | Sjien of Sien zonder punt | `X` of `Shift+S` |
| <span lang="he">שׂ</span> | Sien | `[` |
| <span lang="he">שׁ</span> | Sjien | `]` |
| <span lang="he">ת</span> | Tav | `T` |

### Sluitletters

| Letter | Naam | Toets |
| --- | --- | --- |
| <span lang="he">ך</span> | Kaf (sluitletter) | `Shift+K` |
| <span lang="he">ם</span> | Mem (sluitletter) | `Shift+M` |
| <span lang="he">ן</span> | Noen (sluitletter) | `Shift+N` |
| <span lang="he">ף</span> | Pee (sluitletter) | `Shift+P` |
| <span lang="he">ץ</span> | Tsadee (sluitletter) | `Shift+C` |

### Klinkers

| Klinker | Naam | Toets |
| --- | --- | --- |
| <span lang="he">בַ</span> | Patach | `A` |
| <span lang="he">בָ</span> | Kamets | `O` of `Shift+A` |
| <span lang="he">בֶ</span> | Segol | `E` |
| <span lang="he">בֵ</span> | Tseree | `Shift+E` |
| <span lang="he">בִ</span> | Chirek | `I` of `Shift+I` |
| <span lang="he">בֹ</span> | Cholem | `Shift+O` |
| <span lang="he">בֻ</span> | Kiboets | `U` |
| <span lang="he">וּ</span> | Sjoeroek | `Shift+U` |
| <span lang="he">בְ</span> | Sjwa | `,` of `Shift+,` |
| <span lang="he">בֲ</span> | Chateef patach | `Shift+Z` |
| <span lang="he">בֱ</span> | Chateef segol | `Shift+D` |
| <span lang="he">בֳ</span> | Chateef kamets | `Shift+L` |

### Dageesj

| Teken | Naam | Toets |
| --- | --- | --- |
| <span lang="he">בּ</span> | Dageesj | `.` of `Shift+.` |

## Grieks typen

In een Grieks woordenboek zet het zoekveld wat je typt om in Grieks, volgens de toetsenbordindeling Logos Biblical
Greek.
Typ de letters zonder accenten en spiritus; Dictymus vindt het lemma ook zonder.
Shift met een letter typt de hoofdletter, maar om te zoeken heb je geen hoofdletters nodig.

| Letter | Naam | Toets |
| --- | --- | --- |
| <span lang="grc">α</span> | Alfa | `A` |
| <span lang="grc">β</span> | Bèta | `B` |
| <span lang="grc">γ</span> | Gamma | `G` |
| <span lang="grc">δ</span> | Delta | `D` |
| <span lang="grc">ε</span> | Epsilon | `E` |
| <span lang="grc">ζ</span> | Dzèta | `Z` |
| <span lang="grc">η</span> | Èta | `H` |
| <span lang="grc">θ</span> | Thèta | `Q` |
| <span lang="grc">ι</span> | Iota | `I` |
| <span lang="grc">κ</span> | Kappa | `K` |
| <span lang="grc">λ</span> | Labda | `L` |
| <span lang="grc">μ</span> | Mu | `M` |
| <span lang="grc">ν</span> | Nu | `N` |
| <span lang="grc">ξ</span> | Xi | `X` |
| <span lang="grc">ο</span> | Omikron | `O` |
| <span lang="grc">π</span> | Pi | `P` |
| <span lang="grc">ρ</span> | Rho | `R` |
| <span lang="grc">σ</span> | Sigma | `S` |
| <span lang="grc">ς</span> | Sigma aan het eind van een woord | `V` |
| <span lang="grc">τ</span> | Tau | `T` |
| <span lang="grc">υ</span> | Upsilon | `U` |
| <span lang="grc">φ</span> | Fi | `F` |
| <span lang="grc">χ</span> | Chi | `C` |
| <span lang="grc">ψ</span> | Psi | `Y` |
| <span lang="grc">ω</span> | Omega | `W` |

## Hebreeuws in braille

Dictymus kan Hebreeuws in braille tonen, volgens de International Hebrew Braille Code.
Om dit aan te zetten, kies je Bestand → Opties en vink je in de groep Braille "Hebreeuws als HeBraille tonen" aan.
In Hebreeuwse woordenboeken:

- De lijst Lemma's en de artikelen tonen Hebreeuws als braillecellen, geschreven als Braille-ASCII in kleine letters.
  Je brailleleesregel toont ze als braille.
  Tekst in andere schriften blijft zoals hij is.
- Het zoekveld neemt braille in plaats van de Logos-indeling: typ de Braille-ASCII voor de cellen.
  Net als bij Hebreeuwse letters vindt een kale medeklinkercel elke vocalisatie, en maakt een klinkercel of een letter
  met dageesj de lijst kleiner.

## Beveiligde woordenboeken

Sommige woordenboeken openen alleen met een persoonlijke licentie: een `.dictykey`-bestand dat je van de uitgever
krijgt.
Als je een beveiligd woordenboek opent dat nog door geen enkele licentie wordt ontgrendeld, vraagt Dictymus of je een
licentiebestand wilt importeren.
Kies Ja en kies je `.dictykey`-bestand.

Je kunt je licenties ook zelf beheren.
Kies Bestand → Licenties (Ctrl+L) om elke geïnstalleerde licentie te zien: aan wie en wanneer die is uitgegeven, en
welke woordenboeken die ontgrendelt.
Importeren voegt een licentiebestand toe, en Verwijderen verwijdert de geselecteerde licentie, nadat je dat hebt
bevestigd.

Een licentie werkt ook zonder importeren, als die dezelfde naam heeft als het woordenboek en in dezelfde map staat:
bijvoorbeeld `Lexicon.dictykey` naast `Lexicon.dicty`.

## Opties

Kies Bestand → Opties om deze instellingen te wijzigen:

- Taal: de taal van Dictymus zelf, Engels of Nederlands.
  "Systeemstandaard" volgt de taal van je systeem.
  Een wijziging wordt volledig van kracht nadat je Dictymus opnieuw hebt gestart.
- Bij het opstarten controleren op updates: of Dictymus bij elke start naar een nieuwe versie zoekt.
- Updatekanaal: Stabiel of Ontwikkeling; zie [Updates](#updates).
- Braille: of Hebreeuws als braille wordt getoond; zie [Hebreeuws in braille](#hebreeuws-in-braille).

## Updates

Op Windows en macOS controleert Dictymus bij het opstarten op een nieuwe versie, tenzij je dat in Opties hebt
uitgezet.
Om zelf te controleren, kies je Help → Controleren op updates.
Als er een nieuwe versie is, toont Dictymus wat er nieuw is en vraagt het of je wilt bijwerken.
Elke download wordt met zijn handtekening gecontroleerd voordat die wordt gebruikt.

- Op Windows wordt een geïnstalleerde kopie bijgewerkt door het nieuwe installatieprogramma uit te voeren.
  Een kopie uit het ZIP-bestand vervangt zijn eigen `dictymus.exe`, dus die moet in een map staan waarin je mag
  schrijven.
- Op macOS downloadt Dictymus de nieuwe versie en opent het de schijfkopie ervan.
  Sluit Dictymus af en sleep de nieuwe versie naar de map Programma's.

Er zijn twee updatekanalen:

- Stabiel: uitgebrachte versies.
  Dit is de standaard.
- Ontwikkeling: de nieuwste build, gemaakt bij elke wijziging aan Dictymus.
  Die krijgt nieuwe functies en oplossingen als eerste, maar is minder getest.

## Sneltoetsen

| Actie | Sneltoets |
| --- | --- |
| Een woordenboek openen | Ctrl+O |
| Het huidige woordenboek sluiten | Ctrl+F4 |
| Licenties | Ctrl+L |
| Afsluiten | Ctrl+Q |
| Deze handleiding bekijken | F1 |
| Volgend tabblad | Ctrl+Tab |
| Vorig tabblad | Ctrl+Shift+Tab |
| Wisselen tussen Zoeken, Lemma's en Artikel | Tab, Shift+Tab |
| Het huidige lemma aan de geschiedenis toevoegen | Enter, in het zoekveld of de lijst |
| Terug in de geschiedenis | Omlaag, in het zoekveld |
| Vooruit in de geschiedenis | Omhoog, in het zoekveld |
| De geschiedenislijst openen | Alt+Omlaag, in het zoekveld |

Gebruik op macOS Command in plaats van Ctrl voor de sneltoetsen van de menu's.
Op veel Mac-toetsenborden heb je voor F1 de Fn-toets nodig.

## Hulp en feedback

Voor vragen over Dictymus neem je contact op met Project Didymus via
[didymus.info/contact](https://didymus.info/contact/).
Om een fout te melden of een verbetering voor te stellen, open je een issue op
[GitHub](https://github.com/ProjectDidymus/dictymus/issues).

## Credits en licenties

Dictymus is gemaakt door Project Didymus en valt onder de MIT-licentie.

Dictymus bevat het lettertype SBL BibLit van de
[Society of Biblical Literature](https://www.sbl-site.org/resources/fonts/), dat niet onder de MIT-licentie valt.
De licentie ervan staat gratis niet-commercieel gebruik toe.

De HeBraille-weergave gebruikt de braillevertaler [louis-rs](https://github.com/liblouis/louis-rs) en Hebreeuwse
brailletabellen van [liblouis](https://github.com/liblouis/liblouis), beide gelicentieerd onder de GNU Lesser General
Public License, versie 2.1 of later.
De broncode ervan is via die links beschikbaar.
