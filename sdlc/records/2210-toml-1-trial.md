# TOML 1 trial

## Decision

Keep toml 0.8. The toml 1 candidate passed the configuration cases and removed one compiled crate, but its measured fresh-build and Rust-touch medians were higher. The trial did not meet the requirement that compile time and dependency counts be no worse. Shared host load prevents a causal claim that TOML caused the slowdown. No additional samples were collected, and all candidate manifest, lockfile and test changes were reverted. No changelog change was made.

## Products and method

- OLD: `dd365f45a3ed10385ab5d5c5c603b61262b2d624`.
- CANDIDATE: temporary product commit `5e211596f7ca759724b76e7a557babf5926856ae`, containing only the manifest and lockfile change from OLD.
- Candidate declaration: `toml = { version = "1", default-features = false, features = ["parse", "serde", "std"] }`, resolving to `1.1.8+spec-1.1.0`. The existing `toml::from_str` reader needed no API change.
- Existing measurement helper: `tools/measure-0-9-3-baseline`, frozen from `9593a032b534b6252b1133c657c1088f7c5e908f`, SHA-256 `98ed6f7128e4a2122c0ff69d33b142327c911c1c85fa1d81f9de87ccb9dc8c6b`.
- One configured arm64 build host: 18 logical CPUs, Rust/Cargo 1.95.0, sccache 0.17.0 and Python 3.12.13. Both products used the same host linker and development settings: line-table debug information, no dependency debug information and incremental compilation disabled. `CARGO_INCREMENTAL` and `CARGO_TARGET_DIR` were unset.
- Three paired runs used OLD/CANDIDATE, CANDIDATE/OLD, OLD/CANDIDATE order. Each immutable product clone started each arm with an empty target directory, while the populated compiler cache remained enabled and shared. The helper ran `cold-dev-build`, then `rust-edit-rebuild`; the latter touched `src/render/markdown/gene.rs` without changing its bytes. Tracked-status checks were empty after each arm.
- Other work could run throughout. The table records one-minute load before and after each section. Cache counters include other work and cannot attribute hits to these samples. Queue wait and preparation were outside the measured sections. No quiet-host wait was used.

The fresh command was `tools/with-build-identity cargo build --locked --no-default-features --all-targets --timings`. The Rust-touch command omitted `--timings`. The existing helper recorded wall time and the platform timing utility's CPU and peak resident memory results.

## Configuration and dependency results

The captured OLD binary passed 26 CLI/MCP configuration cases in job `2210/1792359872583-75541`. The candidate binary passed the identical 26 cases during job `2210/1792360201667-69093`. Cases covered every field in the sole settings file, `cache.toml`, environment precedence, missing/empty/comment-only files, malformed syntax, unknown and duplicate keys, invalid types and values, existing CLI/MCP invalid-input behavior, and the existing unreadable-file fallback.

An initial baseline attempt, job `2210/1792359751252-38574`, passed 25 cases and exposed an incorrect test setup: the fixture's cache-directory environment override bypassed validation of the blank directory in the file. Clearing that override corrected the rejection case; the separate environment-precedence control remained. No product change was needed.

The frozen helper collected the OLD graph in job `2210/1792359872583-75541` and the candidate graph in job `2210/1792360136287-51949`. An offline-first candidate graph request required uncached crates; Cargo fetching completed the preflight without compiling. Direct dependency counts remained unchanged.

| Count | OLD | CANDIDATE |
| --- | ---: | ---: |
| Normal/build crates | 333 | 332 |
| Including development crates | 342 | 341 |
| Normal/build duplicate names | 13 | 13 |
| Including development duplicate names | 14 | 14 |
| Lockfile packages | 475 | 475 |
| Lockfile duplicate names | 33 | 33 |
| Lockfile extra versions | 47 | 47 |
| All-target Cargo timing units | 465 | 464 |

## Raw build samples

All twelve measured commands passed in job `2210/1792360201667-69093`, whose total wall time was 590.219 seconds. Wall time below is the helper's reported seconds. User/system CPU totals include concurrent compiler children. Peak resident memory is in bytes; load is the one-minute average.

| Pair | Product | Scenario | Wall s | User s | System s | Peak resident bytes | Load before | Load after |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | OLD | Fresh | 52.5 | 99.92 | 11.48 | 7145586688 | 5.04 | 5.38 |
| 1 | OLD | Rust touch | 37.8 | 80.33 | 5.39 | 7126712320 | 5.38 | 4.47 |
| 1 | CANDIDATE | Fresh | 51.6 | 96.91 | 11.51 | 6577176576 | 4.47 | 4.39 |
| 1 | CANDIDATE | Rust touch | 39.1 | 82.14 | 5.82 | 6601490432 | 4.39 | 3.76 |
| 2 | CANDIDATE | Fresh | 55.0 | 105.87 | 12.79 | 6579060736 | 4.18 | 7.98 |
| 2 | CANDIDATE | Rust touch | 48.7 | 92.32 | 6.02 | 6557270016 | 7.98 | 10.32 |
| 2 | OLD | Fresh | 51.2 | 103.45 | 12.24 | 7111065600 | 10.32 | 10.93 |
| 2 | OLD | Rust touch | 42.4 | 86.59 | 6.35 | 7134380032 | 10.93 | 11.14 |
| 3 | OLD | Fresh | 52.3 | 104.31 | 12.88 | 7128858624 | 10.41 | 8.33 |
| 3 | OLD | Rust touch | 42.7 | 88.02 | 6.99 | 7127662592 | 8.33 | 7.99 |
| 3 | CANDIDATE | Fresh | 58.6 | 112.20 | 12.91 | 6544834560 | 7.99 | 12.18 |
| 3 | CANDIDATE | Rust touch | 50.7 | 95.64 | 7.02 | 6554959872 | 12.18 | 9.98 |

| Scenario | OLD median (range), s | CANDIDATE median (range), s |
| --- | ---: | ---: |
| Fresh development build | 52.3 (51.2–52.5) | 55.0 (51.6–58.6) |
| Rust-touch rebuild | 42.4 (37.8–42.7) | 48.7 (39.1–50.7) |

The smaller graph and lower observed candidate peak memory did not establish no-worse compile time. These shared-load observations support nonadoption under this trial's criterion; they do not establish a TOML performance regression or a quiet-host result.

## Artifact identities

SHA-256 values:

- Candidate source patch: `039ce589e4eb45aa83d0192b1a4ec9ba032ba8dbbe7fddbf5315334119f85699`.
- Candidate lockfile: `0118b7ca3b08311de07ee04a162383c606cc183777461ffe388a4b15d588c64f`.
- Identical compatibility test payload: `384d47be7e9b244457b6f549edac1aa5f0baf5ff7c3f4c8a622c3551c7d0b298`.
- Captured OLD binary: `1bdc92ee5b1a1a29c9fa7122dcec9a100ed648dcdf9e8e27fa61ec550deb646c`, banner revision `dd365f45`.
- Captured candidate binary: `7c93fde654653f6a02ca9bde57a1920535a6a7c72b2919a16adc7ecc42f3d7f0`, banner revision `5e211596`.

The owned ticket build checkout retained raw section logs, full load vectors, cache snapshots, Cargo timing reports, the patch and test payload under `.cache/2210-trial/cycle`; the captured OLD binary was retained under `.cache/2210-baseline`. These disposable artifacts accompany the proof job identities through review.
