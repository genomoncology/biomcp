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

## A second, smaller ask

There is no way to ask a question across specifications. Two that
came up:

- *How do panels write their frequency band edges?* Answered by
  downloading all 122 released specs and grepping: 96 of 117 BS1
  descriptors use an explicit operator, and where two bands share an
  edge, 13 write BA1 `≥X` / BS1 `<X`. That convention let a team
  defend a boundary they were about to guess.
- *Has any panel written a gnomAD quality-filter policy?* No — zero
  hits across all 122. A clean negative that changed what got built.

`biomcp search spec --criterion BS1` would make both a command
instead of a download-and-grep. Lower priority than the attachments;
filed here rather than separately because it is the same entity.

Raised 2026-08-08 from PTEN GN003 research for varclassify2.
