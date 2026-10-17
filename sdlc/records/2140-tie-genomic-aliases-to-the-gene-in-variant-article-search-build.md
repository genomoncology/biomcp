# 2140 build record — tie genomic aliases to the gene in variant article search

- Branch `ticket/2140-tie-genomic-aliases-to-the-gene-in-variant-article-search`
  in worktree `worktrees/biomcp-2140`, started from `origin/main` at
  `ec4f2e309` and rebased onto `b138f6097` (three research-ticket filings, no
  code overlap) before the first push. Final tip `829880675`; stopped without
  landing — the coordinator lands.

## Reproduction on main first

The ticket's Work item says reproduce on main first, and the PM rule for
research-raised lanes demands it before any code. pm job `1792206086379-58813`
on the macOS gate host built main's CLI at `ec4f2e309`
(`cargo build --no-default-features --bin biomcp`) and ran the live searches:

- `variant articles "SAMHD1 I448T" --limit 50`, then `--offset 50`: page 1
  keeps 17 rows whose only match is the bare `chr20:g.35533834A>G`, page 2 six
  more; the cotton paper 28794480 appears on both pages with
  `route=exact_lexical, source=pubmed, matched_alias=chr20:g.35533834A>G`
  (and the same alias through Semantic Scholar on page 1).
- `variant articles "CDKN2A A57V" --limit 50`, then `--offset 50`: the citrus
  paper 41894487 appears on page 2, matched only by `chr9:g.21971188G>A`
  (native position 5; 3 coordinate-only rows on page 1, 8 on page 2).

Both command lines and these outputs are recorded in the ticket's Reproduction
section. Provider-level cause, captured live from the public endpoints on the
research host with a lightweight script (no build): PubMed's ESearch translates
the bare term `chr20:g.35533834A>G` to
`"chr20"[All Fields] AND "G"[All Fields]` — 15 papers that day, cotton among
them — while `SAMHD1 chr20:g.35533834A>G` returns 0 and
`CDKN2A chr9:g.21971188G>A` returns 4 CDKN2A-related papers and no citrus.
Europe PMC answers 0 for the bare coordinates, so PubMed is the one degenerate
leg; the tie removes the class for every leg.

## What changed

- `src/entities/article/variant_search.rs`:
  - `exact_aliases` now gene-prefixes the resolved source identity's genomic id
    through the same `insert_change` closure the protein and coding changes
    use, so `chr20:g.35533834A>G` enters the exact-alias set as
    `SAMHD1 chr20:g.35533834A>G`. The gene selection moved into
    `variant_source_gene`, which prefers the requested gene symbol and falls
    back to the source identity's first gene — the same spelling the strict
    query builder strips, so the strict leg's
    `("<gene>"[Title/Abstract] AND "<alias>"[Title/Abstract])` queries are
    unchanged by the prefix.
  - `exact_aliases_with_equivalence` ties confirmed ClinGen allele-registry
    aliases that carry `:g.` to the same gene; transcript, protein and
    external-identifier aliases stay bare because each names the variant
    uniquely.
  - rsIDs and an authoritative-refseq input's own accession coordinate alias
    are unchanged (scope recorded in the ticket's Decision section).
- Every consumer of the exact-alias set now sends the gene-tied coordinate:
  the strict provider plans strip the prefix and rebuild it (same query as
  before), the exact-lexical federated keyword legs query
  `SAMHD1 chr20:g.35533834A>G`, and the debug plan shows the tied spelling.
- CHANGELOG bullet under 0.9.3 Fixes.
- `tools/zero-coupling-historical.json` carries the new
  `testdata/sources/capture-receipts.json` digest (the receipts file is
  digest-pinned; the 23 new fixture entries changed it). `cargo package
  --list` stays 1,409 (measured on the gate host, pm job
  `1792206973409-76340`), so `MAX_PACKAGE_FILES` is unchanged — the fixtures
  live under the excluded `testdata/` tree.

## Decision recorded in the ticket

Send genomic aliases with the gene symbol (Work option 1). The strict route
already ANDs the gene into every alias query, so the bare exact-lexical
keyword was the one gap; a post-hoc gene-name filter cannot run at candidate
stage, where several providers return title-only rows (abstracts are hydrated
later for the visible page), and the off-topic rows would still spend provider
requests and ranking.

## Proof

- Recorded-reply tests in `src/entities/article/variant_search.rs`:
  `samhd1_i448t_articles_tie_the_genomic_alias_to_the_gene` and
  `cdkn2a_a57v_articles_tie_the_genomic_alias_to_the_gene` drive the real
  `search_variant_articles_with_deadline` Union search against a local fixture
  origin serving the byte-faithful 2026-10-17 provider captures under
  `testdata/sources/variant_articles_2140/` (23 files: MyVariant resolution
  and hydration, PubMed ESearch/ESummary for the bare, gene-tied and primary
  keyword terms, Europe PMC keyword and zero-hit, PubTator autocomplete,
  entity search and zero-hit; receipted in `capture-receipts.json`, described
  in the directory README). Each asserts the off-topic paper is absent, a
  gene-naming paper is kept, no provenance carries the bare coordinate alias,
  and no provider leg ever queried the bare coordinate term.
- Red on main: commit `c210ee055` is main plus the tests and fixtures only;
  pm job `1792206635365-35061` on the macOS gate host fails both —
  "the cotton paper matched only by the bare coordinate must drop; got 1" and
  "the citrus paper matched only by the bare coordinate must drop; got 1".
- Green at the fix: pm job `1792206812604-57415` at `e59abd027` passes the two
  recorded tests (0.16 s each), the updated
  `resolved_exact_aliases_include_validated_source_forms_once` (the BRAF
  genomic alias expectation becomes `BRAF chr7:g.140453136A>T`), and the new
  `confirmed_car_genomic_aliases_travel_with_the_gene_symbol`.
- `tools/check-source-capture-receipts.py --root testdata/sources` passes with
  the 23 new entries; `tools/check-zero-coupling.py` passes with the refreshed
  digest.

## Gates

- `make test` (the Linux gate host, offline routine lane): job
  `1792207086972-2205411` at `54d952080` ran the Rust lane 4,304/4,304 passed
  (33 skipped) and failed the Python lane's quality ratchet on the un-refreshed
  receipts digest ("missing-or-renamed:testdata/sources/capture-receipts.json")
  — fixed by the digest commit; job `1792212391095-2487834` at the final tip
  `829880675` passed both lanes — Rust 4,304 run, 4,304 passed (33 skipped),
  Python 1,159 passed (3 skipped), exit 0. (An intermediate identical run at
  `6e02dd8e6`, job `1792210046801-2352099`, passed the same way.)
- CI on the branch: run `39001366565` at `6e02dd8e6` failed the lint gate's
  code-comment ticket-citation scan (two test comments said "Ticket 2140";
  rewritten to state the constraint); run `39002962569` at `829880675` is green
  in every lane (build, lint, rust-tests, python-lane, spec-lane,
  repository-contracts, release-checks, generated-sources) — the landing bar
  run at the code tip.
- The macOS gate host: `cargo fmt --check` passes at the tip; `cargo clippy
  --no-default-features --all-targets -- -D warnings` reports only three
  pre-existing errors on untouched files (`src/cache/manager.rs`'s unused
  import, two `needless_borrow` in `src/sources/tests/provider_network.rs` —
  the same class the 2130 record documents for its rust-1.95 clippy being
  stricter than CI's pinned toolchain). Jobs `1792209491527-69932` and
  `1792209550434-73273`.

## Findings

- The probe's CDKN2A input spelling was never recorded; `CDKN2A A57V` resolves
  to the reported alias `chr9:g.21971188G>A` (rs372266620, c.170C>T,
  p.Ala57Val), so the tests and the reproduction use it. Noted in the ticket.
- Provider drift: PubMed held 15 papers for the bare chr20 coordinate and 139
  for the bare chr9 coordinate on 2026-10-17 (the probe saw 28 and 19
  coordinate-only matches across the two pages); the defect is unchanged and
  the recorded fixtures pin that day's answers.
- The gene-tied CDKN2A coordinate query has four real PubMed hits (42284679,
  34256839, 33335285, 11025389 — chromosome-9 aneuploidy and CDKN2A
  copy-number papers), so a gene-tied coordinate alias still keeps papers that
  are about the gene; PubMed's synonym expansion matches the gene concept, not
  only its literal spelling.
- The source-citation route's MyVariant hydration leg fires in the recorded
  tests (the live probe's invocation deadline cut it), so the fixtures include
  the two hydration replies; neither carries CIViC evidence, so no citation
  rows appear.

## Cleanup

Research-host scratch (`/tmp/2140-*`: the recording proxy, capture trees, the
extra-capture and fixture-capture scripts, output JSONs) deleted after the
fixtures were committed. The gate-host job logs are pm's own. The worktree and
branch stay for the coordinator.
