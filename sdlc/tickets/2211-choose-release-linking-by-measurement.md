---
priority: 3
---

# 2211 — Choose release linking by measurement

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 9885a40, accept

## Outcome

The release build profile's link settings are chosen from measured release build time and measured program speed, under the standing rule that program speed matters more than build speed.

- Evidence: the 0.9.3 dependency design (step 10) asked to measure codegen units and link-time optimization before deciding; 2116 only described the current profile.
- Starts from: the current release profile; compare it with at least one cheaper setting.
- Proof: release build time, binary size and the existing runtime cases for each setting on one build machine; keep the current profile unless another is no slower at runtime.

- Changes: `sdlc/records/2211-release-linking.md` and this ticket. Freeze the existing measurement helper; temporary profile candidates and build outputs stay outside tracked source. Report measurements and the chosen profile first; any adoption edit to `Cargo.toml` needs coordination with the concurrent cache migration before it begins.

## Measurement brief

Restated outcome: choose release linking from measured build cost and executable runtime, retaining current settings unless a cheaper candidate demonstrates no worse program speed. This is a measurement decision, not a reproduced product defect; no failing product test is appropriate.

Current main `08b74a340658f2c30e158fd00459330baebacd74` uses thin LTO, one codegen unit, unwind and stripping. Its full qualification `2156/1792409329667-86161` passed in 409.862 seconds but is not release-build timing. Earlier 2116 timing used an older revision and different host; 2191/2210 do not compare release profiles.

Compare only current settings against sixteen codegen units, with all other settings fixed. Freeze existing `tools/measure-0-9-3-baseline` at SHA256 `98ed6f7128e4a2122c0ff69d33b142327c911c1c85fa1d81f9de87ccb9dc8c6b`. Run three sequential alternating build pairs, empty per-arm target output and ordinary populated compiler cache, incremental unset. Existing commands are `bash <frozen-helper> <arm-clone> --only release-build,stripped-binary` and its timed `tools/with-build-identity cargo build --release --locked --timings`. Record wall/CPU/peak memory, binary size, toolchain/settings, load before/after and cache context. No quiet wait or additional candidates.

Use existing local output-footprint replay cases for runtime with fixed output checks, alternating the two frozen binaries over five pairs per case. Tokenization stays outside timings. Reuse existing live-provider observations as context; do not repeat provider calls to claim executable speed. A failed command, output mismatch, or inconclusive runtime prevents adoption. Stop after the bounded sample set. Only the report path is claimed; no tracked manifest/source/helper edit. Report any adoption recommendation before changing profile ownership.

## Progress

- 2026-10-19 started
