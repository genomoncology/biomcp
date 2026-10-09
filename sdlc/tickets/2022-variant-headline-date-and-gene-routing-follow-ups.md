# 2022 — Variant headline date and gene routing follow-ups

Status: complete.

Milestone: 0.9.2

## Outcome

The variant headline pairs a ClinVar classification with that classification's own evaluation date. Variant search routes a first token to a gene only when it is an official symbol, and a routed search that returns nothing says how it was read.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, findings 11 and 12).

- `src/transform/variant.rs:687`: `significance_evaluated` is the newest date across all ClinVar submission records, printed beside the most severe classification. `get variant 'BRAF V600E'` prints `Significance: Pathogenic — MyVariant.info (evaluated 2025-01-23)`. The 2025-01-23 record (RCV005089260) says Uncertain significance; the Pathogenic records date from 2014 to 2023. Ticket 1290 asked for the newest date, so the code follows the ticket, but the line misleads.
- `src/cli/variant/query.rs:196` accepts gene aliases. `search variant 'HCC liver cancer'` becomes `gene=HYCC1`, 0 rows, no hint. `'MODY diabetes'` routes to HNF4A only and `'HHT telangiectasia'` to ACVRL1 only. `'BRAF V600E melanoma'` becomes `gene=BRAF, condition=V600E melanoma`, 0 rows, no hint.
- The open 1291 branch adds a second newest-date helper with different rules from `newest_rcv_evaluation_date`; keep one.

- Starts from: tickets 1290 and 1301.
- Keeps: the current-classification headline and official-symbol routing.
- Changes: print the newest date among records carrying the shown classification; route on official symbols only; recognise a protein change after the gene; when a routed search returns zero rows, print the parsed form and the working alternative.
- Proof: outside-in tests for BRAF V600E's date, HCC, MODY and 'BRAF V600E melanoma', each failing on `37631c357`.
- Defers: fixture coverage for the phrase form `get variant 'BRAF V600E'` (the 2026-08-06 resolution search recording predates ClinVar fields; the spec pins the coordinate form and the phrase form is live-verified only).

## Root cause

1. `src/transform/variant.rs` `newest_rcv_evaluation_date` takes the newest `last_evaluated` across every RCV row, while `pick_significance` prints the most severe classification. The two helpers answer different questions, so the headline pairs one record's classification with another record's date whenever the newest date belongs to a less severe record.
2. `src/cli/variant/query.rs` `confirm_gene_first_candidate` resolves the first token through `resolve_unique_canonical_alias`, which accepts the token when it matches a gene's alias OR its official symbol. Alias matches are many-to-one and unreviewed for search routing, so 'HCC' routes to HYCC1 and returns nothing.
3. `split_gene_first_candidate` routes the whole remainder after the gene to the condition, so 'BRAF V600E melanoma' becomes `condition='V600E melanoma'`. Nothing splits a leading protein change out of the remainder, and a routed zero-row search carries no `GeneFirstFallback`, so no hint prints.

## Success criteria

1. `get variant 'BRAF V600E'` prints the Pathogenic headline with `evaluated 2023-10-22` (the newest date among the Pathogenic records, RCV003458334), not `2025-01-23` (the Uncertain significance record RCV005089260).
2. The date helper keeps one rule: a record's date counts only when the record carries the shown classification, and only day-shaped `YYYY-MM-DD` values surface, aligned with the 1291 fallback label's day-shape rule. Whichever of tickets 2022 and 1291 lands second collapses the two checks into one helper.
3. `search variant 'HCC liver cancer'`, `'MODY diabetes'` and `'HHT telangiectasias'` do not route an alias first token; each keeps the whole-phrase condition search.
4. `search variant 'BRAF V600E melanoma'` reads `gene=BRAF, hgvsp=V600E, condition=melanoma`, not `condition='V600E melanoma'`.
5. A routed search that returns zero rows prints the parsed form and a loosened alternative that names one dropped filter; a refused zero-row search keeps the 1301 hint.
6. The 1301 wins hold: SCN5A routes, BRUGADA refuses with the working-form hint, and `BIOMCP_VARIANT_QUERY_GENE_ROUTING=off` still restores the whole-phrase condition search.

## Decision: MODY and HHT stop routing

Official symbols only means the familiar alias abbreviations stop routing too: 'MODY diabetes' no longer narrows to HNF4A and 'HHT telangiectasias' no longer narrows to ACVRL1; both search the whole phrase as a condition. Recorded reason: alias routing either applies to every alias or needs a curated allow-list, and no admitted source provides one. The HCC failure is the same mechanism as the MODY and HHT shortcuts, so one predictable rule (the token must be the official symbol) replaces alias routing everywhere, matching the outcome. Operators who want the narrow search can still write `biomcp search variant -g HNF4A --condition diabetes`.

## Implementation

- `src/transform/variant.rs`: `newest_rcv_evaluation_date` now takes the shown classification and counts only records that carry it, so the headline date belongs to the printed term. The day-shape gate lives in one helper, `crate::utils::date::is_day_shaped`. Ticket 1291 still carries its own inline check on its branch; whichever of tickets 2022 and 1291 lands second collapses the duplicate onto the shared helper.
- `src/entities/gene.rs`: `resolve_unique_official_symbol` confirms a first token only when it is the gene's official symbol; `resolve_unique_canonical_alias` keeps serving the `discover` path unchanged.
- `src/cli/variant/query.rs`: `split_leading_protein_change` moves a leading protein change to the hgvsp filter; `GeneFirstNote` replaces `GeneFirstFallback` so both the refused and the routed zero-row case carry their hint.
- `src/cli/variant/dispatch.rs`: a routed zero-row search prints the parsed form and the same filters with the condition dropped (repeating the parsed filters repeats the empty result), and pushes that alternative into JSON `next_commands`.
- Fixtures: recorded the dated MyVariant GRCh37 BRAF V600E GET (`get_braf_v600e_grch37_20261007.json`, receipted) and the MyGene shapes for HCC (HYCC1), MODY (HNF4A) and HHT (ACVRL1).

## Proof

Red on pristine `37631c357` (built in a disposable clone; the worktree base is byte-identical across every file this ticket touches):

1. BRAF V600E date: `get variant 'chr7:g.140453136A>T'` against the routine fixtures printed `Significance: Pathogenic — MyVariant.info (evaluated 2025-01-23)`, the Uncertain record's date. The spec pins the coordinate form because the fixture's phrase-form resolution search recording (`search_braf_v600e_20260806.json`) predates ClinVar fields; the phrase form is live-verified only (its live search response carries the same 17 dated rows).
2. HCC: `search variant 'HCC liver cancer'` with live sources printed `Query: gene=HYCC1, condition=liver cancer`, count 0, no hint.
3. MODY: `search variant 'MODY diabetes'` with live sources printed `Query: gene=HNF4A, condition=diabetes`.
4. 'BRAF V600E melanoma': `search variant 'BRAF V600E melanoma'` with live sources printed `Query: gene=BRAF, condition=V600E melanoma`.

Green after the fix:

- `spec/entity/variant.md`: 109 passed, 1 skipped (the new BRAF date cases beside the kept TP53 cases).
- `spec/entity/variant-gene-first-routing.md`: 12 passed (HCC, MODY, HHT refusal rows; the routed zero-row parse and working form; the kept SCN5A/BRUGADA/off cases).
- `cargo nextest run --no-default-features -p biomcp-cli -E 'test(/variant/) + test(/gene_first/) + package(biomcp-cli) & test(/::date::/)'`: 420 passed.
- `cargo nextest run --no-default-features -p biomcp-cli -E 'test(/entities::gene::/) + test(/official_symbol/)'`: 81 passed.
- `make lint`: passes (one capture-receipt entry, one zero-coupling digest repin, two authorized size-baseline increases recorded for this ticket).
- `make sync-python-dev` ran before every spec run.

Environment note: the full `biomcp-cli` suite currently flakes on cache/network-sensitive tests under this machine's parallel lane load (`stale_json_note_tests`, stale disease cards, one 595 s population timeout). The identical failure set reproduces on pristine `37631c357`, and every flake passes on retry in isolation, so none of it comes from this change.

## Build status

- Built on branch `tickets/2022-variant-headline-date-and-gene-routing-follow-ups`,
  commits 15225975b, fe6b3ae8f, fold aa8317a4f, 2026-10-07, across two
  timeout revivals with checkpoints (nothing lost). The 2026-10-08 rebase
  onto main (after 2023 and 1300 landed) rewrote those as 77003a2e7,
  3c021ab0d, and fd2522e16; a second rebase the same day, after 2017
  landed, rewrote the chain again to b427eb233..c3bf14929; the work is
  unchanged.
- Code review: REJECT 2026-10-07, findings fixed the same day. The P1
  (configuration.md still describing alias routing after the change)
  and two P2s (the contradicting enum doc; the ticket's missing
  fixture-path reason and wrong Defers) fixed in aa8317a4f, which also
  adopted the report-only hint wording (drop-or-loosen-a-filter
  instead of the misleading try-the-working-form).
- Code re-review (fold delta fe6b3ae8f..aa8317a4f): ACCEPT 2026-10-07.
  All four folds verified at the named seams with no collateral edits
  and no stale wording anywhere.

## Second-review fix round

- Code re-review (the 2026-10-07 second review of the work since 0.9.1,
  `sdlc/issues/2026-10-07-second-review-of-the-work-since-0.9.1.md`,
  findings 12 and 15 and the review-line gap of findings 10 and 11):
  REJECT 2026-10-07, fixed 2026-10-08. Finding 12: the branch re-indented
  `capture-receipts.json` from two spaces to one; the rebase unions by
  path in main's two-space formatting (327 entries, the branch's one
  MyVariant receipt over main's 326) and the re-indent folds away.
  Finding 15: the routed zero-row hint repeated the command that
  returned zero rows; the hint now drops the condition filter and
  prints the same filters without it (845ae0719). Finding 10: the
  recorded REJECT-and-ACCEPT history now sits on the branch ticket in
  the house grammar instead of main's copy alone. Finding 11: the
  the 2026-10-08 rebase onto main after 2023 and 1300 landed unions receipts,
  the size inventory, and the ticket with measured counts kept (gene.rs 3963,
  transform/variant.rs 1496; main's 1_405 package count holds because
  `testdata/` is package-excluded). Re-review of this round: ACCEPT 2026-10-08 (fresh reviewer, head
  7b0e8bfd6). The receipts delta is exactly the 12 real entry lines in
  main's formatting; the hint carries the dropped-condition alternative
  with the refused path unchanged; the review history parses; the
  inventory unions are measured.

## Third-review fix round

- Code re-review (ticket 2033 finding 13): REJECT 2026-10-08, fixed
  2026-10-08. Item 1: the routed zero-row alternative dropped explicit
  `--hgvsp` and `--consequence` flags while the hint said "the same
  filters"; the alternative now keeps the resolved `--hgvsp` filter and
  any explicit `--consequence` flag, so it differs from the routed search
  by exactly the dropped condition (c3bf14929, with unit pins for both
  flags). Item 2: the transform and date comments claimed ticket 1291
  already uses `is_day_shaped`; they now state the pre-merge truth, that
  1291 still carries its own inline check on its branch, and success
  criterion 2 records the collapse rule: whichever of tickets 2022 and
  1291 lands second collapses the duplicate onto the shared helper. The
  stale "no longer conflicts" sentence is replaced by the dated rebase
  notes in Build status. This round's rebase lands on main at `b872c234a`
  (2017 landed, third-review tickets filed): receipts union to 333 entries
  in main's formatting, the zero-coupling digest is repinned, and the
  inventory keeps the measured counts. Re-review of this round pending.
