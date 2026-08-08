# Feature: ClinGen criteria specifications as a first-class entity

Severity: nice-to-have with a high ceiling. This is a whole class of
question BioMCP currently cannot be pointed at.

## What is missing

ClinGen's Criteria Specification (CSpec) Registry holds the
gene-and-disease-specific rules that expert panels use to classify
variants — the thresholds, the strengths, the scope. 205 documents,
122 of them Released. BioMCP knows nothing about it.

It is a plain JSON API, no key, no scraping:

    # one specification, current release
    https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN003

    # pinned version
    …/SequenceVariantInterpretation/id/GN003/version/3.2.1

    # attachments, by the entity id inside the payload
    https://cspec.genome.network/cspec/File/id/<entId>/data

Each payload carries every criterion with its default strength, each
strength descriptor with `applicability` / `status` / text, the gene
scope, the disease and mode of inheritance, release notes, and the
attachment manifest.

## Shape

    biomcp get spec GN003                 # criteria table
    biomcp get spec GN003 --version 3.2.1 # pinned
    biomcp get spec GN003 criteria BS1    # one criterion in full
    biomcp get spec GN003 files           # attachment manifest
    biomcp search spec --gene PTEN        # which panels cover a gene

The `files` verb matters more than it looks. Specifications defer
real content to attachments — the PTEN one defers PVS1 to a decision
tree three times and hands the entire classification-combining step
to a screenshot. Those files are marked `"public": true` and served
at a stable URL, but the only place their ids appear is inside the
payload. A researcher reading the rendered page sees five filenames
with no link and reasonably concludes they are unpublished.

## The cross-specification query is the sleeper feature

    biomcp search spec --criterion BS1 --grep "≥"

Two questions from one week of work needed a sweep across all
specifications, and both had to be answered by downloading 19MB and
grepping:

- *How do panels write their frequency band edges?* 96 of 117
  applicable BS1 descriptors use an explicit `≥ ≤ > <`; only two use
  prose. Where two bands share an edge value, 13 specs write BA1
  `≥X` and BS1 `<X` — a house convention that let a downstream team
  defend a boundary decision they had been about to guess at.
- *Has any panel written a gnomAD quality-filter policy?* No. Zero
  hits across all 122. A clean negative that changed what got built.

Neither is answerable one specification at a time, and both are the
kind of question a person asks precisely because they cannot afford
to guess.

## Why BioMCP

This is the normative layer directly above the data BioMCP already
serves. It answers "what does this variant look like" today; the
specifications answer "what is the rule I am supposed to apply to
it". Pairs naturally with
`feature-clingen-expert-panel-assertions.md`, which covers what
panels actually did — prose and practice, the two halves of the same
question.

Raised 2026-08-08 from PTEN GN003 research for varclassify2.
