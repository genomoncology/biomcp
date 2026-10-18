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
