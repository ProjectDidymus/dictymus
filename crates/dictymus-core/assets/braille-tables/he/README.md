# Hebrew braille tables (IHBC)

Vendored verbatim from [liblouis](https://github.com/liblouis/liblouis),
commit `e8c0c9a07cbdcae3cde5ea2f6a621af87107e6ac` (v3.38.0-39-ge8c0c9a0),
directory `tables/`. Licensed under the GNU LGPL 2.1 or later — see the header
of each file.

`hbo-ihbc-rules.uti` is the entry table: the 1946 International Hebrew Braille
Code for Classical Hebrew. The other files are its `include` closure. The
upstream top-level table `hbo.utb` additionally pulls in Latin/digit/UEB
character definitions and braille indicators; those are omitted because
Dictymus only feeds pure Hebrew-script runs to the translator.

To update: copy the same five files from a newer liblouis checkout and record
the new commit here.
