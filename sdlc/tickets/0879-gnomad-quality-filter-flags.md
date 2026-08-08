---
flow: build
priority: 5
---
# Surface gnomAD's filter flags, exomes and genomes kept apart

## Done when

`biomcp get variant <id> population` lists the filter flags per data
type, so a variant AC0 in exomes and PASS in genomes shows both. Each
flag is expanded to its meaning in the readable output and kept as a
bare name in `--json`.

## The finding

Raised as an issue during BioMCP research on 2026-08-08, then folded in
here and the issue file removed. The text below is the issue as filed.

`feature-gnomad-v4-and-filtering-allele-frequency.md`; either alone
is half an answer.

## The question it blocks

"Is this frequency usable?" comes before "what is this frequency?".
gnomAD answers it with filter flags, and BioMCP reports none of
them. Grepping the whole of `get variant … all` for `filter`, `AC0`,
`VQSR` returns nothing.

The distinction matters because the flags do not all mean the same
thing, and reading them as one blob either over-refuses or
under-refuses.

**`AC0`** is a genotype-level filter. From gnomAD's own help:

> "A variant flagged with the 'AC0' flag indicates that the allele
> count for that variant in the specified data type is 0 after
> removing low quality genotypes. We filter out genotypes as being
> low quality according to the following criteria: Genotype quality
> (GQ) < 20; Depth (DP) < 10 for diploid genotype calls or DP < 5
> for haploid genotype calls; Allele balance < 0.2 for heterozygous
> genotype calls"

The site was called and the denominator is real; no carrier survived
quality filtering. That can only support "not common". For the
high-frequency criteria it is a legitimate not-met.

**`AS_VQSR`** is a call-confidence filter — allele-specific VQSR
computes "a confidence score for each allele in our data to be real
or artifactual". A failure there means the observation may not exist,
so the frequency is untrustworthy in both directions and refusing is
correct.

Same word "filtered", opposite consequences.

## The part that is easy to get wrong

Filter status is per data type, and the two can disagree:

> "As the filtering process was performed on exomes and genomes
> separately, users will notice that for some variants, we have 2
> filter statuses which may be discordant in some cases."

So a single combined `filter: FAIL` field would be worse than none.
A variant can be `AC0` in exomes and `PASS` with real carriers in
genomes. Whatever is surfaced has to keep the two apart.

## Shape

- Report `filters` per data type — exomes and genomes as separate
  entries, each a list, each with the flag name.
- Expand the flag to its meaning in the human-readable output. `AC0`
  on its own is opaque; `AC0 (no carrier survived genotype quality
  filtering)` is actionable.
- Keep the raw flag names in `--json` so callers can branch on them.

## Prior art check, so nobody has to redo it

No released ClinGen expert panel specification states a policy for
gnomAD quality filters. All 122 released specs in the CSpec registry
were searched, every text field, for `AC0`, `VQSR`, `random forest`,
`quality filter`, `non-PASS`, `failed filter`. Zero hits. There is
no convention to encode — the right move is to report the flags
faithfully and let the caller decide, which is what this asks for.

Raised 2026-08-08 from PTEN GN003 research for varclassify2.
