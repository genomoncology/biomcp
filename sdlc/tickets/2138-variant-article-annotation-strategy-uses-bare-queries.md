---
flow: build
priority: 2
deps: [2130]
---

# 2138: Search PubTator by variant entity in the annotation strategy

Status: OPEN.

Milestone: 0.9.3

## Outcome

`variant articles --strategy annotation` searches PubTator3 by the variant's entity id, so "ABCC8 R1436Q" returns the papers PubTator annotates with that variant.

## Evidence

Reported 2026-10-16 by a literature probe of 0.9.2.

- `biomcp --json variant articles "ABCC8 R1436Q" --strategy annotation --limit 50` returns 0 results with `complete: true`.
- PubTator3 `/search/?text=@VARIANT_p.R1436Q_ABCC8_human` returns 7 papers; the same form gives 15 for CDKN2A A57V and 3 for SAMHD1 I448T.
- The debug plan shows bare queries such as `R1436Q` and `chr11:g.17417157C>T` with no entity id.

## Work

- Reproduce on main first.
- Build the annotation route's query from the PubTator variant entity id (`@VARIANT_p.<change>_<GENE>_human`) that autocomplete returns. Keep bare text queries out of this route.

## Acceptance

1. A recorded-reply test of the ABCC8 R1436Q route sends the entity-id query and returns the 7 papers; it fails on current main.
2. A run that finds no entity says so in the result instead of reporting `complete: true` with nothing.
3. The changelog carries a bullet for this ticket.

## Reproduction on main

Run 2026-10-17 on the macOS gate host at `43252305f` (origin/main tip), pm job
`1792206072419-57493`, live providers, `--no-cache`:

```
biomcp --json variant articles "ABCC8 R1436Q" --strategy annotation --limit 50 --no-cache
```

exits 0 with `"complete": true`, `"error": null`, `results: []`,
`"retrieval_path": "PubTator variant annotation recall"`, and the
`pubtator_variant/pubtator` route reporting `ok`. The debug-plan run of the same
command (pm job `1792206158984-70753`) plans only bare text queries for the
route — `chr11:g.17417157C>T`, `rs387906407`, `R1436Q`, and neighboring
misspellings — and no entity id, because the autocomplete reply's display name
carries only `p.R1436Q` while the gene lives in the entity id
`@VARIANT_p.R1436Q_ABCC8_human`, which the row matcher drops as an
indeterminate gene. The provider side is receipted:
`entity/autocomplete/?query=ABCC8%20R1436Q` returns that one variant entity, and
`search/?text=@VARIANT_p.R1436Q_ABCC8_human` returns 7 papers.
