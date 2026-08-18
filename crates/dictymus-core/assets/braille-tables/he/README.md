# Hebrew braille tables (IHBC)

Vendored verbatim from [liblouis](https://github.com/liblouis/liblouis),
commit `a3d985689484ace0dfba06249eaee57c46264ba4` (the head of
[liblouis#2068](https://github.com/liblouis/liblouis/pull/2068), which replaces
the positional `$w`-`$z` class references with named classes), directory
`tables/`. Licensed under the GNU LGPL 2.1 or later — see the header of each
file.

`hbo-ihbc-rules.uti` is the entry table: the 1946 International Hebrew Braille
Code for Classical Hebrew. The other files are its `include` closure. The
upstream top-level table `hbo.utb` additionally pulls in Latin/digit/UEB
character definitions and braille indicators; those are omitted because
Dictymus only feeds pure Hebrew-script runs to the translator.

To update: copy the same five files from a newer liblouis checkout and record
the new commit here.
