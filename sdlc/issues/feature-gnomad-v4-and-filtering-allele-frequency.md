# Feature: gnomAD v4 frequencies, and filtering allele frequency

Severity: should-fix. Not a defect — BioMCP reports what its source
gives it. But it puts variant classification work out of reach.

## The question it blocks

Every ACMG/AMP frequency criterion — BA1, BS1, PM2 — is defined on
the **filtering allele frequency** (FAF), the lower bound of the 95%
confidence interval on the highest non-bottlenecked ancestry group's
frequency. Not on the raw allele frequency. Current ClinGen expert
panel specifications say so in as many words. The PTEN panel's BA1
reads:

> "gnomAD Filtering allele frequency >0.00056 (0.056%)"

`biomcp get variant … population` cannot answer that question. It
returns raw per-population AF and no FAF, no confidence interval,
and no grpmax.

## What is actually being served today

    gnomAD AF: 0.000012 (< 0.01%)
    African/African American: 0
      African/African American (female): 0
      African/African American (male): 0
    …
    ExAC AF: 0.000008

Sex-split subpopulation fields (`af_afr_female`, `af_afr_male`) and
an ExAC line are gnomAD **v2.1** shape. Confirmed against
MyVariant directly: `gnomad_exome` carries `af_afr_female` and no
`filter` key, and `gnomad_genome` comes back empty for the variant
tested. v2.1 is a 2019 dataset built on GRCh37. gnomAD v4.1 has
roughly five times the samples and a different ancestry grouping.

So a caller doing frequency work gets old numbers, in the wrong
statistic, with no signal that either is the case.

## Shape

- Source FAF from gnomAD directly rather than through MyVariant.
  gnomAD's GraphQL API returns `faf95` per ancestry group and the
  grpmax selection, which is exactly the quantity the criteria name.
- Report the dataset version in the output. `gnomAD v4.1 (exomes)`
  on the line is worth as much as the number, because a reader
  currently has no way to tell which release they are looking at.
- Keep the raw per-population AF; it is still useful. Add FAF, do
  not replace.
- Carry the FAF caveat that gnomAD documents: the calculation
  excludes Amish (`ami`), Ashkenazi Jewish (`asj`), European Finnish
  (`fin`) and Remaining Individuals (`rmi`). Anyone applying BA1 to
  a founder variant needs that on the same screen.

## Why it is worth the work

This is the single field that separates "interesting genomics
lookup" from "usable in a variant classification pipeline". It is
also the field with the least ambiguity about what is wanted — the
expert panels have already written the thresholds down, in FAF, with
numbers.

Raised 2026-08-08 from PTEN GN003 research for varclassify2, where
BioMCP could not be used as the frequency source and gnomAD had to
be consulted separately.
