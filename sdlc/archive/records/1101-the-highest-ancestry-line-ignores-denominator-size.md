---
flow: build
priority: 3
---

# Make the highest population-row frequency disclose its allele denominator

## Outcome

The compact variant-population Markdown identifies its raw row maximum as an
observed population-row value and prints that row's allele count and allele
number, so neither a human nor an agent can repeat the frequency without seeing
the denominator that produced it.

## Current facts

At HEAD `9a90858c31ba66356e705d57eea79d3ee06db32a`,
`biomcp get variant rs1426654 population` renders:

```text
Exome highest ancestry frequency: eas_XX (0.994495)
gnomAD v4 exome grpmax FAF95: 0.98581 (eas)
Genome highest ancestry frequency: 1kg:cdx_XY (1)
gnomAD v4 genome grpmax FAF95: 0.961989 (eas)
gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF.
```

The direct `gnomad_r4` GraphQL query in `src/sources/gnomad.rs` retrieves `ac`
and `an` for the exome and genome totals and for every returned population row.
Parsing computes `allele_frequency` as `ac / an` when `an > 0`; `an == 0`
becomes a null frequency. `GnomadAncestryPopulation` retains the raw provider
identifier, frequency, AC, AN, homozygote count, and hemizygote count. Here AN
means the number of alleles with a defined genotype call at that site and is the
denominator used for AF; it is not a count of people, participants, or samples.
Because the parser derives AF from two `u64` values only when AN is positive,
provider-produced non-null values are finite and nonnegative.

`highest_ancestry_frequency` in `src/render/markdown/variant.rs` currently
filters only null frequencies and uses `max_by(total_cmp)`. It therefore:

- includes aggregate ancestry rows, sex-stratified rows such as `eas_XX`, and
  prefixed 1KG/HGDP cohort rows in one candidate set;
- includes an observed zero (`AC = 0, AN > 0`) but excludes `AN = 0` rows;
- selects the last provider row when multiple rows have the same frequency;
  and
- supplies only the row identifier and frequency to `templates/variant.md.j2`.

The original issue's exact denominators have drifted and must not become test
or product constants. A no-cache live check on 2026-09-04 still selected
`1kg:cdx_XY (1)`, but that row was then `AC 84 / AN 84`; the same response had
many other frequency-1 genome rows, ranging down to `AN 2`, and an aggregate
`1kg:cdx` row at `AC 176 / AN 176`. The defect is the denominator-free raw
summary and order-dependent tie, not one enduring rs1426654 row.
This live observation is reproducer/prevalence evidence only, not a fixture or
an oracle for selection, wording, or exact counts.

The neighbouring FAF95 value is a different provider statistic with its own
gnomAD bottleneck-exclusion semantics. BioMCP has no supported clinical
minimum-AN rule for the raw rows.

## Required behavior

For exome and genome independently, the compact Markdown summary must:

1. Consider rows whose stored `allele_frequency` is non-null and finite. A
   defensive in-memory `NaN` or infinity is not an observed frequency and must
   be treated like null, even though the provider parser cannot produce one.
   Do not recompute AF in the renderer, introduce a minimum AN, exclude a
   provider cohort class, or substitute grpmax FAF95.
2. Select the greatest frequency using `f64::total_cmp`. Break a
   `total_cmp`-equal frequency tie by greater `an`, then by ascending raw `id`
   using case-sensitive `str::cmp`, so provider row order cannot change the
   answer. Compare the unmodified provider ID, not its display label. This is a
   single best-row selection; do not reorder the stored population rows.
3. Render the selected row as, for example,
   `Exome highest observed population-row frequency: broad (1; allele count 200 / allele number 200)`.
   Use the existing ancestry-label filter for the displayed identifier.
4. Render an observed zero with its nonzero allele number. Rows with null
   frequency do not compete; if every row is null or the row list is empty,
   render `Not reported` exactly as today.
5. Apply the same rule in compact `population` output and the summary above
   each `population-details` table. A missing exome or genome remains the
   existing `No ... result` state; the other side is unaffected.

The words `observed population-row` are intentional. The candidate collection
contains more than one kind of population partition, so the line must not imply
that it is a prevalence estimate for one uniformly defined ancestry group.
Documentation must define allele number as the AF denominator of alleles with
defined genotype calls at that site, and must not call it sample size or infer a
number of people from it.

## Scope and compatibility

Own the change in the Markdown selection/rendering layer:

- `src/render/markdown/variant.rs`
- `templates/variant.md.j2`
- focused assertions in `src/render/markdown/variant/tests.rs`
- the compact-output wording in `docs/user-guide/variant.md` and
  `docs/sources/gnomad.md`

Do not change the gnomAD query, parsing, entity structs, JSON field names or row
order, source provenance, FAF95 value/caveat, filter rendering, detailed-table
rows, CLI grammar, or section names. `--json` CLI output and JSON batch results
must remain structurally and numerically unchanged.

The ordinary CLI, human-mode typed/raw MCP `get`, and human `batch variant
... --sections population` all reuse `variant_markdown`, so the Markdown fix
reaches those surfaces at the owning renderer. MCP or batch requests for JSON
continue through entity serialization and are intentionally unchanged. Do not
add a parallel summary field to JSON.

The Cargo package listing is already at its 1,300-file ceiling. Add no files.
The two likely Rust edit targets are currently below the 1,000-line source-size
ratchet (`variant.rs`: 590; its test module: 832), and this ticket does not
authorize a raised baseline or a new over-limit file.

## Acceptance, test first

First extend the existing renderer fixture tests with synthetic, deterministic
rows; do not record or query live rs1426654 data in tests.

- Pin the exact compact exome and genome line shape, including the words
  `highest observed population-row frequency` and the selected row's allele
  count and allele number.
- Render otherwise-identical fixtures with equal maximum frequencies in both
  provider orders to prove that the larger-AN row wins in either order. Add an
  equal-frequency/equal-AN pair whose raw IDs prove ascending-ID final
  tie-breaking in both orders.
- Cover an `AC = 0, AN > 0, allele_frequency = 0` row and an `AN = 0,
  allele_frequency = null` row: the observed zero is rendered with its
  denominator, while an all-null side renders `Not reported`. Directly exercise
  the selector with a non-finite in-memory frequency and prove it cannot beat a
  finite row or turn an otherwise unreported side into a reported maximum.
- Keep assertions that compact output omits detail rows, `population-details`
  retains all provider rows and their order, the `remaining` display label is
  preserved, and serialized JSON retains raw IDs, frequencies, AC, AN, and row
  order without a new summary field. Retain the existing FAF95 value and
  caveat, quality flags, source provenance, and overall-frequency lines.
- Exercise a missing exome or genome alongside a populated opposite side so
  partial results remain independent.

No existing mustmatch spec pins this summary line, and the renderer unit is the
narrow deterministic contract; do not add a live-data spec fixture for this
ticket. Run the focused Rust test target, then `make lint`, `make test`, and
`make spec`.

## Dependencies

None. Ticket 1035 already established the compact/detail split, and ticket
1062's `remaining` display label is preserved rather than reopened.

## Review

- Evidence/design: ACCEPT for independent design review at HEAD `9a90858c`.
- Design review (2026-09-04): **ACCEPT** after clarifying that non-finite
  in-memory frequencies are excluded, ties use `f64::total_cmp`, raw-ID order
  is case-sensitive `str::cmp`, live counts are prevalence evidence only, and
  AN is the AF denominator of alleles with defined genotype calls rather than a
  sample count. The comparator is implementable without a dependency, the
  renderer is shared by CLI and human MCP/batch paths, JSON bypasses it, and
  the 1,300-file package ceiling requires no new file.
- Test-first evidence (2026-09-04): focused renderer tests failed before the
  implementation because the old wording remained, equal-frequency selection
  followed provider order, and a non-finite value could win. After the shared
  selector and template change, all four focused population renderer tests pass.
- Independent code review (2026-09-04): **ACCEPT with no findings**. Verified
  the finite-only deterministic comparator, zero/null/partial behavior, shared
  compact/detail selection, exact wording and AC/AN semantics, unchanged JSON
  and detailed rows, documentation, and package/source-size rails. All four
  focused population tests, all 22 variant renderer tests, formatting, the
  focused source-size audit, and `git diff --check` passed. The 995-line test
  module is below the enforced 1,000-line limit and needs no allowance.

## Completed 2026-09-04

Implemented a shared deterministic selector for the compact gnomAD population
summary. It ignores non-finite frequencies, preserves observed zeroes, and
orders candidates by frequency descending, allele number descending, then raw
population ID ascending. Compact output now describes the highest observed
population-row frequency and includes its allele count and allele number;
detailed rows, JSON, FAF95, provenance, and quality information remain intact.

Primary verification passed at the reviewed worktree: `make lint`; `make test`
(3,128 Rust tests passed with 30 skipped, 892 Python tests passed with 3
skipped, and strict documentation build passed); and `make spec` (all routine
mustmatch groups passed, including 140 serialized cases with 4 skipped, 39
parallel-isolation cases, and 8 static cases). `cargo package --list
--allow-dirty --locked --offline --no-verify` remains exactly 1,300 entries,
and `git diff --check` passed.
