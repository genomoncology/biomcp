# Feature: `variant erepo` has no gene-wide sweep and drops the guideline version

Severity: should-fix.

**Correction, 2026-08-08.** An earlier version of this file asked
for expert-panel assertions as if they were absent. They are not —
`biomcp variant erepo <CAid>` returns classification, condition,
MOI, VCEP, HGVS set, dates, summary text, and the met criteria at
their applied strength (`met: PM2`, `met: PS2_Very`). Two narrower
gaps remain, and together they block the analysis that made this
worth filing.

## Gap 1 — the guideline version is dropped

`variant erepo CA000559 --detail --json` reports
`"doc_version": "1.0.0"` and `"vcep": "PTEN VCEP"`. That is the
*assertion document's* version, not the specification version the
curators worked under.

The upstream record carries it. In the raw API response, each
assertion's `guidelines[].label` reads e.g. *"ClinGen PTEN Expert
Panel Specifications … Version 3.2.0"*, and older ones read
*"ACMG-PTEN Variant Curation Guideline"*. BioMCP parses the block
and discards the label.

That field is the whole analysis. The PTEN specification contradicts
itself on PM2 — Moderate approved but Not Applicable, Supporting
Applicable but not approved, no strength both. Unanswerable from the
document. Cut the assertions by guideline version and it resolves:

| Spec version | PM2 (Moderate) | PM2_Supporting |
|---|---|---|
| v1 | 44 | 0 |
| v2 | 9 | 0 |
| 3.0.0 | 0 | 53 |
| 3.1.0 | 16 | 34 |
| 3.2.0 | 4 | 28 |

The switch lands at 3.0.0, unanimously — and the residual bare-PM2
records are not stale carry-overs, since both strengths appear in
the same publication batches. So the honest answer includes a known
inconsistency rate. Without the version field the same data is an
undifferentiated 73-against-115 and says almost nothing.

Fix shape: carry `guideline_label` and a parsed `guideline_version`
alongside `doc_version`. Small change, large payoff.

## Gap 2 — no way to ask for a gene's assertions

`variant erepo` takes a CAID, or a batch of CAIDs via `--input`.
Both require already knowing which variants to ask about. There is
no `--gene`.

The upstream endpoint supports it directly:

    https://erepo.genome.network/evrepo/api/classifications?gene=PTEN&matchLimit=500

229 PTEN interpretations in one call. That is how the table above
was built, and there is no path to it through BioMCP today short of
enumerating CAIDs from somewhere else first.

Shape: `biomcp search assertion --gene PTEN [--code PM2]`, returning
the same per-assertion shape `variant erepo` already produces.

## What it also settles

The same sweep answers scope questions the specifications leave
open. `?gene=KLLN` returns six assertions under the PTEN panel's
guideline, all of them PTEN promoter variants, two curated within
the last year — though the PTEN specification's declared gene scope
is PTEN alone and the string `KLLN` appears nowhere in it. The gene
symbol is a downstream annotation, not the membership key. One
query; unreachable any other way.

Raised 2026-08-08 from PTEN GN003 research for varclassify2.
