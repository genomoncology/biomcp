---
flow: build
priority: 4
---
# Surface a criteria specification's file attachments

## Done when

`biomcp gene cspec PTEN --version <iri> files` lists the five GN003
attachments with label, filename, media type, size and resolved URL,
and the criteria output states how many attachments exist. The listing
carries the same capture provenance the criteria already carry.

## Raise allowance

The attachment manifest reuses the capture machinery `gene cspec`
already has. The `src` line ceiling may rise by at most 150 lines.

## The finding

Raised as an issue during BioMCP research on 2026-08-08, then folded
in here and the issue file removed. Reproduced in full below.

<!-- from feature-clingen-criteria-specifications-as-an-entity.md -->

# Feature: `gene cspec` does not surface a specification's attachments

Severity: should-fix.

**Correction, 2026-08-08.** An earlier version of this file asked
for CSpec support as if it did not exist. It does —
`biomcp gene cspec <symbol>` returns the version manifest,
`--version <iri>` returns the parsed criteria with capture id and
sha256, and `gene cspec document <capture-id>` returns the exact
stored bytes. That is a better-built entity than the one this issue
originally proposed. What follows is the narrow gap that remains.

## The gap

A criteria specification defers real content to attached files, and
`gene cspec` does not mention them.

For PTEN GN003 v3.2.1 there are five, and two of them carry content
the specification cannot be applied without:

- PVS1's approved strength descriptors say only *"Use PTEN PVS1
  decision tree."* — three times, at Very Strong, Strong and
  Moderate. The tree is `PVS1_DecisionTree_PTEN.pdf`.
- The entire criteria-combining step — the last stage of any
  classification — is a screenshot,
  `Screen-Shot-2023-02-06-at-3.03.48-PM.png`.

The other three are a pediatric phenotype scoring sheet, the
Cleveland Clinic score table, and a BLOSUM matrix.

`gene cspec PTEN --version …/3.2.1` returns 328,450 bytes of
criteria and none of these. Grepping its output for the file ids or
`fileLabel` gives zero hits.

## Why it matters more than it sounds

Without them a reader concludes the files are unpublished. They are
not. The captured payload holds `File` entities marked
`"public": true`, each with an `entId`, and the registry serves them
at a stable URL:

    https://cspec.genome.network/cspec/File/id/<entId>/data

All five downloaded first try, no key. A downstream team had written
these off as unobtainable and was preparing to build PVS1 with the
tree left as an unfilled parameter.

## Shape

- `biomcp gene cspec <symbol> --version <iri> files` — the
  attachment manifest: label, filename, media type, size, and the
  resolved URL.
- Include the attachment list in the criteria output too, even as a
  bare count, so their existence is visible without a second call.
- Follow the entity's existing capture discipline: the files are
  part of the specification's bytes, so they want the same
  content-hash provenance the criteria already carry. That is the
  argument for doing this inside `gene cspec` rather than leaving it
  to the caller.

## A second, smaller ask — split out

The cross-specification search ask that used to be bundled here
(`biomcp search spec --criterion BS1`) moved to ticket 0894 on
2026-08-09 during review triage, so one flight does one thing. This
ticket is the attachment manifest only, exactly as the "Done when"
above states.

Raised 2026-08-08 from PTEN GN003 research for varclassify2.
