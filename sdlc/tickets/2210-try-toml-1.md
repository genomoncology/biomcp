---
priority: 3
---

# 2210 — Try toml 1.x

Status: OPEN.

Milestone: 0.9.4

## Outcome

Moving BioMCP to toml 1.x either lands with measured build and dependency numbers no worse than today's, or is recorded as not worth it.

- Evidence: the 0.9.3 dependency design listed a toml 1.x trial; 2072 left it out of 0.9.3 unless a measured ticket was filed.
- Changes: `Cargo.toml`, `Cargo.lock`, `tests/test_cache_invalid_settings.py`, `src/cache/config.rs` only if an actual API adaptation is necessary, `sdlc/records/2210-toml-1-trial.md`, `CHANGELOG.md` if adopted, and this ticket.
- Proof: configuration and cache settings parse identically (existing cases plus one for each setting file), and compile time, crate count and duplicate-version count before and after on one build machine.

## Progress

- 2026-10-18 started

## Trial brief

Restated outcome: adopt toml1 only if supported cache settings, invalid-input behavior and measured compilation/dependency cost remain acceptable; otherwise record nonadoption honestly. This is a compatibility and measured dependency trial, not a reproduced functional defect requiring an invented failing baseline.

Study found one Rust reader, toml::from_str in src/cache/config.rs, and one cache.toml with dir, max_size, min_disk_free and max_age_secs. Python tomllib readers are unaffected. Lockfile-only counts are475packages,33duplicate names and47extra versions; selected Cargo graph counts remain to be measured. Proposed parse-only dependency enables parse,serde,std with default features disabled. Do not change the reader unless compilation proves an adaptation necessary.

Use existing isolated CLI settings checks for complete supported settings, environment precedence, missing/empty/comment-only files, and invalid syntax/unknown fields/duplicates/types/zero sizes or ages/blank directory. Establish baseline behavior before the candidate and preserve useful invalid-input classification and file/field context. New TOML syntax acceptance alone is not a claim of universal parser equivalence.

Measure immutable OLDdd365f45a and CANDIDATE on one configured build host with the existing2191helper frozen at9593a032b534b6252b1133c657c1088f7c5e908f, SHA25698ed6f7128e4a2122c0ff69d33b142327c911c1c85fa1d81f9de87ccb9dc8c6b. Three alternating old/candidate pairs, each cold-dev-build then rust-edit-rebuild, give12measured commands. Empty per-arm targets, shared populated compiler cache, ordinary compiler/linker/profile, CARGO_INCREMENTAL/CARGO_TARGET_DIR unset, no quiet waits. Collect graph counts once per arm and retain raw timings/load/configuration until review. One sampling cycle; regression or inconclusive results justify nonadoption rather than a new sampling spiral.

## Captured baseline

Job2210/1792359872583-75541 on exactdd365f45a3ed10385ab5d5c5c603b61262b2d624 passed in3.723s:26compatibility cases passed in2.95s; compatibility exit0 and graph exit0. Immutable qualifiedbinarySHA2561bdc92ee5b1a1a29c9fa7122dcec9a100ed648dcdf9e8e27fa61ec550deb646c has banner0.9.4-dev+gdd365f45. Corrected test payloadSHA256384d47be7e9b244457b6f549edac1aa5f0baf5ff7c3f4c8a622c3551c7d0b298.

The frozen helper collected the graph without network: normal/build333crates and13duplicate names; including development342crates and14duplicate names. Complete lockfile475packages,33duplicate names,47extra versions. Direct normal59,development6,Windows1,Unix-development1; optional5. The first baseline attempt had25passes and one invalid test assumption: the fixture directory environment override legitimately bypassed blank file-dir validation. Clearing that override fixed the case and retained the explicit environment-precedence control. No production defect or dependency edit came from baseline setup.

Proceed with the bounded candidate. First collect its selected graph. If a required dependency-count criterion already clearly regresses, record nonadoption and skip needless timing builds. Otherwise run the identical compatibility cases and single approved alternating measurement cycle. Do not claim an unmeasured compile result. Regression or inconclusive evidence ends in a nonadoption record; no additional sampling cycle is authorized.
