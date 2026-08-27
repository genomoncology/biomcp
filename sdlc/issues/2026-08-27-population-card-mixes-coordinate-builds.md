# The population card mixes coordinate builds between its header and its follow-up commands

`biomcp get variant rs1426654 population` resolves and displays a GRCh38
coordinate in the section header — `chr15:g.48134287A>G (dbSNP)` — but the
`More:` and `All:` follow-up commands beneath it print the GRCh37 spelling of
the same variant, `chr15:g.48426484A>G`. Both spellings parse and both fetch
(the parser serves either build), so nothing is broken functionally; a reader
who copies the header coordinate and compares it against a copied follow-up
command sees two different positions for one variant with no explanation of
the build switch.

Verified 2026-08-27 against 0.9.0-dev.6, twice, identical output (capture:
experiments/184-biomcp-slide-lab/calls/1062-gnomad-verify.txt). The header
resolution is new from ticket 1062's landing; the follow-up spelling
predates it.
