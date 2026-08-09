---
flow: build
priority: 3
---
# Search across criteria specifications

Split out of ticket 0880 on 2026-08-09 during review triage: 0880
keeps the attachment manifest (its "Done when" already covered only
that); this ticket carries the second ask that was bundled with it.

## Done when

`biomcp search spec --criterion BS1` answers a question across
released criteria specifications from the captured corpus, with the
same capture provenance `gene cspec` output carries.

## The finding, from 0880's original text

There is no way to ask a question across specifications. Two that
came up during PTEN GN003 research for varclassify2 (2026-08-08):

- *How do panels write their frequency band edges?* Answered by
  downloading all 122 released specs and grepping: 96 of 117 BS1
  descriptors use an explicit operator, and where two bands share an
  edge, 13 write BA1 `≥X` / BS1 `<X`. That convention let a team
  defend a boundary they were about to guess.
- *Has any panel written a gnomAD quality-filter policy?* No — zero
  hits across all 122. A clean negative that changed what got built.

`biomcp search spec --criterion BS1` would make both a command
instead of a download-and-grep.
