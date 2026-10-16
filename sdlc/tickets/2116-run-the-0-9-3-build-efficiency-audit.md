# 2116 — Run the 0.9.3 build efficiency audit

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 594fe68, accept

## Outcome

The ticket holds a measured audit of BioMCP's build and test cost after the 0.9.3 speed work already landed, and a list of findings, each with its measured cost now and the expected cost after a fix, ready to file as one ticket each.

## Evidence

- Asked for on 2026-10-16: every project audits its build and test efficiency once per release against shared rules (development, ticket-close, release, coverage and performance runs kept apart; shared output built once; named build modes; coverage reported in five groups; a fixed set of development benchmarks).
- 2069 holds the 0.9.3 before numbers at v0.9.2. Since then 2070, 2075, 2083, 2086, 2071, 2101, 2102, 2103 and 2105 landed; 2076, 2079, 2080, 2081, 2094 and 2095 are open.

- Starts from: `tools/measure-0-9-3-baseline`, `sdlc/records/2069-0-9-3-baseline.md`, `.github/workflows/ci.yml`, `Makefile`, `.config/nextest.toml`, `Cargo.toml`.
- Work, on the Linux gate host through the job runner, at one main revision:
  1. Machine time: total the last two weeks of gate-host job records and green CI runs; split into routine, release and other runs, waiting against running, and repeat runs on identical code.
  2. The twenty costliest tests or steps from the routine suite and CI; for the top five, setup time against work time.
  3. Shared output built more than once per revision: dev build, nextest archive, spec binaries, Python lane, CI jobs.
  4. Every build flag set in use (Makefile modes, CI jobs, spec profile, coverage), and whether each keeps the compiler cache and the fast linker.
  5. The coverage block in `tools/measure-0-9-3-baseline` against the rules: one stated denominator, five groups, a failed collection marked incomplete (2117 owns that last fix).
  6. The development benchmarks: a fresh build with a warm cache, a no-change rebuild, a small code edit, a docs-only edit, the routine suite and the release suite, each with wall time, peak memory and target growth; several samples, raw samples kept in the ticket.
- Decisions: findings already owned by an open ticket name that ticket and add only new numbers; this ticket fixes nothing itself.
- Changes: claims this ticket only.
- Proof: the six sections above written into this ticket with commands and raw numbers; a findings list, each with cost now and expected cost after.
- Order: safe alongside anything; run it once, at one main revision.

## Amendment, 2026-10-16

The audit, measured 2026-10-16 at main 9a58dbed8. All sections below belong to this amendment.

One revision, one job on yellow through the job runner (`pm job run --machine yellow 2116 9a58dbed8`, job `1792165746217-1331137`, 67 minutes), plus one follow-up for the failed routine-suite attempt (`1792171112471-1494889`, 11 minutes). The job ran `tools/measure-0-9-3-baseline` sections with extra samples appended, and dumped the gate host's own job records. CI numbers come from `gh api` on the Beelink (`gh` is not authenticated on yellow; CI numbers do not depend on the host, the 2069 method). Job log: `/home/ian/workspace/job-runs/biomcp/2116/1792165746217-1331137/log` on yellow; the worktree's `.cache/measure/` holds the raw logs (make-test.log, census.jsonl, pytest-durations.log, release-time.txt).

Host conditions match 2069 (yellow, 4 cores, 14 GB, rustc 1.93.1, host cargo config with incremental off and line-tables-only dev debug), with one material difference: the host was not idle. The audit's whole window overlapped ticket 2114's `make test` (11:58–12:14) and ticket 2117's coverage proof (11:55–12:59); the follow-up overlapped 2114's second `make test` (13:23–13:40) and 2117's proof again (13:20–13:36). No idle window appeared all day; polling for one was the first 50 minutes of this ticket's work. Wall times below are contended upper bounds; user/CPU time and the CI runner numbers carry the comparisons. 2069 ran idle, so its walls are not directly comparable.

### 1. Machine time (two weeks, 2026-10-03 to 2026-10-16)

Gate host (yellow, 55 biomcp job records in 14 days; no biodata jobs ran):

- Total 26,017 s ≈ 7.2 h. By class (rubric: `make test`/`test-rust`/`test-python`/`lint`/`spec`/nextest/pytest/clippy/stress = routine; `verify`/`release-gate`/release builds = release; `measure-0-9-3-baseline` and this ticket's runs = measurement; the rest = diagnostics):
  - routine 40 runs, 15,866 s = 4.41 h (six `make test` runs, 703–1,944 s each)
  - measurement 10,243 s = 2.85 h (2116's audit 4,021 s + rerun 659 s; 2117's two coverage proofs 3,887 s + 1,676 s)
  - diagnostics (probes, log reads, ruff greps) the remaining ≈ 0.3 h
  - release: zero runs on the gate host — all release-shaped work happens in CI.
- Waiting against running: the job runner records no queue wait (state files hold start, sha, args only), so gate-host waiting is unmeasured; the runner's waits (ticket lock, slot, fetch) print one line in the log head and none of the 55 logs shows a wait line. CI queue waits are ≈ 0.1 min per job.
- Repeat runs on identical code (gate host): 2073's `make test` twice at 44220861b (1,944 s then 484 s), 2073's contract nextest twice at the same sha, 2114's `make test` retried with `NEXTEST_SLOW_TIMEOUT=600000` (13:08 fail, 13:23 rerun), and this ticket's own make-test rerun. Gate-host repeats are failure retries, not reconfirmations.

CI (`ci.yml`, push events only; 1,360 runs):

- Totals: 1,360 runs, 423 wall-hours. 618 green (210.4 wall-hours), 685 failed (173.9), 52 cancelled, 5 in flight. Green runner-minutes ≈ 479 h (estimated: 227 green main × 76.3 + 308 full ticket-branch × 36.4 + 83 docs-only × 2.4, means from 20 sampled runs' job data).
- Routine against release: per green main push, routine lanes ≈ 36.1 runner-min and main-only release-shaped jobs ≈ 40.2 (release-panic 15.9 + full-features 11.7 + windows-contracts 9.2 + stress 2.7 + generated-sources 0.7). Two-week estimate: routine ≈ 331 runner-h, main-only ≈ 149 runner-h. Failed runs add another ≈ 174 wall-hours.
- Waiting against running: green main median wall 24.1 min against 76.3 runner-min (3.2× lane parallelism); per-job queue wait ≈ 0.1 min.
- Repeats on identical code: 93 shas ran more than once (98 extra runs, 7% of all runs); 66 shas have more than one green run (71 extra green runs); 7 shas failed then went green.

### 2. The twenty costliest tests or steps

CI steps, mean minutes over 12 green main runs (and 5 full ticket-branch runs where they differ), 2026-10-15/16: release-panic gate 15.2; spec gate 11.8 (main shape; 1.0–1.3 on 2080's own branch); full-features 10.6; windows compile 5.8; python gate 4.3–4.5; lane setups 2.5–2.8 (lint/spec/python), 1.8–2.0 (rust/stress), 0.8 (build/full-features); lint gate 2.1–2.3; rust gate 1.8–1.9; build archive 1.7–1.8; repository-contracts 1.3–1.7.

Local routine suite (this audit, contended): `make prepare-test` 32.8 s warm; Rust lane nextest Summary 439.9 s (includes a 240 s timeout burn); census of the same suite from the archive 338.3 s wall, 4,263 tests, 663.7 s summed exec; Python lane pytest 275.05 s (1,269 passed, 1 load-flaky failure, 3 skipped; 1,274 collected — 2069 had 1,254); mkdocs ≈ 2.5 s (2069).

Top Rust tests (census, contended upper bounds): tls_ca_bundle_contract fda_orphan 26.7 s; protein_change_get_notes_unchecked_numbering 20.2; a_stale_mcp_call_tells_the_clinician_the_cache_age 14.0; a_stale_article_search_json_states_the_cache_age 13.5; a_stale_clingen_prefetch_note 11.3; a_stale_disease_card_tells_the_clinician_the_cache_age 11.0; root_configuration_stays_additive 10.6; manifest_and_bytes_preserve_induced_archive_failure 10.1; a_stale_search_all_json 8.6; no_cache_skips_the_cache 8.4; gencc cross_process_first_use 8.4; label_fallback_fetch_error 8.0; stale_search_json_carries_cache_age 7.5; degraded_strict_leg_partials 7.3; article_entities_skin_neoplasms 6.5; article_entities_disease_rows 5.7; mcp_who_region_search_error 5.6; human_runtime_source_error 5.0; citation_evidence_keeps_the_dead_end 4.8. Top five are 47.4% and top twenty 65.3% of the suite's summed exec time.

Top Python tests (durations census, contended): test_wrapper_writes_summary_artifacts_for_pass_fixture 88.4 s; test_dead_code_allowance_audit_passes_for_repo 79.6; test_repository_historical_inventory_is_exact 30.8; test_spec_lint_rejects_preparation_owned_cargo_commands 28.2; test_repository_audit_classifies_every_source_file 26.4; test_quality_ratchet_runs_whole_surface_cli_contract 25.9; test_wheel_installs_skills_without_network 17.6; test_installer_accepts_one_valid_checksum_record 14.8; test_strict_docs_build_stays_clean 13.7; test_run_wrapper_sigkill_reaps_server_group 10.4. The top six are repo-audit ratchet contract tests.

Setup against work, the five costliest CI jobs (mean min, main): release-panic queue 0.1 / setup 0.6 / work 15.2; spec-lane 0.1 / 2.8 / 11.8; full-features 0.1 / 1.0 / 10.6; windows-contracts 0.1 / 3.0 / 6.1; python-lane 0.1 / 2.8 / 4.3. Locally, `make test`: prepare 32.8 s / Rust lane 439.9 s / Python lane 275 s + mkdocs 2.5 s.

### 3. Shared output built more than once per revision

- Local: `make test` builds the dev output once — `prepare-test`'s single `nextest archive` invocation builds bin, example and every test target, and the contract build after it is a freshness check that compiles nothing (2069 measured 1.96 s; this audit's prepare was 32.8 s from a new worktree). `make spec` reuses the dev binaries (its cargo build is a locate-only freshness pass). Coverage uses its own `target/llvm-cov-target` by design. Release and all-features are distinct outputs, each built once per revision.
- CI: the build job builds the dev archive once and the five routine lanes download it; the lanes' `shared-restore` cache restores fingerprints, so no lane recompiles — the spec lane's gate step is provider waits, proven by 2080's branch running the same step in 1.0–1.3 min with pages cut. `cargo fetch --locked` runs twice per revision (build job and python lane); the second is a warm no-op.
- The real repeat cost: the main-only jobs build release-profile output from zero on every main push because `cache: job` gives each run its own key. release-panic 15.2 min + full-features 10.6 min ≈ 25.8 runner-min per push, ≈ 97.6 runner-h over the 227 green main pushes in two weeks. Finding 1.
- Gate-host job worktrees keep their target dir between jobs (warm rebuilds across tickets). ✓

### 4. Build flag sets

1. Local routine: `--no-default-features`, dev profile (line-tables-only debug, dependency debug off), host sets incremental off; compiler cache is cargo's own target dir (no sccache), linker is default ld (no mold) — a documented host choice.
2. CI build job and lanes: the same flags and profile; Swatinem rust-cache `shared` (build job saves) / `shared-restore` (lanes) with the registry cached. CI leaves cargo incremental at its default on — a drift from the host that changes nothing about correctness and little about cost, since the CI build is per-revision anyway.
3. Clippy (lint lane): `--no-default-features --all-targets -D warnings`, its own fingerprints, separate output from the dev build.
4. Release: default features, thin LTO, one codegen unit, strip (the shipped profile); built cold per main push in release-panic and by the release workflow's platform matrix.
5. All-features (main-only): clippy `--all-features --all-targets` plus release `--all-features`; a distinct output, also cold.
6. Coverage: `llvm-cov show-env`, `CARGO_INCREMENTAL=0`, its own target directory, absolute `LLVM_PROFILE_FILE` (the 2069 fix).
7. The spec profile is gone since 2070/2075 (`SPEC_PROFILE ?= dev`, no `[profile.spec]`) — one fewer mode than 2069 measured.

### 5. The coverage block against the coverage rules

- LLVM source coverage through cargo-llvm-cov, subprocesses included (absolute profile path; the 2069 fix). ✓
- One denominator: the path regex removes tests, examples, benches, test_support and generated code; inline `#[cfg(test)]` modules stay counted. The rule is stated in the 2069 record's prose and in the script's regex, not printed by the block — 2117 owns printing it once.
- Five groups: not reported. The block prints three line percentages (outside-in, unit, combined); nothing distinguishes reached-only-by-internal-tests, both, unobserved, or failed/missing collections. The failed-suite marking is 2117's; the missing five-group shape is finding 3.
- Wrapper language: the Python lane is test code driving the Rust binary; there is no separate wrapper-language product code. N/A.

### 6. Development benchmarks (yellow; wall s / user s / peak RSS KB / target bytes; raw samples)

2069's idle v0.9.2 numbers in brackets. Contention caveat above applies to every wall here.

| Scenario | Samples (wall) | User | Peak RSS | Notes |
|---|---|---|---|---|
| Fresh build, empty target (new job worktree) | 589.0 [250.9] | 818.6 [759.4] | 6,108,696 [6,190,000] | user +7.8% is the honest compare; the wall is contention |
| Workspace-only rebuild (`cargo clean -p biomcp-cli`, deps warm) | 178.3, 634.0, 273.8 | 452.9, 445.2, 445.4 | 6,175,244 / 5,615,192 / 6,255,448 | the repeatable "warm cache" build |
| No-change rebuild | 3.09, 0.35, 0.35 | ~3 / ~0.3 | ~1,000,000 / ~104,000 | [2069: not measured] |
| Small code edit, lib file | 313.3 (gene.rs), 180.7 (lib.rs) [161.4] | 540.5 / 439.5 [432.1] | 6,290,564 / 6,253,988 | user close to 2069; wall contended |
| Small code edit, bin-only file | 2.62 (main.rs) | 3.0 | 968,136 | a bin edit does not rebuild the 30 test binaries |
| Docs-only edit | 15.5, 66.3, 0.35 [156.8–176, full recompile] | 0.7 / 1.0 / 0.3 | ~104,000 | 2070's build.rs deletion holds: a spec-page touch no longer recompiles |
| Routine suite | two attempts failed (below); Rust lane Summary 439.9 s incl. 240 s timeout burn; census wall 338.3 s; Python lane 275.05 s + mkdocs ~2.5 s [646.3 / 640.0 total, idle] | 528.0 (rerun make) | 6,403,092 (make tree) | make-test peak memory from the rerun's `/usr/bin/time -v` around `make` |
| Release build (cold) | 931.8 [837.0 / 838.8] | 1,568.6 | 4,637,328 [4,597,860] | contended tail (2117 overlap) |
| Target growth | 0 → 2,038,013,569 after make test [3,131,372,745] → 3,067,891,302 after release [4,441,874,624] | | | 2072's dependency trim shows: ~1 GB smaller than v0.9.2 at both points |

The routine suite never completed green: attempt 1 (script section) died at 1,174 of 4,263 when `entities::drug::test_support::a_stalled_drug_refusal_costs_one_discover_budget` panicked on a fixture BrokenPipe under load; attempt 2 (direct `make test`, 651 s wall) reached 4,262 passed with `rmcp_client_contract::raw_and_typed_article_query_validation_converges_before_provider_work` terminated at the 240 s slow-timeout — the same contract test 2069 saw fail at its boundary in both passes. Both attempts ran against concurrent jobs. The failure class, not a code defect: main's CI at this revision is green.

### Findings

Each finding names its measured cost now and the expected cost after a fix; the coordinator files one ticket per finding worth fixing.

1. **Cold release rebuilds on every main push** (new). The release-panic and full-features jobs use `cache: job`, so their release-profile output compiles from zero each push: 15.2 + 10.6 = 25.8 runner-min per green main push, ≈ 97.6 runner-h over two weeks (227 green main pushes), on top of ≈ 40.2 runner-min of main-only work generally. Expected after a shared release cache keyed the way the build job's shared key works (only main pushes save it): both jobs become incremental builds ≈ 2–5 min each, saving ≈ 15–20 runner-min per push (≈ 57–76 runner-h per two weeks at this pace). 2094 owns lane setup and keeps the heavy jobs main-only; it does not own their cache shape.
2. **Repo-audit ratchet tests sit in the routine Python gate** (new). The six costliest Python tests (88.4, 79.6, 30.8, 28.2, 26.4, 25.9 s — contended upper bounds, together ≈ 280 s of a 275 s-wall 4-worker lane) run whole-repo audits (dead-code allowance, zero-coupling inventory, whole-surface CLI contract, spec lint) inside the gate. Expected after moving them to repository-contracts or computing the audits once per revision and asserting against the result: the python-lane gate step drops from 4.3 min toward ≈ 2 min, ≈ 2.3 runner-min × 535 full green runs ≈ 20 runner-h per two weeks. Their existence is 2076/2081 territory (2076 landed); the placement is unowned.
3. **Coverage reports three percentages, not the five groups** (new). Outside-in / unit / combined only; no internal-only, both, unobserved, or failed/missing groups (2117 owns the failed-suite marking and the stated denominator). Cost now: every five-group question — 2081's deletions, 2082's after-record — needs the design doc's per-file method by hand. Expected after the script derives the five groups from the profraws it already merges: one report step, no extra runs.
4. **Spec lane still costs 11.8 min per full CI run on main shape** (owned by 2080; new numbers only). ≈ 105 runner-h over two weeks of green full runs; 2080's own branch already shows the fixed shape at 1.0–1.3 min. No new ticket.
5. **Slow tests and the timeout class** (owned by 2081/2095; new numbers only). 73 tests over one second in the census (62 at v0.9.2), top twenty 65.3% of summed exec time, top five 47.4%; the 240 s timeout class killed both local make-test attempts today. No new ticket.
6. **CI setup cost per lane** (owned by 2094; new numbers only). Lane setup 2.5–2.8 min on lint/spec/python and 1.8–2.0 on rust/stress; a green full run is ≈ 36 runner-min (main shape) against 2094's 11-minute wall target. No new ticket.
7. **`make test` cannot finish on a busy gate host** (new). Two attempts today failed on different load-flaky tests (fixture BrokenPipe; 240 s timeout), each burning ≈ 11 min; 2073's 1,944 s make test and 2114's retried make test show the same. `concurrentBuilds: 2` admits two heavy gates onto 4 cores, and the flaky class is pinned only in the stress lane. Expected after serializing heavy gates on the gate host (one slot for full-suite jobs) or pinning the flaky class everywhere: a routine gate that completes on the first try; saves the ≈ 11-min failed attempts and the requeues behind them. The machine-time total this audit records is small (7.2 h/fortnight) precisely because most work moved to CI, so this finding is about reliability, not hours.

### Raw evidence pointers

Job `1792165746217-1331137` (audit) and `1792171112471-1494889` (make-test retry) on yellow, with `.cache/measure/` intact in the job worktree: make-test.log, make-test-rerun.log, census.jsonl (4,263 tests with exec times), pytest-durations.log, release-time.txt, maketest-rerun-time.txt. CI raw tables: the two-week run list and the 20 sampled runs' job/step data collected 2026-10-16 from the Actions API (green-run means quoted above; run-level totals from all 1,360 runs).
