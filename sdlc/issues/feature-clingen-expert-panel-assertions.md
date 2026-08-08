# Feature: expert panel assertions with their applied evidence codes

Severity: nice-to-have with a high ceiling. Pairs with
`feature-clingen-criteria-specifications-as-an-entity.md`.

## What is missing

`biomcp get variant … clinvar` gives the aggregate: a star rating, a
condition list with report counts, a variation id. What it cannot
give is **which criteria a submitter applied and at what strength**.

    - Review: 2 star(s) (criteria provided, multiple submitters, no conflicts)

That is the end of the trail. There is no per-submitter breakdown
and no evidence codes anywhere in the output.

ClinGen's Evidence Repository publishes exactly that, over an open
JSON API with no key:

    https://erepo.genome.network/evrepo/api/classifications?gene=PTEN&matchLimit=500

Each record carries the CAID, the condition, the guideline **and its
version**, the outcome, and every evidence code with its status
(`Met` / `Not Met`) at the strength that was applied — `PM2` versus
`PM2_Supporting`, `PVS1` versus `PVS1_Strong`, and so on.

## Shape

    biomcp variant assertions <id>          # panel assertions for one variant
    biomcp search assertion --gene PTEN     # every assertion for a gene
    biomcp search assertion --gene PTEN --code PM2

Report, per assertion: panel, guideline and version, published date,
outcome, and the code list. The version is not decoration — see
below.

## What this answers that nothing else does

When a specification's prose is ambiguous, the panel's own record of
practice settles it, and it settles it better than prose because it
is what they did rather than what they meant.

Worked example. The PTEN specification contradicts itself on PM2:
Moderate is approved but marked Not Applicable, Supporting is
Applicable but not approved, and Supporting carries the criterion
text. No strength is both. Unanswerable from the document. From the
assertions, cut by guideline version:

| Spec version | PM2 (Moderate) | PM2_Supporting |
|---|---|---|
| v1 | 44 | 0 |
| v2 | 9 | 0 |
| 3.0.0 | 0 | 53 |
| 3.1.0 | 16 | 34 |
| 3.2.0 | 4 | 28 |

The switch lands at 3.0.0, unanimously. And the residual bare-PM2
records are not stale carry-overs — both strengths appear in the
same publication batches — so the honest answer includes a known
inconsistency rate rather than a clean rule. A downstream calculator
that knows to expect ~15% divergence will not chase its own tail
when reproduction mismatches show up.

None of that is reachable without version-tagged evidence codes.

## A second thing it settles

The same API answers scope questions the specifications leave open.
Querying `?gene=KLLN` returns six assertions under the PTEN panel's
guideline, all of them PTEN promoter variants, two curated within
the last year — even though the PTEN specification's declared gene
scope is PTEN alone and the string `KLLN` appears nowhere in it. The
gene symbol is a downstream annotation, not the membership key. That
was found in one query and could not be found any other way.

Raised 2026-08-08 from PTEN GN003 research for varclassify2.
