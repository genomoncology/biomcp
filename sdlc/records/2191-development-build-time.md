# Development build time after 0.9.3

## Decision

Keep the existing development profile and build graph. The three paired measurements do not reproduce the earlier large development-build regression. Current fresh-build median is 54.3 seconds against 55.8 seconds for v0.9.2. Rust-touch median is 39.4 seconds against 39.0 seconds. Shared load and three samples do not establish a causal speed improvement or a meaningful 0.4-second regression.

The dominant retained cost is compiling the library test executable, which runs alongside the production library and finishes last in all six fresh builds. It occupies 72.9–83.7% of Cargo wall time. These measurements do not identify a redundant behavior or a specific avoidable compiler cost. Keep those tests and product features; do not change profiles, dependencies or generic code on speculation. No source fix, new benchmark, LLVM collection or additional suite run was made for this finding.

## Inputs and method

- Previous release: `4049666f5cce2794f16cdbdf0ded49878e5bb147` (`v0.9.2`).
- Current product: `3dc15da27888454aac4a4e43b685f1ebabbbeff5`.
- Existing helper frozen from `9593a032b534b6252b1133c657c1088f7c5e908f`: `tools/measure-0-9-3-baseline`, SHA-256 `98ed6f7128e4a2122c0ff69d33b142327c911c1c85fa1d81f9de87ccb9dc8c6b`.
- Collection: build job `2191/1792352213116-26987`, all twelve measured commands passed; total job wall time 565.098 seconds.
- One shared build host: Apple arm64, 18 logical CPUs; Rust/Cargo 1.95.0, LLVM 22.1.2, Python 3.12.13 and Bash 5.3.0. These tools differ from the historical measurements, so their wall times are context, not an additional paired sample.
- Both products used the same Python environment, ordinary `sccache` 0.17.0 and platform linker. Host Cargo configuration disables incremental compilation, uses line-table debug information and removes dependency debug information. `CARGO_INCREMENTAL` and `CARGO_TARGET_DIR` were unset; no compiler/profile environment override was present. The old Python wrapper received a Python supporting `tomllib`; neither product source was patched.
- Pair order: old/current, current/old, old/current. Both immutable product clones started each arm with an empty `target/`; source and locked dependencies remained unchanged. The existing populated compiler cache stayed enabled and shared. Cache counters include other work and cannot attribute hits to these samples. The first production-library compilation in each arm was longer than its subsequent cached compilations.
- Other work could run throughout. One-minute host load before/after each section appears below; full load vectors, raw command output, cache snapshots and Cargo reports remain in the owned disposable experiment `2077-biomcp-development-build-cost` until review. No sample is described as idle. Queue wait and experiment preparation are outside the measured section wall times.

Each clone ran the frozen helper with `--only cold-dev-build`, then `--only rust-edit-rebuild`. The fresh command was:

```sh
tools/with-build-identity cargo build --locked --no-default-features --all-targets --timings
```

The rebuild touched `src/render/markdown/gene.rs` without changing its bytes, then ran:

```sh
tools/with-build-identity cargo build --locked --no-default-features --all-targets
```

The same commands intentionally compile each revision's own shipped targets and dependency graph. The existing helper wraps each command in the platform timing utility. All tracked-status checks were empty after each arm.

## Raw samples

Wall time is the helper's reported seconds; user/system CPU seconds and peak resident bytes come from the timing utility. CPU totals include concurrent compiler children and need not equal wall time. Load is the host's one-minute average.

| Pair | Product | Scenario | Wall s | User s | System s | Peak resident bytes | Load before | Load after |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | old | Fresh | 55.8 | 100.64 | 13.82 | 6299271168 | 5.90 | 14.17 |
| 1 | old | Rust touch | 37.4 | 78.04 | 5.04 | 6335004672 | 14.17 | 10.39 |
| 1 | current | Fresh | 54.3 | 102.50 | 12.24 | 6620692480 | 10.39 | 14.04 |
| 1 | current | Rust touch | 39.4 | 82.74 | 5.59 | 6640697344 | 14.04 | 10.74 |
| 2 | current | Fresh | 46.6 | 98.07 | 11.83 | 6612893696 | 10.74 | 7.62 |
| 2 | current | Rust touch | 39.0 | 82.37 | 5.42 | 6636699648 | 7.62 | 6.04 |
| 2 | old | Fresh | 45.2 | 91.96 | 12.44 | 6339870720 | 6.04 | 5.90 |
| 2 | old | Rust touch | 39.0 | 80.75 | 5.58 | 6282821632 | 5.90 | 6.87 |
| 3 | old | Fresh | 55.9 | 105.93 | 13.66 | 6251069440 | 6.87 | 8.11 |
| 3 | old | Rust touch | 46.9 | 89.58 | 5.97 | 6270746624 | 8.11 | 8.88 |
| 3 | current | Fresh | 55.2 | 110.73 | 12.83 | 6571769856 | 8.88 | 9.41 |
| 3 | current | Rust touch | 46.1 | 93.73 | 6.01 | 6594134016 | 9.41 | 8.91 |

| Scenario | Old median (range), s | Current median (range), s |
| --- | ---: | ---: |
| Fresh development build | 55.8 (45.2–55.9) | 54.3 (46.6–55.2) |
| Rust-touch rebuild | 39.0 (37.4–46.9) | 39.4 (39.0–46.1) |

## Cargo timing attribution

Each fresh build retained Cargo's HTML report. The library test unit finishes last in each report. Its wall share below is its duration divided by Cargo's total build duration; overlapping units mean shares are not additive. Cargo's duration excludes the surrounding identity wrapper and helper timing overhead.

| Pair | Product | Cargo units | Cargo wall s | Library s | Library test s | Library test wall share |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1 | old | 494 | 55.5 | 25.14 | 40.44 | 72.9% |
| 1 | current | 465 | 53.9 | 24.61 | 41.37 | 76.8% |
| 2 | old | 494 | 45.1 | 1.73 | 35.79 | 79.4% |
| 2 | current | 465 | 46.5 | 1.75 | 38.91 | 83.7% |
| 3 | old | 494 | 55.8 | 1.72 | 43.83 | 78.5% |
| 3 | current | 465 | 55.1 | 2.03 | 46.14 | 83.7% |

The current graph has 465 Cargo units against 494 old units. First-pair production-library durations are comparable (24.61 versus 25.14 seconds); later production-library durations are 1.72–2.03 seconds under the shared compiler cache. The library test executable still takes 35.79–46.14 seconds across both revisions. This locates the persistent development cost in the existing library/test compilation rather than demonstrating a new dependency bottleneck.

Inspection separates three proposed explanations:

- The full-main development-profile selection changed in 2171 through Makefile recipes. It does not change this direct `cargo build` command. The explicit current Cargo development profile also matches the host's effective line-table/dependency-debug configuration already applied to the old product.
- The current manifests add workbook parsing for supplement search and replace the article extraction stack, while removing the wrapper executable, old archive codecs and other dependency units. Added crate names alone cannot explain a slowdown; the measured overall graph shrank and no new dependency owns the longest unit.
- Historical release-library IR counts grew from 6,897,914 lines/138,788 copies to roughly 7.20 million lines/143,558 copies. Those are optimized library measurements at their recorded product identities, not a profile of this all-target development workload. They do not prove a generic-code cause, so this ticket makes no such claim and does not repeat the collection.

## Qualification and limits

Exact current main already passed `make main-gate` in `2156/1792351422343-29859` (408.025 seconds), covering routine tests/specifications and the full main checks. Its accepted merged-product landing gate also passed in `2216/1792351080065-83519`. This record changes no product or gate input and reuses that qualification; it does not invent new isolated `make test` or `make spec` timing rows.

The experiment resolves the decision to retain the present build configuration. It does not prove statistical equality, estimate a quiet-host result, attribute global cache activity, or assign the historical slowdown to one code change. A future specific build-cost change needs its own observed bottleneck and comparable paired evidence.
