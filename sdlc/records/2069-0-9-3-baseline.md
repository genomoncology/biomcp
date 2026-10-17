# 2069 — The 0.9.3 baseline

Measured 2026-10-13 and 2026-10-14 on the BioMCP 0.9 Linux build host, against a fresh clone of tag `v0.9.2` (commit `4049666f5cce2794f16cdbdf0ded49878e5bb147`) for each pass, with the machine otherwise idle. `tools/measure-0-9-3-baseline` printed every number below with the command behind it; this record holds its output from two passes. Ticket 2082 repeats the same commands after the 0.9.3 plan lands and compares.

## Method

- Each pass: a fresh clone of tag v0.9.2 under `~/2069-baseline/passN`, then the script run once, holding `~/.yr-checkout.lock` and `~/.<build-host>-gate.lock` for the whole run. The script refuses to start while any cargo, rustc, nextest, clippy, pytest, mkdocs, make, uv, cargo-deny, or cargo-llvm-cov process is alive; the quiet check's output is in each pass's log.
- Section order: host and toolchain conditions, clone identity, quiet check, cold dev build of every target, rebuild after one Rust edit, rebuild after one doc edit, dependency counts, one un-timed `make prepare-test` (context only, so the timed gates start with fresh artifacts), `make test`, build-folder size, test-duration census, Python collection count, spec-page block census, `make spec`, `make lint`, cold release build, build-folder size again, stripped binary size, `cargo llvm-lines`, `deny.toml` advisory count, the three coverage reports, the run-time block, and closing host conditions.
- Routine gates use the Makefile's own feature set (`--no-default-features`); the release build uses the default features and the shipped profile (thin LTO, one codegen unit, strip), as the release workflow builds it. Nothing was tuned.
- The coverage numbers are `cargo llvm-cov report` line percentages with test-only files removed by path (regex in each log). Inline `#[cfg(test)]` modules stay counted, so these are the llvm-cov view; the iteration-speed study's span-reduced production-line figures (65.1 / 73.9 / 84.1) are not reproducible by this script and stay quoted from that study.
- The run-time block uses the release binary. Warm means `BIOMCP_CACHE_MODE=infinite` against a cache directory prefetched once by each command; cache off means `BIOMCP_CACHE_MODE=off` against the same directory. Five runs each, median and spread. The five commands: `get gene BRAF`, `get variant "BRAF V600E"`, `get drug imatinib`, `search article -g BRAF --limit 10`, `search all --gene BRAF --disease melanoma --counts-only`. These runs use the live network; their spread between passes reflects the providers, not the machine.
- CI history comes from the same script's `ci-history` section run on the agent host (`gh` is not authenticated on the Linux check host); CI numbers do not depend on the host.

## Pass-shape notes (what is not one straight run)

Each pass has two logs because each pass's first attempt stopped at `make test` and the resumed attempt completed it:

1. The same contract test, `rmcp_client_contract::raw_and_typed_article_query_validation_converges_before_provider_work`, failed at its boundary in both passes' first `make test` (60.2 s then 120.3 s) and passed on every rerun with all 4205 tests green; the failed attempts' timings are not used. Both first-attempt logs are embedded below.
2. Three script defects surfaced during pass 1 and a fourth at pass 2's reports; the landed script carries all four fixes, and this record quotes the fixed numbers for both passes: the `llvm-lines` sum double-counted the table's own TOTAL row (pass 1 re-ran that section, which prints the raw table); `LLVM_PROFILE_FILE` was a relative path, so every suite subprocess that changed working directory silently dropped its profile (pass 1's outside-in suites were rerun with an absolute path — the first attempt's 23.56% is an artifact, the corrected number is 52.66%); `cargo llvm-cov report` discovery breaks when the show-env exports are present, so the reports now unset `CARGO_TARGET_DIR`, `CARGO_LLVM_COV` and `CARGO_LLVM_COV_SHOW_ENV` (pass 2's three reports were produced by hand with exactly that command after the script's first report attempt failed; both report runs are in the logs).
3. Two outlier rows were rerun once each after pass 2, inside the pass 2 log: the doc-edit rebuild (168.5 s, confirming pass 2's 170.7 against pass 1's low 156.8) and `get drug imatinib` warm and cache off (the widest run-time spread). Pass 1's `--version` rows were re-measured in milliseconds by the addendum block in its log, and pass 2's script printed them in milliseconds natively.

## The numbers

The table gives pass 1, pass 2, and the difference in percent of pass 1. Commands for every number are in the embedded logs; each row names its RESULT key. Network-dependent rows are marked.

| Number (RESULT key) | Pass 1 | Pass 2 | Delta |
|---|---|---|---|
| Stripped release binary, bytes (`stripped_release_binary_bytes`) | 39122976 | 39122976 | identical |
| Build folder bytes after make test (`build_folder_bytes_after_make_test`) | 3131372745 | 3131377964 | +0.0% |
| Build folder bytes after release build (`build_folder_bytes_after_release_build`) | 4441874624 | 4441880999 | +0.0% |
| Cold release build, wall s (`release_build_wall_s`) | 837.0 | 838.8 | +0.2% |
| Cold dev build of all targets, wall s (`cold_dev_build_wall_s`) | 250.9 | 260.3 | +3.7% |
| Rebuild after one Rust edit, wall s (`rust_edit_rebuild_wall_s`) | 161.4 | 162.6 | +0.7% |
| Rebuild after one doc edit, wall s (`doc_edit_rebuild_wall_s`) | 156.8 | 168.5 | rerun once: 168.5; cluster 157-176 |
| Direct dependencies (normal, optional) (`direct_dependencies_normal`) | 58 (optional: 3: prost, tonic, zstd) | 58 (optional: 3: prost, tonic, zstd) | identical |
| Direct dev dependencies (`direct_dependencies_dev`) | 5 | 5 | identical |
| Windows-only / Unix dev dependencies (`direct_dependencies_windows_only`) | 1 | 1 | 1 / 1, identical |
| Cargo.lock packages (`cargo_lock_packages`) | 488 | 488 | identical |
| Cargo.lock duplicate names (`cargo_lock_duplicate_names`) | 50 (extra copies: 68) | 50 (extra copies: 68) | identical |
| Crates compiled, normal+build (`crates_compiled_normal_build`) | 338 | 338 | identical |
| Crates compiled, with dev (`crates_compiled_with_dev`) | 349 | 349 | identical |
| Duplicate names in the routine tree (`duplicate_crate_names_normal_build`) | 24 (base64, block-buffer, cipher, cpufeatures, crypto-common, getrandom, html5ever, inout, markup5ever, phf, phf_codegen, phf_generator, phf_macros, phf_shared, rand, rand_core, siphasher, string_cache, string_cache_codegen, syn, tendril, thiserror, thiserror-impl, unicode-width) | 24 (base64, block-buffer, cipher, cpufeatures, crypto-common, getrandom, html5ever, inout, markup5ever, phf, phf_codegen, phf_generator, phf_macros, phf_shared, rand, rand_core, siphasher, string_cache, string_cache_codegen, syn, tendril, thiserror, thiserror-impl, unicode-width) | 24 in both, identical |
| make test, wall s (`make_test_wall_s`) | 646.3 | 640.0 | -1.0% |
| make spec, wall s (`make_spec_wall_s`) | 657.2 | 668.3 | +1.7% |
| make lint, wall s (`make_lint_wall_s`) | 195.9 | 196.5 | +0.3% |
| Rust tests run / skipped (`rust_tests_run`) | 4205 | 4205 | 4205 run, 33 skipped in both |
| Python lane (make test) (`python_lane_summary`) | 1251 passed, 3 skipped in 476.00s (0:07:56) | 1251 passed, 3 skipped in 477.50s (0:07:57) | identical counts |
| Python tests collected (`python_tests_collected`) | 1254 | 1254 | identical |
| Spec bash blocks / pages (`spec_bash_blocks`) | 496 | 496 | 496 blocks in 38 pages, identical |
| Outside-in line coverage, % (`coverage_outside_in_lines_pct`) | 52.66 | 52.65 | -0.02% |
| Unit line coverage, % (`coverage_unit_lines_pct`) | 78.30 | 78.30 | identical |
| Combined line coverage, % (`coverage_combined_lines_pct`) | 86.26 | 86.27 | +0.01% |
| Tests over one second (`tests_over_1s`) | 62 | 62 | identical count; summed time 296.2 vs 282.2 s |
| LLVM lines / copies (`llvm_lines_total`) | 6819551 | 6819551 | 6819551 lines, 138257 copies, identical |
| Peak release build memory, KB (`release_peak_rss_kb`) | 4597860 | 4597756 | -0.0% |
| Ignored advisories in deny.toml (`ignored_advisories`) | 6 | 6 | identical |

Run-time numbers (amendment). Network-dependent rows are marked; the medians are of five runs each.

| Case (median of 5, seconds) | Pass 1 | Pass 2 |
|---|---|---|
| nocache get drug imatinib (network) | 3.38 | 4.85 |
| nocache get gene BRAF (network) | 0.45 | 0.47 |
| nocache get variant BRAF V600E (network) | 0.52 | 0.52 |
| nocache search all BRAF melanoma (network) | 9.74 | 10.45 |
| nocache search article BRAF (network) | 1.93 | 1.90 |
| warm get drug imatinib (network) | 3.13 | 1.61 |
| warm get gene BRAF | 0.19 | 0.19 |
| warm get variant BRAF V600E | 0.02 | 0.02 |
| warm search all BRAF melanoma (network) | 12.02 | 11.35 |
| warm search article BRAF (network) | 1.72 | 1.71 |

Version start-up in milliseconds (pass 1 from its addendum block, pass 2 from its run): empty cache 4.1 ms vs 3.9 ms; the host's large cache 4.0 ms vs 3.9 ms; spreads 0.2 ms. `--version` exits before any cache work in both passes, so a large cache does not slow start-up.

The widest run-time row, `get drug imatinib`, was rerun once after pass 2: warm median 1.39 s (pass 1: 3.13, pass 2: 1.61), cache off median 6.62 s (pass 1: 3.38, pass 2: 4.85). Every run exits 0 with the same 2410-byte output; the wait varies because one of the drug providers is slow and not cacheable. The spread, not the median, is the honest number for that row.

Output sizes (bytes, one warm run each): gene 1646, variant 1371, drug 2410 — identical in both passes; article 2555 then 2861 and search all 1381 then 2015 — the network-shaped answers change with the providers' live responses, which is the same variance their medians carry.

## CI history (the last ten green runs of ci.yml)

Collected with the script's ci-history section on the agent host, once at each pass (gh is not authenticated on the Linux check host; CI numbers do not depend on the host). The raw tables follow.

### At pass 1

```

### locks — skipped by --no-locks

### ci-history — the last ten green CI runs of ci.yml (wall time and runner-minutes)
$ gh api + python3 (per run: updated_at - run_started_at; runner-minutes summed over jobs)
run 38486201452 bbe6e5998 2026-10-13T23:45:52Z wall 25.2m runner 70.9m
run 38485640028 35f42b858 2026-10-13T23:40:12Z wall 24.8m runner 70.3m
run 38485622606 fc8921610 2026-10-13T23:40:02Z wall 28.7m runner 68.9m
run 38483210845 77ac56243 2026-10-13T23:16:21Z wall 28.8m runner 73.4m
run 38482969496 486476226 2026-10-13T23:14:08Z wall 25.4m runner 70.1m
run 38470498225 8fbce35e6 2026-10-13T21:25:32Z wall 27.4m runner 68.5m
run 38469225000 7bc323c58 2026-10-13T21:15:35Z wall 22.9m runner 70.8m
run 38468767226 7bc323c58 2026-10-13T21:12:03Z wall 3.4m runner 3.5m
run 38468744713 5c0a7a047 2026-10-13T21:11:53Z wall 2.3m runner 3.9m
run 38468637378 10d60f569 2026-10-13T21:11:01Z wall 28.2m runner 73.5m
RESULT ci_green_runs = 10
RESULT ci_wall_minutes_median = 25.4
RESULT ci_wall_minutes_range = 2.3-28.8
RESULT ci_runner_minutes_mean = 57.4
RESULT ci_full_runs = 8 (runner-minutes over 30; docs-only runs excluded)
RESULT ci_full_wall_minutes_median = 27.4
RESULT ci_full_wall_minutes_range = 22.9-28.8
RESULT ci_full_runner_minutes_mean = 70.8

### done — measurement complete
```

### At pass 2

```

### locks — skipped by --no-locks

### ci-history — the last ten green CI runs of ci.yml (wall time and runner-minutes)
$ gh api + python3 (per run: updated_at - run_started_at; runner-minutes summed over jobs)
run 38511664227 c139e21b5 2026-10-14T04:29:39Z wall 24.9m runner 72.0m
run 38486201452 bbe6e5998 2026-10-13T23:45:52Z wall 25.2m runner 70.9m
run 38485640028 35f42b858 2026-10-13T23:40:12Z wall 24.8m runner 70.3m
run 38485622606 fc8921610 2026-10-13T23:40:02Z wall 28.7m runner 68.9m
run 38483210845 77ac56243 2026-10-13T23:16:21Z wall 28.8m runner 73.4m
run 38482969496 486476226 2026-10-13T23:14:08Z wall 25.4m runner 70.1m
run 38470498225 8fbce35e6 2026-10-13T21:25:32Z wall 27.4m runner 68.5m
run 38469225000 7bc323c58 2026-10-13T21:15:35Z wall 22.9m runner 70.8m
run 38468767226 7bc323c58 2026-10-13T21:12:03Z wall 3.4m runner 3.5m
run 38468744713 5c0a7a047 2026-10-13T21:11:53Z wall 2.3m runner 3.9m
RESULT ci_green_runs = 10
RESULT ci_wall_minutes_median = 25.2
RESULT ci_wall_minutes_range = 2.3-28.8
RESULT ci_runner_minutes_mean = 57.2
RESULT ci_full_runs = 8 (runner-minutes over 30; docs-only runs excluded)
RESULT ci_full_wall_minutes_median = 25.4
RESULT ci_full_wall_minutes_range = 22.9-28.8
RESULT ci_full_runner_minutes_mean = 70.6

### done — measurement complete
```

Eight of the ten newest green runs are full runs (the two 2-3 minute ones are docs-only pushes the classify job skips Rust jobs for). Full runs: wall median 27.4 then 25.4 minutes, range 22.9-28.8, mean runner-minutes 70.8 then 70.6 — matching the iteration-speed study's 22-28 minutes and about 70 runner-minutes.

## Machine conditions

Linux check host: Ubuntu, Linux 6.17, 4 cores, 14 GB RAM, one spinning disk (df and load lines are in every log). Toolchain: rustc/cargo 1.93.1, cargo-nextest 0.9.132, cargo-deny 0.19.4, cargo-llvm-cov 0.9.1, cargo-llvm-lines 0.4.48, ruff 0.13.2, Python 3.13.7, uv 0.9.26, Node 22.22.3, mustmatch 0.1.0. The host cargo config turns incremental builds off, keeps line-tables-only dev debug, and drops dependency debug info (full text in each log). Both passes started with the one-minute load average at or below 0.2 and both locks held; no other build or gate ran (the quiet-check output is in each log). Disk stayed above 430 GB free. Pass 1 ran 2026-10-13 evening, pass 2 finished 2026-10-14 early morning; the host's large HTTP cache held 19,689 files, 197 MB.

## What the numbers say (for 0.9.3)

- Every count is deterministic across passes (dependencies, Cargo.lock, crates, duplicates, test counts, spec blocks, LLVM lines, advisory count, stripped bytes) and every timing agrees within a few percent, with the doc-edit rebuild the widest build row (157-176 s across four measurements; it pays the same full recompile as a Rust edit because v0.9.2's empty build.rs makes cargo scan every package file).
- The run-time amendment's answer: start-up does not read the cache (`--version` is ~4 ms with an empty and with the host's 19,689-file cache); a warm cache serves `get gene BRAF` in 0.19 s and `get variant "BRAF V600E"` in 0.02 s; `get drug imatinib` and both search commands stay network-bound even warm because their provider requests are not all cacheable, so their medians swing with the providers (drug 1.4-3.1 s warm).
- One contract test, `rmcp_client_contract::raw_and_typed_article_query_validation_converges_before_provider_work`, failed at its boundary in both passes' first `make test` (60.2 s then 120.3 s) and passed on every rerun; under instrumentation the same test family accounts for the one failed integration test in the coverage runs. CI never shows it. It is load-flaky on this host, the class the stress lane pins to two CPUs.
- Under coverage instrumentation the Python lane loses 26 tests (cargo-driving tests see the coverage build environment) and 1225 pass in both passes, identically; the integration suite runs 135 with 1 failing, identically. The coverage percentages reproduce to a hundredth of a point.
- Counting-method differences from the two studies, so nobody reconciles them by hand: this script counts dependency crates excluding the workspace's own two members (338/349 against the study's 339/351 compiled units); duplicate names are counted per graph (24 in the routine tree, 50 across everything Cargo.lock holds, against the study's 34 by `cargo tree -d` on the default feature set); spec blocks are counted by exact bash fence (496 in 38 pages, against the study's 592 in 52 pages by its own grep); tests over one second are counted from per-test durations (62 against the study's 81, which counted from the summary times); coverage here is the llvm-cov view including inline test modules (52.66/78.30/86.26 against the study's span-reduced 65.1/73.9/84.1 — the study's own llvm-cov summaries read 52.4/78.3/86.2).

## Pass outputs (verbatim script output)

Absolute home paths are rewritten to `~` for the repository. Private host names and host-specific lock/configuration file names are replaced with role labels or `<build-host>`; commands, measurements and result identifiers otherwise remain unchanged. Each pass has two logs: the first attempt stopped at the flaky contract test in `make test` (its timing is not used), and the resumed attempt completed the pass. Pass 1's outside-in coverage suites were rerun after the profile-path fix and its reports follow that rerun; pass 2's reports were rerun once after the report-discovery fix; both reruns are inside the logs below.

### Pass 1, first attempt (host through dependency counts, and the flaky failure)

```

### locks — held for the whole run
$ flock -n ~/.yr-checkout.lock && flock -n ~/.<build-host>-gate.lock
both locks held by pid 2271455

### host — machine and toolchain conditions at the start
$ hostname; uname -sr; nproc
Linux check host
Linux 6.17.0-35-generic
4
$ cat /proc/loadavg
0.10 0.14 0.17 1/508 2271464
$ free -g | sed -n 1,2p
               total        used        free      shared  buff/cache   available
Mem:              14           1           4           0          10          13
$ df -h . | tail -1
/dev/sda1       916G  427G  443G  50% /
$ rustc --version; cargo --version; cargo nextest --version; cargo-deny --version; cargo llvm-cov --version; cargo llvm-lines --version; ruff --version; python3 --version; uv --version; node --version 2>/dev/null || true; mustmatch --version 2>/dev/null || true
rustc 1.93.1 (01f6ddf75 2026-02-11)
cargo 1.93.1 (083ac5135 2025-12-15)
cargo-nextest 0.9.132 (6e4a9d6f2 2026-03-20)
release: 0.9.132
commit-hash: 6e4a9d6f2c4964f30ff54a8cd5466f8869267daa
commit-date: 2026-03-20
host: x86_64-unknown-linux-gnu
cargo-deny 0.19.4
cargo-llvm-cov 0.9.1
cargo-llvm-lines 0.4.48
ruff 0.13.2
Python 3.13.7
uv 0.9.26
v22.22.3
mustmatch 0.1.0
$ cat ~/.cargo/config.toml 2>/dev/null || echo "no ~/.cargo/config.toml"
# Lean Rust dev builds on the Linux check host, matching the agent host (user cargo/config.toml).
# Measured on BioMCP 2026-09-22: fresh build 833 s -> 296 s, next ticket 374 s -> 145 s, 13 GB -> 2.1 GB.
# Long edit-compile loop on one crate: CARGO_INCREMENTAL=1. Debugger: CARGO_PROFILE_DEV_DEBUG=true.
# No sccache or mold here, so no rustc-wrapper or linker flags. Record: user configuration, ubuntu/<build-host>-build-host.md.
[build]
incremental = false

[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = false
$ cat .config/nextest.toml 2>/dev/null || echo "no .config/nextest.toml"
# A hung test must die, not stall the gate: nextest marks a test SLOW when
# it passes the slow-timeout period and terminates it after the second
# period elapses, so a blocking wait fails its test instead of hanging the
# run forever (ticket 2038 finding 7, 2035 #10). The suite's own watchdogs
# bound legitimate waits well under one period.
[profile.default]
slow-timeout = { period = "120s", terminate-after = 2 }

### clone — the tree being measured
$ git rev-parse HEAD; git describe --tags --exact-match HEAD 2>/dev/null || true; git log -1 --format="%H %s"; git status --porcelain | wc -l
4049666f5cce2794f16cdbdf0ded49878e5bb147
v0.9.2
4049666f5cce2794f16cdbdf0ded49878e5bb147 Land 2060: Make the release CI check use flags the gh CLI has
0

### quiet — no other build or gate process is running
$ pgrep -x cargo rustc cargo-nextest clippy-driver pytest mkdocs make uv cargo-deny cargo-llvm-cov
quiet: no matching process is running

### cold-dev-build — cold dev build of every target (routine feature set)
$ /usr/bin/time -f "TIMEFMT WALL %e USER %U SYS %S MAXRSS_KB %M" cargo build --locked --no-default-features --all-targets --timings
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling cfg-if v1.0.4
   Compiling libc v0.2.182
   Compiling serde_core v1.0.228
   Compiling syn v2.0.117
   Compiling serde v1.0.228
   Compiling version_check v0.9.5
   Compiling itoa v1.0.17
   Compiling memchr v2.8.0
   Compiling smallvec v1.15.1
   Compiling generic-array v0.14.7
   Compiling typenum v1.19.0
   Compiling pin-project-lite v0.2.17
   Compiling siphasher v1.0.2
   Compiling bytes v1.11.1
   Compiling futures-core v0.3.32
   Compiling once_cell v1.21.3
   Compiling jobserver v0.1.34
   Compiling find-msvc-tools v0.1.9
   Compiling zerocopy v0.8.40
   Compiling shlex v1.3.0
   Compiling stable_deref_trait v1.2.1
   Compiling cc v1.2.56
   Compiling log v0.4.29
   Compiling futures-sink v0.3.32
   Compiling subtle v2.6.1
   Compiling errno v0.3.14
   Compiling signal-hook-registry v1.4.8
   Compiling futures-channel v0.3.32
   Compiling synstructure v0.13.2
   Compiling socket2 v0.6.2
   Compiling mio v1.1.1
   Compiling bitflags v2.11.0
   Compiling futures-io v0.3.32
   Compiling futures-task v0.3.32
   Compiling slab v0.4.12
   Compiling http v1.4.0
   Compiling getrandom v0.2.17
   Compiling rand_core v0.6.4
   Compiling crypto-common v0.1.7
   Compiling zmij v1.0.21
   Compiling serde_derive v1.0.228
   Compiling zerofrom-derive v0.1.6
   Compiling yoke-derive v0.8.1
   Compiling ppv-lite86 v0.2.21
   Compiling tokio-macros v2.6.1
   Compiling zerofrom v0.1.6
   Compiling yoke v0.8.1
   Compiling zerovec-derive v0.11.2
   Compiling futures-macro v0.3.32
   Compiling tokio v1.50.0
   Compiling futures-util v0.3.32
   Compiling zerovec v0.11.5
   Compiling displaydoc v0.2.5
   Compiling rand_chacha v0.3.1
   Compiling rand v0.8.6
   Compiling tinystr v0.8.2
   Compiling parking_lot_core v0.9.12
   Compiling cpufeatures v0.2.17
   Compiling writeable v0.6.2
   Compiling new_debug_unreachable v1.0.6
   Compiling litemap v0.8.1
   Compiling zerotrie v0.2.3
   Compiling icu_locale_core v2.1.1
   Compiling potential_utf v0.1.4
   Compiling http-body v1.0.1
   Compiling block-buffer v0.10.4
   Compiling percent-encoding v2.3.2
   Compiling crc32fast v1.5.0
   Compiling icu_properties_data v2.1.2
   Compiling icu_normalizer_data v2.1.1
   Compiling scopeguard v1.2.0
   Compiling serde_json v1.0.149
   Compiling lock_api v0.4.14
   Compiling digest v0.10.7
   Compiling icu_provider v2.1.1
   Compiling icu_collections v2.1.1
   Compiling tracing-core v0.1.36
   Compiling phf_shared v0.13.1
   Compiling adler2 v2.0.1
   Compiling simd-adler32 v0.3.8
   Compiling siphasher v0.3.11
   Compiling miniz_oxide v0.8.9
   Compiling parking_lot v0.12.5
   Compiling tracing-attributes v0.1.31
   Compiling ring v0.17.14
   Compiling phf_shared v0.11.3
   Compiling httparse v1.10.1
   Compiling fastrand v2.3.0
   Compiling precomputed-hash v0.1.1
   Compiling tower-service v0.3.3
   Compiling phf_generator v0.13.1
   Compiling tracing v0.1.44
   Compiling flate2 v1.1.9
   Compiling icu_normalizer v2.1.1
   Compiling icu_properties v2.1.2
   Compiling try-lock v0.2.5
   Compiling base64 v0.22.1
   Compiling httpdate v1.0.3
   Compiling zeroize v1.8.2
   Compiling rustls-pki-types v1.14.0
   Compiling want v0.3.1
   Compiling phf_generator v0.11.3
   Compiling phf_shared v0.10.0
   Compiling form_urlencoded v1.2.2
   Compiling idna_adapter v1.2.1
   Compiling http-body-util v0.1.3
   Compiling sync_wrapper v1.0.2
   Compiling tower-layer v0.3.3
   Compiling rand_core v0.10.0
   Compiling rustix v1.1.4
   Compiling getrandom v0.4.2
   Compiling untrusted v0.9.0
   Compiling utf8_iter v1.0.4
   Compiling pkg-config v0.3.32
   Compiling utf-8 v0.7.6
   Compiling pin-utils v0.1.0
   Compiling atomic-waker v1.1.2
   Compiling hyper v1.8.1
   Compiling idna v1.1.0
   Compiling phf_generator v0.10.0
   Compiling autocfg v1.5.0
   Compiling compression-core v0.4.31
   Compiling linux-raw-sys v0.12.1
   Compiling thiserror v1.0.69
   Compiling ipnet v2.12.0
   Compiling num-traits v0.2.19
   Compiling hyper-util v0.1.20
   Compiling compression-codecs v0.4.37
   Compiling url v2.5.8
   Compiling phf_codegen v0.10.0
   Compiling tower v0.5.3
   Compiling futures-executor v0.3.32
   Compiling thiserror-impl v1.0.69
   Compiling rustls v0.23.45
   Compiling strsim v0.11.1
   Compiling async-compression v0.4.41
   Compiling string_cache_codegen v0.6.1
   Compiling phf_codegen v0.13.1
   Compiling tokio-util v0.7.18
   Compiling sha2 v0.10.9
   Compiling crossbeam-utils v0.8.21
   Compiling rustls-webpki v0.103.15
   Compiling ryu v1.0.23
   Compiling powerfmt v0.2.0
   Compiling mime v0.3.17
   Compiling same-file v1.0.6
   Compiling unicase v2.9.0
   Compiling iri-string v0.7.10
   Compiling mime_guess v2.0.5
   Compiling walkdir v2.5.0
   Compiling deranged v0.5.8
   Compiling web_atoms v0.2.4
   Compiling tower-http v0.6.8
   Compiling futures v0.3.32
   Compiling phf_macros v0.13.1
   Compiling aho-corasick v1.1.4
   Compiling iana-time-zone v0.1.65
   Compiling syn v1.0.109
   Compiling num_threads v0.1.7
   Compiling regex-syntax v0.8.10
   Compiling num-conv v0.2.0
   Compiling cfg_aliases v0.2.1
   Compiling time-core v0.1.8
   Compiling time v0.3.47
   Compiling tokio-rustls v0.26.4
   Compiling nix v0.31.2
   Compiling regex-automata v0.4.14
   Compiling phf v0.13.1
   Compiling chrono v0.4.44
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling string_cache_codegen v0.5.4
   Compiling webpki-roots v1.0.6
   Compiling string_cache v0.9.0
   Compiling async-trait v0.1.89
   Compiling block-padding v0.3.3
   Compiling hybrid-array v0.4.10
   Compiling anyhow v1.0.103
   Compiling ident_case v1.0.1
   Compiling crossbeam-epoch v0.9.20
   Compiling mac v0.1.1
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.16.1
   Compiling thiserror v2.0.18
   Compiling tempfile v3.26.0
   Compiling futf v0.1.5
   Compiling darling_core v0.23.0
   Compiling indexmap v2.13.0
   Compiling inout v0.1.4
   Compiling phf v0.10.1
   Compiling markup5ever v0.11.0
   Compiling hyper-rustls v0.27.7
   Compiling serde_urlencoded v0.7.1
   Compiling tendril v0.5.0
   Compiling phf_macros v0.11.3
   Compiling tokio-stream v0.1.18
   Compiling thiserror-impl v2.0.18
   Compiling miette-derive v5.10.0
   Compiling byteorder v1.5.0
   Compiling unicode-width v0.1.14
   Compiling ref-cast v1.0.25
   Compiling either v1.15.0
   Compiling utf8parse v0.2.2
   Compiling getrandom v0.3.4
   Compiling dtoa v1.0.11
   Compiling dtoa-short v0.3.5
   Compiling anstyle-parse v0.2.7
   Compiling miette v5.10.0
   Compiling darling_macro v0.23.0
   Compiling phf v0.11.3
   Compiling markup5ever v0.38.0
   Compiling reqwest v0.12.28
   Compiling html5ever v0.26.0
   Compiling cipher v0.4.4
   Compiling tendril v0.4.3
   Compiling selectors v0.25.0
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling string_cache v0.8.9
   Compiling sha-1 v0.10.1
   Compiling sha1 v0.10.6
   Compiling ref-cast-impl v1.0.25
   Compiling cssparser-macros v0.6.1
   Compiling serde_derive_internals v0.29.1
   Compiling ahash v0.8.12
   Compiling anstyle-query v1.1.5
   Compiling zstd-safe v5.0.2+zstd.1.5.2
   Compiling hex v0.4.3
   Compiling base64 v0.21.7
   Compiling xxhash-rust v0.8.15
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.4
   Compiling anstyle v1.0.13
   Compiling cpufeatures v0.3.0
   Compiling rayon-core v1.13.0
   Compiling chacha20 v0.10.2
   Compiling anstream v0.6.21
   Compiling schemars_derive v1.2.1
   Compiling ssri v9.2.0
   Compiling cssparser v0.31.2
   Compiling crossbeam-deque v0.8.6
   Compiling darling v0.23.0
   Compiling fxhash v0.2.1
   Compiling regex v1.12.3
   Compiling block-buffer v0.12.0
   Compiling crypto-common v0.2.1
   Compiling inout v0.2.2
   Compiling reflink-copy v0.1.29
   Compiling http-serde v2.1.1
   Compiling derive_more v0.99.20
   Compiling servo_arc v0.3.0
   Compiling memmap2 v0.5.10
   Compiling base64ct v1.8.3
   Compiling heck v0.5.0
   Compiling tinyvec_macros v0.1.1
   Compiling rmcp v1.7.0
   Compiling unicode-width v0.2.2
   Compiling dyn-clone v1.0.20
   Compiling clap_lex v1.0.0
   Compiling schemars v1.2.1
   Compiling getopts v0.2.24
   Compiling clap_builder v4.5.60
   Compiling clap_derive v4.5.55
   Compiling tinyvec v1.10.0
   Compiling password-hash v0.4.2
   Compiling cipher v0.5.1
   Compiling cacache v13.1.0
   Compiling http-cache-semantics v2.1.1
   Compiling rust-embed-utils v8.11.0
   Compiling rmcp-macros v1.7.0
   Compiling rand v0.10.1
   Compiling reqwest-middleware v0.4.2
   Compiling aes v0.8.4
   Compiling process-wrap v9.1.0
   Compiling xml5ever v0.38.0
   Compiling html5ever v0.38.0
   Compiling reqwest v0.13.4
   Compiling uuid v1.22.0
   Compiling sse-stream v0.2.1
   Compiling hmac v0.12.1
   Compiling bincode v1.3.3
   Compiling toml_datetime v0.6.11
   Compiling serde_spanned v0.6.9
   Compiling winnow v0.7.15
   Compiling lazy_static v1.5.0
   Compiling toml_write v0.1.2
   Compiling unpdf v0.4.5
   Compiling option-ext v0.2.0
   Compiling kuva v0.5.0
   Compiling pastey v0.2.1
   Compiling ego-tree v0.6.3
   Compiling scraper v0.18.1
   Compiling dirs-sys v0.4.1
   Compiling toml_edit v0.22.27
   Compiling http-cache v0.20.1
   Compiling sharded-slab v0.1.7
   Compiling markup5ever_rcdom v0.38.0+unofficial
   Compiling pbkdf2 v0.11.0
   Compiling bzip2 v0.4.4
   Compiling rayon v1.12.0
   Compiling rust-embed-impl v8.11.0
   Compiling retry-policies v0.4.0
   Compiling zstd v0.11.2+zstd.1.5.2
   Compiling rc4 v0.2.0
   Compiling clap v4.5.60
   Compiling unicode-normalization v0.1.25
   Compiling cbc v0.1.2
   Compiling matchers v0.2.0
   Compiling crossbeam-channel v0.5.15
   Compiling axum-core v0.5.6
   Compiling xattr v1.6.1
   Compiling tracing-log v0.2.0
   Compiling md-5 v0.10.6
   Compiling serde_path_to_error v0.1.20
   Compiling csv-core v0.1.13
   Compiling filetime v0.2.27
   Compiling thread_local v1.1.9
   Compiling constant_time_eq v0.1.5
   Compiling nu-ansi-term v0.50.3
   Compiling colorous v1.0.16
   Compiling matchit v0.8.4
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass1)
   Compiling unicode-bidi v0.3.18
   Compiling axum v0.8.8
   Compiling tracing-subscriber v0.3.22
   Compiling zip v0.6.6
   Compiling tar v0.4.46
   Compiling csv v1.4.0
   Compiling rust-embed v8.11.0
   Compiling readability-rust v0.1.0
   Compiling toml v0.8.23
   Compiling reqwest-retry v0.7.0
   Compiling htmd v0.5.4
   Compiling http-cache-reqwest v0.15.1
   Compiling dirs v5.0.1
   Compiling minijinja v2.17.1
   Compiling fs2 v0.4.3
   Compiling roxmltree v0.20.0
   Compiling md5 v0.7.0
   Compiling bytesize v1.3.3
   Compiling humantime v2.3.0
   Compiling semver v1.0.27
   Compiling yasna v0.5.2
   Compiling pem v3.0.6
   Compiling serial_test_derive v3.5.0
   Compiling comma v1.0.0
   Compiling rexpect v0.7.0
   Compiling rcgen v0.13.2
   Compiling serial_test v3.5.0
   Compiling biomcp-mcp-contract-client v0.1.0 (~/2069-baseline/pass1/crates/biomcp-mcp-contract-client)
      Timing report saved to ~/2069-baseline/pass1/target/cargo-timings/cargo-timing-20261014T003124.826616046Z.html
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 10s
TIMEFMT WALL 250.88 USER 759.41 SYS 52.77 MAXRSS_KB 6191256
RESULT cold_dev_build_wall_s = 250.9

### rust-edit-rebuild — rebuild after one Rust edit (dev profile, incremental off)
$ touch src/render/markdown/gene.rs
$ /usr/bin/time -f "TIMEFMT WALL %e USER %U SYS %S MAXRSS_KB %M" cargo build --locked --no-default-features --all-targets
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass1)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 40s
TIMEFMT WALL 161.36 USER 432.09 SYS 19.35 MAXRSS_KB 6200700
RESULT rust_edit_rebuild_wall_s = 161.4

### doc-edit-rebuild — rebuild after one doc edit (a spec page touch; v0.9.2 build.rs scans the whole package)
$ touch spec/entity/gene.md
$ /usr/bin/time -f "TIMEFMT WALL %e USER %U SYS %S MAXRSS_KB %M" cargo build --locked --no-default-features --all-targets
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass1)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 36s
TIMEFMT WALL 156.75 USER 431.74 SYS 18.83 MAXRSS_KB 6183612
RESULT doc_edit_rebuild_wall_s = 156.8

### dep-counts — direct dependencies, Cargo.lock packages, crates compiled, duplicate names
$ python3 (counts from Cargo.toml and Cargo.lock)
RESULT direct_dependencies_normal = 58 (optional: 3: prost, tonic, zstd)
RESULT direct_dependencies_dev = 5
RESULT direct_dependencies_windows_only = 1
RESULT direct_dependencies_unix_dev = 1
RESULT cargo_lock_packages = 488
RESULT cargo_lock_duplicate_names = 50 (extra copies: 68)
$ cargo tree --locked --no-default-features -e normal,build --prefix none > .cache/measure/tree-normal-build.txt
$ cargo tree --locked --no-default-features -e normal,build,dev --prefix none > .cache/measure/tree-normal-build-dev.txt
$ python3 (unique crate name@version pairs in each cargo tree, workspace members excluded)
RESULT crates_compiled_normal_build = 338
RESULT duplicate_crate_names_normal_build = 24 (base64, block-buffer, cipher, cpufeatures, crypto-common, getrandom, html5ever, inout, markup5ever, phf, phf_codegen, phf_generator, phf_macros, phf_shared, rand, rand_core, siphasher, string_cache, string_cache_codegen, syn, tendril, thiserror, thiserror-impl, unicode-width)
RESULT crates_compiled_with_dev = 349
RESULT duplicate_crate_names_with_dev = 25 (base64, block-buffer, cipher, cpufeatures, crypto-common, getrandom, html5ever, inout, markup5ever, phf, phf_codegen, phf_generator, phf_macros, phf_shared, rand, rand_core, reqwest, siphasher, string_cache, string_cache_codegen, syn, tendril, thiserror, thiserror-impl, unicode-width)

### prepare-context — one prepare so the timed gates below start with fresh artifacts (context, not a baseline number)
$ make prepare-test
mkdir -p "~/2069-baseline/pass1/.cache/routine-test-tmp"
tools/with-build-identity cargo build --locked --profile spec --no-default-features --bin biomcp --example rmcp_streamable_http_contract
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling libc v0.2.182
   Compiling cfg-if v1.0.4
   Compiling serde_core v1.0.228
   Compiling syn v2.0.117
   Compiling serde v1.0.228
   Compiling version_check v0.9.5
   Compiling itoa v1.0.17
   Compiling memchr v2.8.0
   Compiling smallvec v1.15.1
   Compiling generic-array v0.14.7
   Compiling typenum v1.19.0
   Compiling pin-project-lite v0.2.17
   Compiling zerocopy v0.8.40
   Compiling bytes v1.11.1
   Compiling futures-core v0.3.32
   Compiling siphasher v1.0.2
   Compiling once_cell v1.21.3
   Compiling jobserver v0.1.34
   Compiling shlex v1.3.0
   Compiling find-msvc-tools v0.1.9
   Compiling stable_deref_trait v1.2.1
   Compiling log v0.4.29
   Compiling cc v1.2.56
   Compiling subtle v2.6.1
   Compiling futures-sink v0.3.32
   Compiling errno v0.3.14
   Compiling signal-hook-registry v1.4.8
   Compiling synstructure v0.13.2
   Compiling futures-channel v0.3.32
   Compiling mio v1.1.1
   Compiling socket2 v0.6.2
   Compiling bitflags v2.11.0
   Compiling zmij v1.0.21
   Compiling slab v0.4.12
   Compiling futures-task v0.3.32
   Compiling futures-io v0.3.32
   Compiling getrandom v0.2.17
   Compiling serde_derive v1.0.228
   Compiling zerofrom-derive v0.1.6
   Compiling yoke-derive v0.8.1
   Compiling tokio-macros v2.6.1
   Compiling zerofrom v0.1.6
   Compiling yoke v0.8.1
   Compiling futures-macro v0.3.32
   Compiling tokio v1.50.0
   Compiling zerovec-derive v0.11.2
   Compiling zerovec v0.11.5
   Compiling futures-util v0.3.32
   Compiling displaydoc v0.2.5
   Compiling http v1.4.0
   Compiling ppv-lite86 v0.2.21
   Compiling rand_core v0.6.4
   Compiling crypto-common v0.1.7
   Compiling rand_chacha v0.3.1
   Compiling rand v0.8.6
   Compiling tinystr v0.8.2
   Compiling writeable v0.6.2
   Compiling new_debug_unreachable v1.0.6
   Compiling litemap v0.8.1
   Compiling parking_lot_core v0.9.12
   Compiling icu_locale_core v2.1.1
   Compiling http-body v1.0.1
   Compiling zerotrie v0.2.3
   Compiling potential_utf v0.1.4
   Compiling block-buffer v0.10.4
   Compiling crc32fast v1.5.0
   Compiling serde_json v1.0.149
   Compiling percent-encoding v2.3.2
   Compiling icu_properties_data v2.1.2
   Compiling icu_normalizer_data v2.1.1
   Compiling scopeguard v1.2.0
   Compiling icu_provider v2.1.1
   Compiling lock_api v0.4.14
   Compiling digest v0.10.7
   Compiling icu_collections v2.1.1
   Compiling tracing-core v0.1.36
   Compiling adler2 v2.0.1
   Compiling simd-adler32 v0.3.8
   Compiling miniz_oxide v0.8.9
   Compiling parking_lot v0.12.5
   Compiling tracing-attributes v0.1.31
   Compiling ring v0.17.14
   Compiling phf_shared v0.13.1
   Compiling precomputed-hash v0.1.1
   Compiling tower-service v0.3.3
   Compiling cpufeatures v0.2.17
   Compiling httparse v1.10.1
   Compiling fastrand v2.3.0
   Compiling tracing v0.1.44
   Compiling phf_generator v0.13.1
   Compiling icu_normalizer v2.1.1
   Compiling icu_properties v2.1.2
   Compiling flate2 v1.1.9
   Compiling phf_shared v0.11.3
   Compiling zeroize v1.8.2
   Compiling try-lock v0.2.5
   Compiling base64 v0.22.1
   Compiling siphasher v0.3.11
   Compiling httpdate v1.0.3
   Compiling phf_shared v0.10.0
   Compiling want v0.3.1
   Compiling idna_adapter v1.2.1
   Compiling rustls-pki-types v1.14.0
   Compiling phf_generator v0.11.3
   Compiling form_urlencoded v1.2.2
   Compiling http-body-util v0.1.3
   Compiling sync_wrapper v1.0.2
   Compiling utf-8 v0.7.6
   Compiling pin-utils v0.1.0
   Compiling pkg-config v0.3.32
   Compiling utf8_iter v1.0.4
   Compiling getrandom v0.4.2
   Compiling rand_core v0.10.0
   Compiling rustix v1.1.4
   Compiling tower-layer v0.3.3
   Compiling untrusted v0.9.0
   Compiling atomic-waker v1.1.2
   Compiling hyper v1.8.1
   Compiling idna v1.1.0
   Compiling phf_generator v0.10.0
   Compiling ipnet v2.12.0
   Compiling linux-raw-sys v0.12.1
   Compiling autocfg v1.5.0
   Compiling thiserror v1.0.69
   Compiling compression-core v0.4.31
   Compiling compression-codecs v0.4.37
   Compiling num-traits v0.2.19
   Compiling hyper-util v0.1.20
   Compiling url v2.5.8
   Compiling phf_codegen v0.10.0
   Compiling tower v0.5.3
   Compiling futures-executor v0.3.32
   Compiling thiserror-impl v1.0.69
   Compiling rustls v0.23.45
   Compiling rustls-webpki v0.103.15
   Compiling async-compression v0.4.41
   Compiling string_cache_codegen v0.6.1
   Compiling phf_codegen v0.13.1
   Compiling sha2 v0.10.9
   Compiling tokio-util v0.7.18
   Compiling crossbeam-utils v0.8.21
   Compiling mime v0.3.17
   Compiling unicase v2.9.0
   Compiling ryu v1.0.23
   Compiling iri-string v0.7.10
   Compiling powerfmt v0.2.0
   Compiling deranged v0.5.8
   Compiling mime_guess v2.0.5
   Compiling web_atoms v0.2.4
   Compiling futures v0.3.32
   Compiling phf_macros v0.13.1
   Compiling tower-http v0.6.8
   Compiling aho-corasick v1.1.4
   Compiling cfg_aliases v0.2.1
   Compiling num_threads v0.1.7
   Compiling num-conv v0.2.0
   Compiling time-core v0.1.8
   Compiling iana-time-zone v0.1.65
   Compiling regex-syntax v0.8.10
   Compiling syn v1.0.109
   Compiling chrono v0.4.44
   Compiling regex-automata v0.4.14
   Compiling tokio-rustls v0.26.4
   Compiling time v0.3.47
   Compiling nix v0.31.2
   Compiling phf v0.13.1
   Compiling string_cache v0.9.0
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling webpki-roots v1.0.6
   Compiling string_cache_codegen v0.5.4
   Compiling async-trait v0.1.89
   Compiling block-padding v0.3.3
   Compiling hybrid-array v0.4.10
   Compiling mac v0.1.1
   Compiling hashbrown v0.16.1
   Compiling crossbeam-epoch v0.9.20
   Compiling ident_case v1.0.1
   Compiling strsim v0.11.1
   Compiling equivalent v1.0.2
   Compiling thiserror v2.0.18
   Compiling anyhow v1.0.103
   Compiling same-file v1.0.6
   Compiling walkdir v2.5.0
   Compiling indexmap v2.13.0
   Compiling tempfile v3.26.0
   Compiling darling_core v0.23.0
   Compiling futf v0.1.5
   Compiling inout v0.1.4
   Compiling markup5ever v0.11.0
   Compiling hyper-rustls v0.27.7
   Compiling phf v0.10.1
   Compiling serde_urlencoded v0.7.1
   Compiling tendril v0.5.0
   Compiling phf_macros v0.11.3
   Compiling tokio-stream v0.1.18
   Compiling miette-derive v5.10.0
   Compiling thiserror-impl v2.0.18
   Compiling either v1.15.0
   Compiling getrandom v0.3.4
   Compiling utf8parse v0.2.2
   Compiling ref-cast v1.0.25
   Compiling byteorder v1.5.0
   Compiling dtoa v1.0.11
   Compiling unicode-width v0.1.14
   Compiling dtoa-short v0.3.5
   Compiling miette v5.10.0
   Compiling anstyle-parse v0.2.7
   Compiling html5ever v0.26.0
   Compiling phf v0.11.3
   Compiling markup5ever v0.38.0
   Compiling reqwest v0.12.28
   Compiling darling_macro v0.23.0
   Compiling cipher v0.4.4
   Compiling tendril v0.4.3
   Compiling string_cache v0.8.9
   Compiling selectors v0.25.0
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling sha-1 v0.10.1
   Compiling sha1 v0.10.6
   Compiling ref-cast-impl v1.0.25
   Compiling cssparser-macros v0.6.1
   Compiling serde_derive_internals v0.29.1
   Compiling ahash v0.8.12
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.4
   Compiling base64 v0.21.7
   Compiling rayon-core v1.13.0
   Compiling zstd-safe v5.0.2+zstd.1.5.2
   Compiling anstyle-query v1.1.5
   Compiling anstyle v1.0.13
   Compiling hex v0.4.3
   Compiling cpufeatures v0.3.0
   Compiling xxhash-rust v0.8.15
   Compiling chacha20 v0.10.2
   Compiling ssri v9.2.0
   Compiling anstream v0.6.21
   Compiling schemars_derive v1.2.1
   Compiling cssparser v0.31.2
   Compiling crossbeam-deque v0.8.6
   Compiling darling v0.23.0
   Compiling fxhash v0.2.1
   Compiling crypto-common v0.2.1
   Compiling block-buffer v0.12.0
   Compiling inout v0.2.2
   Compiling regex v1.12.3
   Compiling reflink-copy v0.1.29
   Compiling http-serde v2.1.1
   Compiling derive_more v0.99.20
   Compiling servo_arc v0.3.0
   Compiling memmap2 v0.5.10
   Compiling heck v0.5.0
   Compiling tinyvec_macros v0.1.1
   Compiling dyn-clone v1.0.20
   Compiling unicode-width v0.2.2
   Compiling rmcp v1.7.0
   Compiling clap_lex v1.0.0
   Compiling base64ct v1.8.3
   Compiling password-hash v0.4.2
   Compiling clap_builder v4.5.60
   Compiling getopts v0.2.24
   Compiling schemars v1.2.1
   Compiling tinyvec v1.10.0
   Compiling clap_derive v4.5.55
   Compiling cacache v13.1.0
   Compiling http-cache-semantics v2.1.1
   Compiling cipher v0.5.1
   Compiling rmcp-macros v1.7.0
   Compiling rust-embed-utils v8.11.0
   Compiling rand v0.10.1
   Compiling reqwest-middleware v0.4.2
   Compiling process-wrap v9.1.0
   Compiling aes v0.8.4
   Compiling xml5ever v0.38.0
   Compiling html5ever v0.38.0
   Compiling reqwest v0.13.4
   Compiling uuid v1.22.0
   Compiling sse-stream v0.2.1
   Compiling hmac v0.12.1
   Compiling toml_datetime v0.6.11
   Compiling serde_spanned v0.6.9
   Compiling bincode v1.3.3
   Compiling kuva v0.5.0
   Compiling unpdf v0.4.5
   Compiling option-ext v0.2.0
   Compiling lazy_static v1.5.0
   Compiling winnow v0.7.15
   Compiling toml_write v0.1.2
   Compiling ego-tree v0.6.3
   Compiling pastey v0.2.1
   Compiling scraper v0.18.1
   Compiling toml_edit v0.22.27
   Compiling sharded-slab v0.1.7
   Compiling dirs-sys v0.4.1
   Compiling http-cache v0.20.1
   Compiling markup5ever_rcdom v0.38.0+unofficial
   Compiling pbkdf2 v0.11.0
   Compiling rust-embed-impl v8.11.0
   Compiling bzip2 v0.4.4
   Compiling retry-policies v0.4.0
   Compiling rayon v1.12.0
   Compiling rc4 v0.2.0
   Compiling clap v4.5.60
   Compiling zstd v0.11.2+zstd.1.5.2
   Compiling unicode-normalization v0.1.25
   Compiling cbc v0.1.2
   Compiling crossbeam-channel v0.5.15
   Compiling matchers v0.2.0
   Compiling axum-core v0.5.6
   Compiling xattr v1.6.1
   Compiling tracing-log v0.2.0
   Compiling md-5 v0.10.6
   Compiling serde_path_to_error v0.1.20
   Compiling csv-core v0.1.13
   Compiling filetime v0.2.27
   Compiling thread_local v1.1.9
   Compiling colorous v1.0.16
   Compiling nu-ansi-term v0.50.3
   Compiling constant_time_eq v0.1.5
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass1)
   Compiling unicode-bidi v0.3.18
   Compiling matchit v0.8.4
   Compiling axum v0.8.8
   Compiling zip v0.6.6
   Compiling tracing-subscriber v0.3.22
   Compiling tar v0.4.46
   Compiling csv v1.4.0
   Compiling rust-embed v8.11.0
   Compiling readability-rust v0.1.0
   Compiling reqwest-retry v0.7.0
   Compiling http-cache-reqwest v0.15.1
   Compiling htmd v0.5.4
   Compiling dirs v5.0.1
   Compiling toml v0.8.23
   Compiling minijinja v2.17.1
   Compiling fs2 v0.4.3
   Compiling roxmltree v0.20.0
   Compiling md5 v0.7.0
   Compiling humantime v2.3.0
   Compiling semver v1.0.27
   Compiling bytesize v1.3.3
   Compiling yasna v0.5.2
   Compiling pem v3.0.6
   Compiling serial_test_derive v3.5.0
   Compiling comma v1.0.0
   Compiling rexpect v0.7.0
   Compiling serial_test v3.5.0
   Compiling rcgen v0.13.2
   Compiling biomcp-mcp-contract-client v0.1.0 (~/2069-baseline/pass1/crates/biomcp-mcp-contract-client)
    Finished `spec` profile [optimized] target(s) in 7m 43s
make sync-python-dev
make[1]: Entering directory '~/2069-baseline/pass1'
uv sync --extra dev --no-install-project
Using CPython 3.11.14
Creating virtual environment at: .venv
Resolved 48 packages in 1ms
Installed 44 packages in 1.58s
 + attrs==25.4.0
 + babel==2.18.0
 + backrefs==6.2
 + certifi==2026.1.4
 + charset-normalizer==3.4.4
 + click==8.3.1
 + colorama==0.4.6
 + execnet==2.1.2
 + ghp-import==2.1.0
 + idna==3.18
 + iniconfig==2.3.0
 + jinja2==3.1.6
 + jsonschema==4.26.0
 + jsonschema-specifications==2025.9.1
 + markdown==3.10.2
 + markupsafe==3.0.3
 + mergedeep==1.3.4
 + mkdocs==1.6.1
 + mkdocs-get-deps==0.2.0
 + mkdocs-material==9.7.2
 + mkdocs-material-extensions==1.3.1
 + packaging==26.0
 + paginate==0.5.7
 + pathspec==1.0.4
 + platformdirs==4.9.2
 + pluggy==1.6.0
 + pygments==2.20.0
 + pymdown-extensions==11.0.1
 + pytest==9.0.3
 + pytest-asyncio==1.3.0
 + pytest-timeout==2.4.0
 + pytest-xdist==3.8.0
 + python-dateutil==2.9.0.post0
 + pyyaml==6.0.3
 + pyyaml-env-tag==1.1
 + referencing==0.37.0
 + regex==2026.7.19
 + requests==2.34.2
 + rpds-py==0.30.0
 + six==1.17.0
 + tiktoken==0.12.0
 + typing-extensions==4.15.0
 + urllib3==2.7.0
 + watchdog==6.0.0
make[1]: Leaving directory '~/2069-baseline/pass1'
mkdir -p "~/2069-baseline/pass1/.cache"
tools/with-build-identity cargo nextest archive --locked --no-default-features --archive-file "~/2069-baseline/pass1/.cache/routine-tests.tar.zst" --zstd-level -7
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.96s
   Archiving 30 binaries (including 1 non-test binary; 1 non-test binary filtered out), 1 build script output directory, 2 linked paths, and 1 standard library to ~/2069-baseline/pass1/.cache/routine-tests.tar.zst
    Archived 106 files to ~/2069-baseline/pass1/.cache/routine-tests.tar.zst in 4.68s
RESULT make_prepare_test_wall_s = 478.6

### make-test — the routine gate, wall time with artifacts already fresh
$ make test > .cache/measure/make-test.log 2>&1
```

### Pass 1, completed

```

### locks — held for the whole run
$ flock -n ~/.yr-checkout.lock && flock -n ~/.<build-host>-gate.lock
both locks held by pid 2302457

### make-test — the routine gate, wall time with artifacts already fresh
$ make test > .cache/measure/make-test.log 2>&1
RESULT make_test_wall_s = 646.3
$ grep -E 'Summary \[|tests? run:' .cache/measure/make-test.log | tail -1 || true
     Summary [ 157.025s] 4205 tests run: 4205 passed, 33 skipped
$ grep -E '^=+ .* (passed|failed|error).* =+$' .cache/measure/make-test.log | tail -2 || true
================= 1251 passed, 3 skipped in 476.00s (0:07:56) ==================
$ grep -iE 'documentation built|docs built' .cache/measure/make-test.log | tail -1 || true
INFO    -  Documentation built in 2.49 seconds
$ python3 (test counts parsed from the make-test log)
RESULT rust_tests_run = 4205
RESULT rust_tests_passed = 4205
RESULT rust_tests_skipped = 33
RESULT python_lane_summary = 1251 passed, 3 skipped in 476.00s (0:07:56)

### build-folder-after-make-test — target/ bytes after make test
$ du -sb target | awk "{print \$1}"
3131372745
$ du -sh target
3.0G	target
RESULT build_folder_bytes_after_make_test = 3131372745

### rust-test-census — test-duration census of the routine Rust suite (from the archive make test built)
$ NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 tools/run-offline -- cargo nextest run --archive-file .cache/routine-tests.tar.zst --config-file .config/nextest.toml --message-format libtest-json-plus > .cache/measure/census.jsonl 2> .cache/measure/census.err || true
RESULT rust_test_census_wall_s = 113.1
$ tail -2 .cache/measure/census.err || true
────────────
     Summary [ 111.811s] 4205 tests run: 4205 passed, 33 skipped
$ python3 (count tests at or over one second from the libtest-json-plus stream)
RESULT rust_tests_with_duration = 4205
RESULT tests_over_1s = 62
RESULT tests_over_1s_sum_s = 296.2
slowest tests:
      24.4s  biomcp-cli::tls_ca_bundle_contract$fda_orphan_client_reaches_a_private_ca_provider
      19.6s  biomcp-cli::biomcp_cli$entities::disease::get::tests::a_stale_mcp_call_tells_the_clinician_the_cache_age_on_both_channels
      14.1s  biomcp-cli::biomcp_cli$entities::variant::get::tests::protein_change_get_notes_unchecked_numbering_when_uniprot_is_unreachable
      13.8s  biomcp-cli::rmcp_client_contract$raw_and_typed_article_query_validation_converges_before_provider_work
      12.3s  biomcp-cli::biomcp_cli$entities::disease::get::tests::a_stale_disease_card_tells_the_clinician_the_cache_age
      11.6s  biomcp-cli::biomcp_cli$cli::stale_json_note_tests::no_cache_skips_the_cache_for_the_spawned_clingen_fetch
      11.3s  biomcp-cli::biomcp_cli$cli::stale_json_note_tests::a_stale_clingen_prefetch_note_reaches_the_gene_card
       9.8s  biomcp-cli::biomcp_cli$cli::stale_json_note_tests::a_stale_article_search_json_states_the_cache_age_in_meta_notes
       9.4s  biomcp-cli::biomcp_cli$entities::article::assets::tests::manifest_and_bytes_preserve_induced_archive_failure_after_figshare_miss
       9.0s  biomcp-cli::rmcp_client_contract$human_mcp_command_dispatches_to_provider_once
       8.9s  biomcp-cli::biomcp_cli$entities::drug::test_support::label_fallback_fetch_error_stays_an_error
       8.4s  biomcp-cli::biomcp_cli$sources::gencc::tests::cross_process_first_use_elects_one_leader_and_settles_followers
       7.9s  biomcp-cli::biomcp_cli$entities::article::variant_search::tests::degraded_strict_leg_partials_mark_the_route_incomplete
       7.2s  biomcp-cli::biomcp_cli$cli::stale_json_note_tests::a_stale_search_all_json_states_the_cache_age_in_meta_notes
       6.4s  biomcp-cli::biomcp_cli$cli::article::tests::entities_disease_links::article_entities_disease_rows_link_the_named_disease_or_search

### python-test-count — python lane test count (collection, no runs)
$ tools/run-offline -- env BIOMCP_BIN=~/2069-baseline/pass1/target/spec/biomcp uv run --no-sync pytest tests/ --collect-only -q > .cache/measure/pytest-collect.txt 2>&1
RESULT python_collect_wall_s = 0.9
$ tail -2 .cache/measure/pytest-collect.txt
======================== 1254 tests collected in 0.42s =========================
offline ownership mapping: host uid/gid 1000/1000 verified
$ python3 (collected count)
RESULT python_tests_collected = 1254

### spec-block-census — executable bash blocks across the spec pages
$ grep -rlE '^[[:space:]]*```bash[[:space:]]*$' spec --include='*.md' | wc -l
38
$ grep -rE '^[[:space:]]*```bash[[:space:]]*$' spec --include='*.md' | wc -l
496
RESULT spec_pages_with_bash_blocks = 38
RESULT spec_bash_blocks = 496

### make-spec — the spec lane, wall time with artifacts already fresh
$ make spec > .cache/measure/make-spec.log 2>&1
RESULT make_spec_wall_s = 657.2
$ tail -6 .cache/measure/make-spec.log
make[1]: Entering directory '~/2069-baseline/pass1'
bash scripts/run-specs.sh spec-static
=== spec pages: spec/surface/docker-image.md spec/surface/homebrew.md ===
7 passed
make[1]: Leaving directory '~/2069-baseline/pass1'
offline ownership mapping: host uid/gid 1000/1000 verified

### make-lint — the lint gate, wall time
$ make lint > .cache/measure/make-lint.log 2>&1
RESULT make_lint_wall_s = 195.9
$ tail -6 .cache/measure/make-lint.log
advisories ok
[PASS] Rust advisory lint (cargo deny check advisories)
[SKIP] JS/TS lint (not detected)
tools/check-quality-ratchet.sh
tools/check-test-wait-ratchet.py
test-wait ratchet ok (41 files with pinned ceilings, 46 watchdog markers under the ceiling 46)

### release-build — cold release build (default features, shipped profile: thin LTO, one codegen unit, strip)
$ /usr/bin/time -v -o .cache/measure/release-time.txt cargo build --release --locked --timings > .cache/measure/release-build.log 2>&1
RESULT release_build_wall_s = 837.0
$ grep -E 'Elapsed \(wall clock|Maximum resident set size|User time|System time' .cache/measure/release-time.txt
	User time (seconds): 1602.30
	System time (seconds): 49.28
	Elapsed (wall clock) time (h:mm:ss or m:ss): 13:57.00
	Maximum resident set size (kbytes): 4597860
$ tail -3 .cache/measure/release-build.log
   Compiling humantime v2.3.0
      Timing report saved to ~/2069-baseline/pass1/target/cargo-timings/cargo-timing-20261014T014033.069348457Z.html
    Finished `release` profile [optimized] target(s) in 13m 56s
RESULT release_peak_rss_kb = 4597860

### build-folder-after-release — target/ bytes after the release build
$ du -sb target | awk "{print \$1}"
4441874624
$ du -sh target
4.2G	target
RESULT build_folder_bytes_after_release_build = 4441874624

### stripped-binary — the stripped release binary's size in bytes
$ stat -c %s target/release/biomcp
39122976
$ file target/release/biomcp | head -1
target/release/biomcp: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 3.2.0, BuildID[sha1]=846f624c8a7777f1c9b63ef1ce98522939f47e4b, stripped
RESULT stripped_release_binary_bytes = 39122976

### llvm-lines — LLVM IR lines and function copies of the library, release profile
$ cargo llvm-lines --release --lib > .cache/measure/llvm-lines.txt 2>&1
RESULT llvm_lines_wall_s = 85.8
$ head -8 .cache/measure/llvm-lines.txt
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass1)
    Finished `release` profile [optimized] target(s) in 1m 24s
  Lines                  Copies                Function name
  -----                  ------                -------------
  6819551                138257                (TOTAL)
   181284 (2.7%,  2.7%)     447 (0.3%,  0.3%)  <&mut serde_json::de::Deserializer<R> as serde_core::de::Deserializer>::deserialize_struct
    97818 (1.4%,  4.1%)     660 (0.5%,  0.8%)  serde_core::ser::Serializer::collect_seq
    94978 (1.4%,  5.5%)    1345 (1.0%,  1.8%)  core::iter::traits::iterator::Iterator::try_fold
$ awk (sum of the Lines and Copies columns)
RESULT llvm_lines_total = 13639102
RESULT llvm_function_copies = 138257

### deny-advisories — advisories ignored in deny.toml
$ python3 (count of [advisories].ignore entries)
RESULT ignored_advisories = 6
  RUSTSEC-2025-0141
  RUSTSEC-2025-0057
  RUSTSEC-2024-0384
  RUSTSEC-2025-0134
  RUSTSEC-2026-0186
  RUSTSEC-2026-0192

### coverage — line coverage of three suites through cargo-llvm-cov (dev build instrumented)
$ cargo llvm-cov clean --workspace
$ eval each line of "cargo llvm-cov show-env" as an export; CARGO_TARGET_DIR=target/llvm-cov-target; LLVM_PROFILE_FILE per suite
$ cargo build --locked --no-default-features --all-targets > .cache/measure/cov-build.log 2>&1
RESULT coverage_build_wall_s = 263.9
$ tools/run-offline -- cargo nextest run --locked --no-default-features -E 'kind(test)' > .cache/measure/cov-integration.log 2>&1 || true
RESULT coverage_integration_tests_wall_s = 165.1
$ grep -E 'Summary \[|tests? run:' .cache/measure/cov-integration.log | tail -1 || true
     Summary [ 157.209s] 135 tests run: 134 passed (2 slow), 1 failed, 5 skipped
$ tools/run-offline -- env BIOMCP_ROUTINE_TEST_LANE=1 BIOMCP_BIN=$PWD/target/llvm-cov-target/debug/biomcp TMPDIR=$PWD/.cache/measure/tmp uv run --no-sync pytest tests/ -n 4 --dist loadfile --basetemp .cache/measure/pytest-basetemp > .cache/measure/cov-pytest.log 2>&1 || true
RESULT coverage_python_lane_wall_s = 272.8
$ grep -E '^=+ .* (passed|failed|error).* =+$' .cache/measure/cov-pytest.log | tail -2 || true
============ 26 failed, 1225 passed, 3 skipped in 271.76s (0:04:31) ============

### coverage — spec lane against the instrumented dev binary (hand-written artifacts env)
$ cat > .cache/spec-artifacts.env <<'ENVEOF'
export BIOMCP_SPEC_ARTIFACT_MODE=spec
export BIOMCP_SPEC_FEATURE_OFF_BIN=~/2069-baseline/pass1/target/llvm-cov-target/debug/biomcp
export BIOMCP_SPEC_MCP_EXAMPLE_BIN=~/2069-baseline/pass1/target/llvm-cov-target/debug/examples/rmcp_streamable_http_contract
export BIOMCP_SPEC_CARGO_TREE=~/2069-baseline/pass1/.cache/spec-artifacts/cargo-tree-no-default-features.txt
export BIOMCP_SPEC_CARGO_METADATA=~/2069-baseline/pass1/.cache/spec-artifacts/cargo-metadata.json
export BIOMCP_BIN=~/2069-baseline/pass1/target/llvm-cov-target/debug/biomcp
ENVEOF
cat .cache/spec-artifacts.env
export BIOMCP_SPEC_ARTIFACT_MODE=spec
export BIOMCP_SPEC_FEATURE_OFF_BIN=~/2069-baseline/pass1/target/llvm-cov-target/debug/biomcp
export BIOMCP_SPEC_MCP_EXAMPLE_BIN=~/2069-baseline/pass1/target/llvm-cov-target/debug/examples/rmcp_streamable_http_contract
export BIOMCP_SPEC_CARGO_TREE=~/2069-baseline/pass1/.cache/spec-artifacts/cargo-tree-no-default-features.txt
export BIOMCP_SPEC_CARGO_METADATA=~/2069-baseline/pass1/.cache/spec-artifacts/cargo-metadata.json
export BIOMCP_BIN=~/2069-baseline/pass1/target/llvm-cov-target/debug/biomcp
$ PATH="~/2069-baseline/pass1/target/llvm-cov-target/debug:$PATH" BIOMCP_SPEC_ARTIFACTS_PREPARED=1 ROUTINE_CARGO_FEATURES=--no-default-features SPEC_PROFILE=dev tools/run-offline -- bash scripts/run-specs.sh spec > .cache/measure/cov-spec.log 2>&1 || true
RESULT coverage_spec_lane_wall_s = 717.4
$ tail -4 .cache/measure/cov-spec.log || true
============================== 41 passed in 1.79s ==============================
spec run failed (1):
  stage run_article_markdown_specs
offline ownership mapping: host uid/gid 1000/1000 verified
$ tools/run-offline -- cargo nextest run --locked --no-default-features -E 'kind(lib) | kind(bin)' > .cache/measure/cov-unit.log 2>&1 || true
RESULT coverage_unit_tests_wall_s = 80.2
$ grep -E 'Summary \[|tests? run:' .cache/measure/cov-unit.log | tail -1 || true
     Summary [  78.880s] 4070 tests run: 4070 passed, 28 skipped

### coverage — report: outside-in
$ mv target/llvm-cov-target/unit-*.profraw .cache/measure/cov-hold/ (report the other set)
$ cargo llvm-cov report --ignore-filename-regex '(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)$|test_support|(^|/)generated/' | tail -4
error: failed to load coverage: 'target/debug/biomcp, target/debug/biomcp-cli, target/debug/deps/adverse_event_route_contract-c18845c880ccecac, target/debug/deps/article_asset_terminal-f1c09987b8e8fe7f, target/debug/deps/article_batch_outcome-6ec6e0977301fc93, target/debug/deps/article_cli_tests_structure-4643c50bf6cb0165, target/debug/deps/article_transform_structure-fc50e09e5568a69d, target/debug/deps/article_usage_stderr-cb74f257755a4dca, target/debug/deps/benchmark_cli_structure-4c17176d3746474b, target/debug/deps/biomcp-60cefaac99ca9a53, target/debug/deps/biomcp-bcc0ea64f3178014, target/debug/deps/biomcp_cli-62d7f580ca0837e6, target/debug/deps/biomcp_cli-cf2396d15ea6e691, target/debug/deps/biomcp_cli-d712d7a36816cdc5, target/debug/deps/biomcp_cli_shim-5f4cae94a5254436, target/debug/deps/cache_clear-692b0478bfeb8ac6, target/debug/deps/cli_global_modifiers-d091df9de00abcc0, target/debug/deps/cli_line_cap_absorption-ddcd76b66a15397a, target/debug/deps/health_cli_structure-1c2645d6e8f7692d, target/debug/deps/json_error_contract-42a560f2bd561c6b, target/debug/deps/legacy_cache_helper_cleanup-a7536cfaeeb67189, target/debug/deps/list_cli_structure-88f037cb242b7ab9, target/debug/deps/live_smoke-0bcbd89b6c402c5b, target/debug/deps/managed_state_permissions-b34905a17ff0fb03, target/debug/deps/package_build_identity-4a311094889a43bd, target/debug/deps/pgx_get_paging-4cf15a4aa78c8095, target/debug/deps/plugin_manifest-2bdd09abbe8ff654, target/debug/deps/pty_helpers-6dac2b6de696dfb4, target/debug/deps/rmcp_client_contract-56f2d888ad060f88, target/debug/deps/search_all_cli_structure-32c85e0a06fb1a26, target/debug/deps/skill_cli_structure-4b8712ef7740fdb9, target/debug/deps/tls_ca_bundle_contract-208b130f840f59fa, target/debug/deps/windows_stdio_contract-4bcfa43d52db07d1, target/debug/examples/rmcp_streamable_http_contract, target/debug/examples/rmcp_streamable_http_contract-51c3aa4bee5bed8e': no coverage data found
error: failed to generate report: process didn't exit successfully: `~/.rustup/toolchains/1.93.1-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov report -use-color=0 -instr-profile=~/2069-baseline/pass1/target/pass1.profdata -object target/debug/biomcp -object target/debug/biomcp-cli -object target/debug/deps/adverse_event_route_contract-c18845c880ccecac -object target/debug/deps/article_asset_terminal-f1c09987b8e8fe7f -object target/debug/deps/article_batch_outcome-6ec6e0977301fc93 -object target/debug/deps/article_cli_tests_structure-4643c50bf6cb0165 -object target/debug/deps/article_transform_structure-fc50e09e5568a69d -object target/debug/deps/article_usage_stderr-cb74f257755a4dca -object target/debug/deps/benchmark_cli_structure-4c17176d3746474b -object target/debug/deps/biomcp-60cefaac99ca9a53 -object target/debug/deps/biomcp-bcc0ea64f3178014 -object target/debug/deps/biomcp_cli-62d7f580ca0837e6 -object target/debug/deps/biomcp_cli-cf2396d15ea6e691 -object target/debug/deps/biomcp_cli-d712d7a36816cdc5 -object target/debug/deps/biomcp_cli_shim-5f4cae94a5254436 -object target/debug/deps/cache_clear-692b0478bfeb8ac6 -object target/debug/deps/cli_global_modifiers-d091df9de00abcc0 -object target/debug/deps/cli_line_cap_absorption-ddcd76b66a15397a -object target/debug/deps/health_cli_structure-1c2645d6e8f7692d -object target/debug/deps/json_error_contract-42a560f2bd561c6b -object target/debug/deps/legacy_cache_helper_cleanup-a7536cfaeeb67189 -object target/debug/deps/list_cli_structure-88f037cb242b7ab9 -object target/debug/deps/live_smoke-0bcbd89b6c402c5b -object target/debug/deps/managed_state_permissions-b34905a17ff0fb03 -object target/debug/deps/package_build_identity-4a311094889a43bd -object target/debug/deps/pgx_get_paging-4cf15a4aa78c8095 -object target/debug/deps/plugin_manifest-2bdd09abbe8ff654 -object target/debug/deps/pty_helpers-6dac2b6de696dfb4 -object target/debug/deps/rmcp_client_contract-56f2d888ad060f88 -object target/debug/deps/search_all_cli_structure-32c85e0a06fb1a26 -object target/debug/deps/skill_cli_structure-4b8712ef7740fdb9 -object target/debug/deps/tls_ca_bundle_contract-208b130f840f59fa -object target/debug/deps/windows_stdio_contract-4bcfa43d52db07d1 -object target/debug/examples/rmcp_streamable_http_contract -object target/debug/examples/rmcp_streamable_http_contract-51c3aa4bee5bed8e -ignore-filename-regex '(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)$|test_support|(^|/)generated/|/rustc/([0-9a-f]+|[0-9]+\.[0-9]+\.[0-9]+)/|^~/2069\-baseline/pass1(/.*)?/(tests|examples|benches)/|^~/2069\-baseline/pass1(/.*)?/(tests\.rs|[0-9a-zA-Z_-]+[_-]tests\.rs)$|^~/2069\-baseline/pass1/target($|/)|^~/\.cargo/(registry|git)/|^~/\.rustup/toolchains($|/)'` (exit status: 1)

### coverage — report: outside-in (rerun with CARGO_TARGET_DIR unset after the discovery bug)
$ mv target/llvm-cov-target/unit-*.profraw .cache/measure/cov-hold/ (report the other set)
$ env -u CARGO_TARGET_DIR cargo llvm-cov report --ignore-filename-regex "(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)\$|test_support|(^|/)generated/" > .cache/measure/cov-report-outside_in.txt 2>&1; tail -4 .cache/measure/cov-report-outside_in.txt
workflow_ladders.rs                                     106                52    50.94%           7                 3    57.14%          82                44    46.34%           0                 0         -
xml.rs                                                  246               151    38.62%          17                14    17.65%         162                94    41.98%           0                 0         -
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                                                201832            156206    22.61%       14853             10888    26.69%      142085            108613    23.56%           0                 0         -
$ mv .cache/measure/cov-hold/*.profraw target/llvm-cov-target/ (restore)

### coverage — report: unit
$ mv target/llvm-cov-target/out-*.profraw .cache/measure/cov-hold/ (report the other set)
$ env -u CARGO_TARGET_DIR cargo llvm-cov report --ignore-filename-regex "(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)\$|test_support|(^|/)generated/" > .cache/measure/cov-report-unit.txt 2>&1; tail -4 .cache/measure/cov-report-unit.txt
workflow_ladders.rs                                     106                18    83.02%           7                 1    85.71%          82                25    69.51%           0                 0         -
xml.rs                                                  246                 5    97.97%          17                 0   100.00%         162                 1    99.38%           0                 0         -
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                                                201832             47155    76.64%       14853              3223    78.30%      142085             30835    78.30%           0                 0         -
$ mv .cache/measure/cov-hold/*.profraw target/llvm-cov-target/ (restore)

### coverage — report: combined
$ env -u CARGO_TARGET_DIR cargo llvm-cov report --ignore-filename-regex "(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)\$|test_support|(^|/)generated/" > .cache/measure/cov-report-combined.txt 2>&1; tail -4 .cache/measure/cov-report-combined.txt
workflow_ladders.rs                                     106                18    83.02%           7                 1    85.71%          82                25    69.51%           0                 0         -
xml.rs                                                  246                 5    97.97%          17                 0   100.00%         162                 1    99.38%           0                 0         -
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                                                201832             41920    79.23%       14853              2823    80.99%      142085             27181    80.87%           0                 0         -
RESULT coverage_outside_in_lines_pct = 23.56
RESULT coverage_unit_lines_pct = 78.30
RESULT coverage_combined_lines_pct = 80.87

### coverage — outside-in suites rerun (absolute LLVM_PROFILE_FILE after the relative-path bug dropped subprocess profiles)
$ export LLVM_PROFILE_FILE="$CARGO_TARGET_DIR/out-%8m.profraw" (absolute)
$ tools/run-offline -- cargo nextest run --locked --no-default-features -E 'kind(test)'
RESULT coverage_integration_tests_wall_s = 41.1
     Summary [  39.804s] 135 tests run: 135 passed, 5 skipped
$ tools/run-offline -- env BIOMCP_ROUTINE_TEST_LANE=1 BIOMCP_BIN=$PWD/target/llvm-cov-target/debug/biomcp TMPDIR=$PWD/.cache/measure/tmp uv run --no-sync pytest tests/ -n 4 --dist loadfile --basetemp .cache/measure/pytest-basetemp
RESULT coverage_python_lane_wall_s = 271.1
============ 26 failed, 1225 passed, 3 skipped in 270.70s (0:04:30) ============
$ PATH="$PWD/target/llvm-cov-target/debug:$PATH" BIOMCP_SPEC_ARTIFACTS_PREPARED=1 ROUTINE_CARGO_FEATURES=--no-default-features SPEC_PROFILE=dev tools/run-offline -- bash scripts/run-specs.sh spec
RESULT coverage_spec_lane_wall_s = 708.2
tests/surface/test_parallel_isolation_contract.py::test_ticket_379_timing_docs_record_pruned_ownership PASSED [100%]

============================== 41 passed in 1.73s ==============================
offline ownership mapping: host uid/gid 1000/1000 verified
suites done

### coverage — reports after the outside-in rerun (CARGO_TARGET_DIR unset for report discovery)
$ mv target/llvm-cov-target/unit-*.profraw .cache/measure/cov-hold/ (report the outside-in set)
$ env -u CARGO_TARGET_DIR cargo llvm-cov report --ignore-filename-regex ... (outside-in)
xml.rs                                                  246               139    43.50%          17                10    41.18%         162                82    49.38%           0                 0         -
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                                                201832             97487    51.70%       14853              6262    57.84%      142085             67268    52.66%           0                 0         -
$ mv target/llvm-cov-target/out-*.profraw .cache/measure/cov-hold/ (report the unit set)
xml.rs                                                  246                 5    97.97%          17                 0   100.00%         162                 1    99.38%           0                 0         -
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                                                201832             47155    76.64%       14853              3223    78.30%      142085             30835    78.30%           0                 0         -
$ (all restored; report everything)
xml.rs                                                  246                 5    97.97%          17                 0   100.00%         162                 1    99.38%           0                 0         -
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                                                201832             30767    84.76%       14853              2043    86.25%      142085             19525    86.26%           0                 0         -
RESULT coverage_outside_in_lines_pct = 52.66
RESULT coverage_unit_lines_pct = 78.30
RESULT coverage_combined_lines_pct = 86.26

### coverage — reports with the script regex (final)
$ mv target/llvm-cov-target/unit-*.profraw .cache/measure/cov-hold/ (report the outside-in set)
xml.rs                                                  246               139    43.50%          17                10    41.18%         162                82    49.38%           0                 0         -
TOTAL                                                201832             97487    51.70%       14853              6262    57.84%      142085             67268    52.66%           0                 0         -
$ mv target/llvm-cov-target/out-*.profraw .cache/measure/cov-hold/ (report the unit set)
TOTAL                                                201832             47155    76.64%       14853              3223    78.30%      142085             30835    78.30%           0                 0         -
$ (all restored; report everything)
TOTAL                                                201832             30767    84.76%       14853              2043    86.25%      142085             19525    86.26%           0                 0         -
RESULT coverage_outside_in_lines_pct = 52.66
RESULT coverage_unit_lines_pct = 78.30
RESULT coverage_combined_lines_pct = 86.26

### locks — held for the whole run
$ flock -n ~/.yr-checkout.lock && flock -n ~/.<build-host>-gate.lock
both locks held by pid 2489308

### llvm-lines — LLVM IR lines and function copies of the library, release profile
$ cargo llvm-lines --release --lib > .cache/measure/llvm-lines.txt 2>&1
RESULT llvm_lines_wall_s = 93.6
$ head -8 .cache/measure/llvm-lines.txt
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass1)
    Finished `release` profile [optimized] target(s) in 1m 32s
  Lines                  Copies                Function name
  -----                  ------                -------------
  6819551                138257                (TOTAL)
   181284 (2.7%,  2.7%)     447 (0.3%,  0.3%)  <&mut serde_json::de::Deserializer<R> as serde_core::de::Deserializer>::deserialize_struct
    97818 (1.4%,  4.1%)     660 (0.5%,  0.8%)  serde_core::ser::Serializer::collect_seq
    94978 (1.4%,  5.5%)    1345 (1.0%,  1.8%)  core::iter::traits::iterator::Iterator::try_fold
$ awk (sum of the Lines and Copies columns)
RESULT llvm_lines_total = 6819551
RESULT llvm_function_copies = 138257

### runtime — run-time measurements with the release binary (thin LTO, one codegen unit)
$ { find ~/.cache/biomcp/http -type f 2>/dev/null || true; } | wc -l; du -sh ~/.cache/biomcp 2>/dev/null || echo "no host cache"
19799
198M	~/.cache/biomcp
$ python3 (five timed runs per case; median, min, max in seconds)
== biomcp --version start-up, empty cache dir ==
version_empty_cache: runs 0.01 0.00 0.00 0.00 0.00 median 0.00s spread 0.00s exit [0]
RESULT runtime_version_empty_cache_median_s = 0.00
RESULT runtime_version_empty_cache_spread_s = 0.00
== biomcp --version start-up, the host's large cache ==
version_large_cache: runs 0.00 0.00 0.00 0.00 0.00 median 0.00s spread 0.00s exit [0]
RESULT runtime_version_large_cache_median_s = 0.00
RESULT runtime_version_large_cache_spread_s = 0.00
== prefetch (one network run per command into the warm cache dir) ==
prefetch get_gene_BRAF: 0.81s exit 0
RESULT runtime_prefetch_get_gene_BRAF_s = 0.81
prefetch get_variant_BRAF_V600E: 0.58s exit 0
RESULT runtime_prefetch_get_variant_BRAF_V600E_s = 0.58
prefetch get_drug_imatinib: 5.84s exit 0
RESULT runtime_prefetch_get_drug_imatinib_s = 5.84
prefetch search_article_BRAF: 3.76s exit 0
RESULT runtime_prefetch_search_article_BRAF_s = 3.76
prefetch search_all_BRAF_melanoma: 12.02s exit 0
RESULT runtime_prefetch_search_all_BRAF_melanoma_s = 12.02
== warm cache (BIOMCP_CACHE_MODE=infinite), five runs each ==
warm_get_gene_BRAF: runs 0.19 0.19 0.19 0.18 0.18 median 0.19s spread 0.01s exit [0]
RESULT runtime_warm_get_gene_BRAF_median_s = 0.19
RESULT runtime_warm_get_gene_BRAF_spread_s = 0.01
warm_get_variant_BRAF_V600E: runs 0.02 0.02 0.02 0.02 0.02 median 0.02s spread 0.00s exit [0]
RESULT runtime_warm_get_variant_BRAF_V600E_median_s = 0.02
RESULT runtime_warm_get_variant_BRAF_V600E_spread_s = 0.00
warm_get_drug_imatinib: runs 2.06 5.52 3.13 4.51 1.28 median 3.13s spread 4.23s exit [0]
RESULT runtime_warm_get_drug_imatinib_median_s = 3.13
RESULT runtime_warm_get_drug_imatinib_spread_s = 4.23
warm_search_article_BRAF: runs 1.75 1.72 3.17 1.70 1.70 median 1.72s spread 1.47s exit [0]
RESULT runtime_warm_search_article_BRAF_median_s = 1.72
RESULT runtime_warm_search_article_BRAF_spread_s = 1.47
warm_search_all_BRAF_melanoma: runs 12.02 12.02 12.02 11.57 10.24 median 12.02s spread 1.78s exit [0]
RESULT runtime_warm_search_all_BRAF_melanoma_median_s = 12.02
RESULT runtime_warm_search_all_BRAF_melanoma_spread_s = 1.78
== cache off (BIOMCP_CACHE_MODE=off, network), five runs each ==
nocache_get_gene_BRAF: runs 0.60 0.45 0.45 0.44 0.47 median 0.45s spread 0.16s exit [0]
RESULT runtime_nocache_get_gene_BRAF_median_s = 0.45
RESULT runtime_nocache_get_gene_BRAF_spread_s = 0.16
nocache_get_variant_BRAF_V600E: runs 0.55 0.53 0.49 0.52 0.49 median 0.52s spread 0.07s exit [0]
RESULT runtime_nocache_get_variant_BRAF_V600E_median_s = 0.52
RESULT runtime_nocache_get_variant_BRAF_V600E_spread_s = 0.07
nocache_get_drug_imatinib: runs 3.46 3.49 2.27 2.00 3.38 median 3.38s spread 1.49s exit [0]
RESULT runtime_nocache_get_drug_imatinib_median_s = 3.38
RESULT runtime_nocache_get_drug_imatinib_spread_s = 1.49
nocache_search_article_BRAF: runs 1.93 2.81 1.88 1.91 2.80 median 1.93s spread 0.93s exit [0]
RESULT runtime_nocache_search_article_BRAF_median_s = 1.93
RESULT runtime_nocache_search_article_BRAF_spread_s = 0.93
nocache_search_all_BRAF_melanoma: runs 10.57 10.42 9.74 9.72 9.20 median 9.74s spread 1.37s exit [0]
RESULT runtime_nocache_search_all_BRAF_melanoma_median_s = 9.74
RESULT runtime_nocache_search_all_BRAF_melanoma_spread_s = 1.37

### host-after — machine conditions at the end
$ cat /proc/loadavg
0.04 0.19 0.33 1/500 2490112
$ free -g | sed -n 1,2p
               total        used        free      shared  buff/cache   available
Mem:              14           1           5           0           8          13
$ df -h . | tail -1
/dev/sda1       916G  438G  433G  51% /

### done — measurement complete

### runtime — finer-precision version runs and output sizes (added after the first pass printed whole seconds for --version)
$ python3 (five timed --version runs per cache, milliseconds; then one run per command capturing output size)
version_empty_cache: runs_ms 4.1 4.1 4.1 4.1 4.0 median 4.1ms spread 0.2ms
RESULT runtime_version_empty_cache_median_ms = 4.1
RESULT runtime_version_empty_cache_spread_ms = 0.2
version_large_cache: runs_ms 4.0 4.1 3.9 3.9 4.0 median 4.0ms spread 0.2ms
RESULT runtime_version_large_cache_median_ms = 4.0
RESULT runtime_version_large_cache_spread_ms = 0.2
RESULT runtime_output_bytes_get_gene_BRAF = 1646 exit 0 (0.75s)
RESULT runtime_output_bytes_get_variant_BRAF_V600E = 1371 exit 0 (0.57s)
RESULT runtime_output_bytes_get_drug_imatinib = 2410 exit 0 (7.95s)
RESULT runtime_output_bytes_search_article_BRAF = 2555 exit 0 (2.88s)
RESULT runtime_output_bytes_search_all_BRAF_melanoma = 1381 exit 0 (12.02s)
addendum done
```

### Pass 2, first attempt (host through dependency counts, and the flaky failure)

```

### locks — held for the whole run
$ flock -n ~/.yr-checkout.lock && flock -n ~/.<build-host>-gate.lock
both locks held by pid 2490856

### host — machine and toolchain conditions at the start
$ hostname; uname -sr; nproc
Linux check host
Linux 6.17.0-35-generic
4
$ cat /proc/loadavg
0.27 0.09 0.19 1/506 2490865
$ free -g | sed -n 1,2p
               total        used        free      shared  buff/cache   available
Mem:              14           1           5           0           8          13
$ df -h . | tail -1
/dev/sda1       916G  438G  432G  51% /
$ rustc --version; cargo --version; cargo nextest --version; cargo-deny --version; cargo llvm-cov --version; cargo llvm-lines --version; ruff --version; python3 --version; uv --version; node --version 2>/dev/null || true; mustmatch --version 2>/dev/null || true
rustc 1.93.1 (01f6ddf75 2026-02-11)
cargo 1.93.1 (083ac5135 2025-12-15)
cargo-nextest 0.9.132 (6e4a9d6f2 2026-03-20)
release: 0.9.132
commit-hash: 6e4a9d6f2c4964f30ff54a8cd5466f8869267daa
commit-date: 2026-03-20
host: x86_64-unknown-linux-gnu
cargo-deny 0.19.4
cargo-llvm-cov 0.9.1
cargo-llvm-lines 0.4.48
ruff 0.13.2
Python 3.13.7
uv 0.9.26
v22.22.3
mustmatch 0.1.0
$ cat ~/.cargo/config.toml 2>/dev/null || echo "no ~/.cargo/config.toml"
# Lean Rust dev builds on the Linux check host, matching the agent host (user cargo/config.toml).
# Measured on BioMCP 2026-09-22: fresh build 833 s -> 296 s, next ticket 374 s -> 145 s, 13 GB -> 2.1 GB.
# Long edit-compile loop on one crate: CARGO_INCREMENTAL=1. Debugger: CARGO_PROFILE_DEV_DEBUG=true.
# No sccache or mold here, so no rustc-wrapper or linker flags. Record: user configuration, ubuntu/<build-host>-build-host.md.
[build]
incremental = false

[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = false
$ cat .config/nextest.toml 2>/dev/null || echo "no .config/nextest.toml"
# A hung test must die, not stall the gate: nextest marks a test SLOW when
# it passes the slow-timeout period and terminates it after the second
# period elapses, so a blocking wait fails its test instead of hanging the
# run forever (ticket 2038 finding 7, 2035 #10). The suite's own watchdogs
# bound legitimate waits well under one period.
[profile.default]
slow-timeout = { period = "120s", terminate-after = 2 }

### clone — the tree being measured
$ git rev-parse HEAD; git describe --tags --exact-match HEAD 2>/dev/null || true; git log -1 --format="%H %s"; git status --porcelain | wc -l
4049666f5cce2794f16cdbdf0ded49878e5bb147
v0.9.2
4049666f5cce2794f16cdbdf0ded49878e5bb147 Land 2060: Make the release CI check use flags the gh CLI has
0

### quiet — no other build or gate process is running
$ pgrep -x cargo rustc cargo-nextest clippy-driver pytest mkdocs make uv cargo-deny cargo-llvm-cov
quiet: no matching process is running

### cold-dev-build — cold dev build of every target (routine feature set)
$ /usr/bin/time -f "TIMEFMT WALL %e USER %U SYS %S MAXRSS_KB %M" cargo build --locked --no-default-features --all-targets --timings
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling cfg-if v1.0.4
   Compiling libc v0.2.182
   Compiling serde_core v1.0.228
   Compiling syn v2.0.117
   Compiling serde v1.0.228
   Compiling version_check v0.9.5
   Compiling itoa v1.0.17
   Compiling memchr v2.8.0
   Compiling smallvec v1.15.1
   Compiling generic-array v0.14.7
   Compiling typenum v1.19.0
   Compiling pin-project-lite v0.2.17
   Compiling siphasher v1.0.2
   Compiling bytes v1.11.1
   Compiling futures-core v0.3.32
   Compiling once_cell v1.21.3
   Compiling jobserver v0.1.34
   Compiling zerocopy v0.8.40
   Compiling find-msvc-tools v0.1.9
   Compiling shlex v1.3.0
   Compiling stable_deref_trait v1.2.1
   Compiling cc v1.2.56
   Compiling log v0.4.29
   Compiling futures-sink v0.3.32
   Compiling subtle v2.6.1
   Compiling errno v0.3.14
   Compiling signal-hook-registry v1.4.8
   Compiling futures-channel v0.3.32
   Compiling synstructure v0.13.2
   Compiling mio v1.1.1
   Compiling socket2 v0.6.2
   Compiling bitflags v2.11.0
   Compiling futures-task v0.3.32
   Compiling futures-io v0.3.32
   Compiling slab v0.4.12
   Compiling http v1.4.0
   Compiling getrandom v0.2.17
   Compiling rand_core v0.6.4
   Compiling crypto-common v0.1.7
   Compiling zmij v1.0.21
   Compiling serde_derive v1.0.228
   Compiling zerofrom-derive v0.1.6
   Compiling yoke-derive v0.8.1
   Compiling ppv-lite86 v0.2.21
   Compiling tokio-macros v2.6.1
   Compiling futures-macro v0.3.32
   Compiling zerofrom v0.1.6
   Compiling yoke v0.8.1
   Compiling zerovec-derive v0.11.2
   Compiling tokio v1.50.0
   Compiling futures-util v0.3.32
   Compiling zerovec v0.11.5
   Compiling displaydoc v0.2.5
   Compiling rand_chacha v0.3.1
   Compiling rand v0.8.6
   Compiling tinystr v0.8.2
   Compiling new_debug_unreachable v1.0.6
   Compiling cpufeatures v0.2.17
   Compiling writeable v0.6.2
   Compiling litemap v0.8.1
   Compiling parking_lot_core v0.9.12
   Compiling icu_locale_core v2.1.1
   Compiling zerotrie v0.2.3
   Compiling potential_utf v0.1.4
   Compiling http-body v1.0.1
   Compiling block-buffer v0.10.4
   Compiling percent-encoding v2.3.2
   Compiling serde_json v1.0.149
   Compiling icu_normalizer_data v2.1.1
   Compiling icu_properties_data v2.1.2
   Compiling crc32fast v1.5.0
   Compiling scopeguard v1.2.0
   Compiling lock_api v0.4.14
   Compiling digest v0.10.7
   Compiling icu_provider v2.1.1
   Compiling icu_collections v2.1.1
   Compiling tracing-core v0.1.36
   Compiling phf_shared v0.13.1
   Compiling adler2 v2.0.1
   Compiling siphasher v0.3.11
   Compiling simd-adler32 v0.3.8
   Compiling miniz_oxide v0.8.9
   Compiling parking_lot v0.12.5
   Compiling tracing-attributes v0.1.31
   Compiling ring v0.17.14
   Compiling phf_shared v0.11.3
   Compiling fastrand v2.3.0
   Compiling precomputed-hash v0.1.1
   Compiling tower-service v0.3.3
   Compiling httparse v1.10.1
   Compiling phf_generator v0.13.1
   Compiling flate2 v1.1.9
   Compiling icu_properties v2.1.2
   Compiling tracing v0.1.44
   Compiling icu_normalizer v2.1.1
   Compiling try-lock v0.2.5
   Compiling zeroize v1.8.2
   Compiling httpdate v1.0.3
   Compiling base64 v0.22.1
   Compiling rustls-pki-types v1.14.0
   Compiling want v0.3.1
   Compiling idna_adapter v1.2.1
   Compiling phf_generator v0.11.3
   Compiling phf_shared v0.10.0
   Compiling form_urlencoded v1.2.2
   Compiling http-body-util v0.1.3
   Compiling sync_wrapper v1.0.2
   Compiling tower-layer v0.3.3
   Compiling atomic-waker v1.1.2
   Compiling rand_core v0.10.0
   Compiling pkg-config v0.3.32
   Compiling getrandom v0.4.2
   Compiling rustix v1.1.4
   Compiling utf8_iter v1.0.4
   Compiling utf-8 v0.7.6
   Compiling pin-utils v0.1.0
   Compiling untrusted v0.9.0
   Compiling hyper v1.8.1
   Compiling idna v1.1.0
   Compiling phf_generator v0.10.0
   Compiling compression-core v0.4.31
   Compiling autocfg v1.5.0
   Compiling ipnet v2.12.0
   Compiling thiserror v1.0.69
   Compiling linux-raw-sys v0.12.1
   Compiling hyper-util v0.1.20
   Compiling num-traits v0.2.19
   Compiling url v2.5.8
   Compiling compression-codecs v0.4.37
   Compiling phf_codegen v0.10.0
   Compiling tower v0.5.3
   Compiling futures-executor v0.3.32
   Compiling thiserror-impl v1.0.69
   Compiling strsim v0.11.1
   Compiling rustls v0.23.45
   Compiling async-compression v0.4.41
   Compiling phf_codegen v0.13.1
   Compiling string_cache_codegen v0.6.1
   Compiling tokio-util v0.7.18
   Compiling sha2 v0.10.9
   Compiling powerfmt v0.2.0
   Compiling rustls-webpki v0.103.15
   Compiling mime v0.3.17
   Compiling ryu v1.0.23
   Compiling iri-string v0.7.10
   Compiling crossbeam-utils v0.8.21
   Compiling unicase v2.9.0
   Compiling same-file v1.0.6
   Compiling mime_guess v2.0.5
   Compiling walkdir v2.5.0
   Compiling deranged v0.5.8
   Compiling web_atoms v0.2.4
   Compiling tower-http v0.6.8
   Compiling futures v0.3.32
   Compiling phf_macros v0.13.1
   Compiling aho-corasick v1.1.4
   Compiling syn v1.0.109
   Compiling cfg_aliases v0.2.1
   Compiling num-conv v0.2.0
   Compiling num_threads v0.1.7
   Compiling regex-syntax v0.8.10
   Compiling iana-time-zone v0.1.65
   Compiling time-core v0.1.8
   Compiling time v0.3.47
   Compiling chrono v0.4.44
   Compiling tokio-rustls v0.26.4
   Compiling regex-automata v0.4.14
   Compiling nix v0.31.2
   Compiling phf v0.13.1
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling string_cache_codegen v0.5.4
   Compiling webpki-roots v1.0.6
   Compiling string_cache v0.9.0
   Compiling async-trait v0.1.89
   Compiling block-padding v0.3.3
   Compiling hybrid-array v0.4.10
   Compiling anyhow v1.0.103
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.16.1
   Compiling ident_case v1.0.1
   Compiling mac v0.1.1
   Compiling crossbeam-epoch v0.9.20
   Compiling thiserror v2.0.18
   Compiling tempfile v3.26.0
   Compiling indexmap v2.13.0
   Compiling futf v0.1.5
   Compiling darling_core v0.23.0
   Compiling inout v0.1.4
   Compiling markup5ever v0.11.0
   Compiling phf v0.10.1
   Compiling hyper-rustls v0.27.7
   Compiling serde_urlencoded v0.7.1
   Compiling tendril v0.5.0
   Compiling phf_macros v0.11.3
   Compiling tokio-stream v0.1.18
   Compiling thiserror-impl v2.0.18
   Compiling miette-derive v5.10.0
   Compiling unicode-width v0.1.14
   Compiling utf8parse v0.2.2
   Compiling getrandom v0.3.4
   Compiling ref-cast v1.0.25
   Compiling byteorder v1.5.0
   Compiling either v1.15.0
   Compiling dtoa v1.0.11
   Compiling dtoa-short v0.3.5
   Compiling miette v5.10.0
   Compiling anstyle-parse v0.2.7
   Compiling phf v0.11.3
   Compiling darling_macro v0.23.0
   Compiling markup5ever v0.38.0
   Compiling reqwest v0.12.28
   Compiling html5ever v0.26.0
   Compiling cipher v0.4.4
   Compiling tendril v0.4.3
   Compiling selectors v0.25.0
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling string_cache v0.8.9
   Compiling sha1 v0.10.6
   Compiling sha-1 v0.10.1
   Compiling cssparser-macros v0.6.1
   Compiling ref-cast-impl v1.0.25
   Compiling serde_derive_internals v0.29.1
   Compiling ahash v0.8.12
   Compiling anstyle-query v1.1.5
   Compiling rayon-core v1.13.0
   Compiling zstd-safe v5.0.2+zstd.1.5.2
   Compiling hex v0.4.3
   Compiling colorchoice v1.0.4
   Compiling xxhash-rust v0.8.15
   Compiling anstyle v1.0.13
   Compiling is_terminal_polyfill v1.70.2
   Compiling base64 v0.21.7
   Compiling cpufeatures v0.3.0
   Compiling chacha20 v0.10.2
   Compiling ssri v9.2.0
   Compiling anstream v0.6.21
   Compiling schemars_derive v1.2.1
   Compiling cssparser v0.31.2
   Compiling crossbeam-deque v0.8.6
   Compiling darling v0.23.0
   Compiling fxhash v0.2.1
   Compiling regex v1.12.3
   Compiling crypto-common v0.2.1
   Compiling block-buffer v0.12.0
   Compiling inout v0.2.2
   Compiling reflink-copy v0.1.29
   Compiling http-serde v2.1.1
   Compiling derive_more v0.99.20
   Compiling servo_arc v0.3.0
   Compiling memmap2 v0.5.10
   Compiling tinyvec_macros v0.1.1
   Compiling heck v0.5.0
   Compiling dyn-clone v1.0.20
   Compiling clap_lex v1.0.0
   Compiling unicode-width v0.2.2
   Compiling rmcp v1.7.0
   Compiling base64ct v1.8.3
   Compiling clap_builder v4.5.60
   Compiling getopts v0.2.24
   Compiling password-hash v0.4.2
   Compiling clap_derive v4.5.55
   Compiling cacache v13.1.0
   Compiling schemars v1.2.1
   Compiling tinyvec v1.10.0
   Compiling http-cache-semantics v2.1.1
   Compiling cipher v0.5.1
   Compiling rust-embed-utils v8.11.0
   Compiling rmcp-macros v1.7.0
   Compiling rand v0.10.1
   Compiling reqwest-middleware v0.4.2
   Compiling aes v0.8.4
   Compiling process-wrap v9.1.0
   Compiling xml5ever v0.38.0
   Compiling html5ever v0.38.0
   Compiling reqwest v0.13.4
   Compiling uuid v1.22.0
   Compiling sse-stream v0.2.1
   Compiling hmac v0.12.1
   Compiling bincode v1.3.3
   Compiling serde_spanned v0.6.9
   Compiling toml_datetime v0.6.11
   Compiling lazy_static v1.5.0
   Compiling toml_write v0.1.2
   Compiling winnow v0.7.15
   Compiling ego-tree v0.6.3
   Compiling option-ext v0.2.0
   Compiling unpdf v0.4.5
   Compiling pastey v0.2.1
   Compiling kuva v0.5.0
   Compiling scraper v0.18.1
   Compiling dirs-sys v0.4.1
   Compiling http-cache v0.20.1
   Compiling toml_edit v0.22.27
   Compiling sharded-slab v0.1.7
   Compiling markup5ever_rcdom v0.38.0+unofficial
   Compiling pbkdf2 v0.11.0
   Compiling rayon v1.12.0
   Compiling bzip2 v0.4.4
   Compiling retry-policies v0.4.0
   Compiling rust-embed-impl v8.11.0
   Compiling rc4 v0.2.0
   Compiling unicode-normalization v0.1.25
   Compiling zstd v0.11.2+zstd.1.5.2
   Compiling clap v4.5.60
   Compiling cbc v0.1.2
   Compiling matchers v0.2.0
   Compiling crossbeam-channel v0.5.15
   Compiling axum-core v0.5.6
   Compiling xattr v1.6.1
   Compiling tracing-log v0.2.0
   Compiling md-5 v0.10.6
   Compiling serde_path_to_error v0.1.20
   Compiling csv-core v0.1.13
   Compiling filetime v0.2.27
   Compiling thread_local v1.1.9
   Compiling matchit v0.8.4
   Compiling nu-ansi-term v0.50.3
   Compiling constant_time_eq v0.1.5
   Compiling colorous v1.0.16
   Compiling unicode-bidi v0.3.18
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass2)
   Compiling axum v0.8.8
   Compiling tracing-subscriber v0.3.22
   Compiling zip v0.6.6
   Compiling tar v0.4.46
   Compiling csv v1.4.0
   Compiling rust-embed v8.11.0
   Compiling readability-rust v0.1.0
   Compiling toml v0.8.23
   Compiling reqwest-retry v0.7.0
   Compiling htmd v0.5.4
   Compiling http-cache-reqwest v0.15.1
   Compiling dirs v5.0.1
   Compiling minijinja v2.17.1
   Compiling fs2 v0.4.3
   Compiling bytesize v1.3.3
   Compiling humantime v2.3.0
   Compiling semver v1.0.27
   Compiling roxmltree v0.20.0
   Compiling md5 v0.7.0
   Compiling yasna v0.5.2
   Compiling pem v3.0.6
   Compiling serial_test_derive v3.5.0
   Compiling comma v1.0.0
   Compiling rexpect v0.7.0
   Compiling rcgen v0.13.2
   Compiling serial_test v3.5.0
   Compiling biomcp-mcp-contract-client v0.1.0 (~/2069-baseline/pass2/crates/biomcp-mcp-contract-client)
      Timing report saved to ~/2069-baseline/pass2/target/cargo-timings/cargo-timing-20261014T031657.169876566Z.html
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 19s
TIMEFMT WALL 260.24 USER 743.63 SYS 52.32 MAXRSS_KB 6201180
RESULT cold_dev_build_wall_s = 260.3

### rust-edit-rebuild — rebuild after one Rust edit (dev profile, incremental off)
$ touch src/render/markdown/gene.rs
$ /usr/bin/time -f "TIMEFMT WALL %e USER %U SYS %S MAXRSS_KB %M" cargo build --locked --no-default-features --all-targets
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass2)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 42s
TIMEFMT WALL 162.59 USER 434.45 SYS 19.56 MAXRSS_KB 6185576
RESULT rust_edit_rebuild_wall_s = 162.6

### doc-edit-rebuild — rebuild after one doc edit (a spec page touch; v0.9.2 build.rs scans the whole package)
$ touch spec/entity/gene.md
$ /usr/bin/time -f "TIMEFMT WALL %e USER %U SYS %S MAXRSS_KB %M" cargo build --locked --no-default-features --all-targets
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass2)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 50s
TIMEFMT WALL 170.67 USER 429.40 SYS 19.20 MAXRSS_KB 6181388
RESULT doc_edit_rebuild_wall_s = 170.7

### dep-counts — direct dependencies, Cargo.lock packages, crates compiled, duplicate names
$ python3 (counts from Cargo.toml and Cargo.lock)
RESULT direct_dependencies_normal = 58 (optional: 3: prost, tonic, zstd)
RESULT direct_dependencies_dev = 5
RESULT direct_dependencies_windows_only = 1
RESULT direct_dependencies_unix_dev = 1
RESULT cargo_lock_packages = 488
RESULT cargo_lock_duplicate_names = 50 (extra copies: 68)
$ cargo tree --locked --no-default-features -e normal,build --prefix none > .cache/measure/tree-normal-build.txt
$ cargo tree --locked --no-default-features -e normal,build,dev --prefix none > .cache/measure/tree-normal-build-dev.txt
$ python3 (unique crate name@version pairs in each cargo tree, workspace members excluded)
RESULT crates_compiled_normal_build = 338
RESULT duplicate_crate_names_normal_build = 24 (base64, block-buffer, cipher, cpufeatures, crypto-common, getrandom, html5ever, inout, markup5ever, phf, phf_codegen, phf_generator, phf_macros, phf_shared, rand, rand_core, siphasher, string_cache, string_cache_codegen, syn, tendril, thiserror, thiserror-impl, unicode-width)
RESULT crates_compiled_with_dev = 349
RESULT duplicate_crate_names_with_dev = 25 (base64, block-buffer, cipher, cpufeatures, crypto-common, getrandom, html5ever, inout, markup5ever, phf, phf_codegen, phf_generator, phf_macros, phf_shared, rand, rand_core, reqwest, siphasher, string_cache, string_cache_codegen, syn, tendril, thiserror, thiserror-impl, unicode-width)

### prepare-context — one prepare so the timed gates below start with fresh artifacts (context, not a baseline number)
$ make prepare-test
mkdir -p "~/2069-baseline/pass2/.cache/routine-test-tmp"
tools/with-build-identity cargo build --locked --profile spec --no-default-features --bin biomcp --example rmcp_streamable_http_contract
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling libc v0.2.182
   Compiling cfg-if v1.0.4
   Compiling serde_core v1.0.228
   Compiling syn v2.0.117
   Compiling serde v1.0.228
   Compiling version_check v0.9.5
   Compiling itoa v1.0.17
   Compiling smallvec v1.15.1
   Compiling memchr v2.8.0
   Compiling generic-array v0.14.7
   Compiling typenum v1.19.0
   Compiling pin-project-lite v0.2.17
   Compiling zerocopy v0.8.40
   Compiling bytes v1.11.1
   Compiling once_cell v1.21.3
   Compiling siphasher v1.0.2
   Compiling futures-core v0.3.32
   Compiling jobserver v0.1.34
   Compiling shlex v1.3.0
   Compiling find-msvc-tools v0.1.9
   Compiling stable_deref_trait v1.2.1
   Compiling cc v1.2.56
   Compiling log v0.4.29
   Compiling subtle v2.6.1
   Compiling futures-sink v0.3.32
   Compiling errno v0.3.14
   Compiling signal-hook-registry v1.4.8
   Compiling synstructure v0.13.2
   Compiling futures-channel v0.3.32
   Compiling mio v1.1.1
   Compiling socket2 v0.6.2
   Compiling bitflags v2.11.0
   Compiling slab v0.4.12
   Compiling futures-task v0.3.32
   Compiling zmij v1.0.21
   Compiling futures-io v0.3.32
   Compiling getrandom v0.2.17
   Compiling http v1.4.0
   Compiling serde_derive v1.0.228
   Compiling zerofrom-derive v0.1.6
   Compiling zerofrom v0.1.6
   Compiling yoke-derive v0.8.1
   Compiling tokio-macros v2.6.1
   Compiling yoke v0.8.1
   Compiling zerovec-derive v0.11.2
   Compiling tokio v1.50.0
   Compiling futures-macro v0.3.32
   Compiling zerovec v0.11.5
   Compiling futures-util v0.3.32
   Compiling displaydoc v0.2.5
   Compiling ppv-lite86 v0.2.21
   Compiling rand_core v0.6.4
   Compiling crypto-common v0.1.7
   Compiling rand_chacha v0.3.1
   Compiling tinystr v0.8.2
   Compiling rand v0.8.6
   Compiling parking_lot_core v0.9.12
   Compiling writeable v0.6.2
   Compiling litemap v0.8.1
   Compiling new_debug_unreachable v1.0.6
   Compiling icu_locale_core v2.1.1
   Compiling zerotrie v0.2.3
   Compiling potential_utf v0.1.4
   Compiling http-body v1.0.1
   Compiling block-buffer v0.10.4
   Compiling crc32fast v1.5.0
   Compiling scopeguard v1.2.0
   Compiling serde_json v1.0.149
   Compiling icu_normalizer_data v2.1.1
   Compiling percent-encoding v2.3.2
   Compiling icu_properties_data v2.1.2
   Compiling lock_api v0.4.14
   Compiling digest v0.10.7
   Compiling icu_provider v2.1.1
   Compiling icu_collections v2.1.1
   Compiling tracing-core v0.1.36
   Compiling adler2 v2.0.1
   Compiling simd-adler32 v0.3.8
   Compiling parking_lot v0.12.5
   Compiling miniz_oxide v0.8.9
   Compiling tracing-attributes v0.1.31
   Compiling ring v0.17.14
   Compiling phf_shared v0.13.1
   Compiling cpufeatures v0.2.17
   Compiling httparse v1.10.1
   Compiling fastrand v2.3.0
   Compiling tower-service v0.3.3
   Compiling precomputed-hash v0.1.1
   Compiling phf_generator v0.13.1
   Compiling tracing v0.1.44
   Compiling icu_normalizer v2.1.1
   Compiling icu_properties v2.1.2
   Compiling flate2 v1.1.9
   Compiling phf_shared v0.11.3
   Compiling zeroize v1.8.2
   Compiling try-lock v0.2.5
   Compiling siphasher v0.3.11
   Compiling httpdate v1.0.3
   Compiling base64 v0.22.1
   Compiling phf_shared v0.10.0
   Compiling want v0.3.1
   Compiling rustls-pki-types v1.14.0
   Compiling phf_generator v0.11.3
   Compiling idna_adapter v1.2.1
   Compiling form_urlencoded v1.2.2
   Compiling http-body-util v0.1.3
   Compiling sync_wrapper v1.0.2
   Compiling getrandom v0.4.2
   Compiling untrusted v0.9.0
   Compiling rand_core v0.10.0
   Compiling utf-8 v0.7.6
   Compiling tower-layer v0.3.3
   Compiling utf8_iter v1.0.4
   Compiling pin-utils v0.1.0
   Compiling pkg-config v0.3.32
   Compiling rustix v1.1.4
   Compiling atomic-waker v1.1.2
   Compiling idna v1.1.0
   Compiling phf_generator v0.10.0
   Compiling hyper v1.8.1
   Compiling compression-core v0.4.31
   Compiling autocfg v1.5.0
   Compiling linux-raw-sys v0.12.1
   Compiling ipnet v2.12.0
   Compiling thiserror v1.0.69
   Compiling hyper-util v0.1.20
   Compiling num-traits v0.2.19
   Compiling url v2.5.8
   Compiling compression-codecs v0.4.37
   Compiling phf_codegen v0.10.0
   Compiling tower v0.5.3
   Compiling futures-executor v0.3.32
   Compiling thiserror-impl v1.0.69
   Compiling rustls v0.23.45
   Compiling rustls-webpki v0.103.15
   Compiling async-compression v0.4.41
   Compiling string_cache_codegen v0.6.1
   Compiling phf_codegen v0.13.1
   Compiling sha2 v0.10.9
   Compiling tokio-util v0.7.18
   Compiling unicase v2.9.0
   Compiling ryu v1.0.23
   Compiling iri-string v0.7.10
   Compiling powerfmt v0.2.0
   Compiling crossbeam-utils v0.8.21
   Compiling mime v0.3.17
   Compiling deranged v0.5.8
   Compiling mime_guess v2.0.5
   Compiling web_atoms v0.2.4
   Compiling tower-http v0.6.8
   Compiling futures v0.3.32
   Compiling phf_macros v0.13.1
   Compiling aho-corasick v1.1.4
   Compiling time-core v0.1.8
   Compiling num_threads v0.1.7
   Compiling num-conv v0.2.0
   Compiling iana-time-zone v0.1.65
   Compiling syn v1.0.109
   Compiling regex-syntax v0.8.10
   Compiling cfg_aliases v0.2.1
   Compiling nix v0.31.2
   Compiling chrono v0.4.44
   Compiling regex-automata v0.4.14
   Compiling tokio-rustls v0.26.4
   Compiling time v0.3.47
   Compiling phf v0.13.1
   Compiling string_cache v0.9.0
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling webpki-roots v1.0.6
   Compiling string_cache_codegen v0.5.4
   Compiling async-trait v0.1.89
   Compiling block-padding v0.3.3
   Compiling hybrid-array v0.4.10
   Compiling ident_case v1.0.1
   Compiling equivalent v1.0.2
   Compiling thiserror v2.0.18
   Compiling same-file v1.0.6
   Compiling hashbrown v0.16.1
   Compiling anyhow v1.0.103
   Compiling crossbeam-epoch v0.9.20
   Compiling mac v0.1.1
   Compiling strsim v0.11.1
   Compiling indexmap v2.13.0
   Compiling darling_core v0.23.0
   Compiling futf v0.1.5
   Compiling tempfile v3.26.0
   Compiling walkdir v2.5.0
   Compiling inout v0.1.4
   Compiling markup5ever v0.11.0
   Compiling hyper-rustls v0.27.7
   Compiling phf v0.10.1
   Compiling serde_urlencoded v0.7.1
   Compiling tendril v0.5.0
   Compiling phf_macros v0.11.3
   Compiling tokio-stream v0.1.18
   Compiling miette-derive v5.10.0
   Compiling thiserror-impl v2.0.18
   Compiling getrandom v0.3.4
   Compiling byteorder v1.5.0
   Compiling unicode-width v0.1.14
   Compiling dtoa v1.0.11
   Compiling utf8parse v0.2.2
   Compiling ref-cast v1.0.25
   Compiling either v1.15.0
   Compiling anstyle-parse v0.2.7
   Compiling dtoa-short v0.3.5
   Compiling miette v5.10.0
   Compiling html5ever v0.26.0
   Compiling phf v0.11.3
   Compiling markup5ever v0.38.0
   Compiling reqwest v0.12.28
   Compiling cipher v0.4.4
   Compiling darling_macro v0.23.0
   Compiling tendril v0.4.3
   Compiling string_cache v0.8.9
   Compiling selectors v0.25.0
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling sha1 v0.10.6
   Compiling sha-1 v0.10.1
   Compiling cssparser-macros v0.6.1
   Compiling ref-cast-impl v1.0.25
   Compiling serde_derive_internals v0.29.1
   Compiling ahash v0.8.12
   Compiling zstd-safe v5.0.2+zstd.1.5.2
   Compiling anstyle-query v1.1.5
   Compiling hex v0.4.3
   Compiling colorchoice v1.0.4
   Compiling anstyle v1.0.13
   Compiling base64 v0.21.7
   Compiling rayon-core v1.13.0
   Compiling cpufeatures v0.3.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling xxhash-rust v0.8.15
   Compiling ssri v9.2.0
   Compiling anstream v0.6.21
   Compiling chacha20 v0.10.2
   Compiling schemars_derive v1.2.1
   Compiling cssparser v0.31.2
   Compiling crossbeam-deque v0.8.6
   Compiling darling v0.23.0
   Compiling fxhash v0.2.1
   Compiling crypto-common v0.2.1
   Compiling inout v0.2.2
   Compiling block-buffer v0.12.0
   Compiling regex v1.12.3
   Compiling reflink-copy v0.1.29
   Compiling http-serde v2.1.1
   Compiling derive_more v0.99.20
   Compiling servo_arc v0.3.0
   Compiling memmap2 v0.5.10
   Compiling heck v0.5.0
   Compiling tinyvec_macros v0.1.1
   Compiling clap_lex v1.0.0
   Compiling dyn-clone v1.0.20
   Compiling base64ct v1.8.3
   Compiling rmcp v1.7.0
   Compiling unicode-width v0.2.2
   Compiling password-hash v0.4.2
   Compiling getopts v0.2.24
   Compiling clap_builder v4.5.60
   Compiling schemars v1.2.1
   Compiling tinyvec v1.10.0
   Compiling clap_derive v4.5.55
   Compiling cacache v13.1.0
   Compiling http-cache-semantics v2.1.1
   Compiling cipher v0.5.1
   Compiling rmcp-macros v1.7.0
   Compiling rand v0.10.1
   Compiling rust-embed-utils v8.11.0
   Compiling reqwest-middleware v0.4.2
   Compiling process-wrap v9.1.0
   Compiling aes v0.8.4
   Compiling xml5ever v0.38.0
   Compiling html5ever v0.38.0
   Compiling reqwest v0.13.4
   Compiling uuid v1.22.0
   Compiling sse-stream v0.2.1
   Compiling hmac v0.12.1
   Compiling serde_spanned v0.6.9
   Compiling bincode v1.3.3
   Compiling toml_datetime v0.6.11
   Compiling lazy_static v1.5.0
   Compiling ego-tree v0.6.3
   Compiling option-ext v0.2.0
   Compiling kuva v0.5.0
   Compiling toml_write v0.1.2
   Compiling unpdf v0.4.5
   Compiling pastey v0.2.1
   Compiling winnow v0.7.15
   Compiling http-cache v0.20.1
   Compiling toml_edit v0.22.27
   Compiling dirs-sys v0.4.1
   Compiling scraper v0.18.1
   Compiling sharded-slab v0.1.7
   Compiling pbkdf2 v0.11.0
   Compiling markup5ever_rcdom v0.38.0+unofficial
   Compiling rust-embed-impl v8.11.0
   Compiling bzip2 v0.4.4
   Compiling retry-policies v0.4.0
   Compiling rayon v1.12.0
   Compiling rc4 v0.2.0
   Compiling zstd v0.11.2+zstd.1.5.2
   Compiling clap v4.5.60
   Compiling unicode-normalization v0.1.25
   Compiling cbc v0.1.2
   Compiling crossbeam-channel v0.5.15
   Compiling matchers v0.2.0
   Compiling axum-core v0.5.6
   Compiling xattr v1.6.1
   Compiling tracing-log v0.2.0
   Compiling md-5 v0.10.6
   Compiling serde_path_to_error v0.1.20
   Compiling csv-core v0.1.13
   Compiling filetime v0.2.27
   Compiling thread_local v1.1.9
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass2)
   Compiling unicode-bidi v0.3.18
   Compiling matchit v0.8.4
   Compiling constant_time_eq v0.1.5
   Compiling nu-ansi-term v0.50.3
   Compiling colorous v1.0.16
   Compiling tracing-subscriber v0.3.22
   Compiling zip v0.6.6
   Compiling axum v0.8.8
   Compiling tar v0.4.46
   Compiling csv v1.4.0
   Compiling rust-embed v8.11.0
   Compiling readability-rust v0.1.0
   Compiling reqwest-retry v0.7.0
   Compiling htmd v0.5.4
   Compiling dirs v5.0.1
   Compiling toml v0.8.23
   Compiling http-cache-reqwest v0.15.1
   Compiling minijinja v2.17.1
   Compiling fs2 v0.4.3
   Compiling bytesize v1.3.3
   Compiling semver v1.0.27
   Compiling md5 v0.7.0
   Compiling humantime v2.3.0
   Compiling roxmltree v0.20.0
   Compiling yasna v0.5.2
   Compiling pem v3.0.6
   Compiling serial_test_derive v3.5.0
   Compiling comma v1.0.0
   Compiling rexpect v0.7.0
   Compiling serial_test v3.5.0
   Compiling rcgen v0.13.2
   Compiling biomcp-mcp-contract-client v0.1.0 (~/2069-baseline/pass2/crates/biomcp-mcp-contract-client)
    Finished `spec` profile [optimized] target(s) in 7m 46s
make sync-python-dev
make[1]: Entering directory '~/2069-baseline/pass2'
uv sync --extra dev --no-install-project
Using CPython 3.11.14
Creating virtual environment at: .venv
Resolved 48 packages in 1ms
Installed 44 packages in 1.64s
 + attrs==25.4.0
 + babel==2.18.0
 + backrefs==6.2
 + certifi==2026.1.4
 + charset-normalizer==3.4.4
 + click==8.3.1
 + colorama==0.4.6
 + execnet==2.1.2
 + ghp-import==2.1.0
 + idna==3.18
 + iniconfig==2.3.0
 + jinja2==3.1.6
 + jsonschema==4.26.0
 + jsonschema-specifications==2025.9.1
 + markdown==3.10.2
 + markupsafe==3.0.3
 + mergedeep==1.3.4
 + mkdocs==1.6.1
 + mkdocs-get-deps==0.2.0
 + mkdocs-material==9.7.2
 + mkdocs-material-extensions==1.3.1
 + packaging==26.0
 + paginate==0.5.7
 + pathspec==1.0.4
 + platformdirs==4.9.2
 + pluggy==1.6.0
 + pygments==2.20.0
 + pymdown-extensions==11.0.1
 + pytest==9.0.3
 + pytest-asyncio==1.3.0
 + pytest-timeout==2.4.0
 + pytest-xdist==3.8.0
 + python-dateutil==2.9.0.post0
 + pyyaml==6.0.3
 + pyyaml-env-tag==1.1
 + referencing==0.37.0
 + regex==2026.7.19
 + requests==2.34.2
 + rpds-py==0.30.0
 + six==1.17.0
 + tiktoken==0.12.0
 + typing-extensions==4.15.0
 + urllib3==2.7.0
 + watchdog==6.0.0
make[1]: Leaving directory '~/2069-baseline/pass2'
mkdir -p "~/2069-baseline/pass2/.cache"
tools/with-build-identity cargo nextest archive --locked --no-default-features --archive-file "~/2069-baseline/pass2/.cache/routine-tests.tar.zst" --zstd-level -7
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass2)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.98s
   Archiving 30 binaries (including 1 non-test binary; 1 non-test binary filtered out), 1 build script output directory, 2 linked paths, and 1 standard library to ~/2069-baseline/pass2/.cache/routine-tests.tar.zst
    Archived 106 files to ~/2069-baseline/pass2/.cache/routine-tests.tar.zst in 5.01s
RESULT make_prepare_test_wall_s = 477.9

### make-test — the routine gate, wall time with artifacts already fresh
$ make test > .cache/measure/make-test.log 2>&1
```

### Pass 2, completed (includes the doc-edit and drug reruns)

```

### locks — held for the whole run
$ flock -n ~/.yr-checkout.lock && flock -n ~/.<build-host>-gate.lock
both locks held by pid 2523718

### make-test — the routine gate, wall time with artifacts already fresh
$ make test > .cache/measure/make-test.log 2>&1
RESULT make_test_wall_s = 640.0
$ grep -E 'Summary \[|tests? run:' .cache/measure/make-test.log | tail -1 || true
     Summary [ 150.086s] 4205 tests run: 4205 passed, 33 skipped
$ grep -E '^=+ .* (passed|failed|error).* =+$' .cache/measure/make-test.log | tail -2 || true
================= 1251 passed, 3 skipped in 477.50s (0:07:57) ==================
$ grep -iE 'documentation built|docs built' .cache/measure/make-test.log | tail -1 || true
INFO    -  Documentation built in 2.49 seconds
$ python3 (test counts parsed from the make-test log)
RESULT rust_tests_run = 4205
RESULT rust_tests_passed = 4205
RESULT rust_tests_skipped = 33
RESULT python_lane_summary = 1251 passed, 3 skipped in 477.50s (0:07:57)

### build-folder-after-make-test — target/ bytes after make test
$ du -sb target | awk "{print \$1}"
3131377964
$ du -sh target
3.0G	target
RESULT build_folder_bytes_after_make_test = 3131377964

### rust-test-census — test-duration census of the routine Rust suite (from the archive make test built)
$ NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 tools/run-offline -- cargo nextest run --archive-file .cache/routine-tests.tar.zst --config-file .config/nextest.toml --message-format libtest-json-plus > .cache/measure/census.jsonl 2> .cache/measure/census.err || true
RESULT rust_test_census_wall_s = 109.5
$ tail -2 .cache/measure/census.err || true
────────────
     Summary [ 108.240s] 4205 tests run: 4205 passed, 33 skipped
$ python3 (count tests at or over one second from the libtest-json-plus stream)
RESULT rust_tests_with_duration = 4205
RESULT tests_over_1s = 62
RESULT tests_over_1s_sum_s = 282.2
slowest tests:
      23.7s  biomcp-cli::tls_ca_bundle_contract$fda_orphan_client_reaches_a_private_ca_provider
      17.7s  biomcp-cli::biomcp_cli$entities::variant::get::tests::protein_change_get_notes_unchecked_numbering_when_uniprot_is_unreachable
      16.6s  biomcp-cli::rmcp_client_contract$raw_and_typed_article_query_validation_converges_before_provider_work
      14.2s  biomcp-cli::biomcp_cli$entities::disease::get::tests::a_stale_mcp_call_tells_the_clinician_the_cache_age_on_both_channels
      12.9s  biomcp-cli::biomcp_cli$entities::article::assets::tests::manifest_and_bytes_preserve_induced_archive_failure_after_figshare_miss
      11.8s  biomcp-cli::biomcp_cli$entities::disease::get::tests::a_stale_disease_card_tells_the_clinician_the_cache_age
      10.7s  biomcp-cli::biomcp_cli$cli::stale_json_note_tests::a_stale_article_search_json_states_the_cache_age_in_meta_notes
      10.1s  biomcp-cli::biomcp_cli$cli::stale_json_note_tests::a_stale_clingen_prefetch_note_reaches_the_gene_card
       9.8s  biomcp-cli::biomcp_cli$cli::stale_json_note_tests::no_cache_skips_the_cache_for_the_spawned_clingen_fetch
       8.4s  biomcp-cli::biomcp_cli$sources::gencc::tests::cross_process_first_use_elects_one_leader_and_settles_followers
       8.0s  biomcp-cli::rmcp_client_contract$human_mcp_command_dispatches_to_provider_once
       7.4s  biomcp-cli::biomcp_cli$entities::drug::test_support::label_fallback_fetch_error_stays_an_error
       5.9s  biomcp-cli::biomcp_cli$entities::trial::search::eligibility::keep_paths::a_failed_detail_fetch_keeps_the_study_and_reports_it
       5.6s  biomcp-cli::biomcp_cli$cli::article::tests::entities_disease_links::article_entities_disease_rows_link_the_named_disease_or_search
       5.5s  biomcp-cli::biomcp_cli$entities::article::variant_search::tests::degraded_strict_leg_partials_mark_the_route_incomplete

### python-test-count — python lane test count (collection, no runs)
$ tools/run-offline -- env BIOMCP_BIN=~/2069-baseline/pass2/target/spec/biomcp uv run --no-sync pytest tests/ --collect-only -q > .cache/measure/pytest-collect.txt 2>&1
RESULT python_collect_wall_s = 0.9
$ tail -2 .cache/measure/pytest-collect.txt
======================== 1254 tests collected in 0.43s =========================
offline ownership mapping: host uid/gid 1000/1000 verified
$ python3 (collected count)
RESULT python_tests_collected = 1254

### spec-block-census — executable bash blocks across the spec pages
$ grep -rlE '^[[:space:]]*```bash[[:space:]]*$' spec --include='*.md' | wc -l
38
$ grep -rE '^[[:space:]]*```bash[[:space:]]*$' spec --include='*.md' | wc -l
496
RESULT spec_pages_with_bash_blocks = 38
RESULT spec_bash_blocks = 496

### make-spec — the spec lane, wall time with artifacts already fresh
$ make spec > .cache/measure/make-spec.log 2>&1
RESULT make_spec_wall_s = 668.3
$ tail -6 .cache/measure/make-spec.log
make[1]: Entering directory '~/2069-baseline/pass2'
bash scripts/run-specs.sh spec-static
=== spec pages: spec/surface/docker-image.md spec/surface/homebrew.md ===
7 passed
make[1]: Leaving directory '~/2069-baseline/pass2'
offline ownership mapping: host uid/gid 1000/1000 verified

### make-lint — the lint gate, wall time
$ make lint > .cache/measure/make-lint.log 2>&1
RESULT make_lint_wall_s = 196.5
$ tail -6 .cache/measure/make-lint.log
advisories ok
[PASS] Rust advisory lint (cargo deny check advisories)
[SKIP] JS/TS lint (not detected)
tools/check-quality-ratchet.sh
tools/check-test-wait-ratchet.py
test-wait ratchet ok (41 files with pinned ceilings, 46 watchdog markers under the ceiling 46)

### release-build — cold release build (default features, shipped profile: thin LTO, one codegen unit, strip)
$ /usr/bin/time -v -o .cache/measure/release-time.txt cargo build --release --locked --timings > .cache/measure/release-build.log 2>&1
RESULT release_build_wall_s = 838.8
$ grep -E 'Elapsed \(wall clock|Maximum resident set size|User time|System time' .cache/measure/release-time.txt
	User time (seconds): 1613.38
	System time (seconds): 50.11
	Elapsed (wall clock) time (h:mm:ss or m:ss): 13:58.75
	Maximum resident set size (kbytes): 4597756
$ tail -3 .cache/measure/release-build.log
   Compiling roxmltree v0.20.0
      Timing report saved to ~/2069-baseline/pass2/target/cargo-timings/cargo-timing-20261014T042914.061573306Z.html
    Finished `release` profile [optimized] target(s) in 13m 58s
RESULT release_peak_rss_kb = 4597756

### build-folder-after-release — target/ bytes after the release build
$ du -sb target | awk "{print \$1}"
4441880999
$ du -sh target
4.2G	target
RESULT build_folder_bytes_after_release_build = 4441880999

### stripped-binary — the stripped release binary's size in bytes
$ stat -c %s target/release/biomcp
39122976
$ file target/release/biomcp | head -1
target/release/biomcp: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 3.2.0, BuildID[sha1]=1afe53be889cdf51023aa5da65c688becf3faea0, stripped
RESULT stripped_release_binary_bytes = 39122976

### llvm-lines — LLVM IR lines and function copies of the library, release profile
$ cargo llvm-lines --release --lib > .cache/measure/llvm-lines.txt 2>&1
RESULT llvm_lines_wall_s = 85.9
$ head -8 .cache/measure/llvm-lines.txt
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass2)
    Finished `release` profile [optimized] target(s) in 1m 24s
  Lines                  Copies                Function name
  -----                  ------                -------------
  6819551                138257                (TOTAL)
   181284 (2.7%,  2.7%)     447 (0.3%,  0.3%)  <&mut serde_json::de::Deserializer<R> as serde_core::de::Deserializer>::deserialize_struct
    97818 (1.4%,  4.1%)     660 (0.5%,  0.8%)  serde_core::ser::Serializer::collect_seq
    94978 (1.4%,  5.5%)    1345 (1.0%,  1.8%)  core::iter::traits::iterator::Iterator::try_fold
$ awk (sum of the Lines and Copies columns)
RESULT llvm_lines_total = 6819551
RESULT llvm_function_copies = 138257

### deny-advisories — advisories ignored in deny.toml
$ python3 (count of [advisories].ignore entries)
RESULT ignored_advisories = 6
  RUSTSEC-2025-0141
  RUSTSEC-2025-0057
  RUSTSEC-2024-0384
  RUSTSEC-2025-0134
  RUSTSEC-2026-0186
  RUSTSEC-2026-0192

### coverage — line coverage of three suites through cargo-llvm-cov (dev build instrumented)
$ cargo llvm-cov clean --workspace
$ eval each line of "cargo llvm-cov show-env" as an export; CARGO_TARGET_DIR=target/llvm-cov-target; LLVM_PROFILE_FILE per suite
$ cargo build --locked --no-default-features --all-targets > .cache/measure/cov-build.log 2>&1
RESULT coverage_build_wall_s = 272.0
$ tools/run-offline -- cargo nextest run --locked --no-default-features -E 'kind(test)' > .cache/measure/cov-integration.log 2>&1 || true
RESULT coverage_integration_tests_wall_s = 171.6
$ grep -E 'Summary \[|tests? run:' .cache/measure/cov-integration.log | tail -1 || true
     Summary [ 161.859s] 135 tests run: 134 passed (2 slow), 1 failed, 5 skipped
$ tools/run-offline -- env BIOMCP_ROUTINE_TEST_LANE=1 BIOMCP_BIN=$PWD/target/llvm-cov-target/debug/biomcp TMPDIR=$PWD/.cache/measure/tmp uv run --no-sync pytest tests/ -n 4 --dist loadfile --basetemp .cache/measure/pytest-basetemp > .cache/measure/cov-pytest.log 2>&1 || true
RESULT coverage_python_lane_wall_s = 275.2
$ grep -E '^=+ .* (passed|failed|error).* =+$' .cache/measure/cov-pytest.log | tail -2 || true
============ 26 failed, 1225 passed, 3 skipped in 273.41s (0:04:33) ============

### coverage — spec lane against the instrumented dev binary (hand-written artifacts env)
$ cat > .cache/spec-artifacts.env <<'ENVEOF'
export BIOMCP_SPEC_ARTIFACT_MODE=spec
export BIOMCP_SPEC_FEATURE_OFF_BIN=~/2069-baseline/pass2/target/llvm-cov-target/debug/biomcp
export BIOMCP_SPEC_MCP_EXAMPLE_BIN=~/2069-baseline/pass2/target/llvm-cov-target/debug/examples/rmcp_streamable_http_contract
export BIOMCP_SPEC_CARGO_TREE=~/2069-baseline/pass2/.cache/spec-artifacts/cargo-tree-no-default-features.txt
export BIOMCP_SPEC_CARGO_METADATA=~/2069-baseline/pass2/.cache/spec-artifacts/cargo-metadata.json
export BIOMCP_BIN=~/2069-baseline/pass2/target/llvm-cov-target/debug/biomcp
ENVEOF
cat .cache/spec-artifacts.env
export BIOMCP_SPEC_ARTIFACT_MODE=spec
export BIOMCP_SPEC_FEATURE_OFF_BIN=~/2069-baseline/pass2/target/llvm-cov-target/debug/biomcp
export BIOMCP_SPEC_MCP_EXAMPLE_BIN=~/2069-baseline/pass2/target/llvm-cov-target/debug/examples/rmcp_streamable_http_contract
export BIOMCP_SPEC_CARGO_TREE=~/2069-baseline/pass2/.cache/spec-artifacts/cargo-tree-no-default-features.txt
export BIOMCP_SPEC_CARGO_METADATA=~/2069-baseline/pass2/.cache/spec-artifacts/cargo-metadata.json
export BIOMCP_BIN=~/2069-baseline/pass2/target/llvm-cov-target/debug/biomcp
$ PATH="~/2069-baseline/pass2/target/llvm-cov-target/debug:$PATH" BIOMCP_SPEC_ARTIFACTS_PREPARED=1 ROUTINE_CARGO_FEATURES=--no-default-features SPEC_PROFILE=dev tools/run-offline -- bash scripts/run-specs.sh spec > .cache/measure/cov-spec.log 2>&1 || true
RESULT coverage_spec_lane_wall_s = 717.6
$ tail -4 .cache/measure/cov-spec.log || true
tests/surface/test_parallel_isolation_contract.py::test_ticket_379_timing_docs_record_pruned_ownership PASSED [100%]

============================== 41 passed in 1.74s ==============================
offline ownership mapping: host uid/gid 1000/1000 verified
$ tools/run-offline -- cargo nextest run --locked --no-default-features -E 'kind(lib) | kind(bin)' > .cache/measure/cov-unit.log 2>&1 || true
RESULT coverage_unit_tests_wall_s = 77.4
$ grep -E 'Summary \[|tests? run:' .cache/measure/cov-unit.log | tail -1 || true
     Summary [  76.052s] 4070 tests run: 4070 passed, 28 skipped

### coverage — report: outside-in
$ mv target/llvm-cov-target/unit-*.profraw .cache/measure/cov-hold/ (report the other set)
$ env -u CARGO_TARGET_DIR cargo llvm-cov report --ignore-filename-regex '(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)$|test_support|(^|/)generated/' > .cache/measure/cov-report-outside_in.txt 2>&1; tail -4 .cache/measure/cov-report-outside_in.txt
error: failed to load coverage: 'target/debug/biomcp, target/debug/biomcp-cli, target/debug/deps/adverse_event_route_contract-c18845c880ccecac, target/debug/deps/article_asset_terminal-f1c09987b8e8fe7f, target/debug/deps/article_batch_outcome-6ec6e0977301fc93, target/debug/deps/article_cli_tests_structure-4643c50bf6cb0165, target/debug/deps/article_transform_structure-fc50e09e5568a69d, target/debug/deps/article_usage_stderr-cb74f257755a4dca, target/debug/deps/benchmark_cli_structure-4c17176d3746474b, target/debug/deps/biomcp-60cefaac99ca9a53, target/debug/deps/biomcp-bcc0ea64f3178014, target/debug/deps/biomcp_cli-62d7f580ca0837e6, target/debug/deps/biomcp_cli-cf2396d15ea6e691, target/debug/deps/biomcp_cli-d712d7a36816cdc5, target/debug/deps/biomcp_cli_shim-5f4cae94a5254436, target/debug/deps/cache_clear-692b0478bfeb8ac6, target/debug/deps/cli_global_modifiers-d091df9de00abcc0, target/debug/deps/cli_line_cap_absorption-ddcd76b66a15397a, target/debug/deps/health_cli_structure-1c2645d6e8f7692d, target/debug/deps/json_error_contract-42a560f2bd561c6b, target/debug/deps/legacy_cache_helper_cleanup-a7536cfaeeb67189, target/debug/deps/list_cli_structure-88f037cb242b7ab9, target/debug/deps/live_smoke-0bcbd89b6c402c5b, target/debug/deps/managed_state_permissions-b34905a17ff0fb03, target/debug/deps/package_build_identity-4a311094889a43bd, target/debug/deps/pgx_get_paging-4cf15a4aa78c8095, target/debug/deps/plugin_manifest-2bdd09abbe8ff654, target/debug/deps/pty_helpers-6dac2b6de696dfb4, target/debug/deps/rmcp_client_contract-56f2d888ad060f88, target/debug/deps/search_all_cli_structure-32c85e0a06fb1a26, target/debug/deps/skill_cli_structure-4b8712ef7740fdb9, target/debug/deps/tls_ca_bundle_contract-208b130f840f59fa, target/debug/deps/windows_stdio_contract-4bcfa43d52db07d1, target/debug/examples/rmcp_streamable_http_contract, target/debug/examples/rmcp_streamable_http_contract-51c3aa4bee5bed8e': no coverage data found
error: failed to generate report: process didn't exit successfully: `~/.rustup/toolchains/1.93.1-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov report -use-color=0 -instr-profile=~/2069-baseline/pass2/target/pass2.profdata -object target/debug/biomcp -object target/debug/biomcp-cli -object target/debug/deps/adverse_event_route_contract-c18845c880ccecac -object target/debug/deps/article_asset_terminal-f1c09987b8e8fe7f -object target/debug/deps/article_batch_outcome-6ec6e0977301fc93 -object target/debug/deps/article_cli_tests_structure-4643c50bf6cb0165 -object target/debug/deps/article_transform_structure-fc50e09e5568a69d -object target/debug/deps/article_usage_stderr-cb74f257755a4dca -object target/debug/deps/benchmark_cli_structure-4c17176d3746474b -object target/debug/deps/biomcp-60cefaac99ca9a53 -object target/debug/deps/biomcp-bcc0ea64f3178014 -object target/debug/deps/biomcp_cli-62d7f580ca0837e6 -object target/debug/deps/biomcp_cli-cf2396d15ea6e691 -object target/debug/deps/biomcp_cli-d712d7a36816cdc5 -object target/debug/deps/biomcp_cli_shim-5f4cae94a5254436 -object target/debug/deps/cache_clear-692b0478bfeb8ac6 -object target/debug/deps/cli_global_modifiers-d091df9de00abcc0 -object target/debug/deps/cli_line_cap_absorption-ddcd76b66a15397a -object target/debug/deps/health_cli_structure-1c2645d6e8f7692d -object target/debug/deps/json_error_contract-42a560f2bd561c6b -object target/debug/deps/legacy_cache_helper_cleanup-a7536cfaeeb67189 -object target/debug/deps/list_cli_structure-88f037cb242b7ab9 -object target/debug/deps/live_smoke-0bcbd89b6c402c5b -object target/debug/deps/managed_state_permissions-b34905a17ff0fb03 -object target/debug/deps/package_build_identity-4a311094889a43bd -object target/debug/deps/pgx_get_paging-4cf15a4aa78c8095 -object target/debug/deps/plugin_manifest-2bdd09abbe8ff654 -object target/debug/deps/pty_helpers-6dac2b6de696dfb4 -object target/debug/deps/rmcp_client_contract-56f2d888ad060f88 -object target/debug/deps/search_all_cli_structure-32c85e0a06fb1a26 -object target/debug/deps/skill_cli_structure-4b8712ef7740fdb9 -object target/debug/deps/tls_ca_bundle_contract-208b130f840f59fa -object target/debug/deps/windows_stdio_contract-4bcfa43d52db07d1 -object target/debug/examples/rmcp_streamable_http_contract -object target/debug/examples/rmcp_streamable_http_contract-51c3aa4bee5bed8e -ignore-filename-regex '(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)$|test_support|(^|/)generated/|/rustc/([0-9a-f]+|[0-9]+\.[0-9]+\.[0-9]+)/|^~/2069\-baseline/pass2(/.*)?/(tests|examples|benches)/|^~/2069\-baseline/pass2(/.*)?/(tests\.rs|[0-9a-zA-Z_-]+[_-]tests\.rs)$|^~/2069\-baseline/pass2/target($|/)|^~/\.cargo/(registry|git)/|^~/\.rustup/toolchains($|/)'` (exit status: 1)
RESULT coverage_outside_in_lines_pct = parse-failed
$ mv .cache/measure/cov-hold/*.profraw target/llvm-cov-target/ (restore)

### coverage — report: unit
$ mv target/llvm-cov-target/out-*.profraw .cache/measure/cov-hold/ (report the other set)
$ env -u CARGO_TARGET_DIR cargo llvm-cov report --ignore-filename-regex '(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)$|test_support|(^|/)generated/' > .cache/measure/cov-report-unit.txt 2>&1; tail -4 .cache/measure/cov-report-unit.txt
error: failed to load coverage: 'target/debug/biomcp, target/debug/biomcp-cli, target/debug/deps/adverse_event_route_contract-c18845c880ccecac, target/debug/deps/article_asset_terminal-f1c09987b8e8fe7f, target/debug/deps/article_batch_outcome-6ec6e0977301fc93, target/debug/deps/article_cli_tests_structure-4643c50bf6cb0165, target/debug/deps/article_transform_structure-fc50e09e5568a69d, target/debug/deps/article_usage_stderr-cb74f257755a4dca, target/debug/deps/benchmark_cli_structure-4c17176d3746474b, target/debug/deps/biomcp-60cefaac99ca9a53, target/debug/deps/biomcp-bcc0ea64f3178014, target/debug/deps/biomcp_cli-62d7f580ca0837e6, target/debug/deps/biomcp_cli-cf2396d15ea6e691, target/debug/deps/biomcp_cli-d712d7a36816cdc5, target/debug/deps/biomcp_cli_shim-5f4cae94a5254436, target/debug/deps/cache_clear-692b0478bfeb8ac6, target/debug/deps/cli_global_modifiers-d091df9de00abcc0, target/debug/deps/cli_line_cap_absorption-ddcd76b66a15397a, target/debug/deps/health_cli_structure-1c2645d6e8f7692d, target/debug/deps/json_error_contract-42a560f2bd561c6b, target/debug/deps/legacy_cache_helper_cleanup-a7536cfaeeb67189, target/debug/deps/list_cli_structure-88f037cb242b7ab9, target/debug/deps/live_smoke-0bcbd89b6c402c5b, target/debug/deps/managed_state_permissions-b34905a17ff0fb03, target/debug/deps/package_build_identity-4a311094889a43bd, target/debug/deps/pgx_get_paging-4cf15a4aa78c8095, target/debug/deps/plugin_manifest-2bdd09abbe8ff654, target/debug/deps/pty_helpers-6dac2b6de696dfb4, target/debug/deps/rmcp_client_contract-56f2d888ad060f88, target/debug/deps/search_all_cli_structure-32c85e0a06fb1a26, target/debug/deps/skill_cli_structure-4b8712ef7740fdb9, target/debug/deps/tls_ca_bundle_contract-208b130f840f59fa, target/debug/deps/windows_stdio_contract-4bcfa43d52db07d1, target/debug/examples/rmcp_streamable_http_contract, target/debug/examples/rmcp_streamable_http_contract-51c3aa4bee5bed8e': no coverage data found
error: failed to generate report: process didn't exit successfully: `~/.rustup/toolchains/1.93.1-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov report -use-color=0 -instr-profile=~/2069-baseline/pass2/target/pass2.profdata -object target/debug/biomcp -object target/debug/biomcp-cli -object target/debug/deps/adverse_event_route_contract-c18845c880ccecac -object target/debug/deps/article_asset_terminal-f1c09987b8e8fe7f -object target/debug/deps/article_batch_outcome-6ec6e0977301fc93 -object target/debug/deps/article_cli_tests_structure-4643c50bf6cb0165 -object target/debug/deps/article_transform_structure-fc50e09e5568a69d -object target/debug/deps/article_usage_stderr-cb74f257755a4dca -object target/debug/deps/benchmark_cli_structure-4c17176d3746474b -object target/debug/deps/biomcp-60cefaac99ca9a53 -object target/debug/deps/biomcp-bcc0ea64f3178014 -object target/debug/deps/biomcp_cli-62d7f580ca0837e6 -object target/debug/deps/biomcp_cli-cf2396d15ea6e691 -object target/debug/deps/biomcp_cli-d712d7a36816cdc5 -object target/debug/deps/biomcp_cli_shim-5f4cae94a5254436 -object target/debug/deps/cache_clear-692b0478bfeb8ac6 -object target/debug/deps/cli_global_modifiers-d091df9de00abcc0 -object target/debug/deps/cli_line_cap_absorption-ddcd76b66a15397a -object target/debug/deps/health_cli_structure-1c2645d6e8f7692d -object target/debug/deps/json_error_contract-42a560f2bd561c6b -object target/debug/deps/legacy_cache_helper_cleanup-a7536cfaeeb67189 -object target/debug/deps/list_cli_structure-88f037cb242b7ab9 -object target/debug/deps/live_smoke-0bcbd89b6c402c5b -object target/debug/deps/managed_state_permissions-b34905a17ff0fb03 -object target/debug/deps/package_build_identity-4a311094889a43bd -object target/debug/deps/pgx_get_paging-4cf15a4aa78c8095 -object target/debug/deps/plugin_manifest-2bdd09abbe8ff654 -object target/debug/deps/pty_helpers-6dac2b6de696dfb4 -object target/debug/deps/rmcp_client_contract-56f2d888ad060f88 -object target/debug/deps/search_all_cli_structure-32c85e0a06fb1a26 -object target/debug/deps/skill_cli_structure-4b8712ef7740fdb9 -object target/debug/deps/tls_ca_bundle_contract-208b130f840f59fa -object target/debug/deps/windows_stdio_contract-4bcfa43d52db07d1 -object target/debug/examples/rmcp_streamable_http_contract -object target/debug/examples/rmcp_streamable_http_contract-51c3aa4bee5bed8e -ignore-filename-regex '(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)$|test_support|(^|/)generated/|/rustc/([0-9a-f]+|[0-9]+\.[0-9]+\.[0-9]+)/|^~/2069\-baseline/pass2(/.*)?/(tests|examples|benches)/|^~/2069\-baseline/pass2(/.*)?/(tests\.rs|[0-9a-zA-Z_-]+[_-]tests\.rs)$|^~/2069\-baseline/pass2/target($|/)|^~/\.cargo/(registry|git)/|^~/\.rustup/toolchains($|/)'` (exit status: 1)
RESULT coverage_unit_lines_pct = parse-failed
$ mv .cache/measure/cov-hold/*.profraw target/llvm-cov-target/ (restore)

### coverage — report: combined
$ mv .cache/measure/cov-hold/*.profraw target/llvm-cov-target/ (report everything)
$ env -u CARGO_TARGET_DIR cargo llvm-cov report --ignore-filename-regex '(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)$|test_support|(^|/)generated/' > .cache/measure/cov-report-combined.txt 2>&1; tail -4 .cache/measure/cov-report-combined.txt
error: failed to load coverage: 'target/debug/biomcp, target/debug/biomcp-cli, target/debug/deps/adverse_event_route_contract-c18845c880ccecac, target/debug/deps/article_asset_terminal-f1c09987b8e8fe7f, target/debug/deps/article_batch_outcome-6ec6e0977301fc93, target/debug/deps/article_cli_tests_structure-4643c50bf6cb0165, target/debug/deps/article_transform_structure-fc50e09e5568a69d, target/debug/deps/article_usage_stderr-cb74f257755a4dca, target/debug/deps/benchmark_cli_structure-4c17176d3746474b, target/debug/deps/biomcp-60cefaac99ca9a53, target/debug/deps/biomcp-bcc0ea64f3178014, target/debug/deps/biomcp_cli-62d7f580ca0837e6, target/debug/deps/biomcp_cli-cf2396d15ea6e691, target/debug/deps/biomcp_cli-d712d7a36816cdc5, target/debug/deps/biomcp_cli_shim-5f4cae94a5254436, target/debug/deps/cache_clear-692b0478bfeb8ac6, target/debug/deps/cli_global_modifiers-d091df9de00abcc0, target/debug/deps/cli_line_cap_absorption-ddcd76b66a15397a, target/debug/deps/health_cli_structure-1c2645d6e8f7692d, target/debug/deps/json_error_contract-42a560f2bd561c6b, target/debug/deps/legacy_cache_helper_cleanup-a7536cfaeeb67189, target/debug/deps/list_cli_structure-88f037cb242b7ab9, target/debug/deps/live_smoke-0bcbd89b6c402c5b, target/debug/deps/managed_state_permissions-b34905a17ff0fb03, target/debug/deps/package_build_identity-4a311094889a43bd, target/debug/deps/pgx_get_paging-4cf15a4aa78c8095, target/debug/deps/plugin_manifest-2bdd09abbe8ff654, target/debug/deps/pty_helpers-6dac2b6de696dfb4, target/debug/deps/rmcp_client_contract-56f2d888ad060f88, target/debug/deps/search_all_cli_structure-32c85e0a06fb1a26, target/debug/deps/skill_cli_structure-4b8712ef7740fdb9, target/debug/deps/tls_ca_bundle_contract-208b130f840f59fa, target/debug/deps/windows_stdio_contract-4bcfa43d52db07d1, target/debug/examples/rmcp_streamable_http_contract, target/debug/examples/rmcp_streamable_http_contract-51c3aa4bee5bed8e': no coverage data found
error: failed to generate report: process didn't exit successfully: `~/.rustup/toolchains/1.93.1-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov report -use-color=0 -instr-profile=~/2069-baseline/pass2/target/pass2.profdata -object target/debug/biomcp -object target/debug/biomcp-cli -object target/debug/deps/adverse_event_route_contract-c18845c880ccecac -object target/debug/deps/article_asset_terminal-f1c09987b8e8fe7f -object target/debug/deps/article_batch_outcome-6ec6e0977301fc93 -object target/debug/deps/article_cli_tests_structure-4643c50bf6cb0165 -object target/debug/deps/article_transform_structure-fc50e09e5568a69d -object target/debug/deps/article_usage_stderr-cb74f257755a4dca -object target/debug/deps/benchmark_cli_structure-4c17176d3746474b -object target/debug/deps/biomcp-60cefaac99ca9a53 -object target/debug/deps/biomcp-bcc0ea64f3178014 -object target/debug/deps/biomcp_cli-62d7f580ca0837e6 -object target/debug/deps/biomcp_cli-cf2396d15ea6e691 -object target/debug/deps/biomcp_cli-d712d7a36816cdc5 -object target/debug/deps/biomcp_cli_shim-5f4cae94a5254436 -object target/debug/deps/cache_clear-692b0478bfeb8ac6 -object target/debug/deps/cli_global_modifiers-d091df9de00abcc0 -object target/debug/deps/cli_line_cap_absorption-ddcd76b66a15397a -object target/debug/deps/health_cli_structure-1c2645d6e8f7692d -object target/debug/deps/json_error_contract-42a560f2bd561c6b -object target/debug/deps/legacy_cache_helper_cleanup-a7536cfaeeb67189 -object target/debug/deps/list_cli_structure-88f037cb242b7ab9 -object target/debug/deps/live_smoke-0bcbd89b6c402c5b -object target/debug/deps/managed_state_permissions-b34905a17ff0fb03 -object target/debug/deps/package_build_identity-4a311094889a43bd -object target/debug/deps/pgx_get_paging-4cf15a4aa78c8095 -object target/debug/deps/plugin_manifest-2bdd09abbe8ff654 -object target/debug/deps/pty_helpers-6dac2b6de696dfb4 -object target/debug/deps/rmcp_client_contract-56f2d888ad060f88 -object target/debug/deps/search_all_cli_structure-32c85e0a06fb1a26 -object target/debug/deps/skill_cli_structure-4b8712ef7740fdb9 -object target/debug/deps/tls_ca_bundle_contract-208b130f840f59fa -object target/debug/deps/windows_stdio_contract-4bcfa43d52db07d1 -object target/debug/examples/rmcp_streamable_http_contract -object target/debug/examples/rmcp_streamable_http_contract-51c3aa4bee5bed8e -ignore-filename-regex '(^|/)(tests?|examples?|benches)/|(^|/)(tests?\.rs|_tests?\.rs|tests?_[^/]*\.rs)$|test_support|(^|/)generated/|/rustc/([0-9a-f]+|[0-9]+\.[0-9]+\.[0-9]+)/|^~/2069\-baseline/pass2(/.*)?/(tests|examples|benches)/|^~/2069\-baseline/pass2(/.*)?/(tests\.rs|[0-9a-zA-Z_-]+[_-]tests\.rs)$|^~/2069\-baseline/pass2/target($|/)|^~/\.cargo/(registry|git)/|^~/\.rustup/toolchains($|/)'` (exit status: 1)
RESULT coverage_combined_lines_pct = parse-failed

### runtime — run-time measurements with the release binary (thin LTO, one codegen unit)
$ { find ~/.cache/biomcp/http -type f 2>/dev/null || true; } | wc -l; du -sh ~/.cache/biomcp 2>/dev/null || echo "no host cache"
19903
198M	~/.cache/biomcp
$ python3 (five timed runs per case; median, min, max in seconds)
== biomcp --version start-up, empty cache dir ==
version_empty_cache: runs 0.0056 0.0039 0.0038 0.0039 0.0040 median 0.0039s spread 0.0018s exit [0]
RESULT runtime_version_empty_cache_median_s = 0.0039
RESULT runtime_version_empty_cache_spread_s = 0.0018
== biomcp --version start-up, the host's large cache ==
version_large_cache: runs 0.0039 0.0038 0.0039 0.0040 0.0040 median 0.0039s spread 0.0001s exit [0]
RESULT runtime_version_large_cache_median_s = 0.0039
RESULT runtime_version_large_cache_spread_s = 0.0001
== prefetch (one network run per command into the warm cache dir) ==
prefetch get_gene_BRAF: 0.79s exit 0
RESULT runtime_prefetch_get_gene_BRAF_s = 0.79
prefetch get_variant_BRAF_V600E: 0.55s exit 0
RESULT runtime_prefetch_get_variant_BRAF_V600E_s = 0.55
prefetch get_drug_imatinib: 3.32s exit 0
RESULT runtime_prefetch_get_drug_imatinib_s = 3.32
prefetch search_article_BRAF: 1.79s exit 0
RESULT runtime_prefetch_search_article_BRAF_s = 1.79
prefetch search_all_BRAF_melanoma: 10.18s exit 0
RESULT runtime_prefetch_search_all_BRAF_melanoma_s = 10.18
== output size of each command from the warm cache (one run, bytes) ==
RESULT runtime_output_bytes_get_gene_BRAF = 1646 exit 0
RESULT runtime_output_bytes_get_variant_BRAF_V600E = 1371 exit 0
RESULT runtime_output_bytes_get_drug_imatinib = 2410 exit 0
RESULT runtime_output_bytes_search_article_BRAF = 2861 exit 0
RESULT runtime_output_bytes_search_all_BRAF_melanoma = 2015 exit 0
== warm cache (BIOMCP_CACHE_MODE=infinite), five runs each ==
warm_get_gene_BRAF: runs 0.20 0.19 0.20 0.19 0.19 median 0.19s spread 0.02s exit [0]
RESULT runtime_warm_get_gene_BRAF_median_s = 0.19
RESULT runtime_warm_get_gene_BRAF_spread_s = 0.02
warm_get_variant_BRAF_V600E: runs 0.02 0.02 0.02 0.02 0.02 median 0.02s spread 0.00s exit [0]
RESULT runtime_warm_get_variant_BRAF_V600E_median_s = 0.02
RESULT runtime_warm_get_variant_BRAF_V600E_spread_s = 0.00
warm_get_drug_imatinib: runs 1.61 1.66 1.48 3.31 1.27 median 1.61s spread 2.04s exit [0]
RESULT runtime_warm_get_drug_imatinib_median_s = 1.61
RESULT runtime_warm_get_drug_imatinib_spread_s = 2.04
warm_search_article_BRAF: runs 1.71 1.71 1.28 1.75 1.77 median 1.71s spread 0.49s exit [0]
RESULT runtime_warm_search_article_BRAF_median_s = 1.71
RESULT runtime_warm_search_article_BRAF_spread_s = 0.49
warm_search_all_BRAF_melanoma: runs 9.77 12.02 12.02 10.71 11.35 median 11.35s spread 2.25s exit [0]
RESULT runtime_warm_search_all_BRAF_melanoma_median_s = 11.35
RESULT runtime_warm_search_all_BRAF_melanoma_spread_s = 2.25
== cache off (BIOMCP_CACHE_MODE=off, network), five runs each ==
nocache_get_gene_BRAF: runs 0.88 0.47 0.46 0.60 0.47 median 0.47s spread 0.42s exit [0]
RESULT runtime_nocache_get_gene_BRAF_median_s = 0.47
RESULT runtime_nocache_get_gene_BRAF_spread_s = 0.42
nocache_get_variant_BRAF_V600E: runs 0.60 0.49 0.52 0.52 0.51 median 0.52s spread 0.10s exit [0]
RESULT runtime_nocache_get_variant_BRAF_V600E_median_s = 0.52
RESULT runtime_nocache_get_variant_BRAF_V600E_spread_s = 0.10
nocache_get_drug_imatinib: runs 4.03 6.50 5.87 4.85 2.89 median 4.85s spread 3.61s exit [0]
RESULT runtime_nocache_get_drug_imatinib_median_s = 4.85
RESULT runtime_nocache_get_drug_imatinib_spread_s = 3.61
nocache_search_article_BRAF: runs 1.78 2.70 1.77 1.90 2.77 median 1.90s spread 1.00s exit [0]
RESULT runtime_nocache_search_article_BRAF_median_s = 1.90
RESULT runtime_nocache_search_article_BRAF_spread_s = 1.00
nocache_search_all_BRAF_melanoma: runs 10.85 9.44 10.45 10.82 9.83 median 10.45s spread 1.41s exit [0]
RESULT runtime_nocache_search_all_BRAF_melanoma_median_s = 10.45
RESULT runtime_nocache_search_all_BRAF_melanoma_spread_s = 1.41

### host-after — machine conditions at the end
$ cat /proc/loadavg
0.04 0.36 0.97 1/503 2666622
$ free -g | sed -n 1,2p
               total        used        free      shared  buff/cache   available
Mem:              14           1           5           0           9          13
$ df -h . | tail -1
/dev/sda1       916G  448G  422G  52% /

### done — measurement complete

### coverage — reports (rerun without the show-env exports; the script env breaks report discovery)
$ mv target/llvm-cov-target/unit-*.profraw .cache/measure/cov-hold/ (report the outside-in set)
TOTAL                                                201832             97488    51.70%       14853              6263    57.83%      142085             67277    52.65%           0                 0         -
$ mv target/llvm-cov-target/out-*.profraw .cache/measure/cov-hold/ (report the unit set)
TOTAL                                                201832             47155    76.64%       14853              3223    78.30%      142085             30835    78.30%           0                 0         -
$ (all restored; report everything)
TOTAL                                                201832             30736    84.77%       14853              2042    86.25%      142085             19510    86.27%           0                 0         -
RESULT coverage_outside_in_lines_pct = 52.65
RESULT coverage_unit_lines_pct = 78.30
RESULT coverage_combined_lines_pct = 86.27

### locks — held for the whole run
$ flock -n ~/.yr-checkout.lock && flock -n ~/.<build-host>-gate.lock
both locks held by pid 2667277

### doc-edit-rebuild — rebuild after one doc edit (a spec page touch; v0.9.2 build.rs scans the whole package)
$ touch spec/entity/gene.md
$ /usr/bin/time -f "TIMEFMT WALL %e USER %U SYS %S MAXRSS_KB %M" cargo build --locked --no-default-features --all-targets
   Compiling biomcp-cli v0.9.2 (~/2069-baseline/pass2)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 48s
TIMEFMT WALL 168.45 USER 428.62 SYS 19.86 MAXRSS_KB 6226980
RESULT doc_edit_rebuild_wall_s = 168.5

### done — measurement complete

### runtime — get drug imatinib rerun (one repeat of the widest-spread command)
rerun_warm_get_drug_imatinib: runs 2.75 2.06 1.39 1.27 1.30 median 1.39s spread 1.48s exit [0]
RESULT runtime_rerun_warm_get_drug_imatinib_median_s = 1.39
RESULT runtime_rerun_warm_get_drug_imatinib_spread_s = 1.48
rerun_nocache_get_drug_imatinib: runs 7.46 8.68 3.00 6.62 2.37 median 6.62s spread 6.31s exit [0]
RESULT runtime_rerun_nocache_get_drug_imatinib_median_s = 6.62
RESULT runtime_rerun_nocache_get_drug_imatinib_spread_s = 6.31
```

Ticket 2069 stays OPEN; this record is the baseline 2082 measures against.
