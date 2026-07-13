# Spec Lane Audit

## Canary Lane Contract

| Target | Run when | Timeout | Scope | Cache contract |
|---|---|---|---|---|
| `make spec-contracts` | profile-compatible deterministic subset | `180s` per heading | offline Markdown executable contracts, including local MCP transport proof | uses the spec-profile binary selected by `PATH` and `BIOMCP_BIN`; no live-smoke commands or Python pytest contracts run in this lane |
| `make verify` | explicit opt-in operator confidence before releases or upstream checks | n/a | live public-upstream matrix for discover/OLS4, disease, article source status, variant normalization, phenotype, protein, pathway, CPIC PGx, NIH Reporter funding, and other live entity/surface specs | commands go through `tools/biomcp-ci` for cache/XDG roots and optional-key stripping; CPIC and NIH Reporter specs are additionally routed through `tools/biomcp-verify-live` so known source/auth unavailability is reported as operator-pending |
| `make release-live-smoke` | compatibility alias for operators that still use the old live-lane name | n/a | delegates to `make verify` | not part of routine gates |
| `make spec-pr` | PR CI canary and repo-local debugging of the offline executable corpus | `180s` per heading | explicit `SPEC_ROUTINE_PATHS`: local/fixture-backed CLI/MCP Markdown specs plus the parallel-isolation pytest canary | CI restores `.cache/biomcp-specs/`; cache hits export `BIOMCP_SPEC_CACHE_HIT=1`, which makes `tools/biomcp-ci` replay the warm HTTP cache with `BIOMCP_CACHE_MODE=infinite` |
| `make spec` | repo-local routine spec gate and spec debugging | `180s` per heading | the same offline `SPEC_ROUTINE_PATHS` set as `make spec-pr` | uses the same wrapper/cache root; it should pass with external network blocked while local mock servers remain reachable |
| `make test-contracts` | PR contracts lane and local docs/Python validation | n/a | selected contract build plus Python/docs contract checks | routine runs share `target/spec/biomcp` with `make spec`; `release-gate` selects `target/release/biomcp` for both consumers |

Routine validation now uses offline/deterministic lanes: `make spec` and
`make spec-pr` run only explicit `SPEC_ROUTINE_PATHS`, and `make spec-contracts`
keeps a legacy deterministic subset available for profile compatibility. Public upstream confidence is
live and opt-in through `make verify` (`make release-live-smoke` remains a
compatibility alias). In that live lane, CPIC `web_anon` auth/permission denial
and NIH Reporter funding-source/table unavailability are explicit
operator-pending outcomes, not silent skips; unexpected output shapes and other
unclassified failures stay product-red. Ticket 395 moves every live public-upstream spec out of
routine collection: phenotype/Monarch, protein/UniProt and ComplexPortal,
disease/discover OLS4 paths, pathway Reactome/WikiPathways/KEGG, plus the other
entity/surface specs that still exercise public APIs. Deterministic request,
source, fixture, renderer, local study, variant guardrail, and local MCP
contracts own routine proof, while `make verify` owns the operator live matrix.
Ticket 379 pruned representative live public-upstream assertions from
`spec/entity/article.md`, `spec/entity/variant.md`, `spec/entity/disease.md`,
and `spec/surface/discover.md`: deterministic request, source, fixture, and
renderer contracts own routine proof, while `make verify` owns live confidence
and `make release-live-smoke` remains the compatibility name for that operator
lane.
The executable docs themselves call `tools/biomcp-ci`; `make spec` and
`make spec-pr` choose timeout over the same offline path set. `scripts/run-specs.sh`
sets up the local fixtures, defaults routine modes to `target/spec/biomcp`,
keeps the caller-selected biomcp binary directory on `PATH` and the same binary
in `BIOMCP_BIN`, runs Markdown specs with the standalone `mustmatch test`
binary, and runs the lone `tests/surface/test_parallel_isolation_contract.py`
pytest canary. Other Python static contracts live under `tests/surface/` and
run through `make test`, not the Markdown runner.

## Active Corpus

| Path | Purpose |
|---|---|
| `spec/entity/gene.md` | gene search/get canary for identity, tissue-expression context, druggability, and funding/diagnostics pivots |
| `spec/entity/variant.md` | variant canary for gene-scoped search, protein-filter normalization, residue aliases, and clinical/population context |
| `spec/entity/article.md` | article canary for typed vs keyword search, source-aware result structure, annotations, and fulltext fallback |
| `spec/entity/disease-survival-fixture.md` | deterministic disease-survival canary for MyDisease grounding, SEER fixture rendering, and bounded CLI exit |
| `spec/entity/trial.md` | trial canary for condition/status search, alias normalization, age-count transparency, and eligibility/location detail |
| `spec/entity/drug.md` | drug canary for multi-region search, brand bridging, structured-indication truthfulness, and regulatory/target pivots |
| `spec/entity/disease.md` | disease canary for MONDO grounding, synonym rescue, genes/diagnostics gating, funding, and executable pivots |
| `spec/entity/protein.md` | protein canary for reviewed search defaults, UniProt identity, complexes/structures, and JSON follow-up contracts |
| `spec/entity/pathway.md` | live-smoke-only pathway canary for alias normalization, exact-title ranking, concise KEGG defaults, and source-aware section rejection |
| `spec/entity/study.md` | study canary for local cBioPortal discovery, typed analytics validation, comparison summaries, and chart output |
| `spec/entity/pgx.md` | pgx canary for gene/drug CPIC interaction search, opt-in recommendations, and population-frequency detail |
| `spec/entity/phenotype.md` | phenotype canary for HPO/symptom inputs, similarity-ranked disease output, and typed disease follow-ups |
| `spec/entity/diagnostic.md` | diagnostic canary for source-aware search, gene-first GTR guidance, compact discovery rows, and WHO detail paths |
| `spec/entity/vaers.md` | vaers canary for vaccine-first CDC aggregation, aggregate-only reporting, and explicit source limitations/combined output |
| `spec/surface/cli.md` | CLI surface canary for top-level help/list discovery, operator commands, cache-mode exceptions, and health/admin guidance |
| `spec/surface/mcp.md` | MCP surface canary for stdio/HTTP entrypoints, probe routes, and streamable-HTTP tool execution |
| `spec/surface/discover.md` | onboarding-surface canary for discover resolution, skill guidance, and fallback behavior |

## Bash Mustmatch Lint Rule

Every `##` spec section with at least one non-skipped `bash` block must include
at least one `| mustmatch` line unless the section explicitly opts out with
`<!-- mustmatch-lint: skip -->`.

This rule exists because executable Markdown only runs bash blocks that pipe to
`mustmatch`. A section that only uses `jq -e` or other exit-code checks can be
reported as skipped instead of proving the intended user-visible behavior.

Prefer adding a meaningful `mustmatch` assertion on user-visible output or a
stable JSON anchor even when the section also uses `jq -e` for structured
validation. Reserve the opt-out for genuinely exit-code-only checks or cases
without a stable, meaningful output anchor. For readability, place the opt-out
comment immediately after the `##` heading.

## Audit Method

- Measure in the current worktree after `cargo build --release --locked`.
- Keep Python setup project-free: `uv sync --extra dev --no-install-project`,
  then `uv run --no-sync ...` for pytest/spec commands.
- Run `make spec` and `make spec-contracts` for routine offline/deterministic
  timing. Run `make verify` only when intentionally measuring live upstream
  confidence.
- `tools/biomcp-ci` owns `BIOMCP_CACHE_DIR`, `XDG_CACHE_HOME`,
  `XDG_CONFIG_HOME`, optional-key stripping, and the `BIOMCP_SPEC_CACHE_HIT=1`
  to `BIOMCP_CACHE_MODE=infinite` warm-hit replay switch.
- `tools/biomcp-verify-live` owns source-pending classification for CPIC and NIH
  Reporter inside `make verify`; it does not replace deterministic routine
  request/renderer proof.
- The routine deterministic lane should stay within the spec-v2 design budget:
  `<=5 minutes` warm and `<=15 minutes` cold per cache schema/version key.
- CI's `spec-stable` job restores `.cache/biomcp-specs/` with the key
  `spec-http-${runner.os}-${biomcp-version}-${spec-cache-schema-version}`.
- `spec-cache-schema-version` is a workflow-local literal so incompatible cache
  layouts stay explicit in review.

## Warm Timing Record

Warm timing records before ticket 395 included the old live/cache-backed
`make spec-pr` corpus. After ticket 427, `make spec`/`make spec-pr` are offline
Markdown-only routine lanes and `make verify` is the operator-run live lane.
Before this cleanup, `make spec-contracts` was recorded at `386.98s` on beelink
on `2026-05-23` for the legacy deterministic subset and in the `spec-only`
validation-profile comment. After this cleanup, `/usr/bin/time -p make
spec-contracts` in this worktree recorded `real 337.47`, `user 2.51`, and `sys
5.92` on 2026-06-17.

## Contract Profile Timing Record

Timing is observational rather than a gate. On beelink on 2026-07-11, the
pre-change cold `make spec` run spent `114s` compiling the spec profile and
`704.28s` wall-clock overall; that baseline did not expose separate nextest,
Python, or MkDocs timings. Post-change observations on the same machine were:

| Phase | Cold observation | Warm observation |
|---|---:|---:|
| selected `spec` compile, including the MCP example | `95s` | `0.20s` Cargo freshness |
| nextest | not separately captured | `14.85s` |
| Python contracts | not separately captured | `28.28s` |
| strict MkDocs | not separately captured | `1.36s` |
| mustmatch routine corpus | not separately captured | about `579s` after subtracting the `95s` compile from the `673.69s` gate |
| selected `release` compile, including the MCP example | `313s` | not separately captured |

The complete warm routine `make test` was `46.06s`. Missing cold phase values
are recorded as unavailable rather than inferred from aggregate runs. These
observations make compile reuse visible without introducing a timing SLA.

## Ticket 507 Explicit-Fixture Pacing Result

On beelink on 2026-07-13, the parent binary at `e62b45066d931480b8d4fd38df09ab4216af266b`
and candidate binary at `14a8ec05f4f6716ecbaaca6c79600582542aee70` were each built once before
timing. Each repetition removed the worktree-local routine HTTP cache, started a fresh shared
article fixture, ran the candidate revision's article document, and cleaned up the fixture.
Compilation was excluded.

| Article binary | Cold inclusive samples | Min / median / max | Median setup / command / cleanup | Result |
|---|---:|---:|---:|---|
| Parent | `341693`, `341685`, `343642` ms | `341685 / 341693 / 343642` ms | `117 / 341462 / 113` ms | retained assertions: 23 passed, 3 skipped; the new run and expectation timed out |
| Candidate | `19652`, `20790`, `20670` ms | `19652 / 20670 / 20790` ms | `116 / 20441 / 111` ms | 25 passed, 3 skipped, including the new run and expectation |

The candidate median is 93.95% below the same-harness parent and 93.79% below ticket 505's
`332983` ms inclusive reference median. Existing expected command exits stayed green; the only
parent/candidate exit difference was the deliberately red five-second assertion becoming green.
The candidate median is below the `133193` ms acceptance cap.

Three complete cold candidate routines took `60.62`, `61.92`, and `54.06` seconds: min `54.06`,
median `60.62`, and max `61.92` seconds. Every run reported 25 passed/3 skipped for article,
110 passed/3 skipped for the remaining Markdown, and 30 passed for the Python isolation canary.
The median is below the `584.238` second cap and 89.31% below ticket 505's `567.221` second
pre-regression reference median.

`BIOMCP_TEST_UNPACED_ORIGIN` is an internal fixture-only signal. Only requests to its validated
exact loopback origin skip pacing, and redirects cannot leave that origin unpaced. The runner
sources the shared article fixture's source bases and signal only inside the article mustmatch
subshell; later Markdown and the Python canary retain the caller environment instead of
inheriting article overrides.

## Per-Section Warm Ceilings

| Section | Lane | Ceiling | Why |
|---|---|---|---|
| `spec/entity/gene.md::All-Section Warm Budget` | quarantined from routine `make spec-pr` by ticket 372 | n/a | This timing-only canary failed twice during routine `make spec-pr` at 45599ms and 43332ms against the former 12000ms ceiling. Per ticket 371's request-contract strategy, restore it only as a deterministic benchmark/ratchet or explicit performance lane, not as a default live-heavy spec blocker. |
