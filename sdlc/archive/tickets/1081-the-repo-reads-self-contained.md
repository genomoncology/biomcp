---
flow: build
priority: 2
---
# The repo reads self-contained to an outside reader

Living content names private projects. The structural-variant experiment's
explore.md and harden.md name the Trials3/Nucleus alteration-grammar work.
Two open issues from 2026-08-26 cite capture files by their repos/mktg
paths. Active ticket 1060 cites the repos/rolodex CLI.

Required behavior: private ecosystem names leave living content. Replace
"Trials3/Nucleus alteration grammar" with a neutral phrase such as "a
downstream alteration-grammar consumer". Restate the issue capture
provenance without the private path, or copy the captures into the
experiment folder. History stays untouched: sdlc/records/ and
sdlc/tickets/archive/ keep their VarClassify citations. The genomoncology
org name stays wherever it is this repo's own public hosting, packaging, or
attribution.

## Done when

`git grep -niE "trials[0-9]|varclassify|picohr|imaurer|rolodex|mktg" -- ':!sdlc/records' ':!sdlc/tickets/archive' ':!testdata' ':!*fixtures*'`
returns no hits, and "nucleus" outside records and fixtures never names the
private project.

## Boundary

No history rewrites. No code behavior changes. "Nucleus" as the cell
organelle in biomedical text is correct and stays.
