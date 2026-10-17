# 2140 build record — tie genomic aliases to the gene in variant article search

- Branch `ticket/2140-tie-genomic-aliases-to-the-gene-in-variant-article-search`
  in worktree `worktrees/biomcp-2140`, started from `origin/main` at
  `ec4f2e309`, rebased onto `b138f6097` (three research-ticket filings), onto
  `2e707f75a` after the 2138, 2147 and 2148 landings, and a third time onto
  `159c68610` after ticket 2131 landed LitVar2 into the variant-article union
  — 2131 and 2138 both edit the same `variant_search.rs` and append their own
  capture receipts, so the rebases merge all three: the 2140 fixture tests,
  the alias fix, 2138's entity-id route and 2131's LitVar2 route coexist, and
  the zero-coupling digest is recomputed over the merged receipts file (412
  entries, 26 of them this ticket's). Final tip `f82edc2f9`; stopped without
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
  digest-pinned; the new fixture entries changed it). `cargo package
  --list` stays 1,409 (measured on the gate host, pm job
  `1792206973409-76340`), so `MAX_PACKAGE_FILES` is unchanged — the fixtures
  live under the excluded `testdata/` tree.
- After the third rebase the two recorded tests also serve the LitVar2 route
  2131 added to the union: the SAMHD1 test replays two new byte-faithful
  LitVar2 captures under `testdata/sources/litvar/`
  (`autocomplete_samhd1_i448t_20261017.json`, five papers via
  `publications_rs774964432_20261017.json`), and the CDKN2A test reuses
  2131's own recorded `rs372266620` chain; both pin `BIOMCP_LITVAR_BASE` to
  the fixture.
- Two main-repair pins carried on this branch under the coordinator's ruling
  (2131's landing left main's rust-tests lane red; details in Findings and in
  the ticket's Carried main-repair pins section): the no-entity annotation
  run pins `BIOMCP_LITVAR_BASE` with a healthy empty autocomplete reply, and
  the credential-redaction live run pins `BIOMCP_LITVAR_BASE` to its fixture
  (503 like its pinned siblings), unpaces the fixture origin, and drops its
  retry sleeps.

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
  `confirmed_car_genomic_aliases_travel_with_the_gene_symbol`. After the
  second rebase, pm job `1792219146262-28106` at `2d3296434` passes all four
  again plus 2138's `gene_change_search_keeps_the_pubtator_entity_and_searches_by_its_id`,
  so the alias fix and 2138's entity-id route agree. After the third rebase,
  pm job `1792224425262-6245` at `39a1fc3ef` passes both recorded tests with
  the LitVar2 legs wired (plus 2131's own litvar tests), and pm job
  `1792227583461-43468` at `f82edc2f9` passes the two recorded tests, the
  pinned no-entity run, and the repaired credential-redaction run (7.1 s,
  inside its 10 s watchdog).
- `tools/check-source-capture-receipts.py --root testdata/sources` passes with
  the new entries; `tools/check-zero-coupling.py` passes with the refreshed
  digest.

## Gates

- `make test` (the Linux gate host, offline routine lane): job
  `1792207086972-2205411` at `54d952080` ran the Rust lane 4,304/4,304 passed
  (33 skipped) and failed the Python lane's quality ratchet on the un-refreshed
  receipts digest ("missing-or-renamed:testdata/sources/capture-receipts.json")
  — fixed by the digest commit; job `1792212391095-2487834` at `829880675`
  passed both lanes (Rust 4,304 run, 4,304 passed, 33 skipped; Python 1,159
  passed, 3 skipped; exit 0). After the second rebase, job
  `1792219229468-3415128` at `2d3296434` passed both lanes — Rust 4,309 run,
  4,309 passed (33 skipped; 2138's five new tests included), Python 1,160
  passed (3 skipped), exit 0. An interim run at `39a1fc3ef` (job
  `1792224480291-3602159`) and one at `87df8dd8b` (job
  `1792225447767-3660479`) straddled the third rebase: the earlier failed the
  credential-redaction live run inherited from main, the later passed it after
  the first pin; the final run at `f82edc2f9` (job `1792227607178-3699307`)
  passed both lanes — Rust 4,325 run, 4,325 passed (34 skipped; 2131's litvar
  tests included), Python 1,160 passed (3 skipped), exit 0.
- CI on the branch: run `39001366565` at `6e02dd8e6` failed the lint gate's
  code-comment ticket-citation scan (two test comments said "Ticket 2140";
  rewritten to state the constraint); run `39002962569` at `829880675` was
  green in every lane; after the second rebase, run `39010693830` at
  `2d3296434` was green in every lane; after the third rebase, runs
  `39016157944` (at `39a1fc3ef`) and `39017145415` (at `87df8dd8b`), and
  `39018661534` (at `3d3cbe6c2`) failed the rust-tests lane on the two
  main-inherited breaks described in Findings, each fixed on the branch; run
  `39019428156` at the final tip `f82edc2f9` is green in every lane (build,
  lint, rust-tests, python-lane, spec-lane, repository-contracts,
  release-checks, generated-sources) — that run at the exact post-rebase tip
  is the landing bar.
- The macOS gate host: `cargo fmt --check` passes at the tip; `cargo clippy
  --no-default-features --all-targets -- -D warnings` reports only three
  pre-existing errors on untouched files (`src/cache/manager.rs`'s unused
  import, two `needless_borrow` in `src/sources/tests/provider_network.rs` —
  the same class the 2130 record documents for its rust-1.95 clippy being
  stricter than CI's pinned toolchain). Jobs `1792209491527-69932` and
  `1792209550434-73273`.

## Findings

- Ticket 2131's landing left main's CI red in the rust-tests lane (main at
  `159c68610`, run `39015559033`; its own full-features lane fails too). Two
  tests broke because the Annotation strategy's new LitVar2 leg escaped their
  fixtures: `no_entity_annotation_run_names_the_miss_instead_of_complete_empty`
  (no `BIOMCP_LITVAR_BASE` pin; CI hit the real provider, run `39016157944`)
  and `swallowed_source_failures_do_not_log_credentials` (myvariant+pubtator
  pinned, litvar denied through the loopback proxy; on the gate host it fails
  the exit-code assertion at pm job `1792226138855-15222`, on CI runners it
  blows the run's 10 s watchdog). Per the coordinator's 2026-10-17 ruling
  both are carried on this branch as mechanical-consequence pins, so landing
  this branch heals main; the LitVar2 semantics themselves belong to 2131's
  lane, which should see the note in the ticket.
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
