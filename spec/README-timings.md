# Spec Lane Audit

## Canary Lane Contract

| Target | Run when | Timeout | Scope | Cache contract |
|---|---|---|---|---|
| `make spec-contracts` | profile-compatible deterministic subset | `180s` per heading | offline Markdown executable contracts, including local MCP transport proof | uses the spec-profile binary selected by `PATH` and `BIOMCP_BIN`; no live-smoke commands or Python pytest contracts run in this lane |
| `make verify` | explicit opt-in operator confidence before releases or upstream checks | n/a | live public-upstream matrix for discover/OLS4, disease, article source status, variant normalization, protein, pathway, NIH Reporter funding, and other live entity/surface specs | commands go through `tools/biomcp-ci` for cache/XDG roots and optional-key stripping; NIH Reporter is additionally routed through `tools/biomcp-verify-live` so known source/auth unavailability is reported as operator-pending |
| `make release-live-smoke` | compatibility alias for operators that still use the old live-lane name | n/a | delegates to `make verify` | not part of routine gates |
| `make spec-pr` | PR CI canary and repo-local debugging of the offline executable corpus | `180s` per heading | explicit `SPEC_ROUTINE_PATHS`: local/fixture-backed CLI/MCP Markdown specs plus the parallel-isolation pytest canary | CI restores `.cache/biomcp-specs/`; cache hits export `BIOMCP_SPEC_CACHE_HIT=1`, which makes `tools/biomcp-ci` replay the warm HTTP cache with `BIOMCP_CACHE_MODE=infinite` |
| `make spec` | repo-local routine spec gate and spec debugging | `180s` per heading | the same offline `SPEC_ROUTINE_PATHS` set as `make spec-pr` | uses the same wrapper/cache root; it should pass with external network blocked while local mock servers remain reachable |
| `make test-contracts` | PR contracts lane and local docs/Python validation | n/a | selected contract build plus Python/docs contract checks | owns its selected contract build; the spec runner separately prepares stable artifact paths and a release gate supplies its already-built feature-on CLI |

Routine validation now uses offline/deterministic lanes: `make spec` and
`make spec-pr` run only explicit `SPEC_ROUTINE_PATHS`, and `make spec-contracts`
keeps a legacy deterministic subset available for profile compatibility. Public upstream confidence is
live and opt-in through `make verify` (`make release-live-smoke` remains a
compatibility alias). In that live lane, NIH Reporter funding-source/table
unavailability is an explicit operator-pending outcome, not a silent skip;
unexpected output shapes and other
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
`make spec-pr` choose timeout over the same offline path set. Before fixture
standup, `scripts/run-specs.sh` invokes one artifact-preparation phase. Routine
modes build the feature-off CLI and MCP example together once, copy them to
stable paths under `.cache/spec-artifacts/`, and capture Cargo tree and metadata
evidence once. Live verification additionally prepares a distinct feature-on
CLI and compiles filtered Rust tests once with `cargo test --no-run`. The runner
puts the selected prepared CLI directory on `PATH`, keeps the same path in
`BIOMCP_BIN`, runs Markdown specs with the standalone `mustmatch test`
binary, and runs the lone `tests/surface/test_parallel_isolation_contract.py`
pytest canary. Other Python static contracts live under `tests/surface/` and
run through `make test`, not the Markdown runner.

The routine artifact preparation is exercised through:

```console
make spec
```

After a release artifact exists, its feature-on reuse proof is:

```console
make spec SPEC_PROFILE=release SPEC_BIN="$(pwd)/target/release/biomcp"
```

`spec/surface/mcp.md` owns the nested dry-run contracts that inspect default
profile selection. Those dry runs clear recursive Make command-line propagation
locally; the runner remains the only routine owner of Cargo artifact creation.

## Active Corpus

| Path | Purpose |
|---|---|
| `spec/entity/gene.md` | routine, receipt-backed gene canary for MyGene identity, QuickGO/STRING function, HPA expression, DGIdb/Open Targets druggability, NIH funding, and local GTR diagnostics |
| `spec/entity/variant.md` | variant canary for gene-scoped search, protein-filter normalization, residue aliases, and clinical/population context |
| `spec/entity/variant-article-identity.md` | frozen G5 v2 release gate for verified variant-article positives, collision rejection, pagination, outage truthfulness, audit facts, and bounded candidate-route diagnostics |
| `spec/entity/article.md` | article canary for typed vs keyword search, source-aware result structure, annotations, and fulltext fallback |
| `spec/entity/disease-survival-fixture.md` | deterministic disease-survival canary for MyDisease grounding, SEER fixture rendering, and bounded CLI exit |
| `spec/entity/disease.md` | routine, receipt-backed disease contracts for MyDisease identity, Monarch clinical features, NIH funding, and SEER survival |
| `spec/entity/disease-live.md` | remaining operator-run disease search and broad default-card checks |
| `spec/entity/trial.md` | trial canary for condition/status search, alias normalization, age-count transparency, and eligibility/location detail |
| `spec/entity/drug.md` | routine, receipt-backed drug canary for multi-region search, brand bridging, regulatory output, ChEMBL/Open Targets target evidence, and bounded DDInter states |
| `spec/entity/protein.md` | protein canary for reviewed search defaults, UniProt identity, complexes/structures, and JSON follow-up contracts |
| `spec/entity/pathway.md` | live-smoke-only pathway canary for alias normalization, exact-title ranking, concise KEGG defaults, and source-aware section rejection |
| `spec/entity/study.md` | study canary for local cBioPortal discovery, typed analytics validation, comparison summaries, and chart output |
| `spec/entity/pgx.md` | receipt-backed routine fixture contract for gene/drug CPIC interaction search, opt-in recommendations, guidelines, and population-frequency detail |
| `spec/entity/phenotype.md` | routine, receipt-backed phenotype contracts for HPO phrase resolution, direct IDs, similarity-ranked disease output, and typed disease follow-ups |
| `spec/entity/diagnostic.md` | routine diagnostic contracts for local GTR/WHO IVD data, compact source-aware rows, and receipted OpenFDA regulatory requests |
| `spec/entity/vaers.md` | vaers canary for vaccine-first CDC aggregation, aggregate-only reporting, and explicit source limitations/combined output |
| `spec/surface/cli.md` | CLI surface canary for top-level help/list discovery, operator commands, cache-mode exceptions, and health/admin guidance |
| `spec/surface/mcp.md` | MCP surface canary for stdio/HTTP entrypoints, probe routes, and streamable-HTTP tool execution |
| `spec/surface/discover.md` | routine, receipt-backed OLS4 identity, no-match, and relational redirect contracts plus local skill guidance |
| `spec/surface/discover-live.md` | operator-run discover trial intent and credentialed UMLS code-label checks |

The live `spec/entity/variant-articles-live.md` canaries are verify-only. G5
remains useful as a focused diagnostic, but its existing identity, exact-route,
route-alias, source-status, and terminal-state assertions are hard evidence when
included in one authoritative `make verify`; the unchanged seven-variant recall
canary is also hard evidence and its runner preflights required credentials
before network work.

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
- `tools/biomcp-verify-live` owns source-pending classification for NIH Reporter
  inside `make verify`; it does not replace deterministic routine request/renderer
  proof.
- The routine deterministic lane should stay within the spec-v2 design budget:
  `<=5 minutes` warm and `<=15 minutes` cold per cache schema/version key.
- CI's `spec-stable` job restores `.cache/biomcp-specs/` with the key
  `spec-http-${runner.os}-${biomcp-version}-${spec-cache-schema-version}`.
- `spec-cache-schema-version` is a workflow-local literal so incompatible cache
  layouts stay explicit in review.

## Warm Timing Record

Ticket 0892 moved all build-inducing Cargo calls out of executable pages and
fixture helpers. On 2026-08-11, warm routine preparation took `0.77s`, including
one Cargo freshness check, stable artifact copies, and one capture each for
`cargo tree --locked` and `cargo metadata --no-deps`. A complete four-worker
`make spec` then passed in `185.30s` (`5.23s` user, `9.65s` system), compared
with the post-0968 median of `205.42s` and the intervention baseline of
`592.42s`. Live preparation also proved direct discovery and execution of the
library test binary plus all six filtered integration-test binaries; their
focused executions completed without invoking Cargo.

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

## Ticket 624 Routine Timing

The pre-change routine lane measurement was `real 229.2s` with a warm binary
on 2026-07-25 (recorded in the Ticket 622/624 investigation below). On
2026-07-29, `/usr/bin/time -p make spec` recorded `real 386.65`, `user 21.79`,
and `sys 11.49` seconds in this worktree. The after measurement includes the
new static Docker/Homebrew lane and is observational, not a claim of
compilation savings.

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

## Ticket 635 G5 Live Canary Timing

The pre-change release-binary G5 observation was **228s** against the unchanged
**180-second** heading budget. On 2026-07-31, after authoritative RefSeq exact
retrieval stopped appending CAR-derived aliases, the release-binary runner took
**166.38s** (`user 3.97s`, `sys 16.73s`) on beelink. The 180-second budget is
unchanged. This measurement is live-provider dependent; it records the alias
selection lever and elapsed result rather than a timing SLA.

## Per-Section Warm Ceilings

| Section | Lane | Ceiling | Why |
|---|---|---|---|
| `spec/entity/gene.md::All-Section Warm Budget` | quarantined from routine `make spec-pr` by ticket 372 | n/a | This timing-only canary failed twice during routine `make spec-pr` at 45599ms and 43332ms against the former 12000ms ceiling. Per ticket 371's request-contract strategy, restore it only as a deterministic benchmark/ratchet or explicit performance lane, not as a default live-heavy spec blocker. |

## Ticket 622/624 Investigation Record — 2026-07-25

Measured on beelink while the March queue was concurrently building ticket
617, so these are representative of real queue conditions rather than
clean-room best cases. Machine load ranged from 10 to 24 during collection.

| Phase | Wall | User CPU |
|---|---:|---:|
| cold `spec`-profile build (`opt-level = 3`, pre-change) | `184.5s` | `390.5s` |
| cold `cargo nextest run` | `240.6s` | `373.7s` |
| ... of which actually running 2,749 tests | `46.4s` | — |
| `make lint` | `156.5s` | `92.3s` |
| `make test` | `570.0s` | `546.1s` |
| `make spec` (routine corpus, warm binary) | `229.2s` | `22.9s` |

Two observations drove tickets 622 and 624.

**Compilation dominates the gate, not assertion count.** Only 46.4s of the
240.6s `cargo nextest` run executes tests; the rest is compile and link. The
corpus is already bottom-heavy at 2,749 unit tests against roughly 365 spec
assertions, so pruning specs is not a speed lever.

**The routine spec lane is dominated by waiting, not work.** `make spec`
spends 229.2s of wall clock against 22.9s of user CPU — a 10:1 ratio. Fixture
readiness polling, serialized fixture setup, three `uv run` cold starts, and
the deliberate `time.sleep(65)` endpoints in
`setup-article-fulltext-source-fixture.sh:564` and
`setup-article-federated-timeout-fixture.sh:49` account for the gap rather
than assertion execution. Any future attempt to speed this lane should target
the waiting, not the assertions.

**Build-cache note (not a repo concern).** sccache and mold are already active
for every build via `~/.cargo/config.toml`. However sccache was observed at
`10 GiB` of a `10 GiB` default cap — completely full — with a Rust cache hit
rate of `24.3%`. For a 525-dependency crate built across five or more
worktrees in dev, spec, release and clippy variants, that cap thrashes. Raising
`SCCACHE_CACHE_SIZE` is an operator/dotfiles change, deliberately not made in
this repo.

Earlier numbers in this file recording ~60s routine runs predate the current
fixture set and were not reproducible here.

## Ticket 625 AlphaGenome Feature-Gate Result — 2026-07-26

The 2026-07-25 pre-change cold `spec`-profile observation was `154.7s` after
the profile's `opt-level = 1` pin. With AlphaGenome's gRPC/protobuf dependency
subtree feature-gated, an isolated cold
`cargo build --locked --profile spec --no-default-features --bin biomcp --example rmcp_streamable_http_contract`
on this worktree took `122.33s` wall-clock (`68.64s` user, `19.63s` system).
The measurement used a fresh temporary `CARGO_TARGET_DIR`; it records build
cost only, not the fixture and mustmatch time in the full `make spec` gate.

## Ticket 644 Fixture Lifecycle Timing — 2026-08-02

The six signal-lifecycle cases were measured with:

```console
/usr/bin/time -p uv run --no-sync pytest tests/test_article_spec_fixture_lifecycle.py tests/test_ctgov_spec_fixture_lifecycle.py -k 'runner_signal_cleans_article_fixture or runner_termination_cleans_ctgov_process_group_env_and_port' -v --durations=0
```

The cold state was produced immediately before its run with `uv cache clean`;
there is no fixture virtualenv to remove (the repository `.venv` is the test
runner environment). The cold run was `real 9.48s` and the subsequent warm run
was `real 9.29s`. Per-test cold/warm call durations were `1.37–1.38s` for each
article signal case and `1.68–1.69s` for each CTGov signal case. These are
observations, not a timing SLA. The unchanged `python3` fixture paths avoid
`uv` during setup and readiness, so clearing the uv cache did not affect them.

## Ticket 967 fixture-result reuse — 2026-08-11

Routine pages had three exact repeated computations. The variant-article
identity report ran six times, the same saved JATS rendering ran ten times, and
the same ClinGen CSpec capture report ran twice. Each page now runs its helper
once and applies all of its existing expectations to that named result.

The remaining repeated `run-variant-article-entity-fixture.sh` calls in
`spec/entity/variant.md` have distinct mode arguments and assert distinct human,
JSON, pagination, fallback, and debug-plan scenarios. They are not duplicate
results. Their repeated server setup is a page-level fixture-lifetime and
parallel-runner concern, not something this ticket can cache as one output.

On the loaded intervention worktree, the consolidated variant-article identity
page took 46.87s. The complete `make spec` run fell from the 592.42s baseline to
352.51s even though the post-commit build-identity invalidation added 73s of
compilation to the latter run. The comparable warm-binary spec execution is
therefore about 279.5s, versus 592.1s before: roughly 2.1x faster.

## Ticket 968 bounded routine-page workers — 2026-08-11

The routine runner now gives each independent Markdown page to a bounded
runner-level worker. The default is four workers; set
`BIOMCP_SPEC_WORKERS=1` to reproduce pages serially during diagnosis. Commands
inside one page remain ordered by one Mustmatch process. Article and author
remain together because they share the article server and its mutable request
log. Section outcomes remains in its existing setup/cleanup subshell because it
owns generated inputs. Static and live verification modes retain their former
single Mustmatch invocation.

Each worker has its own process group. An interrupt terminates the group, and a
failure waits for and reports every page in the active batch, prints captured
output in path order, and does not start the next batch. Synthetic lifecycle
tests cover four-worker concurrency, one-worker diagnosis, two simultaneous
failures, deterministic attribution, interruption, and invalid configuration.
The 32-test parallel-isolation contract passed three consecutive runs at the
four-worker default on the loaded intervention machine.

After an explicit prewarm, two complete four-worker `make spec` runs took
216.47s and 194.37s wall time; both passed. Their median is 205.42s, 1.36x
faster than the 279.5s post-ticket-967 serial comparison and 2.88x faster than
the 592.1s intervention baseline. The first run used `11.33s` user CPU,
`8.31s` system CPU, and 350200 KiB peak RSS; the repeat used `5.32s`, `8.41s`,
and 106784 KiB. The Cargo build check was 0.23s in both runs. The prewarm itself
took 65.26s after a record-only commit moved Git `HEAD`, further confirming the
separate build-identity invalidation measured for ticket 970.

## Ticket 970 executable-only build identity — 2026-08-11

Git-derived version, revision, and commit-date values are no longer emitted by
the package build script. `tools/with-build-identity` computes them before a
build command, and only the two thin executable entry points consume those
compile inputs. The reusable library and its unit tests use stable package
metadata. All routine, release, lint, install, and CI build entry points use the
wrapper. A plain build from a Git-free package remains reproducible and reports
the Cargo package version with `unknown` Git/date provenance.

A synthetic Cargo package proves that a metadata-only commit keeps its library
fresh and rebuilds its binary, an executable-source change rebuilds only that
owner, and a library-source change rebuilds the library. It also proves exact
release tags, tracked dirty source, and an archive nested inside an unrelated
Git checkout. On the real BioMCP package, two clean `HEAD`-only rebuilds
reported `biomcp_cli lib true` and `biomcp bin false` in Cargo diagnostics. They
took 2.59s and 1.79s and both binaries reported the new eight-character commit.

For the same no-default-feature development binary, a fresh target-directory
build took 102.76s (`138.53s` user, `25.46s` system, 3488336 KiB peak RSS); a
same-HEAD warm build took 0.38s. The previous record-only commit forced a
65.26s optimized package rebuild. The new 1.79–2.59s HEAD-only path is roughly
25–36x faster and does not recompile the product library or its unit tests.
