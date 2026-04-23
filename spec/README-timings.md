# Spec Lane Audit

## Spec Lane Split

| Lane | Make target | Run when | Timeout | Scope |
|---|---|---|---|---|
| `spec-pr` | `make spec-pr` | every PR and repo-local pre-merge verification | `60s` per heading | Stable PR-blocking live specs after `SPEC_PR_DESELECT_ARGS` exclusions |
| `spec-smoke` | `make spec-smoke` | targeted local smoke rerun for volatile live-network headings | `120s` per heading | Exactly the ten stable smoke section IDs represented by `SPEC_SMOKE_ARGS`, resolved to current mustmatch pytest item IDs at runtime |
| `spec` | `make spec` | scheduled full smoke workflow and manual full smoke reruns | `120s` per heading | Full live spec suite, including every smoke-only heading |
| `test-contracts` | `make test-contracts` | PR contracts lane and local docs/Python validation | n/a | Rust release build plus Python/docs contract checks |

`spec-pr` is the fast blocking lane, `spec-smoke` is the serial local rerun for the ten stable smoke section IDs in `SPEC_SMOKE_ARGS`, `spec` remains the full live suite used by the existing scheduled smoke workflow, and `test-contracts` covers the Python/docs contract surface. The `spec-smoke` target resolves those stable section IDs to current mustmatch pytest item IDs immediately before execution so line-number churn cannot make the smoke lane uncollectable. Use this file as the current audit and smoke-only inventory for `SPEC_PR_DESELECT_ARGS`.

## Bash Mustmatch Lint Rule

Every `##` spec section with at least one non-skipped `bash` block must include
at least one `| mustmatch` line unless the section explicitly opts out with
`<!-- mustmatch-lint: skip -->`.

This rule exists because the mustmatch pytest plugin silently does not collect
bash blocks that never pipe to `mustmatch`. A section that only uses `jq -e` or
other exit-code checks can disappear from pytest output instead of passing,
failing, or skipping.

Prefer adding a meaningful `mustmatch` assertion on user-visible output or a
stable JSON anchor even when the section also uses `jq -e` for structured
validation. Reserve the opt-out for genuinely exit-code-only checks or cases
without a stable, meaningful output anchor. For readability, place the opt-out
comment immediately after the `##` heading.

## Audit Method

- Measured on 2026-04-13 in this worktree after `cargo build --release --locked`.
- Derived the exact commands from `make -n spec-pr`, then added `--durations=0` and saved them in `.march/spec-pr-profile.commands.sh`.
- Removed `.cache/` before the first pass to capture cold-start behavior, then reran the same commands warm without clearing caches.
- Categories: `fast` <10s, `medium` 10-60s, `slow` >60s, `flaky` = passed during the audit but failed the final end-to-end lane due provider rate limiting, `gated` = skipped because an optional-key proof did not execute in this environment.
- `spec/03-variant.md::Searching by c.HGVS` needed a supplemental cold/warm rerun because the saved xdist `--durations=0` tables did not emit a numeric row for that heading even though both lane passes executed it; those targeted measurements are saved in `.march/review-c-hgvs-{cold,warm}.log`.
- The four gated headings in this audit stayed listed in the PR lane, but this environment did not have the provider credentials needed to execute them, so their time cells are `n/a`.

| Phase | Result | Wall Time |
|---|---|---|
| First-pass parallel | passed | `100.50s` |
| First-pass serial | passed | `77.04s` |
| Warm-pass parallel | passed | `78.51s` |
| Warm-pass serial | passed | `57.38s` |
| First-pass total | passed | `177.54s` |
| Warm-pass total | passed | `135.89s` |

The audited lane fit the PR budget before any repair: no heading crossed the 60s per-heading timeout, and the full `spec-pr` lane stayed at `177.54s` cold and `135.89s` warm. The final end-to-end `make spec-pr` verification then hit a PubMed E-utilities `429 Too Many Requests` on `spec/06-article.md::Keyword Search Can Force Lexical Ranking`, so that article-search proof moved to smoke-only coverage as the smallest reliability repair.

## spec-pr Timing Audit

| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |
|---|---|---|---|---|---|---|---|---|
| `spec/07-disease.md` | `Disease Funding Stays Opt-In` | `27.12s` | passed | `17.68s` | passed | medium | keep in spec-pr | The NIH Reporter funding proofs stayed stable and well below the 60s per-heading limit, and the full lane still fits comfortably inside the PR budget. |
| `spec/02-gene.md` | `Gene Funding Stays Opt-In` | `14.74s` | passed | `14.88s` | passed | medium | keep in spec-pr | The NIH Reporter funding proofs stayed stable and well below the 60s per-heading limit, and the full lane still fits comfortably inside the PR budget. |
| `spec/18-source-labels.md` | `Markdown Source Labels` | `13.92s` | passed | `12.28s` | passed | medium | keep in spec-pr | Cold and warm timings stayed below 15s, so the suspected fan-out did not justify a trim or smoke-only move. |
| `spec/18-source-labels.md` | `Article Fulltext Resolver Order` | `n/a` | classified | `n/a` | classified | classified | keep in spec-pr | Fixture-local resolver-order proof added in ticket 274; classified in the PR lane before timing data is collected because it does not use live providers. |
| `spec/09-search-all.md` | `Distinct Disease And Keyword Stay Separate` | `12.27s` | passed | `6.66s` | passed | medium | move to smoke-only | Issue 182 later showed this live federated search timing out under the 60s PR timeout, so ticket 270 moves it to the serial smoke lane with 120s coverage. |
| `spec/18-source-labels.md` | `JSON section_sources — Gene, Drug, Disease` | `11.68s` | passed | `11.43s` | passed | medium | keep in spec-pr | The core section_sources proof stayed around 11-12s on both passes, so the representative provenance contract remains cheap enough for PRs. |
| `spec/06-article.md` | `Article Query Echo Surfaces Explicit Max-Per-Source Overrides` | `11.39s` | passed | `10.29s` | passed | medium | move to smoke-only | Issue 223 reported repeated 60s PR-lane timeouts for this live article fan-out, so ticket 270 moves it to the targeted smoke lane. |
| `spec/07-disease.md` | `Disease Survival` | `10.97s` | passed | `9.05s` | passed | medium | keep in spec-pr | Medium but stable timing; the heading stayed far below the 60s timeout and the audited lane remained well under the 10-minute target. |
| `spec/11-evidence-urls.md` | `Repaired Variant, Disease, and Drug Gaps` | `9.84s` | passed | `8.44s` | passed | fast | keep in spec-pr | The repaired-gap regression proof stayed under 10s on both passes, so there was no measured reason to demote it to smoke-only coverage. |
| `spec/07-disease.md` | `Disease Funding` | `9.20s` | passed | `9.75s` | passed | fast | keep in spec-pr | The NIH Reporter funding proofs stayed stable and well below the 60s per-heading limit, and the full lane still fits comfortably inside the PR budget. |
| `spec/09-search-all.md` | `Debug Plan` | `9.61s` | passed | `9.12s` | passed | fast | move to smoke-only | Issue 182 reported repeated 60s PR-lane timeouts for this multi-call live debug-plan proof, so ticket 270 moves it to the targeted smoke lane. |
| `spec/11-evidence-urls.md` | `JSON Metadata for Repaired Gaps` | `8.18s` | passed | `8.15s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Keyword Search Can Force Lexical Ranking` | `2.53s` | passed | `7.98s` | passed | flaky | move to smoke-only | Timings stayed low during the audit, but the final end-to-end `make spec-pr` run hit a PubMed `429 Too Many Requests`, so this article-search proof moved to the smoke lane. |
| `spec/09-search-all.md` | `Counts-only Mode` | `7.23s` | passed | `4.04s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/09-search-all.md` | `JSON Search All Preserves Article Metadata` | `7.11s` | passed | `4.00s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/09-search-all.md` | `Multi-slot Search` | `6.90s` | passed | `3.01s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `Cache Health Warning` | `n/a` | removed | `n/a` | removed | removed | removed in ticket 258 | Deleted in ticket 258 after the retained overview health heading absorbed the shipped `Cache limits` contract and the warning-path behavior stayed covered by targeted Rust unit tests. |
| `spec/01-overview.md` | `Health Check` | `12.80s` | passed | `8.98s` | passed | medium | keep in spec-pr | Ticket 258 repaired `biomcp health` with bounded ordered fan-out and a health-only per-probe timeout, then consolidated the health contract into this single retained heading. Fresh and warm reruns now stay well below the 60s PR timeout, so the heading returns to the blocking lane. |
| `spec/24-diagnostic.md` | `Local Health Readiness` | `n/a` | removed | `n/a` | removed | removed | removed in ticket 258 | Deleted in ticket 258 because the consolidated overview health heading now proves the shipped local-readiness rows, while the remaining diagnostic file focuses on source-specific search/get behavior. |
| `spec/07-disease.md` | `Disease Survival Hodgkin Mapping` | `6.09s` | passed | `5.08s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Article Batch` | `2.97s` | passed | `5.87s` | passed | fast | move to smoke-only | Issue 182 reported repeated 60s PR-lane timeouts for this live PubMed batch enrichment proof, so ticket 270 moves it to the targeted smoke lane. |
| `spec/09-search-all.md` | `Keyword Search` | `4.83s` | passed | `5.51s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-cross-entity-pivots.md` | `Disease to Articles` | `5.31s` | passed | `1.24s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Type Filter Uses The Compatible Source Set` | `2.59s` | passed | `5.12s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Phenotype Key Features` | `4.69s` | passed | `3.37s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-cross-entity-pivots.md` | `Gene to Articles` | `1.32s` | passed | `4.52s` | passed | fast | move to smoke-only | Issue 223 reported repeated 60s PR-lane timeouts for this live gene-to-article pivot, so ticket 270 moves it to the targeted smoke lane. |
| `spec/14-pathway.md` | `Default KEGG Card Stays Concise` | `4.27s` | passed | `4.51s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `Search-all Positional Query` | `4.37s` | passed | `3.72s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Survival No-Data Note` | `4.01s` | passed | `3.22s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Article Search JSON With Semantic Scholar Key` | `3.73s` | passed | `1.40s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Semantic Scholar TLDR Section` | `3.72s` | passed | `1.85s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Funding Beyond Cancer` | `3.52s` | passed | `3.62s` | passed | fast | keep in spec-pr | The NIH Reporter funding proofs stayed stable and well below the 60s per-heading limit, and the full lane still fits comfortably inside the PR budget. |
| `spec/06-article.md` | `Article Date Flag Help Advertises Accepted Formats` | `3.36s` | passed | `2.95s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Sparse Phenotype Coverage Notes` | `3.31s` | passed | `3.26s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-cross-entity-pivots.md` | `Variant pivots` | `3.08s` | passed | `1.78s` | passed | fast | move to smoke-only | Ticket 246 review found this live variant article pivot still timing out in `make spec-pr`, so ticket 270 moves it to the targeted smoke lane. |
| `spec/07-disease.md` | `Disease Search Discover Fallback` | `3.04s` | passed | `2.26s` | passed | fast | move to smoke-only | OLS4-backed discover fallback stayed fast in the audit, but provider latency is reliable enough only for the smoke lane and not for the PR-blocking lane under parallel load. |
| `spec/09-search-all.md` | `Shared Disease And Keyword Token` | `2.83s` | passed | `3.03s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Article to Entities` | `2.92s` | passed | `1.91s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Full Disease Definitions` | `2.68s` | passed | `1.92s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/23-phenotype.md` | `Symptom phrases` | `2.63s` | passed | `2.45s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Canonical Disease Genes` | `7.94s` | passed | `4.47s` | passed | fast | keep in spec-pr | One parameterized heading now covers one cancer and one non-cancer canonical disease-gene proof, preserving the live table contract while cutting duplicate network fan-out. |
| `spec/09-search-all.md` | `Single Gene Search` | `1.67s` | passed | `2.45s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/23-phenotype.md` | `Top disease follow-up` | `2.40s` | passed | `2.23s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/23-phenotype.md` | `HPO IDs` | `2.31s` | passed | `2.31s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Crosswalk Identifier Resolution` | `2.28s` | passed | `2.19s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/18-source-labels.md` | `JSON section_sources — Variant, Trial, Article` | `1.23s` | passed | `2.23s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Gene Funding` | `2.20s` | passed | `1.98s` | passed | fast | keep in spec-pr | The NIH Reporter funding proofs stayed stable and well below the 60s per-heading limit, and the full lane still fits comfortably inside the PR budget. |
| `spec/02-gene.md` | `Druggability Section` | `2.16s` | passed | `1.91s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `WikiPathways Genes Section` | `1.90s` | passed | `0.81s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Getting Disease Details` | `1.87s` | passed | `1.25s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Article Annotations` | `0.88s` | passed | `1.83s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Human Protein Atlas Section` | `1.81s` | passed | `1.75s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `Explicit KEGG Genes Section Still Renders` | `1.81s` | passed | `1.76s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/18-source-labels.md` | `JSON section_sources — Pathway, Protein, PGX, Adverse Event` | `1.73s` | passed | `1.76s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Getting Article Details` | `1.72s` | passed | `0.85s` | passed | fast | move to smoke-only | Issue 182 reported repeated 60s PR-lane timeouts for this live PubMed/enrichment proof, so ticket 270 moves it to the targeted smoke lane. |
| `spec/16-protein.md` | `Protein Complexes Section` | `1.68s` | passed | `1.68s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Genes` | `1.62s` | passed | `1.55s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Search Discover Fallback Synonym` | `1.62s` | passed | `0.94s` | passed | fast | move to smoke-only | OLS4-backed discover fallback stayed fast in the audit, but provider latency is reliable enough only for the smoke lane and not for the PR-blocking lane under parallel load. |
| `spec/04-trial.md` | `NCI Source Search` | `1.31s` | passed | `1.58s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Constraint Section` | `1.50s` | passed | `1.24s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `Long-Form MAPK Alias` | `1.24s` | passed | `1.47s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Search Fallback Miss` | `1.44s` | passed | `0.89s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Gene Protein Alternative Names` | `1.43s` | passed | `1.20s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/16-protein.md` | `Complexes JSON Next Commands` | `1.43s` | passed | `1.42s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Top Variant Summary` | `1.36s` | passed | `1.36s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Gene Protein Function Full Text` | `1.32s` | passed | `1.32s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/01-overview.md` | `Command Reference` | `1.25s` | passed | `1.12s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Search Discover Fallback for T-PLL` | `1.24s` | passed | `0.93s` | passed | fast | move to smoke-only | OLS4-backed discover fallback stayed fast in the audit, but provider latency is reliable enough only for the smoke lane and not for the PR-blocking lane under parallel load. |
| `spec/02-gene.md` | `Gene Protein Isoforms` | `1.22s` | passed | `1.22s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/15-mcp-runtime.md` | `Cache Family Stays CLI-only` | `1.18s` | passed | `1.20s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/01-overview.md` | `Article Routing Help` | `1.09s` | passed | `1.19s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Gene Card Guidance` | `1.19s` | passed | `0.93s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/15-mcp-runtime.md` | `Stdio Tool Identity` | `1.14s` | passed | `1.15s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `Exact Title Match Ranks First Across Sources` | `1.13s` | passed | `1.14s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/15-mcp-runtime.md` | `Read-only Study Boundary` | `1.14s` | passed | `1.14s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Getting Gene Details` | `1.05s` | passed | `0.82s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `WikiPathways Search Presence` | `0.97s` | passed | `0.99s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/16-protein.md` | `Getting Protein Details` | `0.94s` | passed | `0.99s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/10-workflows.md` | `Skill Overview` | `0.97s` | passed | `0.96s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Progressive Disclosure` | `0.95s` | passed | `0.94s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/16-protein.md` | `JSON Metadata Contract` | `0.93s` | passed | `0.83s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-cross-entity-pivots.md` | `Gene to Pathways` | `0.89s` | passed | `0.91s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Searching by Name` | `0.83s` | passed | `0.29s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/18-source-labels.md` | `Backward Compatibility` | `0.81s` | passed | `0.80s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/08-pgx.md` | `Population Frequencies` | `0.72s` | passed | `0.70s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `Cache Stats Markdown` | `0.72s` | passed | `0.71s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Search Offset Hint` | `0.71s` | passed | `0.69s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/16-protein.md` | `Search Table Structure` | `0.70s` | passed | `0.69s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/16-protein.md` | `Positional Search Query` | `0.68s` | passed | `0.66s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Getting Variant Details` | `0.63s` | passed | `0.16s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Residue Alias Search` | `0.63s` | passed | `0.18s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/08-pgx.md` | `PGx Recommendations` | `0.60s` | passed | `0.62s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Searching by Gene` | `0.59s` | passed | `0.23s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Getting Trial Details` | `0.59s` | passed | `0.58s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Trial Help Explains Special Filter Semantics` | `0.59s` | passed | `0.54s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Exon Deletion Phrase Search` | `0.57s` | passed | `0.22s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `Variant Positional Complex Free Text` | `0.56s` | passed | `0.26s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/15-mcp-runtime.md` | `Streamable HTTP Help` | `0.46s` | passed | `0.55s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Searching by Symbol` | `0.54s` | passed | `0.22s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `NCI Terminated Status Search` | `0.48s` | passed | `0.54s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Trial List Documents NCI Filters` | `0.54s` | passed | `0.44s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `Variant Positional Unquoted (`GENE CHANGE`)` | `0.52s` | passed | `0.24s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `Search Query Is Required Unless `--top-level`` | `0.51s` | passed | `0.51s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Age-Only Count Approximation Signal` | `0.49s` | passed | `0.50s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Invalid Date Fails Before Backend Warnings` | `0.50s` | passed | `0.44s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Legacy Name in Detail Card` | `0.49s` | passed | `0.16s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/08-pgx.md` | `Getting PGx Details` | `0.45s` | passed | `0.49s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/08-pgx.md` | `Searching by Gene` | `0.47s` | passed | `0.49s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-guide-workflows.md` | `Page Structure` | `0.36s` | passed | `0.49s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Missing Filters Fail Before Planner Warnings` | `0.46s` | passed | `0.39s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Trial Help Documents NCI Source Semantics` | `0.44s` | passed | `0.42s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Exact Disease Ranking` | `0.44s` | passed | `0.12s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `Variant Positional With Flag Coexistence` | `0.44s` | passed | `0.17s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Legacy Name Search for Stop-Gain` | `0.43s` | passed | `0.14s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-cross-entity-pivots.md` | `Disease to Drugs` | `0.41s` | passed | `0.14s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Protein Shorthand with Gene Context` | `0.40s` | passed | `0.18s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Zero-Result Positional Hint` | `0.36s` | passed | `0.40s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/08-pgx.md` | `Filtering by CPIC Level` | `0.35s` | passed | `0.40s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/08-pgx.md` | `Searching by Drug` | `0.39s` | passed | `0.38s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/11-evidence-urls.md` | `Trial Locations JSON Shape` | `0.39s` | passed | `0.38s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/15-mcp-runtime.md` | `Legacy SSE Help` | `0.37s` | passed | `0.39s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-guide-workflows.md` | `Exact Workflow Commands` | `0.35s` | passed | `0.39s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Gene Alias Search` | `0.38s` | passed | `0.12s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Searching by rsID` | `0.38s` | passed | `0.14s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Inverted Date Range Is A Clean Invalid Argument` | `0.38s` | passed | `0.30s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `PGx Positional Query` | `0.38s` | passed | `0.37s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `JSON Guidance Metadata` | `0.31s` | passed | `0.36s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Legacy Name Search for Missense` | `0.36s` | passed | `0.08s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Searching by Condition` | `0.27s` | passed | `0.34s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Filtering by Status` | `0.33s` | passed | `0.31s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/10-workflows.md` | `Listing Skills` | `0.31s` | passed | `0.33s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `WikiPathways Pathway Detail` | `0.31s` | passed | `0.20s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Variant to Trials` | `0.24s` | passed | `0.30s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Filtering by Phase` | `0.26s` | passed | `0.29s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `Trial Positional Query` | `0.23s` | passed | `0.29s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Locations Section` | `0.28s` | passed | `0.26s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `Trial Positional Multi-word Query` | `0.27s` | passed | `0.28s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `Trial Positional Plus Status Flag` | `0.25s` | passed | `0.28s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Long-Form Protein Filter` | `0.23s` | passed | `0.27s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Combined Phase 1 and 2 Search` | `0.27s` | passed | `0.27s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Eligibility Section` | `0.25s` | passed | `0.27s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Intervention Code Punctuation Normalization` | `0.25s` | passed | `0.27s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Mutation Search` | `0.24s` | passed | `0.27s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/15-mcp-runtime.md` | `Top-Level Discovery` | `0.27s` | passed | `0.24s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-cross-entity-pivots.md` | `Disease to Trials` | `0.25s` | passed | `0.27s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-cross-entity-pivots.md` | `Drug to Trials` | `0.25s` | passed | `0.27s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-guide-workflows.md` | `Guardrails and Evidence Traceability` | `0.26s` | passed | `0.26s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/01-overview.md` | `Entity Help` | `0.22s` | passed | `0.24s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Gene to Trials` | `0.24s` | passed | `0.24s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease to Trials` | `0.23s` | passed | `0.24s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-cross-entity-pivots.md` | `Gene to Trials` | `0.24s` | passed | `0.24s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Standalone Protein Shorthand Guidance` | `0.20s` | passed | `0.23s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `JSON Flag Exception` | `0.16s` | passed | `0.23s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Finding a Specific Variant` | `0.17s` | passed | `0.22s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/04-trial.md` | `Fractional Age Filter` | `0.19s` | passed | `0.22s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/02-gene.md` | `Search Table Structure` | `0.18s` | passed | `0.21s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Get with Residue Alias Guidance` | `0.19s` | passed | `0.21s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease Search No Fallback` | `0.19s` | passed | `0.21s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/10-workflows.md` | `Viewing a Skill by Slug` | `0.21s` | passed | `0.19s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `Variant Positional Quoted (`GENE CHANGE`)` | `0.18s` | passed | `0.21s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/10-workflows.md` | `Viewing a Skill by Number` | `0.20s` | passed | `0.20s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `Unsupported KEGG Events Section` | `0.16s` | passed | `0.19s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/12-search-positionals.md` | `Variant Positional Long-Form` | `0.17s` | passed | `0.18s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `Unsupported KEGG Enrichment Section` | `0.16s` | passed | `0.18s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `Unsupported WikiPathways Enrichment Section` | `0.18s` | passed | `0.18s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `ClinVar Section` | `0.15s` | passed | `0.17s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Long-Form Exact Variant Details` | `0.17s` | passed | `0.16s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Residue Alias Search with Gene Flag` | `0.15s` | passed | `0.17s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `Unsupported WikiPathways Events Section` | `0.17s` | passed | `0.17s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-cross-entity-pivots.md` | `Gene to Drugs` | `0.12s` | passed | `0.16s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Population Frequencies` | `0.15s` | passed | `0.15s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `Cache Clear Refuses Non-Interactive Destructive Runs` | `0.15s` | passed | `0.13s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `Cache Clear Is Idempotent When the HTTP Cache Is Already Gone` | `0.12s` | passed | `0.14s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `Cache Clear Reports Machine-Readable Results` | `0.13s` | passed | `0.14s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Invalid Identifier Rejection` | `0.12s` | passed | `0.12s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/07-disease.md` | `Disease to Drugs` | `0.12s` | passed | `0.12s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/14-pathway.md` | `Parser Usage Errors Exit 2` | `0.12s` | passed | `0.12s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `Cache Path` | `0.09s` | passed | `0.11s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/01-overview.md` | `Version` | `0.08s` | passed | `0.10s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `Cache Clean Summary` | `0.10s` | passed | `0.08s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `Cache Stats JSON` | `0.09s` | passed | `0.10s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/03-variant.md` | `Population Compact Markdown` | `0.08s` | passed | `0.09s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/22-cache.md` | `Cache Clear Supports Full-Wipe Automation with --yes` | `0.07s` | passed | `0.08s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Article Batch Invalid Identifier` | `0.06s` | passed | `0.07s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/17-guide-workflows.md` | `Discoverability Surfaces` | `0.06s` | passed | `0.07s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Article Batch Limit Enforcement` | `0.06s` | passed | `0.06s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Federated Deep Offset Guard` | `0.06s` | passed | `0.06s` | passed | fast | keep in spec-pr | Fast and stable on both passes, so the heading stays in the PR-blocking lane. |
| `spec/06-article.md` | `Search JSON Next Commands` | `n/a` | classified | `n/a` | classified | classified | keep in spec-pr | Existing PR-stable live article JSON proof; ticket 270 ratchet adds this inventory row without fabricating timing data. |
| `spec/09-search-all.md` | `Counts-only JSON Contract` | `n/a` | classified | `n/a` | classified | classified | keep in spec-pr | Existing PR-stable live search-all JSON proof; ticket 270 ratchet adds this inventory row without fabricating timing data. |
| `spec/18-source-labels.md` | `JSON section_sources — Diagnostic Regulatory` | `n/a` | classified | `n/a` | classified | classified | keep in spec-pr | Existing PR-stable live diagnostic/source-label proof; ticket 270 ratchet adds this inventory row without fabricating timing data. |
| `spec/21-cross-entity-see-also.md` | `Article Curated Pivots` | `n/a` | classified | `n/a` | classified | classified | keep in spec-pr | Existing PR-stable live article see-also proof; ticket 270 ratchet adds this inventory row without fabricating timing data. |
| `spec/06-article.md` | `Article Search Gene Keyword Pivot` | `n/a` | smoke-only | `n/a` | smoke-only | smoke | smoke-only | Live article keyword-pivot test stays in the nightly smoke lane. |
| `spec/06-article.md` | `Article Search Drug Keyword Pivot` | `n/a` | smoke-only | `n/a` | smoke-only | smoke | smoke-only | Live article keyword-pivot test stays in the nightly smoke lane. |
| `spec/03-variant.md` | `Searching by c.HGVS` | `0.49s` | passed | `0.49s` | passed | fast | keep in spec-pr | The targeted cold and warm reruns both stayed under a second, so the heading remains a cheap PR-lane proof. |
| `spec/11-evidence-urls.md` | `JSON Metadata Contract` | `n/a` | skipped | `n/a` | skipped | gated | keep in spec-pr (key-gated) | Optional-key proof skipped without the provider credential; keep the heading in the PR lane when keys are available. |
| `spec/11-evidence-urls.md` | `Markdown Evidence Links` | `n/a` | skipped | `n/a` | skipped | gated | keep in spec-pr (key-gated) | Optional-key proof skipped without the provider credential; keep the heading in the PR lane when keys are available. |
| `spec/12-search-positionals.md` | `Adverse-event Positional Query` | `n/a` | skipped | `n/a` | skipped | gated | keep in spec-pr (key-gated) | Optional-key proof skipped without the provider credential; keep the heading in the PR lane when keys are available. |
| `spec/17-cross-entity-pivots.md` | `Drug to Adverse Events` | `n/a` | skipped | `n/a` | skipped | gated | keep in spec-pr (key-gated) | Optional-key proof skipped without the provider credential; keep the heading in the PR lane when keys are available. |

## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)

| Node ID | Reason |
|---|---|
| `spec/02-gene.md::Gene to Articles` | Entity-to-article live literature pivot stays in the nightly smoke lane. |
| `spec/03-variant.md::Variant to Articles` | Entity-to-article live literature pivot stays in the nightly smoke lane. |
| `spec/06-article.md::Searching by Gene` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Searching by Keyword` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::First Index Date in Article Search` | Live article-search index-date coverage fans out across Europe PMC and PubMed, so it stays in the nightly smoke lane. |
| `spec/06-article.md::Keyword Search Can Force Lexical Ranking` | Moved in ticket 188 after the final `make spec-pr` verification hit a PubMed `429 Too Many Requests`; the article-search proof remains covered in the smoke lane. |
| `spec/06-article.md::Source-Specific PubTator Search Uses Default Retraction Filter` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Source-Specific PubMed Search` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Source-Specific LitSense2 Search` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Live Article Year Range Search` | Live article-search year-range coverage stays smoke-only so the deselect inventory matches the PR lane contract. |
| `spec/06-article.md::Federated Search Preserves Non-EuropePMC Matches Under Default Retraction Filter` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Keyword Anchors Tokenize In JSON Ranking Metadata` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Article Full Text Saved Markdown` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Large Article Full Text Saved Markdown` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Article Fulltext HTML Fallback Saved Markdown` | Live fulltext fallback coverage stays smoke-only so the deselect inventory matches the PR lane contract. |
| `spec/06-article.md::Article Fulltext PDF Fallback Is Opt-In` | Live fulltext fallback coverage stays smoke-only so the deselect inventory matches the PR lane contract. |
| `spec/06-article.md::Optional-Key Get Article Path` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Article Search JSON Without Semantic Scholar Key` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Article Query Echo Surfaces Explicit Max-Per-Source Overrides` | Issue 223 reported repeated PR-lane timeouts, so ticket 270 moves this live article fan-out to `make spec-smoke`. |
| `spec/06-article.md::Article Search Discover Keyword Pivot` | Ticket 246 review found this live article discover pivot timing out in `make spec-pr`, so ticket 270 moves it to `make spec-smoke`. |
| `spec/06-article.md::Getting Article Details` | Issue 182 reported repeated PR-lane timeouts, so ticket 270 moves this live PubMed/enrichment proof to `make spec-smoke`. |
| `spec/06-article.md::Article Batch` | Issue 182 reported repeated PR-lane timeouts, so ticket 270 moves this live PubMed batch proof to `make spec-smoke`. |
| `spec/06-article.md::Article Search Gene Keyword Pivot` | Live article keyword-pivot test stays in the nightly smoke lane. |
| `spec/06-article.md::Article Search Drug Keyword Pivot` | Live article keyword-pivot test stays in the nightly smoke lane. |
| `spec/06-article.md::Article Debug Plan` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Semantic Scholar Citations` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Semantic Scholar References` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Semantic Scholar Recommendations (Single Seed)` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Semantic Scholar Recommendations (Multi Seed)` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/06-article.md::Sort Behavior` | Live article-search fan-out and provider-latency surface already classified as smoke-only. |
| `spec/09-search-all.md::Debug Plan` | Issue 182 reported repeated PR-lane timeouts, so ticket 270 moves this multi-call live search proof to `make spec-smoke`. |
| `spec/09-search-all.md::Distinct Disease And Keyword Stay Separate` | Issue 182 reported repeated PR-lane timeouts, so ticket 270 moves this live federated search proof to `make spec-smoke`. |
| `spec/07-disease.md::Disease to Articles` | Entity-to-article live literature pivot stays in the nightly smoke lane. |
| `spec/07-disease.md::Disease Search Discover Fallback` | OLS4-backed discover fallback stays in the smoke lane because provider latency is acceptable there but not reliable enough for the PR-blocking lane under parallel load. |
| `spec/07-disease.md::Disease Search Discover Fallback Synonym` | OLS4-backed discover fallback stays in the smoke lane because provider latency is acceptable there but not reliable enough for the PR-blocking lane under parallel load. |
| `spec/07-disease.md::Disease Search Discover Fallback for T-PLL` | OLS4-backed discover fallback stays in the smoke lane because provider latency is acceptable there but not reliable enough for the PR-blocking lane under parallel load. |
| `spec/12-search-positionals.md::GWAS Positional Query` | GWAS positional search remains a smoke-only live-network proof. |
| `spec/02-gene.md::Gene DisGeNET Associations` | Optional live DisGeNET association coverage remains smoke-only. |
| `spec/07-disease.md::Disease DisGeNET Associations` | Optional live DisGeNET association coverage remains smoke-only. |
| `spec/17-cross-entity-pivots.md::Gene to Articles` | Issue 223 reported repeated PR-lane timeouts, so ticket 270 moves this live gene-to-article pivot to `make spec-smoke`. |
| `spec/17-cross-entity-pivots.md::Variant pivots` | Ticket 246 review found this live variant article pivot timing out in `make spec-pr`, so ticket 270 moves it to `make spec-smoke`. |
| `spec/18-source-labels.md::Article Fulltext Source Labels` | Live fulltext/source-label coverage stays smoke-only so the deselect inventory matches the PR lane contract. |
| `spec/19-discover.md` | Entire discover file stays smoke-only because its live exploratory fan-out is not part of the stable PR lane. |
| `spec/20-alias-fallback.md` | Alias-fallback live probes stay smoke-only and continue to run in the nightly suite. |
| `spec/26-workflow-ladders.md::Article Follow-up` | Live PubMed/PubTator article get aligns with the established smoke-only lane for all article get operations (ticket 282). |
| `spec/26-workflow-ladders.md::Mechanism Pathway` | Live Reactome pathway probe can be slow under parallel load; smoke-only so the PR lane stays reliable (ticket 282). |
