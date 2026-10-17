# 2069 — The 0.9.3 baseline

## Final same-host comparison, 2026-10-17

This comparison supersedes every earlier before/after column below. Both immutable product revisions used the same reviewed harness, verified toolchain, workloads and cache policy on one shared macOS ARM64 build host. It is not a controlled quiet-host experiment. Other builds were observed; one-minute load ranged from 3.97 to 14.29. Build and gate scenarios have one completed sample per revision. Live timings include provider variability. Faster and slower observations are retained without causal speedup claims.

| Identity | Revision |
| --- | --- |
| Prior release v0.9.2 | 4049666f5cce2794f16cdbdf0ded49878e5bb147 |
| Repaired 0.9.3 product | 234d2b32d3b5068dc27b2543d676997b75ad5baa |
| Reviewed harness | dedbc1307e65c21c6f79bdb49025f209def074ac |
| Harness code component accepted | 5fb5523758e312716684a3fda4d8694cc8dbfa66 |

Formal collection `2082/1792269937662-71934`, source/export `2082/1792271866922-64213`, and integration-test supplement/export `2082/1792271923818-66165` all returned zero. Both final product identity checks passed. The original safe-export SHA-256 was `56a51d1c0af567e73ed83349c5c358010308bc29d65bf42064a1159bd124c760`. The durable raw measurements and per-file counts are embedded below; private provider payloads and machine paths are not committed. Relative raw-log names identify diagnostic artifacts, not additional durable evidence required to read these results.

Two aborted setup attempts (`1792269823964-62289` and `1792269897378-67494`) contribute no performance values. The first reached a system Python without `tomllib`; the second discovered different selected interpreter versions. Fresh product trees and explicit `UV_PYTHON=3.12.13`, frozen dependency resolution and each product virtual environment first on PATH corrected setup identically. Actual Python 3.12.13 and matching Rust/Cargo/uv identities were verified before the completed measurements. No measured product source was patched.

Each section invoked the same harness as `bash5 <harness>/tools/measure-0-9-3-baseline <product> --only <section>`. Exact sanitized argument lists, section statuses and loads are retained below. Ordinary compiler cache and linker settings remained enabled, with ordinary incremental settings unchanged. Qualified coverage scoped its instrumentation and restored the caller environment before the Python/specification consumers. The global compiler cache was shared, so its counters do not attribute hits to one version. Queue wait is outside the section command timings.

### Local measurements and approved targets

| Measurement | v0.9.2 | 0.9.3 candidate | Target/disposition |
| --- | --- | --- | --- |
| Fresh dev build, seconds | 51.5 | 72.8 | Observation; no numeric target |
| Rust-touch rebuild, seconds | 39.3 | 51.5 | Observation; no numeric target |
| Documentation-touch rebuild, seconds | 42.4 | 0.3 | Observation; no numeric target |
| Artifact preparation, seconds | 68.2 | 1.0 | Observation; no numeric target |
| make test, seconds | Unavailable: old Linux-only wrapper | 118.2 | Met: candidate <= 200 s |
| make spec, seconds | Unavailable: old /proc requirement | 24.4 | Met: candidate <= 200 s |
| make lint, seconds | Unavailable: old Darwin-arm64 refusal | 84.6 | Candidate passed; no numeric target |
| Release build, seconds | 260.3 | 256.8 | Observation; no numeric target |
| Release peak RSS, KiB | 4168064 | 4361024 | Observation; no numeric target |
| Stripped binary, bytes | 32775408 | 33882944 | Observation; no numeric target |
| Build folder after release, bytes | 4935225344 | 4240039936 | Observation; no numeric target |
| LLVM IR lines | 6897914 | 7197318 | Observation; no numeric target |
| LLVM function copies | 138788 | 143546 | Observation; no numeric target |
| Cargo.lock packages | 488 | 478 | Observation; no numeric target |
| Duplicate dependency names | 50 (extra copies: 68) | 35 (extra copies: 50) | Observation; no numeric target |

The recorded test/specification targets come from the October 14 suite decision; obsolete hosted-CI targets are not applied. Fresh-build and Rust-touch observations were slower for the candidate. Documentation-touch rebuild and several runtime observations were faster. No optimization work or passing-warm-cache requirement follows from these measurements. Native release peak RSS was 4,268,097,536 and 4,465,688,576 bytes respectively; divide by 1024 for the KiB table values. LLVM IR measures compiler output, not test-code share.

Old gate outcomes are platform refusals, not successful near-zero timings: `tools/run-offline` reports `offline routine-gate enforcement is unsupported on this platform`; specification preparation reads `/proc/$$/stat`; pinned lint bootstrap reports `No pinned lint-tool archive for Darwin-arm64`. The old Rust census and Python collection also cannot run through their unchanged wrappers. Old artifact preparation itself passed in 68.2 seconds; the subsequent specification command still failed. Old coverage was stopped at entry because of these known prerequisites: no suite ran, no coverage suite exit files existed, and zero profiles were produced. It is **not collected**, with no percentage, rather than a claimed failed coverage suite or a zero-percent result.

Candidate routine tests passed: 4,362 Rust cases, 34 skips; 1,253 Python contracts, nine skips; strict documentation passed. The separate Python collection inventory was 1,270, not the executed count. The one required Rust duration census passed with 4,362 durations, 75 over one second, and a slowest case of 17.0 seconds. A Python routine warning named `test_wrapper_writes_summary_artifacts_for_pass_fixture` at 32.190 seconds and load 10.69/10.84/8.84; it warned and passed under the approved policy.

### Test-source share

PM confirmed on October 17 that no numeric test-code-share target was ever set. The table compares physical Rust lines, including blanks/comments, in nongenerated root-crate `src/**/*.rs` plus tracked `tests/**/*.rs`. Test-only source files, inline `#[cfg(test)]` item spans and external integration-test lines form the numerator; overlapping spans are counted once. Generated directories are excluded. Workspace helper crates, examples, benches, root build scripts and other languages are outside this denominator. Python test lines are separate, not mixed into a Rust percentage.

| Source measure | v0.9.2 | 0.9.3 candidate | Verdict |
| --- | --- | --- | --- |
| Primary Rust test lines | 139914 | 149869 | No numeric target |
| Primary Rust denominator | 301538 | 314914 | Same declared method |
| Primary Rust test share | 46.4001% | 47.5905% | Comparison only; +1.1903 percentage points |
| Separate Python test physical lines | 37979 | 35606 | Comparison only |

The src-only figures remain in the raw census summaries and are not substituted for the primary share. Per-file counts, generated exclusions and exact totals are retained below.

### Coverage and collection completeness

Candidate coverage collection completed in 242.287 seconds. Integration 159 passed/five skipped; unit 4,203 passed/29 skipped; Python 1,261 passed/nine skipped; specifications seven pages/zero failures. All four suite exits are zero. Existing clean and forced-failure summary contracts, including the 2117 incomplete-collection behavior, passed within the eleven entry/summary contracts in job `1792268711571-66185`; a failing suite is labeled incomplete instead of printing a bare percentage. No extra coverage preflight or repeated collection was run.

The accepted 2120 filename filters define **145,197 filtered instrumented-source lines**, including inline test modules and some standard-library instrumentation. This is routine-feature coverage, not pure production-source or all-feature coverage. Shipped-feature behavior has its separate full-main qualification. The three fresh reports have valid TOTAL rows and no diagnostic errors; report wrappers returned zero. The harness did not retain each individual `llvm-cov report` exit separately, so those statuses remain unrecorded.

| Reported set | Reached lines | Reported percentage |
| --- | --- | --- |
| Outside-in | 49506 | 34.10% |
| Unit | 115657 | 79.66% |
| Union | 122916 | 84.65% |

| Disjoint group on the same denominator | Lines |
| --- | --- |
| Outside only | 7259 |
| Unit only | 73410 |
| Both | 42247 |
| Unobserved | 22281 |
| Incomplete candidate collections | 0 |

The four line groups sum to 145,197. Old collections are unavailable, so there is no comparable before percentage and no claim that coverage increased.

### Runtime observations

Both arms used the same commands and anonymous OpenFDA conditions: remove only `OPENFDA_API_KEY` from each runtime child environment and preserve other provider configuration. Warm/cache-off pairs alternate after consistent prefetch; each cache-off sample has an independent cache and cannot erase the warm cache. Default day-cache rows are distinct from forced warm rows. All 174 raw samples are retained, comprising 160 measured samples and 14 prefetches. Every sample exited zero and no sample reached the 180-second outer watchdog. Five warm and five cache-off search-all samples per arm reported no GWAS errors. Ten warm and ten cache-off drug samples per arm use a true even-sized median.

| Case | v0.9.2 median seconds | Candidate median seconds | Measured samples per arm |
| --- | --- | --- | --- |
| version_empty_cache | 0.010356 | 0.006612 | 5 |
| version_large_cache | 0.006116 | 0.004730 | 5 |
| warm_get_gene_BRAF | 0.192736 | 0.187100 | 5 |
| nocache_get_gene_BRAF | 0.441477 | 0.434339 | 5 |
| warm_get_variant_BRAF_V600E | 0.015475 | 0.017360 | 5 |
| nocache_get_variant_BRAF_V600E | 0.479658 | 0.474469 | 5 |
| warm_get_drug_imatinib | 1.206165 | 0.229730 | 10 |
| nocache_get_drug_imatinib | 1.905876 | 0.872234 | 10 |
| warm_search_article_BRAF | 1.723926 | 1.453399 | 5 |
| nocache_search_article_BRAF | 2.435817 | 2.960370 | 5 |
| warm_search_all_BRAF_melanoma | 10.883769 | 1.253691 | 5 |
| nocache_search_all_BRAF_melanoma | 10.758906 | 6.994962 | 5 |
| day_gwas_BRAF | 4.274273 | 0.006056 | 5 |
| day_openfda_drug_imatinib | 1.193192 | 0.229197 | 5 |

The version_large_cache row is the harness label for the existing default cache; its entry count was not recorded, so it does not establish a population-specific startup claim. Old drug output checksums were stable. Candidate cache-off drug output was stable; candidate warm drug output had nine hashes across ten samples. Inspection of the retained output differences found only one changing cache-age metadata line, with the first two warm samples identical. Original hashes remain unchanged; no normalization or rerun improved these results. Default drug-card timing mixes OpenFDA and other providers and cannot prove zero OpenFDA requests. Existing counted behavior contracts provide that separate functional evidence. First-run startup outliers are retained.

### Scope of the release evidence

Fresh bounded read-only release review dispatch `review_release_readiness_through_repaired_main` accepted product `234d2b32d3b5068dc27b2543d676997b75ad5baa`, using actual release/source/cache/fixture and relevant record diffs plus existing fresh per-ticket reviews. It did not claim a new line-by-line review of every historical intermediate commit. Full actual-main job `2156/1792269541418-30247` passed in 207.724 seconds, including shipped-feature checks and PNG/SVG/terminal output. The final measurement change has its own review and landing checks. The full untagged nonpublishing release rehearsal must follow its landing; no tag or publication was made by this measurement.

### Landing selection correction

Fresh final measurement review accepted `4160f2f6ffe621c06e4101d1525eb13824ece537` after checking every runtime median, sample status, source sum, coverage total and export/report checksum. The first configured landing gate `2082/1792272548440-79852` then failed the existing test-module selection guard, with 1,256 Python cases passing, one failing and nine skips. The new entry-test module's scratch `target/release` executable path triggers the conservative binary-dependent classification even though the cases use fake executables. Source `cc6a4b4f090511a4258bbb59e0b7cf1e6522c423` adds its conventional `needs_binary` marker and an accurate comment, following the existing fake-wrapper convention. Python-only job `2082/1792272865499-11576` passed the actual guard plus all eleven entry/summary cases and Ruff formatting/lint. This is a test-selection declaration; the guard, measurement tool, measured products and recorded raw data are unchanged. No formal measurement or coverage collection was repeated. A fresh review covers the final delta before the configured landing gate runs again.

### Durable raw data

Host-role wording and default-cache directory labels below are generalized for the public repository. Measurement values, sample ordering, statuses, checksums and source counts are retained. Provider stdout/stderr stay private; the raw sample arrays preserve safe error classes, byte counts and relative diagnostic filenames. The macOS routine wrapper does not provide operating-system network isolation.

#### Method and tools

```json
{
  "harness_sha": "dedbc1307e65c21c6f79bdb49025f209def074ac",
  "product_shas": {
    "old": "4049666f5cce2794f16cdbdf0ded49878e5bb147",
    "candidate": "234d2b32d3b5068dc27b2543d676997b75ad5baa"
  },
  "environment_names": [
    "ALPHAGENOME_API_KEY",
    "DISGENET_API_KEY",
    "HOME",
    "LLVM_COV",
    "LLVM_PROFDATA",
    "NCI_API_KEY",
    "PATH",
    "S2_API_KEY",
    "TMPDIR",
    "UMLS_API_KEY",
    "UV_FROZEN",
    "UV_PYTHON"
  ],
  "anonymous_runtime_removed_environment_names": [
    "OPENFDA_API_KEY"
  ],
  "runtime_environment_names": [
    "BIOMCP_CACHE_DIR",
    "BIOMCP_CACHE_MODE"
  ],
  "tool_version_output": [
    "rustc 1.95.0 (59807616e 2026-04-14) (Homebrew)",
    "binary: rustc",
    "commit-hash: 59807616e1fa2540724bfbac14d7976d7e4a3860",
    "commit-date: 2026-04-14",
    "host: aarch64-apple-darwin",
    "release: 1.95.0",
    "LLVM version: 22.1.3",
    "cargo 1.95.0 (f2d3ce0bd 2026-03-21) (Homebrew)",
    "cargo-nextest 0.9.146 (8af696ddc 2026-09-21)",
    "release: 0.9.146",
    "commit-hash: 8af696ddcce8fff2962d6a5168b6d138b8616a35",
    "commit-date: 2026-09-21",
    "host: aarch64-apple-darwin",
    "cargo-llvm-cov 0.9.1",
    "cargo-llvm-lines 0.4.48",
    "cargo-deny 0.20.2",
    "uv 0.11.14 (Homebrew 2026-05-12 aarch64-apple-darwin)",
    "ruff 0.13.2",
    "mustmatch 0.1.0",
    "GNU bash, version 5.3.0(1)-release (aarch64-apple-darwin25.4.0)",
    "Copyright (C) 2025 Free Software Foundation, Inc.",
    "License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>",
    "",
    "This is free software; you are free to change and redistribute it.",
    "There is NO WARRANTY, to the extent permitted by law.",
    "Homebrew LLVM version 22.1.5",
    "  Optimized build."
  ],
  "measured_python_versions": {
    "old": "Python 3.12.13",
    "candidate": "Python 3.12.13"
  },
  "coverage_environment_names": [
    "LLVM_PROFILE_FILE",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES",
    "RUSTC_WRAPPER",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_PRE_EXISTING",
    "CARGO_LLVM_COV",
    "CARGO_LLVM_COV_SHOW_ENV",
    "CARGO_LLVM_COV_TARGET_DIR",
    "CARGO_LLVM_COV_BUILD_DIR",
    "CARGO_INCREMENTAL",
    "CARGO_TARGET_DIR"
  ],
  "comparison_limitations": [
    "Shared build host; other-team Cargo activity observed, no controlled quiet claim or strict causal speedup claim.",
    "Local build/gate scenarios have one completed measured attempt per product; ordinary shared compiler cache retained, no cache or linker exceptions.",
    "Live runtime samples include providers and their variability; warm cache does not imply every provider request was cached.",
    "Default-mode drug card combines openFDA with other providers; timing alone does not prove absence of openFDA network requests.",
    "Prior setup attempts were stopped for Python setup mismatches and contribute no performance values.",
    "Old routine gates and coverage are unavailable on this host because v0.9.2 explicitly requires Linux facilities; source was not patched or host faked.",
    "No numeric test-code-share target was set; report actual source share without met/missed verdict.",
    "Current macOS routine wrapper reports no network sandbox; passing local gates do not prove OS-level offline isolation."
  ],
  "observed_one_minute_load_range": {
    "min": 3.966796875,
    "max": 14.28955078125
  },
  "runtime_outer_timeout_s": 180,
  "coverage_denominator": "Existing qualified filename filters remove test/example/bench/generated paths and named test files; inline test modules and some instrumented standard-library code remain counted; routine feature set.",
  "host": {
    "role": "shared build host",
    "architecture": "arm64",
    "os": "Darwin 25.4.0"
  }
}
```

#### Section commands, statuses and host loads

```json
[
{"arm":"setup","section":"tool-versions","exit_code":0,"elapsed_s":0.08430886268615723,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"setup/sections/tool-versions.log","command":["bash5","-c","rustc -vV\ncargo --version\ncargo nextest --version\ncargo llvm-cov --version\ncargo llvm-lines --version\ncargo deny --version\nuv --version\nruff --version\nmustmatch --version\nbash5 --version\n$LLVM_COV --version"],"results":{},"completeness":"complete"},
{"arm":"setup","section":"disk-and-memory","exit_code":0,"elapsed_s":0.010003805160522461,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"setup/sections/disk-and-memory.log","command":["bash5","-c","df -k .\nvm_stat\nsysctl -n hw.memsize\nuptime"],"results":{},"completeness":"complete"},
{"arm":"old","section":"checkout","exit_code":0,"elapsed_s":0.3027629852294922,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"old/sections/checkout.log","command":["git","worktree","add","--detach","<harness>/.cache/2082-final-comparison-v3/old/product","4049666f5cce2794f16cdbdf0ded49878e5bb147"],"results":{},"completeness":"complete"},
{"arm":"old","section":"dependency-fetch","exit_code":0,"elapsed_s":0.1488199234008789,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"old/sections/dependency-fetch.log","command":["cargo","fetch","--locked"],"results":{},"completeness":"complete"},
{"arm":"old","section":"python-sync","exit_code":0,"elapsed_s":0.2394108772277832,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"old/sections/python-sync.log","command":["uv","sync","--frozen","--extra","dev","--no-install-project"],"results":{},"completeness":"complete"},
{"arm":"old","section":"measured-python-version","exit_code":0,"elapsed_s":0.005407810211181641,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"old/sections/measured-python-version.log","command":["python3","--version"],"results":{},"completeness":"complete"},
{"arm":"old","section":"measured-toolchain","exit_code":0,"elapsed_s":0.02725386619567871,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"old/sections/measured-toolchain.log","command":["bash5","-c","rustc -vV\ncargo --version\nuv --version"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"checkout","exit_code":0,"elapsed_s":0.34095120429992676,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"candidate/sections/checkout.log","command":["git","worktree","add","--detach","<harness>/.cache/2082-final-comparison-v3/candidate/product","234d2b32d3b5068dc27b2543d676997b75ad5baa"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"dependency-fetch","exit_code":0,"elapsed_s":0.12984704971313477,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"candidate/sections/dependency-fetch.log","command":["cargo","fetch","--locked"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"python-sync","exit_code":0,"elapsed_s":0.245955228805542,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"candidate/sections/python-sync.log","command":["uv","sync","--frozen","--extra","dev","--no-install-project"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"measured-python-version","exit_code":0,"elapsed_s":0.005380868911743164,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"candidate/sections/measured-python-version.log","command":["python3","--version"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"measured-toolchain","exit_code":0,"elapsed_s":0.02771592140197754,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"candidate/sections/measured-toolchain.log","command":["bash5","-c","rustc -vV\ncargo --version\nuv --version"],"results":{},"completeness":"complete"},
{"arm":"old","section":"compiler-cache-before","exit_code":0,"elapsed_s":0.004361867904663086,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"old/sections/compiler-cache-before.log","command":["sccache","--show-stats"],"results":{},"completeness":"complete"},
{"arm":"old","section":"clone","exit_code":0,"elapsed_s":0.16703271865844727,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[3.966796875,4.95703125,4.8330078125],"log_file":"old/sections/clone.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","clone"],"results":{},"completeness":"complete"},
{"arm":"old","section":"cold-dev-build","exit_code":0,"elapsed_s":51.58177161216736,"load_before":[3.966796875,4.95703125,4.8330078125],"load_after":[5.6884765625,5.26904296875,4.95458984375],"log_file":"old/sections/cold-dev-build.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","cold-dev-build"],"results":{"cold_dev_build_wall_s":"51.5"},"completeness":"complete"},
{"arm":"old","section":"rust-edit-rebuild","exit_code":0,"elapsed_s":39.34505486488342,"load_before":[5.6884765625,5.26904296875,4.95458984375],"load_after":[5.12158203125,5.15869140625,4.92626953125],"log_file":"old/sections/rust-edit-rebuild.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","rust-edit-rebuild"],"results":{"rust_edit_rebuild_wall_s":"39.3"},"completeness":"complete"},
{"arm":"old","section":"doc-edit-rebuild","exit_code":0,"elapsed_s":42.4933922290802,"load_before":[5.12158203125,5.15869140625,4.92626953125],"load_after":[11.7841796875,6.8369140625,5.5498046875],"log_file":"old/sections/doc-edit-rebuild.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","doc-edit-rebuild"],"results":{"doc_edit_rebuild_wall_s":"42.4"},"completeness":"complete"},
{"arm":"old","section":"dep-counts","exit_code":0,"elapsed_s":0.27338600158691406,"load_before":[11.7841796875,6.8369140625,5.5498046875],"load_after":[11.7841796875,6.8369140625,5.5498046875],"log_file":"old/sections/dep-counts.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","dep-counts"],"results":{"direct_dependencies_normal":"58 (optional: 3: prost, tonic, zstd)","direct_dependencies_dev":"5","direct_dependencies_windows_only":"1","direct_dependencies_unix_dev":"1","cargo_lock_packages":"488","cargo_lock_duplicate_names":"50 (extra copies: 68)","crates_compiled_normal_build":"337","duplicate_crate_names_normal_build":"23 (base64, block-buffer, cipher, crypto-common, getrandom, html5ever, inout, markup5ever, phf, phf_codegen, phf_generator, phf_macros, phf_shared, rand, rand_core, siphasher, string_cache, string_cache_codegen, syn, tendril, thiserror, thiserror-impl, unicode-width)","crates_compiled_with_dev":"348","duplicate_crate_names_with_dev":"24 (base64, block-buffer, cipher, crypto-common, getrandom, html5ever, inout, markup5ever, phf, phf_codegen, phf_generator, phf_macros, phf_shared, rand, rand_core, reqwest, siphasher, string_cache, string_cache_codegen, syn, tendril, thiserror, thiserror-impl, unicode-width)"},"completeness":"complete"},
{"arm":"old","section":"prepare-context","exit_code":0,"elapsed_s":68.20988488197327,"load_before":[11.7841796875,6.8369140625,5.5498046875],"load_after":[13.73193359375,8.9873046875,6.494140625],"log_file":"old/sections/prepare-context.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","prepare-context"],"results":{"make_prepare_test_wall_s":"68.2"},"completeness":"complete"},
{"arm":"old","section":"make-test","exit_code":2,"elapsed_s":1.0246098041534424,"load_before":[13.73193359375,8.9873046875,6.494140625],"load_after":[13.73193359375,8.9873046875,6.494140625],"log_file":"old/sections/make-test.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","make-test"],"results":{},"completeness":"unavailable"},
{"arm":"old","section":"build-folder-after-make-test","exit_code":0,"elapsed_s":0.09723210334777832,"load_before":[13.73193359375,8.9873046875,6.494140625],"load_after":[13.73193359375,8.9873046875,6.494140625],"log_file":"old/sections/build-folder-after-make-test.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","build-folder-after-make-test"],"results":{"build_folder_bytes_after_make_test":"3646242816"},"completeness":"complete"},
{"arm":"old","section":"rust-test-census","exit_code":0,"elapsed_s":0.07756900787353516,"load_before":[13.73193359375,8.9873046875,6.494140625],"load_after":[13.73193359375,8.9873046875,6.494140625],"log_file":"old/sections/rust-test-census.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","rust-test-census"],"results":{"rust_test_census_wall_s":"0.0","rust_test_census_collection":"incomplete (exit 2)","rust_tests_with_duration":"0","tests_over_1s":"0","tests_over_1s_sum_s":"0.0"},"completeness":"incomplete (exit 2)"},
{"arm":"old","section":"python-test-count","exit_code":2,"elapsed_s":0.04462313652038574,"load_before":[13.73193359375,8.9873046875,6.494140625],"load_after":[13.73193359375,8.9873046875,6.494140625],"log_file":"old/sections/python-test-count.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","python-test-count"],"results":{},"completeness":"unavailable"},
{"arm":"old","section":"spec-block-census","exit_code":0,"elapsed_s":0.05936622619628906,"load_before":[13.73193359375,8.9873046875,6.494140625],"load_after":[13.73193359375,8.9873046875,6.494140625],"log_file":"old/sections/spec-block-census.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","spec-block-census"],"results":{"spec_pages_with_bash_blocks":"      38","spec_bash_blocks":"     496"},"completeness":"complete"},
{"arm":"old","section":"make-spec","exit_code":2,"elapsed_s":0.051773786544799805,"load_before":[13.73193359375,8.9873046875,6.494140625],"load_after":[13.73193359375,8.9873046875,6.494140625],"log_file":"old/sections/make-spec.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","make-spec"],"results":{},"completeness":"unavailable"},
{"arm":"old","section":"make-lint","exit_code":2,"elapsed_s":0.2030949592590332,"load_before":[13.73193359375,8.9873046875,6.494140625],"load_after":[12.9521484375,8.904296875,6.4794921875],"log_file":"old/sections/make-lint.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","make-lint"],"results":{},"completeness":"unavailable"},
{"arm":"old","section":"release-build","exit_code":0,"elapsed_s":260.3784167766571,"load_before":[12.9521484375,8.904296875,6.4794921875],"load_after":[7.59326171875,8.11669921875,6.7763671875],"log_file":"old/sections/release-build.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","release-build"],"results":{"release_build_wall_s":"260.3","release_peak_rss_kb":"4168064"},"completeness":"complete"},
{"arm":"old","section":"build-folder-after-release","exit_code":0,"elapsed_s":0.1200718879699707,"load_before":[7.59326171875,8.11669921875,6.7763671875],"load_after":[7.59326171875,8.11669921875,6.7763671875],"log_file":"old/sections/build-folder-after-release.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","build-folder-after-release"],"results":{"build_folder_bytes_after_release_build":"4935225344"},"completeness":"complete"},
{"arm":"old","section":"stripped-binary","exit_code":0,"elapsed_s":0.03890109062194824,"load_before":[7.59326171875,8.11669921875,6.7763671875],"load_after":[7.59326171875,8.11669921875,6.7763671875],"log_file":"old/sections/stripped-binary.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","stripped-binary"],"results":{"stripped_release_binary_bytes":"32775408"},"completeness":"complete"},
{"arm":"old","section":"llvm-lines","exit_code":0,"elapsed_s":40.74660015106201,"load_before":[7.59326171875,8.11669921875,6.7763671875],"load_after":[14.28955078125,9.853515625,7.47705078125],"log_file":"old/sections/llvm-lines.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","llvm-lines"],"results":{"llvm_lines_wall_s":"40.6","llvm_lines_total":"6897914","llvm_function_copies":"138788"},"completeness":"complete"},
{"arm":"old","section":"deny-advisories","exit_code":0,"elapsed_s":0.055689096450805664,"load_before":[14.28955078125,9.853515625,7.47705078125],"load_after":[14.28955078125,9.853515625,7.47705078125],"log_file":"old/sections/deny-advisories.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","deny-advisories"],"results":{"ignored_advisories":"6"},"completeness":"complete"},
{"arm":"old","section":"coverage-prepare-spec","exit_code":2,"elapsed_s":0.019115686416625977,"load_before":[14.28955078125,9.853515625,7.47705078125],"load_after":[14.28955078125,9.853515625,7.47705078125],"log_file":"old/sections/coverage-prepare-spec.log","command":["make","prepare-spec"],"results":{},"completeness":"unavailable"},
{"arm":"old","section":"coverage","exit_code":-15,"elapsed_s":1.1387591361999512,"load_before":[14.28955078125,9.853515625,7.47705078125],"load_after":[14.28955078125,9.853515625,7.47705078125],"log_file":"old/sections/coverage.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","coverage"],"results":{},"completeness":"not collected: known v0.9.2 Linux-only prerequisites; intentionally stopped before suites"},
{"arm":"old","section":"runtime","exit_code":0,"elapsed_s":218.97544193267822,"load_before":[14.28955078125,9.853515625,7.47705078125],"load_after":[10.7578125,10.10302734375,8.10986328125],"log_file":"old/sections/runtime.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/old/product","--only","runtime"],"results":{"runtime_version_empty_cache_median_s":"0.010356","runtime_version_empty_cache_spread_s":"1.414757","runtime_version_large_cache_median_s":"0.006116","runtime_version_large_cache_spread_s":"0.000284","runtime_warm_get_gene_BRAF_median_s":"0.192736","runtime_warm_get_gene_BRAF_spread_s":"0.026234","runtime_nocache_get_gene_BRAF_median_s":"0.441477","runtime_nocache_get_gene_BRAF_spread_s":"0.017303","runtime_warm_get_variant_BRAF_V600E_median_s":"0.015475","runtime_warm_get_variant_BRAF_V600E_spread_s":"0.007437","runtime_nocache_get_variant_BRAF_V600E_median_s":"0.479658","runtime_nocache_get_variant_BRAF_V600E_spread_s":"0.030834","runtime_warm_get_drug_imatinib_median_s":"1.206165","runtime_warm_get_drug_imatinib_spread_s":"0.158856","runtime_nocache_get_drug_imatinib_median_s":"1.905876","runtime_nocache_get_drug_imatinib_spread_s":"0.539166","runtime_warm_search_article_BRAF_median_s":"1.723926","runtime_warm_search_article_BRAF_spread_s":"2.739123","runtime_nocache_search_article_BRAF_median_s":"2.435817","runtime_nocache_search_article_BRAF_spread_s":"1.190371","runtime_warm_search_all_BRAF_melanoma_median_s":"10.883769","runtime_warm_search_all_BRAF_melanoma_spread_s":"2.362702","runtime_nocache_search_all_BRAF_melanoma_median_s":"10.758906","runtime_nocache_search_all_BRAF_melanoma_spread_s":"1.983846","runtime_day_gwas_BRAF_median_s":"4.274273","runtime_day_gwas_BRAF_spread_s":"2.258973","runtime_day_openfda_drug_imatinib_median_s":"1.193192","runtime_day_openfda_drug_imatinib_spread_s":"0.071266"},"completeness":"complete"},
{"arm":"old","section":"compiler-cache-after","exit_code":0,"elapsed_s":0.008127450942993164,"load_before":[10.7578125,10.10302734375,8.10986328125],"load_after":[10.7578125,10.10302734375,8.10986328125],"log_file":"old/sections/compiler-cache-after.log","command":["sccache","--show-stats"],"results":{},"completeness":"complete"},
{"arm":"old","section":"final-product-identity","exit_code":0,"elapsed_s":0.03763890266418457,"load_before":[10.7578125,10.10302734375,8.10986328125],"load_after":[10.7578125,10.10302734375,8.10986328125],"log_file":"old/sections/final-product-identity.log","command":["bash5","-c","git rev-parse HEAD\ngit status --porcelain"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"compiler-cache-before","exit_code":0,"elapsed_s":0.005028247833251953,"load_before":[10.7578125,10.10302734375,8.10986328125],"load_after":[10.7578125,10.10302734375,8.10986328125],"log_file":"candidate/sections/compiler-cache-before.log","command":["sccache","--show-stats"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"clone","exit_code":0,"elapsed_s":0.6084511280059814,"load_before":[10.7578125,10.10302734375,8.10986328125],"load_after":[10.7578125,10.10302734375,8.10986328125],"log_file":"candidate/sections/clone.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","clone"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"cold-dev-build","exit_code":0,"elapsed_s":72.8904972076416,"load_before":[10.7578125,10.10302734375,8.10986328125],"load_after":[13.7734375,11.2978515625,8.724609375],"log_file":"candidate/sections/cold-dev-build.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","cold-dev-build"],"results":{"cold_dev_build_wall_s":"72.8"},"completeness":"complete"},
{"arm":"candidate","section":"rust-edit-rebuild","exit_code":0,"elapsed_s":51.582035064697266,"load_before":[13.7734375,11.2978515625,8.724609375],"load_after":[10.7470703125,10.8876953125,8.73388671875],"log_file":"candidate/sections/rust-edit-rebuild.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","rust-edit-rebuild"],"results":{"rust_edit_rebuild_wall_s":"51.5"},"completeness":"complete"},
{"arm":"candidate","section":"doc-edit-rebuild","exit_code":0,"elapsed_s":0.3628418445587158,"load_before":[10.7470703125,10.8876953125,8.73388671875],"load_after":[10.7470703125,10.8876953125,8.73388671875],"log_file":"candidate/sections/doc-edit-rebuild.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","doc-edit-rebuild"],"results":{"doc_edit_rebuild_wall_s":"0.3"},"completeness":"complete"},
{"arm":"candidate","section":"dep-counts","exit_code":0,"elapsed_s":0.3307209014892578,"load_before":[10.7470703125,10.8876953125,8.73388671875],"load_after":[10.7470703125,10.8876953125,8.73388671875],"log_file":"candidate/sections/dep-counts.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","dep-counts"],"results":{"direct_dependencies_normal":"58 (optional: 4: prost, tokio-stream, tonic, zstd)","direct_dependencies_dev":"6","direct_dependencies_windows_only":"1","direct_dependencies_unix_dev":"1","cargo_lock_packages":"478","cargo_lock_duplicate_names":"35 (extra copies: 50)","crates_compiled_normal_build":"333","duplicate_crate_names_normal_build":"13 (base64, block-buffer, cipher, crypto-common, getrandom, html5ever, inout, markup5ever, rand, rand_core, thiserror, thiserror-impl, zip)","crates_compiled_with_dev":"342","duplicate_crate_names_with_dev":"14 (base64, block-buffer, cipher, crypto-common, getrandom, html5ever, inout, markup5ever, rand, rand_core, reqwest, thiserror, thiserror-impl, zip)"},"completeness":"complete"},
{"arm":"candidate","section":"prepare-context","exit_code":0,"elapsed_s":1.04653000831604,"load_before":[10.7470703125,10.8876953125,8.73388671875],"load_after":[10.7470703125,10.8876953125,8.73388671875],"log_file":"candidate/sections/prepare-context.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","prepare-context"],"results":{"make_prepare_test_wall_s":"1.0"},"completeness":"complete"},
{"arm":"candidate","section":"make-test","exit_code":0,"elapsed_s":118.2759039402008,"load_before":[10.7470703125,10.8876953125,8.73388671875],"load_after":[10.14404296875,10.8125,8.994140625],"log_file":"candidate/sections/make-test.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","make-test"],"results":{"make_test_wall_s":"118.2","rust_tests_run":"4362","rust_tests_passed":"4362","rust_tests_skipped":"34","python_lane_summary":"1253 passed, 9 skipped in 93.82s (0:01:33)"},"completeness":"complete"},
{"arm":"candidate","section":"build-folder-after-make-test","exit_code":0,"elapsed_s":0.17009425163269043,"load_before":[10.14404296875,10.8125,8.994140625],"load_after":[10.14404296875,10.8125,8.994140625],"log_file":"candidate/sections/build-folder-after-make-test.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","build-folder-after-make-test"],"results":{"build_folder_bytes_after_make_test":"2755584000"},"completeness":"complete"},
{"arm":"candidate","section":"rust-test-census","exit_code":0,"elapsed_s":38.48126292228699,"load_before":[10.14404296875,10.8125,8.994140625],"load_after":[8.755859375,10.39013671875,8.92333984375],"log_file":"candidate/sections/rust-test-census.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","rust-test-census"],"results":{"rust_test_census_wall_s":"38.4","rust_test_census_collection":"complete","rust_tests_with_duration":"4362","tests_over_1s":"75","tests_over_1s_sum_s":"320.2"},"completeness":"complete"},
{"arm":"candidate","section":"python-test-count","exit_code":0,"elapsed_s":0.5597712993621826,"load_before":[8.755859375,10.39013671875,8.92333984375],"load_after":[8.755859375,10.39013671875,8.92333984375],"log_file":"candidate/sections/python-test-count.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","python-test-count"],"results":{"python_collect_wall_s":"0.5","python_tests_collected":"1270"},"completeness":"complete"},
{"arm":"candidate","section":"spec-block-census","exit_code":0,"elapsed_s":0.061020851135253906,"load_before":[8.755859375,10.39013671875,8.92333984375],"load_after":[8.755859375,10.39013671875,8.92333984375],"log_file":"candidate/sections/spec-block-census.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","spec-block-census"],"results":{"spec_pages_with_bash_blocks":"      38","spec_bash_blocks":"     488"},"completeness":"complete"},
{"arm":"candidate","section":"make-spec","exit_code":0,"elapsed_s":24.406344175338745,"load_before":[8.755859375,10.39013671875,8.92333984375],"load_after":[7.6357421875,9.9892578125,8.8203125],"log_file":"candidate/sections/make-spec.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","make-spec"],"results":{"make_spec_wall_s":"24.4"},"completeness":"complete"},
{"arm":"candidate","section":"make-lint","exit_code":0,"elapsed_s":84.6405508518219,"load_before":[7.6357421875,9.9892578125,8.8203125],"load_after":[6.92578125,9.20703125,8.625],"log_file":"candidate/sections/make-lint.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","make-lint"],"results":{"make_lint_wall_s":"84.6"},"completeness":"complete"},
{"arm":"candidate","section":"release-build","exit_code":0,"elapsed_s":256.88951897621155,"load_before":[6.92578125,9.20703125,8.625],"load_after":[5.171875,7.2861328125,7.96826171875],"log_file":"candidate/sections/release-build.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","release-build"],"results":{"release_build_wall_s":"256.8","release_peak_rss_kb":"4361024"},"completeness":"complete"},
{"arm":"candidate","section":"build-folder-after-release","exit_code":0,"elapsed_s":0.2332620620727539,"load_before":[5.171875,7.2861328125,7.96826171875],"load_after":[5.171875,7.2861328125,7.96826171875],"log_file":"candidate/sections/build-folder-after-release.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","build-folder-after-release"],"results":{"build_folder_bytes_after_release_build":"4240039936"},"completeness":"complete"},
{"arm":"candidate","section":"stripped-binary","exit_code":0,"elapsed_s":0.047158002853393555,"load_before":[5.171875,7.2861328125,7.96826171875],"load_after":[5.171875,7.2861328125,7.96826171875],"log_file":"candidate/sections/stripped-binary.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","stripped-binary"],"results":{"stripped_release_binary_bytes":"33882944"},"completeness":"complete"},
{"arm":"candidate","section":"llvm-lines","exit_code":0,"elapsed_s":35.52882719039917,"load_before":[5.171875,7.2861328125,7.96826171875],"load_after":[6.00048828125,7.228515625,7.91748046875],"log_file":"candidate/sections/llvm-lines.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","llvm-lines"],"results":{"llvm_lines_wall_s":"35.4","llvm_lines_total":"7197318","llvm_function_copies":"143546"},"completeness":"complete"},
{"arm":"candidate","section":"deny-advisories","exit_code":0,"elapsed_s":0.049635887145996094,"load_before":[6.00048828125,7.228515625,7.91748046875],"load_after":[6.00048828125,7.228515625,7.91748046875],"log_file":"candidate/sections/deny-advisories.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","deny-advisories"],"results":{"ignored_advisories":"8"},"completeness":"complete"},
{"arm":"candidate","section":"coverage-prepare-spec","exit_code":0,"elapsed_s":0.7740211486816406,"load_before":[6.00048828125,7.228515625,7.91748046875],"load_after":[6.00048828125,7.228515625,7.91748046875],"log_file":"candidate/sections/coverage-prepare-spec.log","command":["make","prepare-spec"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"coverage","exit_code":0,"elapsed_s":242.28694200515747,"load_before":[6.00048828125,7.228515625,7.91748046875],"load_after":[6.67626953125,6.94921875,7.6171875],"log_file":"candidate/sections/coverage.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","coverage"],"results":{"coverage_build_wall_s":"61.8","coverage_integration_tests_wall_s":"21.1","coverage_integration_tests":"exit 0; 159 run: 159 passed, 0 failed, 0 timed out, 5 skipped","coverage_unit_tests_wall_s":"30.6","coverage_unit_tests":"exit 0; 4203 run: 4203 passed, 0 failed, 0 timed out, 29 skipped","coverage_python_lane_wall_s":"104.5","coverage_python_lane":"exit 0; 1261 passed, 0 failed, 0 errors, 9 skipped","coverage_spec_lane_wall_s":"22.1","coverage_spec_lane":"exit 0; 7 pages, 0 failed","coverage_outside_in_lines_pct":"34.10","coverage_unit_lines_pct":"79.66","coverage_combined_lines_pct":"84.65"},"completeness":"complete"},
{"arm":"candidate","section":"runtime","exit_code":0,"elapsed_s":99.19716477394104,"load_before":[6.67626953125,6.94921875,7.6171875],"load_after":[4.39111328125,6.1064453125,7.2158203125],"log_file":"candidate/sections/runtime.log","command":["bash5","<harness>/tools/measure-0-9-3-baseline","<harness>/.cache/2082-final-comparison-v3/candidate/product","--only","runtime"],"results":{"runtime_version_empty_cache_median_s":"0.006612","runtime_version_empty_cache_spread_s":"0.669872","runtime_version_large_cache_median_s":"0.004730","runtime_version_large_cache_spread_s":"0.000302","runtime_warm_get_gene_BRAF_median_s":"0.187100","runtime_warm_get_gene_BRAF_spread_s":"0.017821","runtime_nocache_get_gene_BRAF_median_s":"0.434339","runtime_nocache_get_gene_BRAF_spread_s":"0.018634","runtime_warm_get_variant_BRAF_V600E_median_s":"0.017360","runtime_warm_get_variant_BRAF_V600E_spread_s":"0.004439","runtime_nocache_get_variant_BRAF_V600E_median_s":"0.474469","runtime_nocache_get_variant_BRAF_V600E_spread_s":"0.014489","runtime_warm_get_drug_imatinib_median_s":"0.229730","runtime_warm_get_drug_imatinib_spread_s":"0.020782","runtime_nocache_get_drug_imatinib_median_s":"0.872234","runtime_nocache_get_drug_imatinib_spread_s":"0.634617","runtime_warm_search_article_BRAF_median_s":"1.453399","runtime_warm_search_article_BRAF_spread_s":"0.333644","runtime_nocache_search_article_BRAF_median_s":"2.960370","runtime_nocache_search_article_BRAF_spread_s":"1.249945","runtime_warm_search_all_BRAF_melanoma_median_s":"1.253691","runtime_warm_search_all_BRAF_melanoma_spread_s":"0.692756","runtime_nocache_search_all_BRAF_melanoma_median_s":"6.994962","runtime_nocache_search_all_BRAF_melanoma_spread_s":"0.675412","runtime_day_gwas_BRAF_median_s":"0.006056","runtime_day_gwas_BRAF_spread_s":"0.007765","runtime_day_openfda_drug_imatinib_median_s":"0.229197","runtime_day_openfda_drug_imatinib_spread_s":"0.003858"},"completeness":"complete"},
{"arm":"candidate","section":"compiler-cache-after","exit_code":0,"elapsed_s":0.006332874298095703,"load_before":[4.39111328125,6.1064453125,7.2158203125],"load_after":[4.39111328125,6.1064453125,7.2158203125],"log_file":"candidate/sections/compiler-cache-after.log","command":["sccache","--show-stats"],"results":{},"completeness":"complete"},
{"arm":"candidate","section":"final-product-identity","exit_code":0,"elapsed_s":0.026419878005981445,"load_before":[4.39111328125,6.1064453125,7.2158203125],"load_after":[4.39111328125,6.1064453125,7.2158203125],"log_file":"candidate/sections/final-product-identity.log","command":["bash5","-c","git rev-parse HEAD\ngit status --porcelain"],"results":{},"completeness":"complete"},
{"arm":"setup","section":"closing-disk-and-memory","exit_code":0,"elapsed_s":0.009569168090820312,"load_before":[4.39111328125,6.1064453125,7.2158203125],"load_after":[4.39111328125,6.1064453125,7.2158203125],"log_file":"setup/sections/closing-disk-and-memory.log","command":["bash5","-c","df -k .\nvm_stat\nuptime"],"results":{},"completeness":"complete"}
]
```

#### old: collection and source summary

```json
{
  "product_sha": "4049666f5cce2794f16cdbdf0ded49878e5bb147",
  "suite_exit_codes": {
    "census.exit": "2"
  },
  "source_census": {
    "method": "Rust src/**/*.rs physical lines, including blank and comment lines; directories named generated excluded; build output outside src excluded; test-only paths plus #[cfg(test)] item spans from brace matching with literals/comments ignored; union avoids overlap; integration/Python/spec files outside src excluded; same source method both arms",
    "scope": "Rust source under src only; not an all-language repository share",
    "line_measure": "physical lines, including blanks/comments; nonblank share not collected",
    "excluded_generated_files": [
      {
        "file": "src/generated/google.gdm.gdmscience.alphagenome.v1main.rs",
        "lines": 1486
      }
    ],
    "total_src_rust_lines": 290517,
    "test_only_file_lines": 93858,
    "inline_cfg_test_lines": 35035,
    "test_code_lines": 128893,
    "test_code_share_pct": 44.366766832921996,
    "numeric_share_target": null,
    "target_verdict": "not applicable: no numeric target was set",
    "primary_crate_rust": {
      "method": "nongenerated src Rust physical lines plus tracked tests/**/*.rs; all external Rust test lines count as test code; src test-only/inline span counts reused unchanged",
      "runtime_and_test_lines": 301538,
      "test_code_lines": 139914,
      "test_code_share_pct": 46.40012204100312,
      "src_rust_lines": 290517,
      "src_test_code_lines": 128893,
      "external_test_rust_lines": 11021,
      "external_test_rust_files": 34,
      "numeric_target": null,
      "target_verdict": "not applicable: no numeric target was set"
    },
    "python_tests": {
      "scope": "tracked tests/**/*.py physical lines, including test support and blank/comment lines; separate count, no mixed-language share",
      "physical_lines": 37979
    },
    "external_generated_exclusions": [],
    "scope_limits": [
      "Primary Rust metric covers only root-crate src and tests trees; workspace helper crates are excluded.",
      "Examples, benches, root build scripts and Rust files elsewhere are excluded.",
      "Generated directories and build output are excluded.",
      "Python tests are counted separately; Python source elsewhere, specifications, shell and other languages are not included in a composite share.",
      "All counts use physical lines, including blank/comment lines; no nonblank share is claimed."
    ]
  },
  "coverage": {
    "collection": "not collected",
    "reason": "v0.9.2 offline wrapper refuses non-Linux and specification setup requires /proc",
    "suite_exit_files": [],
    "profile_count": 0,
    "line_percentages": null
  },
  "approved_local_targets": {
    "make-test": {
      "target_s": 200,
      "measured_s": null,
      "verdict": "unavailable: gate did not complete"
    },
    "make-spec": {
      "target_s": 200,
      "measured_s": null,
      "verdict": "unavailable: gate did not complete"
    }
  },
  "native_release_peak_rss": {
    "raw_bytes": 4268097536,
    "conversion": "BSD time raw bytes / 1024 = KiB; existing RESULT key ends in _kb"
  },
  "routine_slow_warnings": {
    "threshold_s": 30,
    "count": 0,
    "lines": [],
    "old_suite_not_executed": true
  }
}
```

#### old: ordered raw runtime samples

```json
[
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":1.4208410419996653,"exit_code":0,"timed_out":false,"stdout_file":"0000-version-measure-default.stdout","stderr_file":"0000-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":0.011983417000010377,"exit_code":0,"timed_out":false,"stdout_file":"0001-version-measure-default.stdout","stderr_file":"0001-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":0.010121332998096477,"exit_code":0,"timed_out":false,"stdout_file":"0002-version-measure-default.stdout","stderr_file":"0002-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":0.010356250000768341,"exit_code":0,"timed_out":false,"stdout_file":"0003-version-measure-default.stdout","stderr_file":"0003-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":0.006084000000555534,"exit_code":0,"timed_out":false,"stdout_file":"0004-version-measure-default.stdout","stderr_file":"0004-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.006115916999988258,"exit_code":0,"timed_out":false,"stdout_file":"0005-version-measure-default.stdout","stderr_file":"0005-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.0061413339972204994,"exit_code":0,"timed_out":false,"stdout_file":"0006-version-measure-default.stdout","stderr_file":"0006-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.0060662919968308415,"exit_code":0,"timed_out":false,"stdout_file":"0007-version-measure-default.stdout","stderr_file":"0007-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.006083166997996159,"exit_code":0,"timed_out":false,"stdout_file":"0008-version-measure-default.stdout","stderr_file":"0008-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.00635020800109487,"exit_code":0,"timed_out":false,"stdout_file":"0009-version-measure-default.stdout","stderr_file":"0009-version-measure-default.stderr","stdout_bytes":13,"stdout_sha256":"0ce2eedb794722333d806fa4b8be1fc4f5d1112418ece41c7e3457349f0b1381","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.7129302079993067,"exit_code":0,"timed_out":false,"stdout_file":"0010-get_gene_BRAF-prefetch-default.stdout","stderr_file":"0010-get_gene_BRAF-prefetch-default.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.1927364579969435,"exit_code":0,"timed_out":false,"stdout_file":"0011-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0011-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/0","elapsed_s":0.4374668750024284,"exit_code":0,"timed_out":false,"stdout_file":"0012-get_gene_BRAF-measure-off.stdout","stderr_file":"0012-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.18055091600035666,"exit_code":0,"timed_out":false,"stdout_file":"0013-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0013-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/1","elapsed_s":0.4547698339993076,"exit_code":0,"timed_out":false,"stdout_file":"0014-get_gene_BRAF-measure-off.stdout","stderr_file":"0014-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.19806737500039162,"exit_code":0,"timed_out":false,"stdout_file":"0015-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0015-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/2","elapsed_s":0.4414766670015524,"exit_code":0,"timed_out":false,"stdout_file":"0016-get_gene_BRAF-measure-off.stdout","stderr_file":"0016-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.2067845830024453,"exit_code":0,"timed_out":false,"stdout_file":"0017-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0017-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/3","elapsed_s":0.43904604199997266,"exit_code":0,"timed_out":false,"stdout_file":"0018-get_gene_BRAF-measure-off.stdout","stderr_file":"0018-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.18591837499843678,"exit_code":0,"timed_out":false,"stdout_file":"0019-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0019-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/4","elapsed_s":0.4464188749989262,"exit_code":0,"timed_out":false,"stdout_file":"0020-get_gene_BRAF-measure-off.stdout","stderr_file":"0020-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.5298971670017636,"exit_code":0,"timed_out":false,"stdout_file":"0021-get_variant_BRAF_V600E-prefetch-default.stdout","stderr_file":"0021-get_variant_BRAF_V600E-prefetch-default.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.017033791002177168,"exit_code":0,"timed_out":false,"stdout_file":"0022-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0022-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/0","elapsed_s":0.4772556670031918,"exit_code":0,"timed_out":false,"stdout_file":"0023-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0023-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.015162249997956678,"exit_code":0,"timed_out":false,"stdout_file":"0024-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0024-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/1","elapsed_s":0.46540337500118767,"exit_code":0,"timed_out":false,"stdout_file":"0025-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0025-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.015475083000637824,"exit_code":0,"timed_out":false,"stdout_file":"0026-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0026-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/2","elapsed_s":0.49192187499647844,"exit_code":0,"timed_out":false,"stdout_file":"0027-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0027-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.012722874998871703,"exit_code":0,"timed_out":false,"stdout_file":"0028-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0028-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/3","elapsed_s":0.4796584169998823,"exit_code":0,"timed_out":false,"stdout_file":"0029-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0029-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.02016024999829824,"exit_code":0,"timed_out":false,"stdout_file":"0030-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0030-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/4","elapsed_s":0.49623774999781745,"exit_code":0,"timed_out":false,"stdout_file":"0031-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0031-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":2.371375041999272,"exit_code":0,"timed_out":false,"stdout_file":"0032-get_drug_imatinib-prefetch-default.stdout","stderr_file":"0032-get_drug_imatinib-prefetch-default.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.218692624999676,"exit_code":0,"timed_out":false,"stdout_file":"0033-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0033-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/0","elapsed_s":1.8383960419996583,"exit_code":0,"timed_out":false,"stdout_file":"0034-get_drug_imatinib-measure-off.stdout","stderr_file":"0034-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.2846837090000918,"exit_code":0,"timed_out":false,"stdout_file":"0035-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0035-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/1","elapsed_s":2.340702166002302,"exit_code":0,"timed_out":false,"stdout_file":"0036-get_drug_imatinib-measure-off.stdout","stderr_file":"0036-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.1936380000006466,"exit_code":0,"timed_out":false,"stdout_file":"0037-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0037-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/2","elapsed_s":2.111508458001481,"exit_code":0,"timed_out":false,"stdout_file":"0038-get_drug_imatinib-measure-off.stdout","stderr_file":"0038-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.1857329170015873,"exit_code":0,"timed_out":false,"stdout_file":"0039-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0039-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/3","elapsed_s":2.0200253750008414,"exit_code":0,"timed_out":false,"stdout_file":"0040-get_drug_imatinib-measure-off.stdout","stderr_file":"0040-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.1704570420006348,"exit_code":0,"timed_out":false,"stdout_file":"0041-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0041-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/4","elapsed_s":1.960622999999032,"exit_code":0,"timed_out":false,"stdout_file":"0042-get_drug_imatinib-measure-off.stdout","stderr_file":"0042-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.2254780420007592,"exit_code":0,"timed_out":false,"stdout_file":"0043-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0043-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/5","elapsed_s":1.8281313340012275,"exit_code":0,"timed_out":false,"stdout_file":"0044-get_drug_imatinib-measure-off.stdout","stderr_file":"0044-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.2470290419987577,"exit_code":0,"timed_out":false,"stdout_file":"0045-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0045-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/6","elapsed_s":1.8015358329976152,"exit_code":0,"timed_out":false,"stdout_file":"0046-get_drug_imatinib-measure-off.stdout","stderr_file":"0046-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.125827832998766,"exit_code":0,"timed_out":false,"stdout_file":"0047-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0047-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/7","elapsed_s":1.854240500000742,"exit_code":0,"timed_out":false,"stdout_file":"0048-get_drug_imatinib-measure-off.stdout","stderr_file":"0048-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.2203065829999105,"exit_code":0,"timed_out":false,"stdout_file":"0049-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0049-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/8","elapsed_s":1.852302833001886,"exit_code":0,"timed_out":false,"stdout_file":"0050-get_drug_imatinib-measure-off.stdout","stderr_file":"0050-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.1424394589994336,"exit_code":0,"timed_out":false,"stdout_file":"0051-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0051-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/9","elapsed_s":1.9575118749999092,"exit_code":0,"timed_out":false,"stdout_file":"0052-get_drug_imatinib-measure-off.stdout","stderr_file":"0052-get_drug_imatinib-measure-off.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":1.5655078339987085,"exit_code":0,"timed_out":false,"stdout_file":"0053-search_article_BRAF-prefetch-default.stdout","stderr_file":"0053-search_article_BRAF-prefetch-default.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":4.110526666998339,"exit_code":0,"timed_out":false,"stdout_file":"0054-search_article_BRAF-measure-infinite.stdout","stderr_file":"0054-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2960,"stdout_sha256":"59fd2ebd72b3951e934edb4deb0062eec9bf4ae5386b599f3edad5fca089baf8","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/0","elapsed_s":1.5385759999990114,"exit_code":0,"timed_out":false,"stdout_file":"0055-search_article_BRAF-measure-off.stdout","stderr_file":"0055-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":2.5178379170029075,"exit_code":0,"timed_out":false,"stdout_file":"0056-search_article_BRAF-measure-infinite.stdout","stderr_file":"0056-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2960,"stdout_sha256":"69cecc3629f9865978acf3531ed3a0c633ddd23c2e6765dafd837460b59d5de2","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/1","elapsed_s":1.4133038330001,"exit_code":0,"timed_out":false,"stdout_file":"0057-search_article_BRAF-measure-off.stdout","stderr_file":"0057-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":1.7239257090004685,"exit_code":0,"timed_out":false,"stdout_file":"0058-search_article_BRAF-measure-infinite.stdout","stderr_file":"0058-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2964,"stdout_sha256":"e3d79744eee2e61339b2a6a6d4efabf4434fe5877f854cb700bd475914035f6a","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/2","elapsed_s":2.435817457997473,"exit_code":0,"timed_out":false,"stdout_file":"0059-search_article_BRAF-measure-off.stdout","stderr_file":"0059-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":1.4217369580001105,"exit_code":0,"timed_out":false,"stdout_file":"0060-search_article_BRAF-measure-infinite.stdout","stderr_file":"0060-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2964,"stdout_sha256":"00352a088b1a8be9aa64dfc8d42dd58bba48fda1197618bbdf15f3f04db0a374","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/3","elapsed_s":2.5379619170016667,"exit_code":0,"timed_out":false,"stdout_file":"0061-search_article_BRAF-measure-off.stdout","stderr_file":"0061-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":1.3714037500030827,"exit_code":0,"timed_out":false,"stdout_file":"0062-search_article_BRAF-measure-infinite.stdout","stderr_file":"0062-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2964,"stdout_sha256":"81cffc5899bd515e57bb6a2f1870be0bd991269faf74ef11b077812543389cb1","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/4","elapsed_s":2.603674417001457,"exit_code":0,"timed_out":false,"stdout_file":"0063-search_article_BRAF-measure-off.stdout","stderr_file":"0063-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":10.261412249998102,"exit_code":0,"timed_out":false,"stdout_file":"0064-search_all_BRAF_melanoma-prefetch-default.stdout","stderr_file":"0064-search_all_BRAF_melanoma-prefetch-default.stderr","stdout_bytes":1605,"stdout_sha256":"e2dc0e7ef5ea86b54cc80858b2838aa449bf42caf9e765c4a927994d909c9beb","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":10.883769125000981,"exit_code":0,"timed_out":false,"stdout_file":"0065-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0065-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2425,"stdout_sha256":"5bbdc02e4115739a3def9a4437e7610ae11abe8657189d530ba75a8c81641732","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/0","elapsed_s":10.703362291002122,"exit_code":0,"timed_out":false,"stdout_file":"0066-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0066-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1605,"stdout_sha256":"cf1fc0bf327724954a40a32862f892630c1265be19f4761872d680a418d44d37","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":9.464748750000581,"exit_code":0,"timed_out":false,"stdout_file":"0067-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0067-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2329,"stdout_sha256":"69be3bf205a76e9a50dccdf1e9565f0e4ee89c87bb2941d2dac941583493fb96","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/1","elapsed_s":10.824284291000367,"exit_code":0,"timed_out":false,"stdout_file":"0068-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0068-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1605,"stdout_sha256":"7b28f9fb9eae7069677bc09599406a8cd431ac5fb14b82e748df00d359104292","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":11.015898790999927,"exit_code":0,"timed_out":false,"stdout_file":"0069-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0069-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2330,"stdout_sha256":"38e338ff66636443dae0d16e86e4e79bec7616c907f9bbc03f09651f4006c171","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/2","elapsed_s":9.83597754099901,"exit_code":0,"timed_out":false,"stdout_file":"0070-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0070-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1604,"stdout_sha256":"34908aae5be186c4c65f56688a8df3f69ddcdc09e24deff68c0dd968bd196b4f","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":10.482956083000317,"exit_code":0,"timed_out":false,"stdout_file":"0071-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0071-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2330,"stdout_sha256":"5e41a25389500854daea631512627f8de38422e9dbf7583c73c493f0bdf4d4dc","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/3","elapsed_s":10.75890633400195,"exit_code":0,"timed_out":false,"stdout_file":"0072-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0072-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1605,"stdout_sha256":"adcab4d0c72ce2a3d4f0c05354b2d237cb6f6a789ce6af92fe9a2ff2a1a17515","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":11.827450332999433,"exit_code":0,"timed_out":false,"stdout_file":"0073-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0073-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2429,"stdout_sha256":"4ebb0e76d3f628cb80ae64e26e6e4a25342b516fa1ed0ea2f646e3808a76c21e","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/4","elapsed_s":11.819823082998482,"exit_code":0,"timed_out":false,"stdout_file":"0074-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0074-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1605,"stdout_sha256":"93faa4cc549727dcd04e40ff07daab1fb6af82fbd944db712589dfe09bfdc054","gwas_error_classes":[null],"stderr_class":null},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"prefetch","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":4.608898458001931,"exit_code":0,"timed_out":false,"stdout_file":"0075-day_gwas_BRAF-prefetch-default.stdout","stderr_file":"0075-day_gwas_BRAF-prefetch-default.stderr","stdout_bytes":4930,"stdout_sha256":"e1dce06839998f8a068251486c82f4e7fd30e99ab16f9b1ec1d291e26118f33b","gwas_error_classes":[],"stderr_class":null},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":6.02307212499727,"exit_code":0,"timed_out":false,"stdout_file":"0076-day_gwas_BRAF-measure-default.stdout","stderr_file":"0076-day_gwas_BRAF-measure-default.stderr","stdout_bytes":4930,"stdout_sha256":"e1dce06839998f8a068251486c82f4e7fd30e99ab16f9b1ec1d291e26118f33b","gwas_error_classes":[],"stderr_class":null},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":4.274272666996694,"exit_code":0,"timed_out":false,"stdout_file":"0077-day_gwas_BRAF-measure-default.stdout","stderr_file":"0077-day_gwas_BRAF-measure-default.stderr","stdout_bytes":4930,"stdout_sha256":"e1dce06839998f8a068251486c82f4e7fd30e99ab16f9b1ec1d291e26118f33b","gwas_error_classes":[],"stderr_class":null},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":3.7640991669977666,"exit_code":0,"timed_out":false,"stdout_file":"0078-day_gwas_BRAF-measure-default.stdout","stderr_file":"0078-day_gwas_BRAF-measure-default.stderr","stdout_bytes":4930,"stdout_sha256":"e1dce06839998f8a068251486c82f4e7fd30e99ab16f9b1ec1d291e26118f33b","gwas_error_classes":[],"stderr_class":null},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":4.137757500000589,"exit_code":0,"timed_out":false,"stdout_file":"0079-day_gwas_BRAF-measure-default.stdout","stderr_file":"0079-day_gwas_BRAF-measure-default.stderr","stdout_bytes":4930,"stdout_sha256":"e1dce06839998f8a068251486c82f4e7fd30e99ab16f9b1ec1d291e26118f33b","gwas_error_classes":[],"stderr_class":null},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":4.515418083999975,"exit_code":0,"timed_out":false,"stdout_file":"0080-day_gwas_BRAF-measure-default.stdout","stderr_file":"0080-day_gwas_BRAF-measure-default.stderr","stdout_bytes":4930,"stdout_sha256":"e1dce06839998f8a068251486c82f4e7fd30e99ab16f9b1ec1d291e26118f33b","gwas_error_classes":[],"stderr_class":null},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"prefetch","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":1.8874395409984572,"exit_code":0,"timed_out":false,"stdout_file":"0081-day_openfda_drug_imatinib-prefetch-default.stdout","stderr_file":"0081-day_openfda_drug_imatinib-prefetch-default.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":1.2443469999998342,"exit_code":0,"timed_out":false,"stdout_file":"0082-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0082-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":1.1730813329995726,"exit_code":0,"timed_out":false,"stdout_file":"0083-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0083-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":1.193191874997865,"exit_code":0,"timed_out":false,"stdout_file":"0084-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0084-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":1.2085638749995269,"exit_code":0,"timed_out":false,"stdout_file":"0085-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0085-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":1.188715249998495,"exit_code":0,"timed_out":false,"stdout_file":"0086-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0086-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2410,"stdout_sha256":"026cfb1aac9b693578aae079f5f2b70592086359cb267ddf507e051fbe00ad1f","gwas_error_classes":[],"stderr_class":null}
]
```

#### candidate: collection and source summary

```json
{
  "product_sha": "234d2b32d3b5068dc27b2543d676997b75ad5baa",
  "suite_exit_codes": {
    "census.exit": "0",
    "cov-integration.exit": "0",
    "cov-pytest.exit": "0",
    "cov-spec.exit": "0",
    "cov-unit.exit": "0"
  },
  "source_census": {
    "method": "Rust src/**/*.rs physical lines, including blank and comment lines; directories named generated excluded; build output outside src excluded; test-only paths plus #[cfg(test)] item spans from brace matching with literals/comments ignored; union avoids overlap; integration/Python/spec files outside src excluded; same source method both arms",
    "scope": "Rust source under src only; not an all-language repository share",
    "line_measure": "physical lines, including blanks/comments; nonblank share not collected",
    "excluded_generated_files": [
      {
        "file": "src/generated/google.gdm.gdmscience.alphagenome.v1main.rs",
        "lines": 1486
      }
    ],
    "total_src_rust_lines": 301877,
    "test_only_file_lines": 98818,
    "inline_cfg_test_lines": 38014,
    "test_code_lines": 136832,
    "test_code_share_pct": 45.327070296842756,
    "numeric_share_target": null,
    "target_verdict": "not applicable: no numeric target was set",
    "primary_crate_rust": {
      "method": "nongenerated src Rust physical lines plus tracked tests/**/*.rs; all external Rust test lines count as test code; src test-only/inline span counts reused unchanged",
      "runtime_and_test_lines": 314914,
      "test_code_lines": 149869,
      "test_code_share_pct": 47.59045326660612,
      "src_rust_lines": 301877,
      "src_test_code_lines": 136832,
      "external_test_rust_lines": 13037,
      "external_test_rust_files": 33,
      "numeric_target": null,
      "target_verdict": "not applicable: no numeric target was set"
    },
    "python_tests": {
      "scope": "tracked tests/**/*.py physical lines, including test support and blank/comment lines; separate count, no mixed-language share",
      "physical_lines": 35606
    },
    "external_generated_exclusions": [],
    "scope_limits": [
      "Primary Rust metric covers only root-crate src and tests trees; workspace helper crates are excluded.",
      "Examples, benches, root build scripts and Rust files elsewhere are excluded.",
      "Generated directories and build output are excluded.",
      "Python tests are counted separately; Python source elsewhere, specifications, shell and other languages are not included in a composite share.",
      "All counts use physical lines, including blank/comment lines; no nonblank share is claimed."
    ]
  },
  "coverage": {
    "collection": "complete",
    "suite_exit_codes": {
      "cov-integration.exit": "0",
      "cov-pytest.exit": "0",
      "cov-spec.exit": "0",
      "cov-unit.exit": "0"
    },
    "reported_groups": {
      "coverage_outside_in_lines_pct": "34.10",
      "coverage_unit_lines_pct": "79.66",
      "coverage_combined_lines_pct": "84.65"
    },
    "reports": {
      "outside_in": {
        "denominator_lines": 145197,
        "missed_lines": 95691,
        "reached_lines": 49506,
        "line_coverage_pct": "34.10%",
        "raw_report_file": "candidate/raw/cov-report-outside_in.txt",
        "safe_report_file": "public-coverage/candidate/cov-report-outside_in.txt",
        "report_wrapper_exit_code": 0,
        "llvm_cov_report_exit_code": null,
        "report_exit_limitation": "individual llvm-cov status was not retained; final tail wrapper returned zero",
        "diagnostic_lines": []
      },
      "unit": {
        "denominator_lines": 145197,
        "missed_lines": 29540,
        "reached_lines": 115657,
        "line_coverage_pct": "79.66%",
        "raw_report_file": "candidate/raw/cov-report-unit.txt",
        "safe_report_file": "public-coverage/candidate/cov-report-unit.txt",
        "report_wrapper_exit_code": 0,
        "llvm_cov_report_exit_code": null,
        "report_exit_limitation": "individual llvm-cov status was not retained; final tail wrapper returned zero",
        "diagnostic_lines": []
      },
      "combined": {
        "denominator_lines": 145197,
        "missed_lines": 22281,
        "reached_lines": 122916,
        "line_coverage_pct": "84.65%",
        "raw_report_file": "candidate/raw/cov-report-combined.txt",
        "safe_report_file": "public-coverage/candidate/cov-report-combined.txt",
        "report_wrapper_exit_code": 0,
        "llvm_cov_report_exit_code": null,
        "report_exit_limitation": "individual llvm-cov status was not retained; final tail wrapper returned zero",
        "diagnostic_lines": []
      }
    },
    "denominator_lines": 145197,
    "denominator_label": "existing filtered instrumented source; inline tests and some standard-library instrumentation remain; routine features only",
    "five_groups": {
      "outside_only_lines": 7259,
      "unit_only_lines": 73410,
      "both_lines": 42247,
      "unobserved_lines": 22281,
      "incomplete_collections": 0
    }
  },
  "approved_local_targets": {
    "make-test": {
      "target_s": 200,
      "measured_s": 118.2,
      "verdict": "met"
    },
    "make-spec": {
      "target_s": 200,
      "measured_s": 24.4,
      "verdict": "met"
    }
  },
  "native_release_peak_rss": {
    "raw_bytes": 4465688576,
    "conversion": "BSD time raw bytes / 1024 = KiB; existing RESULT key ends in _kb"
  },
  "routine_slow_warnings": {
    "threshold_s": 30,
    "count": 1,
    "lines": [
      "SLOW tests/test_quality_ratchet_contract.py::test_wrapper_writes_summary_artifacts_for_pass_fixture (32.190s); load (1, 5, 15 min): 10.69, 10.84, 8.84"
    ],
    "old_suite_not_executed": false
  }
}
```

#### candidate: ordered raw runtime samples

```json
[
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":0.6747697079990758,"exit_code":0,"timed_out":false,"stdout_file":"0000-version-measure-default.stdout","stderr_file":"0000-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":0.01048287500088918,"exit_code":0,"timed_out":false,"stdout_file":"0001-version-measure-default.stdout","stderr_file":"0001-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":0.006611917000554968,"exit_code":0,"timed_out":false,"stdout_file":"0002-version-measure-default.stdout","stderr_file":"0002-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":0.005033624998759478,"exit_code":0,"timed_out":false,"stdout_file":"0003-version-measure-default.stdout","stderr_file":"0003-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"empty-cache","elapsed_s":0.004897957998764468,"exit_code":0,"timed_out":false,"stdout_file":"0004-version-measure-default.stdout","stderr_file":"0004-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.004794917000253918,"exit_code":0,"timed_out":false,"stdout_file":"0005-version-measure-default.stdout","stderr_file":"0005-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.004906624999421183,"exit_code":0,"timed_out":false,"stdout_file":"0006-version-measure-default.stdout","stderr_file":"0006-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.004605042002367554,"exit_code":0,"timed_out":false,"stdout_file":"0007-version-measure-default.stdout","stderr_file":"0007-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.004729708001832478,"exit_code":0,"timed_out":false,"stdout_file":"0008-version-measure-default.stdout","stderr_file":"0008-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"version","argv":["--version"],"phase":"measure","mode":"default","cache_dir":"existing-default-cache","elapsed_s":0.004665958000259707,"exit_code":0,"timed_out":false,"stdout_file":"0009-version-measure-default.stdout","stderr_file":"0009-version-measure-default.stderr","stdout_bytes":23,"stdout_sha256":"ffb64d3048acd755a1122daa63a9a88d175c34176b9a8e09a506dfe27aaa8855","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.48670237500118674,"exit_code":0,"timed_out":false,"stdout_file":"0010-get_gene_BRAF-prefetch-default.stdout","stderr_file":"0010-get_gene_BRAF-prefetch-default.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.17928300000130548,"exit_code":0,"timed_out":false,"stdout_file":"0011-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0011-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/0","elapsed_s":0.4413985829996818,"exit_code":0,"timed_out":false,"stdout_file":"0012-get_gene_BRAF-measure-off.stdout","stderr_file":"0012-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.18709979099730845,"exit_code":0,"timed_out":false,"stdout_file":"0013-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0013-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/1","elapsed_s":0.4322928330002469,"exit_code":0,"timed_out":false,"stdout_file":"0014-get_gene_BRAF-measure-off.stdout","stderr_file":"0014-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.19710420899718883,"exit_code":0,"timed_out":false,"stdout_file":"0015-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0015-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/2","elapsed_s":0.45092687500073225,"exit_code":0,"timed_out":false,"stdout_file":"0016-get_gene_BRAF-measure-off.stdout","stderr_file":"0016-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.19086904199866694,"exit_code":0,"timed_out":false,"stdout_file":"0017-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0017-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/3","elapsed_s":0.4331796249971376,"exit_code":0,"timed_out":false,"stdout_file":"0018-get_gene_BRAF-measure-off.stdout","stderr_file":"0018-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_gene_BRAF","elapsed_s":0.1838951250028913,"exit_code":0,"timed_out":false,"stdout_file":"0019-get_gene_BRAF-measure-infinite.stdout","stderr_file":"0019-get_gene_BRAF-measure-infinite.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_gene_BRAF","argv":["get","gene","BRAF"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_gene_BRAF/4","elapsed_s":0.4343392499977199,"exit_code":0,"timed_out":false,"stdout_file":"0020-get_gene_BRAF-measure-off.stdout","stderr_file":"0020-get_gene_BRAF-measure-off.stderr","stdout_bytes":1646,"stdout_sha256":"940babba82abba4b561a7af00e06f6d0715e7d12b32d3db8df4e240800ee9b2e","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.5274829589980072,"exit_code":0,"timed_out":false,"stdout_file":"0021-get_variant_BRAF_V600E-prefetch-default.stdout","stderr_file":"0021-get_variant_BRAF_V600E-prefetch-default.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.013630292000016198,"exit_code":0,"timed_out":false,"stdout_file":"0022-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0022-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/0","elapsed_s":0.47402866700213053,"exit_code":0,"timed_out":false,"stdout_file":"0023-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0023-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.018069375000777654,"exit_code":0,"timed_out":false,"stdout_file":"0024-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0024-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/1","elapsed_s":0.4744694580003852,"exit_code":0,"timed_out":false,"stdout_file":"0025-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0025-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.017621333001443418,"exit_code":0,"timed_out":false,"stdout_file":"0026-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0026-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/2","elapsed_s":0.4834131659990817,"exit_code":0,"timed_out":false,"stdout_file":"0027-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0027-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.01735983300022781,"exit_code":0,"timed_out":false,"stdout_file":"0028-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0028-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/3","elapsed_s":0.4763022080005612,"exit_code":0,"timed_out":false,"stdout_file":"0029-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0029-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_variant_BRAF_V600E","elapsed_s":0.01601279199894634,"exit_code":0,"timed_out":false,"stdout_file":"0030-get_variant_BRAF_V600E-measure-infinite.stdout","stderr_file":"0030-get_variant_BRAF_V600E-measure-infinite.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_variant_BRAF_V600E","argv":["get","variant","BRAF V600E"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_variant_BRAF_V600E/4","elapsed_s":0.4689238329992804,"exit_code":0,"timed_out":false,"stdout_file":"0031-get_variant_BRAF_V600E-measure-off.stdout","stderr_file":"0031-get_variant_BRAF_V600E-measure-off.stderr","stdout_bytes":1371,"stdout_sha256":"0746f675385674010d7c1d539ff29ded13a50be473f49efd81b73087496a890c","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":1.1347928750001302,"exit_code":0,"timed_out":false,"stdout_file":"0032-get_drug_imatinib-prefetch-default.stdout","stderr_file":"0032-get_drug_imatinib-prefetch-default.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.22771683400060283,"exit_code":0,"timed_out":false,"stdout_file":"0033-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0033-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2374,"stdout_sha256":"c46bd8cd771b4ad29c1d66c4a84c0092780fbb9165d0d363b2cacc4b9def5c05","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/0","elapsed_s":0.8875430830012192,"exit_code":0,"timed_out":false,"stdout_file":"0034-get_drug_imatinib-measure-off.stdout","stderr_file":"0034-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.2234195839992026,"exit_code":0,"timed_out":false,"stdout_file":"0035-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0035-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2374,"stdout_sha256":"c46bd8cd771b4ad29c1d66c4a84c0092780fbb9165d0d363b2cacc4b9def5c05","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/1","elapsed_s":0.9133694999982254,"exit_code":0,"timed_out":false,"stdout_file":"0036-get_drug_imatinib-measure-off.stdout","stderr_file":"0036-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.23809679199985112,"exit_code":0,"timed_out":false,"stdout_file":"0037-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0037-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2374,"stdout_sha256":"d108eb48edf24caed26a6abaae608b08c5d8071ce957ca59bf0dcb0d1bdb7347","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/2","elapsed_s":1.485767958998622,"exit_code":0,"timed_out":false,"stdout_file":"0038-get_drug_imatinib-measure-off.stdout","stderr_file":"0038-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.23051337500146474,"exit_code":0,"timed_out":false,"stdout_file":"0039-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0039-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2374,"stdout_sha256":"d9f29fb3b5892d809f672b31a1e6661c51d9a5e1845f11ec3e96caa2dc6e2f35","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/3","elapsed_s":0.8559267920027196,"exit_code":0,"timed_out":false,"stdout_file":"0040-get_drug_imatinib-measure-off.stdout","stderr_file":"0040-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.22346445899893297,"exit_code":0,"timed_out":false,"stdout_file":"0041-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0041-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2374,"stdout_sha256":"640ecc25294c02c656a00146f57f5a58f3d45019a0ec291f6a9c2e5dfab4a08b","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/4","elapsed_s":0.8657906670014199,"exit_code":0,"timed_out":false,"stdout_file":"0042-get_drug_imatinib-measure-off.stdout","stderr_file":"0042-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.23927445800291025,"exit_code":0,"timed_out":false,"stdout_file":"0043-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0043-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2374,"stdout_sha256":"5be4493638f3f7ff577ba76f6825f8a03dc871ee70e5847574296bbf8c4a80f2","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/5","elapsed_s":0.9952942090021679,"exit_code":0,"timed_out":false,"stdout_file":"0044-get_drug_imatinib-measure-off.stdout","stderr_file":"0044-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.22894579199783038,"exit_code":0,"timed_out":false,"stdout_file":"0045-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0045-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2374,"stdout_sha256":"f234c7cf2883632607d5f006d4ecf4e1461ec6bbf1979ef9569e84956790372a","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/6","elapsed_s":0.8786777500026801,"exit_code":0,"timed_out":false,"stdout_file":"0046-get_drug_imatinib-measure-off.stdout","stderr_file":"0046-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.2406966660018952,"exit_code":0,"timed_out":false,"stdout_file":"0047-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0047-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2374,"stdout_sha256":"0dedf28e8f4c76add8c7c398196e61aa30bb1de2cec4758de631bceb2484fbd8","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/7","elapsed_s":0.8542796250003448,"exit_code":0,"timed_out":false,"stdout_file":"0048-get_drug_imatinib-measure-off.stdout","stderr_file":"0048-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.24420204099806142,"exit_code":0,"timed_out":false,"stdout_file":"0049-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0049-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2374,"stdout_sha256":"9e2d4ed2e4c163e111314e3adaf200df38a0c8ed0b55c6c7fb9047ccbb78a217","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/8","elapsed_s":0.8511506249997183,"exit_code":0,"timed_out":false,"stdout_file":"0050-get_drug_imatinib-measure-off.stdout","stderr_file":"0050-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/get_drug_imatinib","elapsed_s":0.2286593749995518,"exit_code":0,"timed_out":false,"stdout_file":"0051-get_drug_imatinib-measure-infinite.stdout","stderr_file":"0051-get_drug_imatinib-measure-infinite.stderr","stdout_bytes":2375,"stdout_sha256":"fe47d24169a923746f35719e6b0e64ad4bcfa4bc90cf546a01b836977c1d9150","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"get_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"off","cache_dir":"off-cache/get_drug_imatinib/9","elapsed_s":0.8602505000017118,"exit_code":0,"timed_out":false,"stdout_file":"0052-get_drug_imatinib-measure-off.stdout","stderr_file":"0052-get_drug_imatinib-measure-off.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":2.824248125001759,"exit_code":0,"timed_out":false,"stdout_file":"0053-search_article_BRAF-prefetch-default.stdout","stderr_file":"0053-search_article_BRAF-prefetch-default.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":1.410500250000041,"exit_code":0,"timed_out":false,"stdout_file":"0054-search_article_BRAF-measure-infinite.stdout","stderr_file":"0054-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2960,"stdout_sha256":"5fbba863f4d75d9b7bfdf935b321cc54db9cd0970fd09cbab2cf183dc629bf10","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/0","elapsed_s":2.9716585420028423,"exit_code":0,"timed_out":false,"stdout_file":"0055-search_article_BRAF-measure-off.stdout","stderr_file":"0055-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":1.3577471250027884,"exit_code":0,"timed_out":false,"stdout_file":"0056-search_article_BRAF-measure-infinite.stdout","stderr_file":"0056-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2960,"stdout_sha256":"e9cb93cdb168ba7d14f50bfb40cf2d70ed57df337f2697ce590a8f99c429460c","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/1","elapsed_s":1.9502192910003942,"exit_code":0,"timed_out":false,"stdout_file":"0057-search_article_BRAF-measure-off.stdout","stderr_file":"0057-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":1.6913911249976081,"exit_code":0,"timed_out":false,"stdout_file":"0058-search_article_BRAF-measure-infinite.stdout","stderr_file":"0058-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2860,"stdout_sha256":"ea45f243f41dacd867f19757ca173d1616165e89cea3360ef4d9946f81c641e5","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/2","elapsed_s":2.960370167002111,"exit_code":0,"timed_out":false,"stdout_file":"0059-search_article_BRAF-measure-off.stdout","stderr_file":"0059-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":1.4533994590019574,"exit_code":0,"timed_out":false,"stdout_file":"0060-search_article_BRAF-measure-infinite.stdout","stderr_file":"0060-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2964,"stdout_sha256":"13131a2842f0af7b9fd49e9c098c217c4f99d1e0f9c5d0439678413c1b6ba70e","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/3","elapsed_s":2.1115409999983967,"exit_code":0,"timed_out":false,"stdout_file":"0061-search_article_BRAF-measure-off.stdout","stderr_file":"0061-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_article_BRAF","elapsed_s":1.6162915000022622,"exit_code":0,"timed_out":false,"stdout_file":"0062-search_article_BRAF-measure-infinite.stdout","stderr_file":"0062-search_article_BRAF-measure-infinite.stderr","stdout_bytes":2964,"stdout_sha256":"e3d3f0b932f565349a46eae2963913687919d40c750b755a9c71fc24de43b6fd","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_article_BRAF","argv":["search","article","-g","BRAF","--limit","10"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_article_BRAF/4","elapsed_s":3.20016433300043,"exit_code":0,"timed_out":false,"stdout_file":"0063-search_article_BRAF-measure-off.stdout","stderr_file":"0063-search_article_BRAF-measure-off.stderr","stdout_bytes":2555,"stdout_sha256":"02d5d8f137749f0edea30006737948db2c6aaa7f4d70ec63302506d845180d96","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"prefetch","mode":"default","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":6.8054702910012566,"exit_code":0,"timed_out":false,"stdout_file":"0064-search_all_BRAF_melanoma-prefetch-default.stdout","stderr_file":"0064-search_all_BRAF_melanoma-prefetch-default.stderr","stdout_bytes":1604,"stdout_sha256":"b50c3f0ff9a1f42fa234b404ad0d44a6edb467d14150667ac46e2bd88942ee5b","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":1.2536906249988533,"exit_code":0,"timed_out":false,"stdout_file":"0065-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0065-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2522,"stdout_sha256":"c6ad40d02c0df67b312f5076cf5cf4073a12ed306a63efbec4ed1294ed193958","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/0","elapsed_s":7.348312874997646,"exit_code":0,"timed_out":false,"stdout_file":"0066-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0066-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1604,"stdout_sha256":"ca0ecf22213ca17692892ad45299a906090053c8fa0a8a809b54a72f0dd9c7a1","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":1.5920318749995204,"exit_code":0,"timed_out":false,"stdout_file":"0067-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0067-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2629,"stdout_sha256":"50013f4fa0a2ebe37434ceb6ece5bdc45a693907ac672864218bed17c53c7dbf","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/1","elapsed_s":7.205638915998861,"exit_code":0,"timed_out":false,"stdout_file":"0068-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0068-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1604,"stdout_sha256":"53ef295b8440dd35c24fabc768a0b08d6b03dd0fb59b14c4fc143362dca32adc","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":1.226209000000381,"exit_code":0,"timed_out":false,"stdout_file":"0069-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0069-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2630,"stdout_sha256":"bf451d334ca1469e93895f6ad49166d27639b996c2339122ea1d77be2a9e90ae","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/2","elapsed_s":6.942134833996533,"exit_code":0,"timed_out":false,"stdout_file":"0070-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0070-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1604,"stdout_sha256":"af05dead6e81b0730bc0ccbe5c998fce10d763822b2ef623d0140ecca57c271c","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":1.1667762499964738,"exit_code":0,"timed_out":false,"stdout_file":"0071-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0071-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2630,"stdout_sha256":"ac3e118f12bb5f7b8d2b2eac3f3fc886828bec461fc7d6d6d3f36eb48411fbfb","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/3","elapsed_s":6.9949615000005,"exit_code":0,"timed_out":false,"stdout_file":"0072-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0072-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1604,"stdout_sha256":"81e0da8beebccddfb58baf1f28ee8486b7eec6f673424f8ee8bb6b0419677d21","gwas_error_classes":[null],"stderr_class":null},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"infinite","cache_dir":"warm-cache/search_all_BRAF_melanoma","elapsed_s":1.859531916998094,"exit_code":0,"timed_out":false,"stdout_file":"0073-search_all_BRAF_melanoma-measure-infinite.stdout","stderr_file":"0073-search_all_BRAF_melanoma-measure-infinite.stderr","stdout_bytes":2531,"stdout_sha256":"acb5bd657278e30293a8ebc3518d6d45701eabcfa570f78955eec9446936b1bb","gwas_error_classes":[null],"stderr_class":"diagnostic text present"},
{"case":"search_all_BRAF_melanoma","argv":["--json","search","all","--gene","BRAF","--disease","melanoma","--counts-only"],"phase":"measure","mode":"off","cache_dir":"off-cache/search_all_BRAF_melanoma/4","elapsed_s":6.672900583998853,"exit_code":0,"timed_out":false,"stdout_file":"0074-search_all_BRAF_melanoma-measure-off.stdout","stderr_file":"0074-search_all_BRAF_melanoma-measure-off.stderr","stdout_bytes":1604,"stdout_sha256":"c065367f81a7d1ea163f561926b1ffa804f99edea4e90c4fcd63c9df627b0369","gwas_error_classes":[null],"stderr_class":null},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"prefetch","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":3.8963330420010607,"exit_code":0,"timed_out":false,"stdout_file":"0075-day_gwas_BRAF-prefetch-default.stdout","stderr_file":"0075-day_gwas_BRAF-prefetch-default.stderr","stdout_bytes":4930,"stdout_sha256":"e1dce06839998f8a068251486c82f4e7fd30e99ab16f9b1ec1d291e26118f33b","gwas_error_classes":[],"stderr_class":null},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":0.013182624999899417,"exit_code":0,"timed_out":false,"stdout_file":"0076-day_gwas_BRAF-measure-default.stdout","stderr_file":"0076-day_gwas_BRAF-measure-default.stderr","stdout_bytes":5051,"stdout_sha256":"15091861ea95e9f0c5c570b6dc778c5553c5845800965bb311f1ccc2f90526c0","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":0.008741041998291621,"exit_code":0,"timed_out":false,"stdout_file":"0077-day_gwas_BRAF-measure-default.stdout","stderr_file":"0077-day_gwas_BRAF-measure-default.stderr","stdout_bytes":5051,"stdout_sha256":"15091861ea95e9f0c5c570b6dc778c5553c5845800965bb311f1ccc2f90526c0","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":0.006056375001207925,"exit_code":0,"timed_out":false,"stdout_file":"0078-day_gwas_BRAF-measure-default.stdout","stderr_file":"0078-day_gwas_BRAF-measure-default.stderr","stdout_bytes":5051,"stdout_sha256":"15091861ea95e9f0c5c570b6dc778c5553c5845800965bb311f1ccc2f90526c0","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":0.0056758749997243285,"exit_code":0,"timed_out":false,"stdout_file":"0079-day_gwas_BRAF-measure-default.stdout","stderr_file":"0079-day_gwas_BRAF-measure-default.stderr","stdout_bytes":5051,"stdout_sha256":"15091861ea95e9f0c5c570b6dc778c5553c5845800965bb311f1ccc2f90526c0","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"day_gwas_BRAF","argv":["--json","search","gwas","-g","BRAF"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_gwas_BRAF","elapsed_s":0.005417459000454983,"exit_code":0,"timed_out":false,"stdout_file":"0080-day_gwas_BRAF-measure-default.stdout","stderr_file":"0080-day_gwas_BRAF-measure-default.stderr","stdout_bytes":5051,"stdout_sha256":"15091861ea95e9f0c5c570b6dc778c5553c5845800965bb311f1ccc2f90526c0","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"prefetch","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":0.9934897079983784,"exit_code":0,"timed_out":false,"stdout_file":"0081-day_openfda_drug_imatinib-prefetch-default.stdout","stderr_file":"0081-day_openfda_drug_imatinib-prefetch-default.stderr","stdout_bytes":2275,"stdout_sha256":"b220c8b652a781cff42f3d4d8dad1dbe51ed3f2df0a0165543ad133acdef7666","gwas_error_classes":[],"stderr_class":null},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":0.23144037499878323,"exit_code":0,"timed_out":false,"stdout_file":"0082-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0082-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2374,"stdout_sha256":"c46bd8cd771b4ad29c1d66c4a84c0092780fbb9165d0d363b2cacc4b9def5c05","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":0.23159274999852641,"exit_code":0,"timed_out":false,"stdout_file":"0083-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0083-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2374,"stdout_sha256":"c46bd8cd771b4ad29c1d66c4a84c0092780fbb9165d0d363b2cacc4b9def5c05","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":0.22859233299823245,"exit_code":0,"timed_out":false,"stdout_file":"0084-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0084-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2374,"stdout_sha256":"c46bd8cd771b4ad29c1d66c4a84c0092780fbb9165d0d363b2cacc4b9def5c05","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":0.2291972920029366,"exit_code":0,"timed_out":false,"stdout_file":"0085-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0085-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2472,"stdout_sha256":"3a507a1892243f05a105037559211ff0364d5aa532aaf5095fe19092a0c30885","gwas_error_classes":[],"stderr_class":"diagnostic text present"},
{"case":"day_openfda_drug_imatinib","argv":["get","drug","imatinib"],"phase":"measure","mode":"default","cache_dir":"day-cache/day_openfda_drug_imatinib","elapsed_s":0.22773429199878592,"exit_code":0,"timed_out":false,"stdout_file":"0086-day_openfda_drug_imatinib-measure-default.stdout","stderr_file":"0086-day_openfda_drug_imatinib-measure-default.stderr","stdout_bytes":2472,"stdout_sha256":"3a507a1892243f05a105037559211ff0364d5aa532aaf5095fe19092a0c30885","gwas_error_classes":[],"stderr_class":"diagnostic text present"}
]
```

#### Per-file physical source counts

Rust rows are `[file, old_lines, old_test_only_lines, old_inline_test_lines, candidate_lines, candidate_test_only_lines, candidate_inline_test_lines]`; null means absent. External Rust and Python test rows are `[file, old_lines, candidate_lines]`. Generated exclusions are in each source summary.

```json
{"src_rust":[
["src/build_identity.rs",24,0,0,24,0,0],
["src/cache/citation_evidence.rs",183,0,0,183,0,0],
["src/cache/clean.rs",717,0,503,717,0,503],
["src/cache/clear.rs",763,0,264,971,0,378],
["src/cache/config.rs",949,0,547,949,0,547],
["src/cache/http.rs",null,null,null,84,0,0],
["src/cache/limits.rs",218,0,114,279,0,114],
["src/cache/manager.rs",1115,0,640,1300,0,668],
["src/cache/manager/tests/stale_marker_tests.rs",124,124,0,124,124,0],
["src/cache/manager/tests/startup_clean_tests.rs",null,null,null,138,138,0],
["src/cache/manager/tests/startup_totals_tests.rs",null,null,null,109,109,0],
["src/cache/manager/tests/write_security_tests.rs",697,697,0,722,722,0],
["src/cache/manager/tests/write_security_windows_tests.rs",113,113,0,113,113,0],
["src/cache/migration.rs",1088,0,552,1176,0,595],
["src/cache/mod.rs",131,0,30,135,0,30],
["src/cache/planner.rs",877,0,394,877,0,394],
["src/cache/private.rs",806,0,0,869,0,4],
["src/cache/provider_capture.rs",1508,0,463,1506,0,463],
["src/cache/provider_capture/unsupported.rs",25,0,17,25,0,17],
["src/cli/adverse_event/dispatch.rs",522,0,0,522,0,0],
["src/cli/adverse_event/mod.rs",93,0,4,93,0,4],
["src/cli/adverse_event/plan.rs",201,0,0,201,0,0],
["src/cli/adverse_event/tests.rs",546,546,0,546,546,0],
["src/cli/article/annotations.rs",54,0,0,54,0,0],
["src/cli/article/assets.rs",317,0,3,317,0,3],
["src/cli/article/dispatch.rs",677,0,24,756,0,24],
["src/cli/article/dispatch/workflow_tests.rs",75,75,0,75,75,0],
["src/cli/article/export.rs",102,0,0,102,0,0],
["src/cli/article/fulltext_view.rs",349,0,3,349,0,3],
["src/cli/article/mod.rs",518,0,4,540,0,4],
["src/cli/article/render.rs",52,0,0,52,0,0],
["src/cli/article/session.rs",371,0,2,370,0,2],
["src/cli/article/session/tests.rs",427,427,0,427,427,0],
["src/cli/article/tests/citation_evidence.rs",82,82,0,82,82,0],
["src/cli/article/tests/diagnostics.rs",64,64,0,65,65,0],
["src/cli/article/tests/entities_disease_links.rs",181,181,0,237,237,0],
["src/cli/article/tests/exact_lookup.rs",444,444,0,446,446,0],
["src/cli/article/tests/filters.rs",649,649,0,650,650,0],
["src/cli/article/tests/help.rs",392,392,0,265,265,0],
["src/cli/article/tests/json.rs",691,691,0,699,699,0],
["src/cli/article/tests/mod.rs",10,10,0,11,11,0],
["src/cli/article/tests/next_commands.rs",40,40,0,41,41,0],
["src/cli/article/tests/outage.rs",null,null,null,223,223,0],
["src/cli/article/workflow.rs",96,0,0,96,0,0],
["src/cli/author/detail.rs",28,0,0,28,0,0],
["src/cli/author/mod.rs",121,0,76,121,0,76],
["src/cli/author/papers.rs",27,0,0,27,0,0],
["src/cli/author/search.rs",38,0,0,38,0,0],
["src/cli/benchmark/mod.rs",127,0,0,null,null,null],
["src/cli/benchmark/run/execute.rs",428,0,3,null,null,null],
["src/cli/benchmark/run/mod.rs",193,0,0,null,null,null],
["src/cli/benchmark/run/regression.rs",257,0,3,null,null,null],
["src/cli/benchmark/run/render.rs",115,0,0,null,null,null],
["src/cli/benchmark/run/suite.rs",203,0,3,null,null,null],
["src/cli/benchmark/run/tests/execute.rs",9,9,0,null,null,null],
["src/cli/benchmark/run/tests/regression.rs",171,171,0,null,null,null],
["src/cli/benchmark/run/tests/suite.rs",44,44,0,null,null,null],
["src/cli/benchmark/score/mod.rs",172,0,3,null,null,null],
["src/cli/benchmark/score/normalize.rs",196,0,3,null,null,null],
["src/cli/benchmark/score/parse.rs",300,0,0,null,null,null],
["src/cli/benchmark/score/render.rs",71,0,3,null,null,null],
["src/cli/benchmark/score/tests/normalize.rs",29,29,0,null,null,null],
["src/cli/benchmark/score/tests/parse.rs",108,108,0,null,null,null],
["src/cli/benchmark/score/tests/render.rs",1,1,0,null,null,null],
["src/cli/benchmark/types.rs",136,0,0,null,null,null],
["src/cli/cache.rs",402,0,19,402,0,19],
["src/cli/cache/tests.rs",468,468,0,468,468,0],
["src/cli/cell_line/dispatch.rs",115,0,0,115,0,0],
["src/cli/cell_line/mod.rs",85,0,0,85,0,0],
["src/cli/chart.rs",249,0,152,249,0,152],
["src/cli/commands.rs",279,0,0,276,0,0],
["src/cli/commands/selectors.rs",445,0,0,445,0,0],
["src/cli/debug_plan.rs",35,0,0,35,0,0],
["src/cli/diagnostic/dispatch.rs",136,0,0,136,0,0],
["src/cli/diagnostic/mod.rs",66,0,4,66,0,4],
["src/cli/diagnostic/tests.rs",342,342,0,342,342,0],
["src/cli/discover.rs",570,0,431,570,0,431],
["src/cli/disease/dispatch.rs",381,0,32,381,0,32],
["src/cli/disease/mod.rs",114,0,4,114,0,4],
["src/cli/disease/tests.rs",225,225,0,225,225,0],
["src/cli/drug/alias_alignment_tests.rs",70,70,0,70,70,0],
["src/cli/drug/dispatch.rs",676,0,0,676,0,0],
["src/cli/drug/mod.rs",232,0,6,232,0,6],
["src/cli/drug/render.rs",303,0,0,303,0,0],
["src/cli/drug/tests.rs",694,694,0,695,695,0],
["src/cli/drug/workflow.rs",15,0,0,15,0,0],
["src/cli/gene/cspec.rs",236,0,0,236,0,0],
["src/cli/gene/dispatch.rs",175,0,0,175,0,0],
["src/cli/gene/mod.rs",209,0,5,209,0,5],
["src/cli/gene/related.rs",232,0,34,232,0,34],
["src/cli/gwas/dispatch.rs",133,0,58,134,0,59],
["src/cli/gwas/mod.rs",31,0,2,31,0,2],
["src/cli/gwas/tests.rs",158,158,0,158,158,0],
["src/cli/health/catalog.rs",541,0,0,548,0,0],
["src/cli/health/http.rs",589,0,80,625,0,80],
["src/cli/health/local.rs",417,0,0,482,0,0],
["src/cli/health/mod.rs",519,0,257,556,0,259],
["src/cli/health/runner.rs",377,0,20,443,0,20],
["src/cli/health/tests/catalog.rs",207,207,0,208,208,0],
["src/cli/health/tests/fda.rs",null,null,null,156,156,0],
["src/cli/health/tests/http.rs",352,352,0,392,392,0],
["src/cli/health/tests/local.rs",648,648,0,691,691,0],
["src/cli/health/tests/runner.rs",642,642,0,564,564,0],
["src/cli/install.rs",388,0,110,388,0,110],
["src/cli/list/catalog.rs",276,0,0,276,0,0],
["src/cli/list/clinical.rs",363,0,0,363,0,0],
["src/cli/list/helpers.rs",157,0,0,157,0,0],
["src/cli/list/literature.rs",214,0,0,217,0,0],
["src/cli/list/mod.rs",102,0,5,102,0,5],
["src/cli/list/molecular.rs",379,0,0,379,0,0],
["src/cli/list/tests/pages.rs",700,700,0,700,700,0],
["src/cli/list/tests/router.rs",26,26,0,26,26,0],
["src/cli/mcp_config.rs",186,0,84,186,0,84],
["src/cli/mod.rs",103,0,18,97,0,14],
["src/cli/outcome.rs",697,0,6,685,0,6],
["src/cli/outcome/mcp_chart.rs",55,0,0,55,0,0],
["src/cli/outcome/probe_tests.rs",108,108,0,108,108,0],
["src/cli/outcome/tests.rs",33,33,0,33,33,0],
["src/cli/pathway/dispatch.rs",353,0,0,353,0,0],
["src/cli/pathway/mod.rs",106,0,4,106,0,4],
["src/cli/pathway/tests.rs",168,168,0,168,168,0],
["src/cli/pgx/dispatch.rs",85,0,0,85,0,0],
["src/cli/pgx/mod.rs",57,0,4,57,0,4],
["src/cli/pgx/tests.rs",61,61,0,61,61,0],
["src/cli/phenotype/dispatch.rs",84,0,0,84,0,0],
["src/cli/phenotype/mod.rs",21,0,2,21,0,2],
["src/cli/phenotype/tests.rs",115,115,0,115,115,0],
["src/cli/protein/dispatch.rs",184,0,0,184,0,0],
["src/cli/protein/mod.rs",75,0,4,75,0,4],
["src/cli/protein/tests.rs",137,137,0,137,137,0],
["src/cli/response_contract.rs",630,0,272,633,0,272],
["src/cli/search_all/dispatch.rs",458,0,0,458,0,0],
["src/cli/search_all/format.rs",349,0,0,349,0,0],
["src/cli/search_all/links.rs",370,0,0,370,0,0],
["src/cli/search_all/mod.rs",296,0,7,296,0,7],
["src/cli/search_all/plan.rs",650,0,0,650,0,0],
["src/cli/search_all/tests/dispatch.rs",283,283,0,283,283,0],
["src/cli/search_all/tests/format.rs",290,290,0,290,290,0],
["src/cli/search_all/tests/links.rs",218,218,0,218,218,0],
["src/cli/search_all/tests/plan.rs",468,468,0,468,468,0],
["src/cli/search_all_command.rs",161,0,123,161,0,123],
["src/cli/shared.rs",489,0,11,489,0,11],
["src/cli/shared/search_payloads.rs",282,0,0,282,0,0],
["src/cli/skill/assets.rs",55,0,0,55,0,0],
["src/cli/skill/atomic_swap.rs",122,0,23,122,0,23],
["src/cli/skill/catalog.rs",171,0,0,171,0,0],
["src/cli/skill/install.rs",620,0,4,620,0,4],
["src/cli/skill/mod.rs",65,0,5,65,0,5],
["src/cli/skill/status.rs",319,0,0,319,0,0],
["src/cli/skill/tests/catalog.rs",391,391,0,391,391,0],
["src/cli/skill/tests/install.rs",374,374,0,374,374,0],
["src/cli/stale_json_note_tests.rs",504,504,0,539,539,0],
["src/cli/study/dispatch.rs",437,0,0,437,0,0],
["src/cli/study/mod.rs",183,0,2,183,0,2],
["src/cli/study/tests.rs",10,10,0,10,10,0],
["src/cli/study/tests/charts.rs",412,412,0,412,412,0],
["src/cli/study/tests/help.rs",126,126,0,126,126,0],
["src/cli/study/tests/parsing.rs",270,270,0,270,270,0],
["src/cli/study/tests/validation.rs",184,184,0,184,184,0],
["src/cli/system/batch.rs",678,0,425,812,0,530],
["src/cli/system/dispatch.rs",680,0,45,680,0,45],
["src/cli/system/mod.rs",228,0,4,231,0,4],
["src/cli/system/tests.rs",689,689,0,696,696,0],
["src/cli/system/tests/dev2_contracts.rs",303,303,0,307,307,0],
["src/cli/system/tests/who_sync.rs",132,132,0,132,132,0],
["src/cli/system/who_sync.rs",70,0,0,70,0,0],
["src/cli/tests/facade.rs",543,543,0,543,543,0],
["src/cli/tests/facade/cache.rs",212,212,0,214,214,0],
["src/cli/tests/facade/chart.rs",157,157,0,157,157,0],
["src/cli/tests/facade/help.rs",229,229,0,195,195,0],
["src/cli/tests/mod.rs",6,6,0,6,6,0],
["src/cli/tests/next_commands_json_property.rs",67,67,0,67,67,0],
["src/cli/tests/next_commands_json_property/diagnostic.rs",287,287,0,287,287,0],
["src/cli/tests/next_commands_json_property/disease_trial.rs",294,294,0,294,294,0],
["src/cli/tests/next_commands_json_property/gene_article.rs",306,306,0,306,306,0],
["src/cli/tests/next_commands_json_property/pathway_adverse_event.rs",298,298,0,298,298,0],
["src/cli/tests/next_commands_json_property/protein_phenotype.rs",85,85,0,85,85,0],
["src/cli/tests/next_commands_json_property/variant_drug.rs",152,152,0,152,152,0],
["src/cli/tests/next_commands_validity.rs",306,306,0,306,306,0],
["src/cli/tests/outcome.rs",552,552,0,552,552,0],
["src/cli/tests/printed_card_commands.rs",268,268,0,595,595,0],
["src/cli/tests/surface_agreement.rs",398,398,0,398,398,0],
["src/cli/trial/count_tests.rs",117,117,0,117,117,0],
["src/cli/trial/dispatch.rs",630,0,0,630,0,0],
["src/cli/trial/documents.rs",188,0,78,188,0,78],
["src/cli/trial/mod.rs",163,0,8,163,0,8],
["src/cli/trial/tests.rs",689,689,0,689,689,0],
["src/cli/trial/tests_locations.rs",650,650,0,650,650,0],
["src/cli/trial/zero_result.rs",191,0,0,191,0,0],
["src/cli/types.rs",416,0,0,421,0,0],
["src/cli/update.rs",538,0,8,541,0,8],
["src/cli/update/tests.rs",335,335,0,365,365,0],
["src/cli/variant/articles.rs",235,0,0,235,0,0],
["src/cli/variant/car.rs",107,0,19,107,0,19],
["src/cli/variant/dispatch.rs",602,0,79,598,0,79],
["src/cli/variant/erepo.rs",357,0,97,357,0,97],
["src/cli/variant/guidance.rs",28,0,0,28,0,0],
["src/cli/variant/mod.rs",336,0,5,336,0,5],
["src/cli/variant/normalization_json.rs",117,0,55,117,0,55],
["src/cli/variant/query.rs",585,0,0,585,0,0],
["src/cli/variant/trial.rs",33,0,0,33,0,0],
["src/cli/worker.rs",97,0,24,97,0,24],
["src/entities/adverse_event.rs",2663,0,783,2663,0,783],
["src/entities/adverse_event/device.rs",184,0,0,184,0,0],
["src/entities/adverse_event/sections.rs",158,0,46,158,0,46],
["src/entities/article/assets.rs",2909,0,646,3712,0,975],
["src/entities/article/backends.rs",993,0,2,1001,0,2],
["src/entities/article/backends/provider_unit.rs",140,0,0,140,0,0],
["src/entities/article/backends/tests.rs",687,687,0,687,687,0],
["src/entities/article/batch.rs",203,0,30,203,0,30],
["src/entities/article/batch/tests.rs",276,276,0,276,276,0],
["src/entities/article/candidates.rs",368,0,2,368,0,2],
["src/entities/article/candidates/tests.rs",580,580,0,580,580,0],
["src/entities/article/detail.rs",842,0,2,849,0,2],
["src/entities/article/detail/tests.rs",355,355,0,421,421,0],
["src/entities/article/enrichment.rs",515,0,2,595,0,2],
["src/entities/article/enrichment/tests.rs",179,179,0,354,354,0],
["src/entities/article/filters.rs",330,0,2,330,0,2],
["src/entities/article/filters/tests.rs",335,335,0,335,335,0],
["src/entities/article/fulltext.rs",1727,0,713,1903,0,837],
["src/entities/article/graph.rs",503,0,2,503,0,2],
["src/entities/article/graph/citation_evidence.rs",1278,0,19,1278,0,19],
["src/entities/article/graph/tests.rs",2263,2263,0,2263,2263,0],
["src/entities/article/graph/tests/admission.rs",307,307,0,307,307,0],
["src/entities/article/graph/tests/cache.rs",660,660,0,660,660,0],
["src/entities/article/identity_verification.rs",1404,0,559,1533,0,620],
["src/entities/article/mod.rs",1439,0,324,1515,0,324],
["src/entities/article/planner.rs",320,0,2,320,0,2],
["src/entities/article/planner/tests.rs",460,460,0,460,460,0],
["src/entities/article/query.rs",629,0,23,686,0,17],
["src/entities/article/query/tests.rs",333,333,0,458,458,0],
["src/entities/article/ranking.rs",573,0,2,573,0,2],
["src/entities/article/ranking/tests.rs",27,27,0,27,27,0],
["src/entities/article/ranking/tests/calibration.rs",153,153,0,153,153,0],
["src/entities/article/ranking/tests/calibration/hybrid.rs",524,524,0,524,524,0],
["src/entities/article/ranking/tests/calibration/rescue.rs",319,319,0,319,319,0],
["src/entities/article/ranking/tests/directness.rs",208,208,0,208,208,0],
["src/entities/article/ranking/tests/keyword.rs",82,82,0,82,82,0],
["src/entities/article/ranking/tests/policy.rs",144,144,0,144,144,0],
["src/entities/article/search.rs",988,0,2,1015,0,2],
["src/entities/article/search/deadline.rs",142,0,0,215,0,40],
["src/entities/article/search/status.rs",87,0,0,87,0,0],
["src/entities/article/search/tests.rs",168,168,0,169,169,0],
["src/entities/article/search/tests/deadline.rs",669,669,0,663,663,0],
["src/entities/article/search/tests/finalizer.rs",89,89,0,89,89,0],
["src/entities/article/search/tests/integration.rs",82,82,0,82,82,0],
["src/entities/article/search/tests/merge.rs",449,449,0,449,449,0],
["src/entities/article/search/tests/relevance_paging.rs",null,null,null,53,53,0],
["src/entities/article/search/tests/type_capable.rs",120,120,0,120,120,0],
["src/entities/article/supplement_search.rs",null,null,null,580,0,51],
["src/entities/article/test_support.rs",224,224,0,224,224,0],
["src/entities/article/variant_search.rs",6303,0,2189,8958,0,4014],
["src/entities/author/mod.rs",618,0,179,618,0,179],
["src/entities/author/papers.rs",1597,0,869,1597,0,869],
["src/entities/author/search.rs",358,0,123,358,0,123],
["src/entities/cell_line.rs",753,0,11,753,0,11],
["src/entities/cell_line/chembl.rs",138,0,2,138,0,2],
["src/entities/cell_line/chembl/tests.rs",196,196,0,196,196,0],
["src/entities/cell_line/pharmacodb.rs",166,0,2,166,0,2],
["src/entities/cell_line/pharmacodb/tests.rs",243,243,0,243,243,0],
["src/entities/cell_line/tests.rs",467,467,0,467,467,0],
["src/entities/diagnostic/get.rs",695,0,35,695,0,35],
["src/entities/diagnostic/mod.rs",860,0,566,860,0,566],
["src/entities/diagnostic/search.rs",707,0,257,707,0,257],
["src/entities/discover.rs",4220,0,1477,4220,0,1477],
["src/entities/disease/article_links.rs",132,0,0,167,0,2],
["src/entities/disease/article_links/tests.rs",null,null,null,102,102,0],
["src/entities/disease/associations.rs",664,0,7,664,0,7],
["src/entities/disease/associations/tests.rs",128,128,0,128,128,0],
["src/entities/disease/enrichment.rs",872,0,8,872,0,8],
["src/entities/disease/enrichment/tests.rs",574,574,0,574,574,0],
["src/entities/disease/fallback.rs",515,0,2,515,0,2],
["src/entities/disease/fallback/tests.rs",455,455,0,455,455,0],
["src/entities/disease/get.rs",287,0,6,287,0,6],
["src/entities/disease/get/tests.rs",818,818,0,820,820,0],
["src/entities/disease/mod.rs",382,0,4,383,0,4],
["src/entities/disease/resolution.rs",893,0,2,914,0,2],
["src/entities/disease/resolution/ambiguity.rs",173,0,0,226,0,0],
["src/entities/disease/resolution/tests.rs",696,696,0,745,745,0],
["src/entities/disease/search.rs",837,0,14,837,0,14],
["src/entities/disease/search/tests.rs",703,703,0,703,703,0],
["src/entities/disease/test_support.rs",98,98,0,98,98,0],
["src/entities/disease/tests.rs",65,65,0,65,65,0],
["src/entities/drug/cell_lines.rs",231,0,2,231,0,2],
["src/entities/drug/cell_lines/tests.rs",233,233,0,233,233,0],
["src/entities/drug/get.rs",1428,0,6,1709,0,6],
["src/entities/drug/get/tests.rs",765,765,0,1105,1105,0],
["src/entities/drug/interactions.rs",482,0,103,482,0,103],
["src/entities/drug/label.rs",1303,0,4,1349,0,4],
["src/entities/drug/label/tests.rs",5,5,0,5,5,0],
["src/entities/drug/label/tests/extraction.rs",262,262,0,262,262,0],
["src/entities/drug/label/tests/identity.rs",398,398,0,398,398,0],
["src/entities/drug/label/tests/summary.rs",103,103,0,103,103,0],
["src/entities/drug/metadata.rs",508,0,2,530,0,2],
["src/entities/drug/metadata/tests.rs",61,61,0,152,152,0],
["src/entities/drug/mod.rs",717,0,31,717,0,31],
["src/entities/drug/query.rs",278,0,2,278,0,2],
["src/entities/drug/query/tests.rs",137,137,0,137,137,0],
["src/entities/drug/search.rs",999,0,46,999,0,46],
["src/entities/drug/search/tests.rs",271,271,0,271,271,0],
["src/entities/drug/search/tests/fallback.rs",242,242,0,242,242,0],
["src/entities/drug/search/tests/mechanism.rs",55,55,0,55,55,0],
["src/entities/drug/search/tests/who.rs",218,218,0,218,218,0],
["src/entities/drug/targets.rs",436,0,2,466,0,2],
["src/entities/drug/targets/tests.rs",8,8,0,8,8,0],
["src/entities/drug/targets/tests/civic.rs",99,99,0,99,99,0],
["src/entities/drug/targets/tests/family.rs",160,160,0,160,160,0],
["src/entities/drug/targets/tests/indications.rs",39,39,0,39,39,0],
["src/entities/drug/test_support.rs",967,967,0,967,967,0],
["src/entities/drug/test_support/brand_resolution.rs",642,642,0,762,762,0],
["src/entities/drug/test_support/ingredient_resolution.rs",670,670,0,689,689,0],
["src/entities/drug/test_support/label_choice.rs",1288,1288,0,1288,1288,0],
["src/entities/drug/test_support/product_names.rs",638,638,0,638,638,0],
["src/entities/gene.rs",3963,0,677,3974,0,677],
["src/entities/gene/cell_lines.rs",199,0,2,199,0,2],
["src/entities/gene/cell_lines/tests.rs",324,324,0,324,324,0],
["src/entities/gene/clingen.rs",283,0,153,283,0,153],
["src/entities/gene/cspec.rs",1105,0,405,1105,0,405],
["src/entities/gene/gencc.rs",288,0,11,288,0,11],
["src/entities/gene/gencc/tests.rs",1019,1019,0,1037,1037,0],
["src/entities/mod.rs",84,0,0,84,0,0],
["src/entities/pathway.rs",1314,0,419,1300,0,409],
["src/entities/pgx.rs",1139,0,157,1139,0,157],
["src/entities/pgx/recommendation_tests.rs",179,179,0,179,179,0],
["src/entities/pharmacodb.rs",366,0,2,366,0,2],
["src/entities/pharmacodb/tests.rs",425,425,0,425,425,0],
["src/entities/protein.rs",957,0,282,957,0,282],
["src/entities/section_outcome.rs",480,0,159,480,0,159],
["src/entities/source_state_registry.rs",946,0,44,946,0,44],
["src/entities/study.rs",1774,0,535,1774,0,535],
["src/entities/study/tests/missing_root.rs",46,46,0,46,46,0],
["src/entities/trial/design.rs",943,0,45,943,0,45],
["src/entities/trial/documents.rs",496,0,219,500,0,219],
["src/entities/trial/eligibility.rs",810,0,288,810,0,288],
["src/entities/trial/get.rs",306,0,2,306,0,2],
["src/entities/trial/get/tests.rs",907,907,0,907,907,0],
["src/entities/trial/mod.rs",860,0,159,860,0,159],
["src/entities/trial/search/ctgov.rs",1134,0,4,1134,0,4],
["src/entities/trial/search/ctgov/abbreviation_note_tests.rs",247,247,0,247,247,0],
["src/entities/trial/search/ctgov/tests.rs",1123,1123,0,1123,1123,0],
["src/entities/trial/search/eligibility.rs",438,0,4,438,0,4],
["src/entities/trial/search/eligibility/keep_paths.rs",134,0,0,134,0,0],
["src/entities/trial/search/eligibility/tests.rs",334,334,0,334,334,0],
["src/entities/trial/search/essie.rs",316,0,109,316,0,109],
["src/entities/trial/search/mod.rs",416,0,0,416,0,0],
["src/entities/trial/search/nci.rs",247,0,2,266,0,2],
["src/entities/trial/search/nci/tests.rs",714,714,0,782,782,0],
["src/entities/trial/search/normalization.rs",235,0,12,235,0,12],
["src/entities/trial/search/normalization/tests.rs",211,211,0,211,211,0],
["src/entities/trial/test_support.rs",279,279,0,279,279,0],
["src/entities/variant/clinvar.rs",785,0,554,791,0,560],
["src/entities/variant/erepo.rs",1026,0,243,1026,0,243],
["src/entities/variant/get.rs",2026,0,19,2052,0,19],
["src/entities/variant/get/protein_change_tests.rs",1392,1392,0,1392,1392,0],
["src/entities/variant/get/tests.rs",1235,1235,0,1539,1539,0],
["src/entities/variant/gwas.rs",591,0,2,617,0,2],
["src/entities/variant/gwas/tests.rs",299,299,0,479,479,0],
["src/entities/variant/mod.rs",743,0,4,743,0,4],
["src/entities/variant/normalization.rs",429,0,61,429,0,61],
["src/entities/variant/resolution.rs",1436,0,2,1482,0,2],
["src/entities/variant/resolution/tests.rs",631,631,0,699,699,0],
["src/entities/variant/search/diagnostics.rs",377,0,0,377,0,0],
["src/entities/variant/search/mod.rs",1113,0,2,1113,0,2],
["src/entities/variant/search/tests.rs",874,874,0,874,874,0],
["src/entities/variant/structure.rs",514,0,95,518,0,2],
["src/entities/variant/structure/tests.rs",null,null,null,595,595,0],
["src/entities/variant/test_support.rs",4,4,0,4,4,0],
["src/error.rs",1479,0,493,1543,0,517],
["src/gene.rs",3,0,0,3,0,0],
["src/lib.rs",54,0,26,59,0,26],
["src/main.rs",220,0,0,229,0,0],
["src/main_biomcp_cli.rs",61,0,0,null,null,null],
["src/mcp/catalog.rs",102,0,0,119,0,0],
["src/mcp/mod.rs",49,0,0,46,0,0],
["src/mcp/shell.rs",2846,0,1071,2867,0,1073],
["src/mcp/shell/http_server.rs",614,0,127,614,0,127],
["src/mcp/shell/modern.rs",273,0,0,273,0,0],
["src/mcp/shell/pre_session.rs",153,0,0,153,0,0],
["src/mcp/shell/tests/ticket_1120.rs",117,117,0,117,117,0],
["src/mcp/shell/trial_phase.rs",54,0,37,54,0,37],
["src/mcp/shell/typed_get.rs",103,0,0,111,0,0],
["src/mcp/shell/typed_get_tests.rs",932,932,0,975,975,0],
["src/next_command.rs",103,0,17,103,0,17],
["src/render/chart.rs",2068,0,1087,2068,0,1087],
["src/render/chart/bounds.rs",64,0,0,64,0,0],
["src/render/human.rs",535,0,293,535,0,293],
["src/render/json.rs",1598,0,955,1598,0,955],
["src/render/markdown/adverse_event.rs",381,0,2,381,0,2],
["src/render/markdown/adverse_event/tests.rs",475,475,0,475,475,0],
["src/render/markdown/article.rs",789,0,2,789,0,2],
["src/render/markdown/article/tests.rs",1351,1351,0,1351,1351,0],
["src/render/markdown/author.rs",873,0,465,873,0,465],
["src/render/markdown/cell_line.rs",104,0,2,104,0,2],
["src/render/markdown/cell_line/tests.rs",194,194,0,194,194,0],
["src/render/markdown/diagnostic.rs",249,0,2,249,0,2],
["src/render/markdown/diagnostic/tests.rs",424,424,0,424,424,0],
["src/render/markdown/discovery.rs",184,0,2,184,0,2],
["src/render/markdown/discovery/tests.rs",176,176,0,176,176,0],
["src/render/markdown/disease.rs",406,0,2,406,0,2],
["src/render/markdown/disease/tests.rs",128,128,0,128,128,0],
["src/render/markdown/disease/tests/extended.rs",260,260,0,260,260,0],
["src/render/markdown/disease/tests/rendering.rs",918,918,0,918,918,0],
["src/render/markdown/drug.rs",731,0,2,731,0,2],
["src/render/markdown/drug/tests.rs",1149,1149,0,1149,1149,0],
["src/render/markdown/drug/tests/label_sections.rs",164,164,0,164,164,0],
["src/render/markdown/drug/tests/label_warnings.rs",196,196,0,196,196,0],
["src/render/markdown/drug_regulatory.rs",775,0,61,775,0,61],
["src/render/markdown/evidence.rs",494,0,2,494,0,2],
["src/render/markdown/evidence/tests.rs",306,306,0,306,306,0],
["src/render/markdown/funding.rs",85,0,0,85,0,0],
["src/render/markdown/gene.rs",131,0,2,131,0,2],
["src/render/markdown/gene/tests.rs",4,4,0,4,4,0],
["src/render/markdown/gene/tests/extended.rs",414,414,0,414,414,0],
["src/render/markdown/gene/tests/rendering.rs",829,829,0,829,829,0],
["src/render/markdown/mod.rs",932,0,15,922,0,15],
["src/render/markdown/pathway.rs",62,0,2,62,0,2],
["src/render/markdown/pathway/tests.rs",77,77,0,77,77,0],
["src/render/markdown/pgx.rs",78,0,2,78,0,2],
["src/render/markdown/pgx/tests.rs",90,90,0,90,90,0],
["src/render/markdown/pharmacodb.rs",55,0,0,55,0,0],
["src/render/markdown/protein.rs",156,0,2,156,0,2],
["src/render/markdown/protein/tests.rs",90,90,0,90,90,0],
["src/render/markdown/related.rs",1141,0,23,1141,0,23],
["src/render/markdown/related/article_support.rs",435,0,0,435,0,0],
["src/render/markdown/related/tests.rs",8,8,0,8,8,0],
["src/render/markdown/related/tests/article_search.rs",376,376,0,376,376,0],
["src/render/markdown/related/tests/diagnostic.rs",99,99,0,99,99,0],
["src/render/markdown/related/tests/disease.rs",468,468,0,468,468,0],
["src/render/markdown/related/tests/drug_variant_article_trial.rs",734,734,0,734,734,0],
["src/render/markdown/related/tests/gene_drug.rs",196,196,0,196,196,0],
["src/render/markdown/related/tests/misc.rs",126,126,0,126,126,0],
["src/render/markdown/root_tests.rs",708,708,0,708,708,0],
["src/render/markdown/sections.rs",670,0,2,670,0,2],
["src/render/markdown/sections/tests.rs",912,912,0,912,912,0],
["src/render/markdown/study.rs",481,0,2,481,0,2],
["src/render/markdown/study/tests.rs",429,429,0,429,429,0],
["src/render/markdown/support.rs",159,0,2,159,0,2],
["src/render/markdown/support/tests.rs",48,48,0,48,48,0],
["src/render/markdown/test_support.rs",25,25,0,25,25,0],
["src/render/markdown/tests.rs",5,5,0,5,5,0],
["src/render/markdown/trial.rs",472,0,2,472,0,2],
["src/render/markdown/trial/tests.rs",997,997,0,997,997,0],
["src/render/markdown/variant.rs",708,0,2,705,0,2],
["src/render/markdown/variant/tests.rs",1206,1206,0,1251,1251,0],
["src/render/mod.rs",7,0,0,7,0,0],
["src/render/provenance.rs",1916,0,1025,1916,0,1025],
["src/render/provenance/adverse_event.rs",101,0,0,101,0,0],
["src/skill_assets.rs",29,0,0,29,0,0],
["src/sources/alphagenome.rs",551,0,2,552,0,2],
["src/sources/alphagenome/tests/construction.rs",40,40,0,40,40,0],
["src/sources/alphagenome/tests/mod.rs",15,15,0,15,15,0],
["src/sources/alphagenome/tests/parsing.rs",69,69,0,69,69,0],
["src/sources/archive_budget.rs",467,0,283,467,0,283],
["src/sources/ca_bundle.rs",454,0,240,460,0,246],
["src/sources/cancerhotspots.rs",194,0,5,194,0,5],
["src/sources/cancerhotspots/tests/construction.rs",33,33,0,33,33,0],
["src/sources/cancerhotspots/tests/parsing.rs",178,178,0,178,178,0],
["src/sources/cbioportal.rs",330,0,2,330,0,2],
["src/sources/cbioportal/tests/construction.rs",69,69,0,69,69,0],
["src/sources/cbioportal/tests/live.rs",14,14,0,14,14,0],
["src/sources/cbioportal/tests/mod.rs",3,3,0,3,3,0],
["src/sources/cbioportal/tests/parsing.rs",85,85,0,85,85,0],
["src/sources/cbioportal_download.rs",556,0,2,559,0,2],
["src/sources/cbioportal_download/tests/construction.rs",81,81,0,81,81,0],
["src/sources/cbioportal_download/tests/download.rs",127,127,0,127,127,0],
["src/sources/cbioportal_download/tests/mod.rs",3,3,0,3,3,0],
["src/sources/cbioportal_download/tests/parsing.rs",338,338,0,338,338,0],
["src/sources/cbioportal_study.rs",3195,0,974,3195,0,974],
["src/sources/cbioportal_study/tests/dev2_contracts.rs",48,48,0,48,48,0],
["src/sources/cellosaurus.rs",507,0,2,507,0,2],
["src/sources/cellosaurus/tests.rs",124,124,0,124,124,0],
["src/sources/chembl.rs",374,0,21,374,0,21],
["src/sources/chembl/tests/construction.rs",78,78,0,78,78,0],
["src/sources/chembl/tests/parsing.rs",149,149,0,149,149,0],
["src/sources/civic.rs",526,0,2,526,0,2],
["src/sources/civic/tests/construction.rs",46,46,0,46,46,0],
["src/sources/civic/tests/mod.rs",2,2,0,2,2,0],
["src/sources/civic/tests/parsing.rs",120,120,0,120,120,0],
["src/sources/clingen.rs",812,0,36,812,0,36],
["src/sources/clingen/tests/mod.rs",422,422,0,422,422,0],
["src/sources/clingen/tests/parsing.rs",286,286,0,286,286,0],
["src/sources/clingen_allele_registry.rs",757,0,252,762,0,252],
["src/sources/clingen_cspec.rs",334,0,187,335,0,187],
["src/sources/clingen_erepo.rs",337,0,82,337,0,82],
["src/sources/clingen_ldh.rs",119,0,23,124,0,23],
["src/sources/clinicaltrials.rs",962,0,8,968,0,8],
["src/sources/clinicaltrials/tests/construction.rs",266,266,0,266,266,0],
["src/sources/clinicaltrials/tests/live.rs",18,18,0,18,18,0],
["src/sources/clinicaltrials/tests/mod.rs",3,3,0,3,3,0],
["src/sources/clinicaltrials/tests/parsing.rs",469,469,0,469,469,0],
["src/sources/complexportal.rs",237,0,2,237,0,2],
["src/sources/complexportal/tests/construction.rs",32,32,0,32,32,0],
["src/sources/complexportal/tests/mod.rs",2,2,0,2,2,0],
["src/sources/complexportal/tests/parsing.rs",89,89,0,89,89,0],
["src/sources/cpic.rs",485,0,40,485,0,40],
["src/sources/cpic/tests/construction.rs",76,76,0,76,76,0],
["src/sources/cpic/tests/mod.rs",2,2,0,2,2,0],
["src/sources/cpic/tests/parsing.rs",143,143,0,143,143,0],
["src/sources/cvx.rs",880,0,17,880,0,17],
["src/sources/cvx/tests/construction.rs",80,80,0,80,80,0],
["src/sources/cvx/tests/mod.rs",22,22,0,22,22,0],
["src/sources/cvx/tests/parsing.rs",212,212,0,212,212,0],
["src/sources/dbsnp.rs",208,0,0,208,0,0],
["src/sources/ddinter.rs",532,0,2,542,0,2],
["src/sources/ddinter/tests/construction.rs",111,111,0,111,111,0],
["src/sources/ddinter/tests/mod.rs",2,2,0,2,2,0],
["src/sources/ddinter/tests/parsing.rs",208,208,0,208,208,0],
["src/sources/dgidb.rs",391,0,2,391,0,2],
["src/sources/dgidb/tests/construction.rs",34,34,0,34,34,0],
["src/sources/dgidb/tests/mod.rs",2,2,0,2,2,0],
["src/sources/dgidb/tests/parsing.rs",79,79,0,79,79,0],
["src/sources/disable.rs",null,null,null,487,0,139],
["src/sources/disgenet.rs",466,0,2,466,0,2],
["src/sources/disgenet/tests/construction.rs",93,93,0,93,93,0],
["src/sources/disgenet/tests/mod.rs",154,154,0,154,154,0],
["src/sources/disgenet/tests/parsing.rs",178,178,0,178,178,0],
["src/sources/ema.rs",1003,0,8,1003,0,8],
["src/sources/ema/identity.rs",264,0,11,264,0,11],
["src/sources/ema/tests/mod.rs",103,103,0,103,103,0],
["src/sources/ema/tests/parsing.rs",416,416,0,416,416,0],
["src/sources/enrichr.rs",238,0,2,239,0,2],
["src/sources/enrichr/tests/construction.rs",30,30,0,30,30,0],
["src/sources/enrichr/tests/mod.rs",2,2,0,2,2,0],
["src/sources/enrichr/tests/parsing.rs",45,45,0,45,45,0],
["src/sources/europepmc.rs",616,0,2,616,0,2],
["src/sources/europepmc/tests/construction.rs",119,119,0,119,119,0],
["src/sources/europepmc/tests/live.rs",14,14,0,14,14,0],
["src/sources/europepmc/tests/mod.rs",3,3,0,3,3,0],
["src/sources/europepmc/tests/parsing.rs",222,222,0,238,238,0],
["src/sources/fda_orphan.rs",1006,0,346,1042,0,346],
["src/sources/figshare.rs",513,0,2,515,0,2],
["src/sources/figshare/tests/construction.rs",126,126,0,126,126,0],
["src/sources/figshare/tests/mod.rs",52,52,0,52,52,0],
["src/sources/figshare/tests/parsing.rs",135,135,0,135,135,0],
["src/sources/gencc.rs",1016,0,237,1021,0,237],
["src/sources/gencc/model.rs",396,0,8,396,0,8],
["src/sources/gencc/store.rs",1208,0,8,1211,0,8],
["src/sources/gencc/tests.rs",1189,1189,0,1188,1188,0],
["src/sources/gnomad.rs",359,0,2,359,0,2],
["src/sources/gnomad/tests/construction.rs",75,75,0,75,75,0],
["src/sources/gnomad/tests/mod.rs",2,2,0,2,2,0],
["src/sources/gnomad/tests/parsing.rs",167,167,0,167,167,0],
["src/sources/gprofiler.rs",235,0,2,236,0,2],
["src/sources/gprofiler/tests/construction.rs",34,34,0,34,34,0],
["src/sources/gprofiler/tests/mod.rs",2,2,0,2,2,0],
["src/sources/gprofiler/tests/parsing.rs",95,95,0,95,95,0],
["src/sources/gtex.rs",280,0,2,280,0,2],
["src/sources/gtex/tests/construction.rs",39,39,0,39,39,0],
["src/sources/gtex/tests/mod.rs",2,2,0,2,2,0],
["src/sources/gtex/tests/parsing.rs",94,94,0,94,94,0],
["src/sources/gtr.rs",791,0,18,791,0,18],
["src/sources/gtr/tests/construction.rs",110,110,0,110,110,0],
["src/sources/gtr/tests/mod.rs",2,2,0,2,2,0],
["src/sources/gtr/tests/parsing.rs",306,306,0,306,306,0],
["src/sources/gwas.rs",528,0,2,569,0,2],
["src/sources/gwas/tests/construction.rs",58,58,0,58,58,0],
["src/sources/gwas/tests/day_cache.rs",null,null,null,360,360,0],
["src/sources/gwas/tests/mod.rs",4,4,0,4,4,0],
["src/sources/gwas/tests/no_store.rs",35,35,0,null,null,null],
["src/sources/gwas/tests/parsing.rs",133,133,0,133,133,0],
["src/sources/gwas/tests/retired_endpoint.rs",114,114,0,114,114,0],
["src/sources/hpa.rs",661,0,2,661,0,2],
["src/sources/hpa/tests/construction.rs",53,53,0,53,53,0],
["src/sources/hpa/tests/mod.rs",42,42,0,42,42,0],
["src/sources/hpa/tests/parsing.rs",87,87,0,87,87,0],
["src/sources/hpo.rs",310,0,20,310,0,20],
["src/sources/hpo/tests/construction.rs",60,60,0,60,60,0],
["src/sources/hpo/tests/mod.rs",2,2,0,2,2,0],
["src/sources/hpo/tests/parsing.rs",140,140,0,167,167,0],
["src/sources/interpro.rs",197,0,2,197,0,2],
["src/sources/interpro/tests/construction.rs",23,23,0,23,23,0],
["src/sources/interpro/tests/mod.rs",2,2,0,2,2,0],
["src/sources/interpro/tests/parsing.rs",26,26,0,26,26,0],
["src/sources/kegg.rs",332,0,2,332,0,2],
["src/sources/kegg/tests/construction.rs",25,25,0,25,25,0],
["src/sources/kegg/tests/mod.rs",2,2,0,2,2,0],
["src/sources/kegg/tests/parsing.rs",74,74,0,74,74,0],
["src/sources/litsense2.rs",183,0,2,183,0,2],
["src/sources/litsense2/tests/construction.rs",71,71,0,71,71,0],
["src/sources/litsense2/tests/live.rs",14,14,0,14,14,0],
["src/sources/litsense2/tests/mod.rs",3,3,0,3,3,0],
["src/sources/litsense2/tests/parsing.rs",107,107,0,107,107,0],
["src/sources/litvar.rs",null,null,null,215,0,2],
["src/sources/litvar/tests/construction.rs",null,null,null,56,56,0],
["src/sources/litvar/tests/live.rs",null,null,null,26,26,0],
["src/sources/litvar/tests/mod.rs",null,null,null,3,3,0],
["src/sources/litvar/tests/parsing.rs",null,null,null,148,148,0],
["src/sources/medlineplus.rs",235,0,2,235,0,2],
["src/sources/medlineplus/tests/construction.rs",56,56,0,56,56,0],
["src/sources/medlineplus/tests/mod.rs",28,28,0,28,28,0],
["src/sources/medlineplus/tests/parsing.rs",85,85,0,85,85,0],
["src/sources/mod.rs",2826,0,1019,3073,0,1145],
["src/sources/monarch.rs",748,0,2,748,0,2],
["src/sources/monarch/tests/construction.rs",95,95,0,95,95,0],
["src/sources/monarch/tests/mod.rs",2,2,0,2,2,0],
["src/sources/monarch/tests/parsing.rs",333,333,0,333,333,0],
["src/sources/mutalyzer.rs",261,0,2,261,0,2],
["src/sources/mutalyzer/tests/construction.rs",38,38,0,38,38,0],
["src/sources/mutalyzer/tests/mod.rs",2,2,0,2,2,0],
["src/sources/mutalyzer/tests/parsing.rs",109,109,0,109,109,0],
["src/sources/mychem.rs",327,0,2,336,0,2],
["src/sources/mychem/tests/construction.rs",63,63,0,63,63,0],
["src/sources/mychem/tests/live.rs",14,14,0,14,14,0],
["src/sources/mychem/tests/mod.rs",3,3,0,3,3,0],
["src/sources/mychem/tests/parsing.rs",257,257,0,257,257,0],
["src/sources/mydisease.rs",493,0,9,493,0,9],
["src/sources/mydisease/tests/construction.rs",160,160,0,160,160,0],
["src/sources/mydisease/tests/live.rs",25,25,0,25,25,0],
["src/sources/mydisease/tests/mod.rs",3,3,0,3,3,0],
["src/sources/mydisease/tests/parsing.rs",109,109,0,109,109,0],
["src/sources/mygene.rs",888,0,353,888,0,353],
["src/sources/mygene/tests/live.rs",58,58,0,58,58,0],
["src/sources/myvariant.rs",1610,0,23,1610,0,23],
["src/sources/myvariant/filter_conflicts.rs",47,0,0,47,0,0],
["src/sources/myvariant/filter_keys.rs",19,0,0,19,0,0],
["src/sources/myvariant/tests/construction.rs",915,915,0,915,915,0],
["src/sources/myvariant/tests/live.rs",87,87,0,87,87,0],
["src/sources/myvariant/tests/mod.rs",10,10,0,10,10,0],
["src/sources/myvariant/tests/parsing.rs",631,631,0,631,631,0],
["src/sources/ncbi_efetch.rs",1322,0,428,1322,0,428],
["src/sources/ncbi_efetch/tests/construction.rs",60,60,0,60,60,0],
["src/sources/ncbi_efetch/tests/live.rs",15,15,0,15,15,0],
["src/sources/ncbi_efetch/tests/mod.rs",3,3,0,3,3,0],
["src/sources/ncbi_efetch/tests/parsing.rs",85,85,0,85,85,0],
["src/sources/ncbi_idconv.rs",174,0,2,174,0,2],
["src/sources/ncbi_idconv/tests/construction.rs",64,64,0,64,64,0],
["src/sources/ncbi_idconv/tests/live.rs",13,13,0,13,13,0],
["src/sources/ncbi_idconv/tests/mod.rs",3,3,0,3,3,0],
["src/sources/ncbi_idconv/tests/parsing.rs",76,76,0,76,76,0],
["src/sources/nci_cts.rs",672,0,2,672,0,2],
["src/sources/nci_cts/tests/construction.rs",167,167,0,167,167,0],
["src/sources/nci_cts/tests/live.rs",48,48,0,48,48,0],
["src/sources/nci_cts/tests/mod.rs",9,9,0,9,9,0],
["src/sources/nci_cts/tests/parsing.rs",247,247,0,247,247,0],
["src/sources/nih_reporter.rs",353,0,2,353,0,2],
["src/sources/nih_reporter/tests/construction.rs",75,75,0,75,75,0],
["src/sources/nih_reporter/tests/live.rs",14,14,0,14,14,0],
["src/sources/nih_reporter/tests/mod.rs",3,3,0,3,3,0],
["src/sources/nih_reporter/tests/parsing.rs",212,212,0,212,212,0],
["src/sources/ols4.rs",192,0,9,192,0,9],
["src/sources/ols4/tests/construction.rs",37,37,0,37,37,0],
["src/sources/ols4/tests/mod.rs",2,2,0,2,2,0],
["src/sources/ols4/tests/parsing.rs",59,59,0,59,59,0],
["src/sources/oncokb.rs",219,0,2,219,0,2],
["src/sources/oncokb/tests/construction.rs",56,56,0,56,56,0],
["src/sources/oncokb/tests/mod.rs",2,2,0,2,2,0],
["src/sources/oncokb/tests/parsing.rs",60,60,0,60,60,0],
["src/sources/opencitations.rs",391,0,267,388,0,267],
["src/sources/openfda.rs",846,0,7,864,0,9],
["src/sources/openfda/tests/construction.rs",179,179,0,179,179,0],
["src/sources/openfda/tests/day_cache.rs",null,null,null,320,320,0],
["src/sources/openfda/tests/parsing.rs",122,122,0,122,122,0],
["src/sources/opentargets.rs",2088,0,639,2088,0,639],
["src/sources/orcid.rs",697,0,2,710,0,2],
["src/sources/orcid/tests.rs",630,630,0,772,772,0],
["src/sources/ordinary_url_policy.rs",427,0,100,427,0,100],
["src/sources/pharmacodb.rs",679,0,6,685,0,6],
["src/sources/pharmacodb/tests/construction.rs",154,154,0,154,154,0],
["src/sources/pharmacodb/tests/limits.rs",128,128,0,128,128,0],
["src/sources/pharmacodb/tests/parsing.rs",232,232,0,232,232,0],
["src/sources/pharmgkb.rs",367,0,16,367,0,16],
["src/sources/pharmgkb/tests/construction.rs",68,68,0,68,68,0],
["src/sources/pharmgkb/tests/mod.rs",2,2,0,2,2,0],
["src/sources/pharmgkb/tests/parsing.rs",123,123,0,123,123,0],
["src/sources/pmc_article.rs",854,0,350,899,0,368],
["src/sources/pmc_oa.rs",521,0,98,528,0,98],
["src/sources/pmc_oa/tests/construction.rs",37,37,0,37,37,0],
["src/sources/pmc_oa/tests/live.rs",25,25,0,25,25,0],
["src/sources/pmc_oa/tests/mod.rs",3,3,0,3,3,0],
["src/sources/pmc_oa/tests/parsing.rs",293,293,0,293,293,0],
["src/sources/provider_url_policy.rs",978,0,360,1118,0,395],
["src/sources/pubmed.rs",744,0,2,744,0,2],
["src/sources/pubmed/tests/construction.rs",113,113,0,113,113,0],
["src/sources/pubmed/tests/live.rs",20,20,0,20,20,0],
["src/sources/pubmed/tests/mod.rs",3,3,0,3,3,0],
["src/sources/pubmed/tests/parsing.rs",493,493,0,493,493,0],
["src/sources/pubtator.rs",463,0,2,498,0,2],
["src/sources/pubtator/tests/construction.rs",126,126,0,126,126,0],
["src/sources/pubtator/tests/live.rs",17,17,0,17,17,0],
["src/sources/pubtator/tests/mod.rs",3,3,0,3,3,0],
["src/sources/pubtator/tests/parsing.rs",246,246,0,285,285,0],
["src/sources/quickgo.rs",155,0,2,155,0,2],
["src/sources/quickgo/tests/construction.rs",35,35,0,35,35,0],
["src/sources/quickgo/tests/mod.rs",2,2,0,2,2,0],
["src/sources/quickgo/tests/parsing.rs",46,46,0,46,46,0],
["src/sources/rate_limit.rs",734,0,378,747,0,378],
["src/sources/reactome.rs",390,0,2,390,0,2],
["src/sources/reactome/tests/construction.rs",61,61,0,61,61,0],
["src/sources/reactome/tests/mod.rs",2,2,0,2,2,0],
["src/sources/reactome/tests/parsing.rs",61,61,0,61,61,0],
["src/sources/request_plan_transport.rs",220,0,0,220,0,0],
["src/sources/seer.rs",697,0,13,697,0,13],
["src/sources/seer/tests/construction.rs",27,27,0,27,27,0],
["src/sources/seer/tests/mod.rs",2,2,0,2,2,0],
["src/sources/seer/tests/parsing.rs",198,198,0,198,198,0],
["src/sources/semantic_scholar.rs",1048,0,54,1053,0,54],
["src/sources/semantic_scholar/tests/construction.rs",633,633,0,633,633,0],
["src/sources/semantic_scholar/tests/live.rs",14,14,0,14,14,0],
["src/sources/semantic_scholar/tests/mod.rs",3,3,0,3,3,0],
["src/sources/semantic_scholar/tests/parsing.rs",382,382,0,382,382,0],
["src/sources/string.rs",109,0,2,109,0,2],
["src/sources/string/tests/construction.rs",22,22,0,22,22,0],
["src/sources/string/tests/mod.rs",2,2,0,2,2,0],
["src/sources/string/tests/parsing.rs",31,31,0,31,31,0],
["src/sources/tests/clingen_runtime.rs",204,204,0,204,204,0],
["src/sources/tests/provider_network.rs",1006,1006,0,1466,1466,0],
["src/sources/trust_failure_tests/mod.rs",130,0,0,130,0,0],
["src/sources/umls.rs",254,0,2,254,0,2],
["src/sources/umls/tests/construction.rs",32,32,0,32,32,0],
["src/sources/umls/tests/mod.rs",2,2,0,2,2,0],
["src/sources/umls/tests/parsing.rs",105,105,0,105,105,0],
["src/sources/uniprot.rs",794,0,2,794,0,2],
["src/sources/uniprot/tests/construction.rs",74,74,0,74,74,0],
["src/sources/uniprot/tests/mod.rs",18,18,0,18,18,0],
["src/sources/uniprot/tests/parsing.rs",380,380,0,380,380,0],
["src/sources/vaers.rs",462,0,2,465,0,2],
["src/sources/vaers/tests/construction.rs",67,67,0,67,67,0],
["src/sources/vaers/tests/mod.rs",39,39,0,39,39,0],
["src/sources/vaers/tests/parsing.rs",106,106,0,106,106,0],
["src/sources/variantvalidator.rs",275,0,2,275,0,2],
["src/sources/variantvalidator/tests/construction.rs",44,44,0,44,44,0],
["src/sources/variantvalidator/tests/mod.rs",2,2,0,2,2,0],
["src/sources/variantvalidator/tests/parsing.rs",115,115,0,115,115,0],
["src/sources/who_ivd.rs",419,0,17,419,0,17],
["src/sources/who_ivd/tests/construction.rs",112,112,0,112,112,0],
["src/sources/who_ivd/tests/mod.rs",15,15,0,15,15,0],
["src/sources/who_ivd/tests/parsing.rs",71,71,0,71,71,0],
["src/sources/who_pq.rs",1299,0,2,1299,0,2],
["src/sources/who_pq/tests/construction.rs",150,150,0,150,150,0],
["src/sources/who_pq/tests/mod.rs",38,38,0,38,38,0],
["src/sources/who_pq/tests/parsing.rs",398,398,0,398,398,0],
["src/sources/who_pq/tests/sync.rs",310,310,0,310,310,0],
["src/sources/wikipathways.rs",432,0,2,432,0,2],
["src/sources/wikipathways/tests/construction.rs",40,40,0,40,40,0],
["src/sources/wikipathways/tests/mod.rs",2,2,0,2,2,0],
["src/sources/wikipathways/tests/parsing.rs",89,89,0,89,89,0],
["src/test_support.rs",266,266,0,306,306,0],
["src/transform/adverse_event.rs",592,0,115,592,0,115],
["src/transform/article.rs",122,0,43,127,0,43],
["src/transform/article/anchors.rs",91,0,2,91,0,2],
["src/transform/article/anchors/tests.rs",31,31,0,31,31,0],
["src/transform/article/annotations.rs",515,0,2,515,0,2],
["src/transform/article/annotations/tests.rs",323,323,0,323,323,0],
["src/transform/article/annotations/tests/mutation_identity.rs",684,684,0,684,684,0],
["src/transform/article/federation.rs",544,0,2,544,0,2],
["src/transform/article/federation/tests.rs",420,420,0,420,420,0],
["src/transform/article/html.rs",564,0,206,566,0,206],
["src/transform/article/jats.rs",677,0,2,677,0,2],
["src/transform/article/jats/refs.rs",546,0,0,546,0,0],
["src/transform/article/jats/refs/citation.rs",608,0,0,608,0,0],
["src/transform/article/jats/supplements.rs",179,0,28,179,0,28],
["src/transform/article/jats/tables.rs",134,0,0,134,0,0],
["src/transform/article/jats/tests.rs",615,615,0,615,615,0],
["src/transform/article/jats/tests/citation_evidence.rs",553,553,0,553,553,0],
["src/transform/article/pdf.rs",73,0,41,73,0,41],
["src/transform/article/supplement_text.rs",null,null,null,696,0,2],
["src/transform/article/supplement_text/tests.rs",null,null,null,291,291,0],
["src/transform/disease.rs",1154,0,318,1154,0,318],
["src/transform/drug.rs",1698,0,445,1699,0,450],
["src/transform/drug/name_resolution_tests.rs",891,891,0,891,891,0],
["src/transform/drug/shared_name_tests.rs",304,304,0,304,304,0],
["src/transform/gene.rs",360,0,130,384,0,144],
["src/transform/mod.rs",11,0,0,11,0,0],
["src/transform/pathway.rs",205,0,128,205,0,128],
["src/transform/protein.rs",216,0,151,216,0,151],
["src/transform/trial.rs",459,0,3,459,0,3],
["src/transform/trial/tests.rs",673,673,0,673,673,0],
["src/transform/trial/tests/ticket_1107.rs",61,61,0,61,61,0],
["src/transform/trial/tests/ticket_1111.rs",86,86,0,86,86,0],
["src/transform/trial/tests/ticket_1114.rs",122,122,0,122,122,0],
["src/transform/trial/tests/ticket_1115.rs",42,42,0,42,42,0],
["src/transform/variant.rs",1251,0,2,1264,0,2],
["src/transform/variant/tests.rs",446,446,0,446,446,0],
["src/utils/date.rs",225,0,98,225,0,98],
["src/utils/download.rs",469,0,241,469,0,241],
["src/utils/fs_lock.rs",null,null,null,25,0,0],
["src/utils/mod.rs",39,0,18,41,0,18],
["src/utils/query.rs",29,0,10,29,0,10],
["src/utils/serde.rs",46,0,17,46,0,17],
["src/utils/sync.rs",37,0,31,37,0,31],
["src/workflow_ladders.rs",152,0,24,152,0,24],
["src/xml.rs",240,0,122,240,0,122]
],"external_rust_tests":[
["tests/adverse_event_route_contract.rs",453,453],
["tests/article_asset_terminal.rs",280,949],
["tests/article_batch_outcome.rs",1181,1527],
["tests/article_cli_tests_structure.rs",123,null],
["tests/article_transform_structure.rs",127,129],
["tests/article_usage_stderr.rs",360,363],
["tests/benchmark_cli_structure.rs",291,null],
["tests/biomcp_cli_shim.rs",68,null],
["tests/cache_clear.rs",331,333],
["tests/cli_global_modifiers.rs",61,61],
["tests/cli_line_cap_absorption.rs",128,null],
["tests/disable_sources_cli.rs",null,872],
["tests/fulltext_fixture_output_location.rs",null,43],
["tests/health_cli_structure.rs",138,139],
["tests/json_error_contract.rs",1117,1156],
["tests/legacy_cache_helper_cleanup.rs",66,66],
["tests/list_cli_structure.rs",137,137],
["tests/live_smoke.rs",80,80],
["tests/managed_state_permissions.rs",425,428],
["tests/package_build_identity.rs",9,9],
["tests/pgx_get_paging.rs",61,61],
["tests/plugin_manifest.rs",53,53],
["tests/pty_helpers.rs",33,35],
["tests/rmcp_client_contract.rs",1726,2333],
["tests/search_all_cli_structure.rs",138,138],
["tests/skill_cli_structure.rs",138,138],
["tests/tls_ca_bundle_contract.rs",811,814],
["tests/unit/cli/article_assets.rs",54,54],
["tests/unit/cli/article_fulltext.rs",91,91],
["tests/unit/cli/drug_json.rs",521,521],
["tests/unit/cli/gene.rs",179,179],
["tests/unit/cli/variant.rs",1489,1489],
["tests/unit/cli/variant/articles.rs",87,87],
["tests/unit/cli/variant/parsing.rs",86,86],
["tests/unit/render/json_error.rs",36,36],
["tests/windows_lock_contract.rs",null,34],
["tests/windows_stdio_contract.rs",143,143]
],"python_tests":[
["tests/conftest.py",65,130],
["tests/golden/conftest.py",null,34],
["tests/golden/replay_env.py",null,209],
["tests/golden/spec/test_cleanup_cli_behaviors.py",null,40],
["tests/golden/spec/test_spec_page_golden.py",null,109],
["tests/golden/test_golden_output.py",null,153],
["tests/support.py",57,62],
["tests/surface/test_architecture_docs_parity_contract.py",159,105],
["tests/surface/test_biomcp_ci_path_contract.py",80,80],
["tests/surface/test_biomcp_verify_live.py",252,252],
["tests/surface/test_complexportal_fixture_contract.py",28,35],
["tests/surface/test_mcp_live_gate_contract.py",44,47],
["tests/surface/test_parallel_isolation_contract.py",2067,null],
["tests/surface/test_public_entity_contract_docs.py",53,53],
["tests/surface/test_search_all_cli_structure.py",80,80],
["tests/surface/test_semantic_scholar_retry_after_contract.py",288,288],
["tests/surface/test_source_configuration_docs_contract.py",196,200],
["tests/surface/test_ticket_401_surface_ratchets.py",138,142],
["tests/surface/test_ticket_405_architecture_operator_contracts.py",200,199],
["tests/surface/test_trial_help_contract.py",213,213],
["tests/surface/test_variant_normalization_docs_contract.py",112,112],
["tests/test_alphagenome_proto_generation.py",133,134],
["tests/test_article_spec_fixture_lifecycle.py",947,null],
["tests/test_author_surface_docs.py",843,875],
["tests/test_binary_asset_gitattributes_contract.py",57,71],
["tests/test_bioasq_benchmark_contract.py",168,168],
["tests/test_bioasq_ingest.py",285,285],
["tests/test_biomcp_ci_contract.py",242,247],
["tests/test_biomcp_cli_shim_contract.py",30,null],
["tests/test_build_identity_rebuild.py",148,148],
["tests/test_build_identity_wrapper_contract.py",91,98],
["tests/test_capture_receipts.py",1316,1317],
["tests/test_cbioportal_study_download_timeout_contract.py",29,29],
["tests/test_central_test_support_helpers.py",174,171],
["tests/test_changelog_coverage_check.py",685,1163],
["tests/test_ci_classify_push_script.py",208,539],
["tests/test_ci_workflow_contract.py",305,612],
["tests/test_citation_contract.py",108,113],
["tests/test_cli_surface_contract_ratchet.py",380,383],
["tests/test_cross_entity_pivots_docs_contract.py",30,30],
["tests/test_ctgov_spec_fixture_lifecycle.py",171,null],
["tests/test_ctgov_trial_search_detail_reuse.py",503,507],
["tests/test_directory_submission_contract.py",354,357],
["tests/test_disable_sources_replay_walk.py",null,293],
["tests/test_disease_survival_fixture_lifecycle.py",384,null],
["tests/test_docs_changelog_refresh.py",1075,1145],
["tests/test_docs_live_revision_gate.py",85,85],
["tests/test_docs_publication_contract.py",509,566],
["tests/test_documentation_consistency_audit_contract.py",738,738],
["tests/test_documentation_toolchain_contract.py",79,79],
["tests/test_gencc_docs_contract.py",163,163],
["tests/test_gwas_day_cache_contract.py",null,136],
["tests/test_gwas_no_store_contract.py",49,null],
["tests/test_http_cache_fail_soft.py",null,229],
["tests/test_json_error_fixture_lifecycle.py",33,null],
["tests/test_json_list_contract.py",66,66],
["tests/test_lint.py",419,541],
["tests/test_llms_txt_contract.py",106,106],
["tests/test_markdown_twins_contract.py",70,70],
["tests/test_mcp_2026_protocol.py",487,487],
["tests/test_mcp_http_surface.py",537,501],
["tests/test_mcp_stdio_guidance.py",147,147],
["tests/test_mcp_tool_catalog.py",326,326],
["tests/test_measurement_coverage_contract.py",null,116],
["tests/test_measurement_summary_contract.py",null,157],
["tests/test_merge_conflict_markers.py",null,75],
["tests/test_mychem_empty_spec_fixture_contract.py",101,null],
["tests/test_nci_filter_transport.py",212,212],
["tests/test_no_cache_managed_state.py",188,188],
["tests/test_no_climb_tracked_script.py",81,81],
["tests/test_offline_gate_contract.py",414,458],
["tests/test_output_footprint_benchmark.py",91,91],
["tests/test_png_artifact_contract.py",35,35],
["tests/test_prepare_spec_artifacts.py",104,104],
["tests/test_provider_contract_fixture.py",154,154],
["tests/test_provider_network_policy.py",176,187],
["tests/test_public_example_accessions.py",186,186],
["tests/test_public_install_docs_contract.py",152,179],
["tests/test_public_installer_checksum.py",316,464],
["tests/test_public_landing_copy_docs_contract.py",134,134],
["tests/test_public_search_all_docs_contract.py",224,224],
["tests/test_public_skill_docs_contract.py",543,537],
["tests/test_python_contract_parallelism.py",37,94],
["tests/test_python_lane_cache_isolation.py",null,43],
["tests/test_quality_ratchet_contract.py",1961,1734],
["tests/test_release_candidate.py",427,427],
["tests/test_release_container.py",179,179],
["tests/test_release_draft_notes.py",null,86],
["tests/test_release_homebrew.py",84,84],
["tests/test_release_mcpb.py",585,585],
["tests/test_release_metadata_contract.py",300,303],
["tests/test_release_notes.py",145,145],
["tests/test_release_package.py",639,607],
["tests/test_release_promotion.py",733,733],
["tests/test_release_signing.py",226,226],
["tests/test_release_smoke_script.py",235,261],
["tests/test_release_targets.py",85,85],
["tests/test_release_versions_check.py",73,139],
["tests/test_release_workflow_provenance.py",2246,2282],
["tests/test_routine_cargo_feature_contract.py",120,284],
["tests/test_routine_fixture_recovery.py",901,null],
["tests/test_sdlc_clean_contract.py",111,111],
["tests/test_sdlc_project_lifecycle_contract.py",1097,1097],
["tests/test_sdlc_review_status_contract.py",636,null],
["tests/test_shell_workflow_lint_contract.py",46,46],
["tests/test_short_unix_socket_contract.py",40,21],
["tests/test_should_move_latest.py",77,77],
["tests/test_skill_prompt_contract.py",352,352],
["tests/test_slow_test_reporting.py",null,117],
["tests/test_small_rust_test_boundaries.py",39,39],
["tests/test_source_child_stdio_guard.py",677,667],
["tests/test_source_external_xml_guard.py",282,282],
["tests/test_source_licensing_docs_contract.py",564,576],
["tests/test_source_package_boundary.py",207,219],
["tests/test_source_pages_docs_contract.py",1079,1103],
["tests/test_streamable_http_demo.py",95,95],
["tests/test_stress_lane_contract.py",78,95],
["tests/test_test_wait_ratchet_check.py",477,null],
["tests/test_update_command_docs_contract.py",58,58],
["tests/test_update_rust_source_size_inventory.py",218,null],
["tests/test_upstream_planning_analysis_docs.py",1619,1622],
["tests/test_variant_article_live_canary.py",503,503],
["tests/test_variant_normalize_json_contract.py",69,69],
["tests/test_version_sync_script.py",377,430],
["tests/test_wheel_glibc_floor_check.py",218,218],
["tests/test_zero_coupling_ratchet.py",461,462]
]}
```

#### Durable per-file coverage reports

Rows retain the sanitized original report columns without fixed-width padding. Each row is `[filename, outside_in_columns, unit_columns, combined_columns]`; null means absent. The report column order is `["regions", "missed_regions", "region_coverage", "functions", "missed_functions", "functions_executed", "lines", "missed_lines", "line_coverage", "branches", "missed_branches", "branch_coverage"]`. All three report denominators and the sum of their file denominators equal 145,197. Original report SHA-256 values: `{"combined": "c3f9b424a72a65f8f8b6f17766767ae9f3f202bade051eabcad19827903d957b", "outside_in": "27a676826a2d2cd801adb26ad005f7f1e126e9e78b8bdea7ce76a13b04922b6b", "unit": "f219e03d1f55f0c02e75cb04629ec251299995dbbfcf19b3fc0720e0a235a1c9"}`. Individual report exits remain unrecorded as stated above.

```json
[
["<comparison>/candidate/product/src/build_identity.rs",["10","0","100.00%","2","0","100.00%","6","0","100.00%","0","0","-"],["10","4","60.00%","2","1","50.00%","6","3","50.00%","0","0","-"],["10","0","100.00%","2","0","100.00%","6","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cache/citation_evidence.rs",["177","177","0.00%","13","13","0.00%","121","121","0.00%","0","0","-"],["177","12","93.22%","13","0","100.00%","121","5","95.87%","0","0","-"],["177","12","93.22%","13","0","100.00%","121","5","95.87%","0","0","-"]],
["<comparison>/candidate/product/src/cache/clean.rs",["676","593","12.28%","40","33","17.50%","521","420","19.39%","0","0","-"],["676","46","93.20%","40","4","90.00%","521","19","96.35%","0","0","-"],["676","46","93.20%","40","4","90.00%","521","19","96.35%","0","0","-"]],
["<comparison>/candidate/product/src/cache/clear.rs",["1331","937","29.60%","91","58","36.26%","732","481","34.29%","0","0","-"],["1331","139","89.56%","91","5","94.51%","732","66","90.98%","0","0","-"],["1331","139","89.56%","91","5","94.51%","732","66","90.98%","0","0","-"]],
["<comparison>/candidate/product/src/cache/config.rs",["884","694","21.49%","65","47","27.69%","642","499","22.27%","0","0","-"],["884","17","98.08%","65","1","98.46%","642","12","98.13%","0","0","-"],["884","16","98.19%","65","1","98.46%","642","10","98.44%","0","0","-"]],
["<comparison>/candidate/product/src/cache/http.rs",["43","5","88.37%","7","2","71.43%","28","2","92.86%","0","0","-"],["43","1","97.67%","7","0","100.00%","28","0","100.00%","0","0","-"],["43","0","100.00%","7","0","100.00%","28","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cache/limits.rs",["252","201","20.24%","15","10","33.33%","167","126","24.55%","0","0","-"],["252","20","92.06%","15","2","86.67%","167","21","87.43%","0","0","-"],["252","20","92.06%","15","2","86.67%","167","21","87.43%","0","0","-"]],
["<comparison>/candidate/product/src/cache/manager.rs",["1356","954","29.65%","110","74","32.73%","919","647","29.60%","0","0","-"],["1356","126","90.71%","110","12","89.09%","919","67","92.71%","0","0","-"],["1356","123","90.93%","110","12","89.09%","919","67","92.71%","0","0","-"]],
["<comparison>/candidate/product/src/cache/migration.rs",["1563","1109","29.05%","91","51","43.96%","899","577","35.82%","0","0","-"],["1563","178","88.61%","91","4","95.60%","899","78","91.32%","0","0","-"],["1563","176","88.74%","91","3","96.70%","899","77","91.43%","0","0","-"]],
["<comparison>/candidate/product/src/cache/mod.rs",["68","25","63.24%","7","1","85.71%","47","16","65.96%","0","0","-"],["68","1","98.53%","7","0","100.00%","47","1","97.87%","0","0","-"],["68","1","98.53%","7","0","100.00%","47","1","97.87%","0","0","-"]],
["<comparison>/candidate/product/src/cache/planner.rs",["1139","964","15.36%","68","56","17.65%","707","553","21.78%","0","0","-"],["1139","70","93.85%","68","8","88.24%","707","46","93.49%","0","0","-"],["1139","70","93.85%","68","8","88.24%","707","46","93.49%","0","0","-"]],
["<comparison>/candidate/product/src/cache/private.rs",["699","337","51.79%","37","11","70.27%","380","146","61.58%","0","0","-"],["699","49","92.99%","37","4","89.19%","380","16","95.79%","0","0","-"],["699","49","92.99%","37","4","89.19%","380","16","95.79%","0","0","-"]],
["<comparison>/candidate/product/src/cache/provider_capture.rs",["1935","1600","17.31%","96","69","28.12%","1085","841","22.49%","0","0","-"],["1935","190","90.18%","96","5","94.79%","1085","55","94.93%","0","0","-"],["1935","190","90.18%","96","5","94.79%","1085","55","94.93%","0","0","-"]],
["<comparison>/candidate/product/src/cache/provider_capture/unsupported.rs",["26","26","0.00%","2","2","0.00%","12","12","0.00%","0","0","-"],["26","0","100.00%","2","0","100.00%","12","0","100.00%","0","0","-"],["26","0","100.00%","2","0","100.00%","12","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/adverse_event/dispatch.rs",["470","144","69.36%","9","0","100.00%","389","136","65.04%","0","0","-"],["470","437","7.02%","9","7","22.22%","389","355","8.74%","0","0","-"],["470","144","69.36%","9","0","100.00%","389","133","65.81%","0","0","-"]],
["<comparison>/candidate/product/src/cli/adverse_event/plan.rs",["248","88","64.52%","7","2","71.43%","171","53","69.01%","0","0","-"],["248","4","98.39%","7","0","100.00%","171","4","97.66%","0","0","-"],["248","4","98.39%","7","0","100.00%","171","4","97.66%","0","0","-"]],
["<comparison>/candidate/product/src/cli/article/annotations.rs",["69","69","0.00%","9","9","0.00%","50","50","0.00%","0","0","-"],["69","2","97.10%","9","0","100.00%","50","1","98.00%","0","0","-"],["69","2","97.10%","9","0","100.00%","50","1","98.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/article/assets.rs",["386","54","86.01%","29","3","89.66%","288","53","81.60%","0","0","-"],["386","282","26.94%","29","13","55.17%","288","202","29.86%","0","0","-"],["386","47","87.82%","29","1","96.55%","288","39","86.46%","0","0","-"]],
["<comparison>/candidate/product/src/cli/article/dispatch.rs",["855","307","64.09%","44","24","45.45%","590","206","65.08%","0","0","-"],["855","330","61.40%","44","13","70.45%","590","192","67.46%","0","0","-"],["855","153","82.11%","44","8","81.82%","590","82","86.10%","0","0","-"]],
["<comparison>/candidate/product/src/cli/article/export.rs",["137","22","83.94%","10","3","70.00%","95","20","78.95%","0","0","-"],["137","137","0.00%","10","10","0.00%","95","95","0.00%","0","0","-"],["137","22","83.94%","10","3","70.00%","95","20","78.95%","0","0","-"]],
["<comparison>/candidate/product/src/cli/article/fulltext_view.rs",["409","261","36.19%","29","16","44.83%","284","173","39.08%","0","0","-"],["409","227","44.50%","29","15","48.28%","284","154","45.77%","0","0","-"],["409","107","73.84%","29","7","75.86%","284","70","75.35%","0","0","-"]],
["<comparison>/candidate/product/src/cli/article/mod.rs",["197","42","78.68%","29","13","55.17%","156","29","81.41%","0","0","-"],["197","12","93.91%","29","8","72.41%","156","10","93.59%","0","0","-"],["197","11","94.42%","29","7","75.86%","156","9","94.23%","0","0","-"]],
["<comparison>/candidate/product/src/cli/article/render.rs",["61","9","85.25%","2","0","100.00%","45","3","93.33%","0","0","-"],["61","9","85.25%","2","0","100.00%","45","3","93.33%","0","0","-"],["61","9","85.25%","2","0","100.00%","45","3","93.33%","0","0","-"]],
["<comparison>/candidate/product/src/cli/article/session.rs",["400","372","7.00%","32","28","12.50%","291","268","7.90%","0","0","-"],["400","38","90.50%","32","3","90.62%","291","23","92.10%","0","0","-"],["400","38","90.50%","32","3","90.62%","291","23","92.10%","0","0","-"]],
["<comparison>/candidate/product/src/cli/article/workflow.rs",["96","53","44.79%","16","8","50.00%","89","53","40.45%","0","0","-"],["96","4","95.83%","16","0","100.00%","89","10","88.76%","0","0","-"],["96","4","95.83%","16","0","100.00%","89","10","88.76%","0","0","-"]],
["<comparison>/candidate/product/src/cli/author/detail.rs",["20","2","90.00%","3","0","100.00%","17","0","100.00%","0","0","-"],["20","14","30.00%","3","2","33.33%","17","9","47.06%","0","0","-"],["20","2","90.00%","3","0","100.00%","17","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/author/mod.rs",["67","53","20.90%","4","2","50.00%","66","54","18.18%","0","0","-"],["67","15","77.61%","4","2","50.00%","66","13","80.30%","0","0","-"],["67","1","98.51%","4","0","100.00%","66","1","98.48%","0","0","-"]],
["<comparison>/candidate/product/src/cli/author/papers.rs",["34","3","91.18%","2","0","100.00%","20","0","100.00%","0","0","-"],["34","34","0.00%","2","2","0.00%","20","20","0.00%","0","0","-"],["34","3","91.18%","2","0","100.00%","20","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/author/search.rs",["18","18","0.00%","2","2","0.00%","12","12","0.00%","0","0","-"],["18","18","0.00%","2","2","0.00%","12","12","0.00%","0","0","-"],["18","18","0.00%","2","2","0.00%","12","12","0.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/cache.rs",["353","124","64.87%","30","19","36.67%","260","84","67.69%","0","0","-"],["353","199","43.63%","30","16","46.67%","260","122","53.08%","0","0","-"],["353","73","79.32%","30","11","63.33%","260","40","84.62%","0","0","-"]],
["<comparison>/candidate/product/src/cli/cell_line/dispatch.rs",["150","121","19.33%","8","4","50.00%","99","74","25.25%","0","0","-"],["150","150","0.00%","8","8","0.00%","99","99","0.00%","0","0","-"],["150","121","19.33%","8","4","50.00%","99","74","25.25%","0","0","-"]],
["<comparison>/candidate/product/src/cli/chart.rs",["220","220","0.00%","18","18","0.00%","134","134","0.00%","0","0","-"],["220","13","94.09%","18","2","88.89%","134","4","97.01%","0","0","-"],["220","13","94.09%","18","2","88.89%","134","4","97.01%","0","0","-"]],
["<comparison>/candidate/product/src/cli/commands.rs",["4","1","75.00%","1","0","100.00%","3","0","100.00%","0","0","-"],["4","4","0.00%","1","1","0.00%","3","3","0.00%","0","0","-"],["4","1","75.00%","1","0","100.00%","3","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/diagnostic/dispatch.rs",["157","132","15.92%","9","6","33.33%","115","95","17.39%","0","0","-"],["157","81","48.41%","9","4","55.56%","115","48","58.26%","0","0","-"],["157","62","60.51%","9","2","77.78%","115","37","67.83%","0","0","-"]],
["<comparison>/candidate/product/src/cli/diagnostic/mod.rs",["6","6","0.00%","1","1","0.00%","6","6","0.00%","0","0","-"],["6","2","66.67%","1","0","100.00%","6","2","66.67%","0","0","-"],["6","2","66.67%","1","0","100.00%","6","2","66.67%","0","0","-"]],
["<comparison>/candidate/product/src/cli/discover.rs",["583","573","1.72%","43","41","4.65%","414","400","3.38%","0","0","-"],["583","47","91.94%","43","5","88.37%","414","25","93.96%","0","0","-"],["583","47","91.94%","43","5","88.37%","414","25","93.96%","0","0","-"]],
["<comparison>/candidate/product/src/cli/disease/dispatch.rs",["387","314","18.86%","14","8","42.86%","273","213","21.98%","0","0","-"],["387","203","47.55%","14","2","85.71%","273","123","54.95%","0","0","-"],["387","203","47.55%","14","2","85.71%","273","123","54.95%","0","0","-"]],
["<comparison>/candidate/product/src/cli/drug/dispatch.rs",["641","514","19.81%","17","8","52.94%","551","436","20.87%","0","0","-"],["641","510","20.44%","17","8","52.94%","551","427","22.50%","0","0","-"],["641","490","23.56%","17","6","64.71%","551","408","25.95%","0","0","-"]],
["<comparison>/candidate/product/src/cli/drug/render.rs",["242","209","13.64%","14","11","21.43%","226","187","17.26%","0","0","-"],["242","30","87.60%","14","3","78.57%","226","29","87.17%","0","0","-"],["242","30","87.60%","14","3","78.57%","226","29","87.17%","0","0","-"]],
["<comparison>/candidate/product/src/cli/drug/workflow.rs",["15","15","0.00%","1","1","0.00%","12","12","0.00%","0","0","-"],["15","15","0.00%","1","1","0.00%","12","12","0.00%","0","0","-"],["15","15","0.00%","1","1","0.00%","12","12","0.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/gene/cspec.rs",["297","282","5.05%","14","12","14.29%","198","181","8.59%","0","0","-"],["297","297","0.00%","14","14","0.00%","198","198","0.00%","0","0","-"],["297","282","5.05%","14","12","14.29%","198","181","8.59%","0","0","-"]],
["<comparison>/candidate/product/src/cli/gene/dispatch.rs",["197","42","78.68%","12","1","91.67%","153","31","79.74%","0","0","-"],["197","133","32.49%","12","5","58.33%","153","92","39.87%","0","0","-"],["197","42","78.68%","12","1","91.67%","153","31","79.74%","0","0","-"]],
["<comparison>/candidate/product/src/cli/gene/related.rs",["261","261","0.00%","4","4","0.00%","159","159","0.00%","0","0","-"],["261","215","17.62%","4","2","50.00%","159","125","21.38%","0","0","-"],["261","215","17.62%","4","2","50.00%","159","125","21.38%","0","0","-"]],
["<comparison>/candidate/product/src/cli/gwas/dispatch.rs",["111","65","41.44%","7","4","42.86%","96","60","37.50%","0","0","-"],["111","13","88.29%","7","0","100.00%","96","7","92.71%","0","0","-"],["111","13","88.29%","7","0","100.00%","96","7","92.71%","0","0","-"]],
["<comparison>/candidate/product/src/cli/health/catalog.rs",["10","7","30.00%","3","2","33.33%","10","7","30.00%","0","0","-"],["10","0","100.00%","3","0","100.00%","10","0","100.00%","0","0","-"],["10","0","100.00%","3","0","100.00%","10","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/health/http.rs",["565","505","10.62%","40","35","12.50%","440","395","10.23%","0","0","-"],["565","281","50.27%","40","19","52.50%","440","207","52.95%","0","0","-"],["565","270","52.21%","40","17","57.50%","440","198","55.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/health/local.rs",["479","479","0.00%","36","36","0.00%","388","388","0.00%","0","0","-"],["479","164","65.76%","36","12","66.67%","388","92","76.29%","0","0","-"],["479","164","65.76%","36","12","66.67%","388","92","76.29%","0","0","-"]],
["<comparison>/candidate/product/src/cli/health/mod.rs",["521","371","28.79%","37","25","32.43%","348","246","29.31%","0","0","-"],["521","92","82.34%","37","4","89.19%","348","54","84.48%","0","0","-"],["521","60","88.48%","37","0","100.00%","348","31","91.09%","0","0","-"]],
["<comparison>/candidate/product/src/cli/health/runner.rs",["395","189","52.15%","30","5","83.33%","320","135","57.81%","0","0","-"],["395","269","31.90%","30","17","43.33%","320","188","41.25%","0","0","-"],["395","155","60.76%","30","3","90.00%","320","98","69.38%","0","0","-"]],
["<comparison>/candidate/product/src/cli/install.rs",["486","486","0.00%","24","24","0.00%","278","278","0.00%","0","0","-"],["486","57","88.27%","24","3","87.50%","278","26","90.65%","0","0","-"],["486","57","88.27%","24","3","87.50%","278","26","90.65%","0","0","-"]],
["<comparison>/candidate/product/src/cli/list/catalog.rs",["319","86","73.04%","22","5","77.27%","193","36","81.35%","0","0","-"],["319","56","82.45%","22","3","86.36%","193","26","86.53%","0","0","-"],["319","54","83.07%","22","3","86.36%","193","26","86.53%","0","0","-"]],
["<comparison>/candidate/product/src/cli/list/clinical.rs",["24","20","16.67%","6","5","16.67%","357","272","23.81%","0","0","-"],["24","0","100.00%","6","0","100.00%","357","0","100.00%","0","0","-"],["24","0","100.00%","6","0","100.00%","357","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/list/helpers.rs",["35","21","40.00%","6","5","16.67%","147","136","7.48%","0","0","-"],["35","5","85.71%","6","1","83.33%","147","5","96.60%","0","0","-"],["35","5","85.71%","6","1","83.33%","147","5","96.60%","0","0","-"]],
["<comparison>/candidate/product/src/cli/list/literature.rs",["12","12","0.00%","3","3","0.00%","215","215","0.00%","0","0","-"],["12","4","66.67%","3","1","66.67%","215","21","90.23%","0","0","-"],["12","4","66.67%","3","1","66.67%","215","21","90.23%","0","0","-"]],
["<comparison>/candidate/product/src/cli/list/mod.rs",["133","44","66.92%","4","0","100.00%","68","4","94.12%","0","0","-"],["133","10","92.48%","4","0","100.00%","68","1","98.53%","0","0","-"],["133","10","92.48%","4","0","100.00%","68","1","98.53%","0","0","-"]],
["<comparison>/candidate/product/src/cli/list/molecular.rs",["49","49","0.00%","8","8","0.00%","368","368","0.00%","0","0","-"],["49","13","73.47%","8","3","62.50%","368","95","74.18%","0","0","-"],["49","13","73.47%","8","3","62.50%","368","95","74.18%","0","0","-"]],
["<comparison>/candidate/product/src/cli/mcp_config.rs",["152","152","0.00%","13","13","0.00%","112","112","0.00%","0","0","-"],["152","12","92.11%","13","0","100.00%","112","1","99.11%","0","0","-"],["152","12","92.11%","13","0","100.00%","112","1","99.11%","0","0","-"]],
["<comparison>/candidate/product/src/cli/mod.rs",["4","0","100.00%","1","0","100.00%","3","0","100.00%","0","0","-"],["4","0","100.00%","1","0","100.00%","3","0","100.00%","0","0","-"],["4","0","100.00%","1","0","100.00%","3","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/outcome.rs",["1029","410","60.16%","42","12","71.43%","495","157","68.28%","0","0","-"],["1029","590","42.66%","42","9","78.57%","495","228","53.94%","0","0","-"],["1029","363","64.72%","42","5","88.10%","495","125","74.75%","0","0","-"]],
["<comparison>/candidate/product/src/cli/outcome/mcp_chart.rs",["51","45","11.76%","2","0","100.00%","43","37","13.95%","0","0","-"],["51","27","47.06%","2","0","100.00%","43","23","46.51%","0","0","-"],["51","27","47.06%","2","0","100.00%","43","23","46.51%","0","0","-"]],
["<comparison>/candidate/product/src/cli/pathway/dispatch.rs",["450","392","12.89%","15","13","13.33%","267","239","10.49%","0","0","-"],["450","390","13.33%","15","9","40.00%","267","214","19.85%","0","0","-"],["450","358","20.44%","15","9","40.00%","267","204","23.60%","0","0","-"]],
["<comparison>/candidate/product/src/cli/pgx/dispatch.rs",["114","83","27.19%","6","3","50.00%","75","51","32.00%","0","0","-"],["114","92","19.30%","6","4","33.33%","75","52","30.67%","0","0","-"],["114","65","42.98%","6","2","66.67%","75","33","56.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/phenotype/dispatch.rs",["81","60","25.93%","5","2","60.00%","63","45","28.57%","0","0","-"],["81","60","25.93%","5","3","40.00%","63","42","33.33%","0","0","-"],["81","45","44.44%","5","1","80.00%","63","29","53.97%","0","0","-"]],
["<comparison>/candidate/product/src/cli/protein/dispatch.rs",["199","140","29.65%","12","8","33.33%","156","113","27.56%","0","0","-"],["199","142","28.64%","12","8","33.33%","156","103","33.97%","0","0","-"],["199","123","38.19%","12","6","50.00%","156","92","41.03%","0","0","-"]],
["<comparison>/candidate/product/src/cli/response_contract.rs",["446","195","56.28%","25","8","68.00%","486","287","40.95%","0","0","-"],["446","49","89.01%","25","1","96.00%","486","43","91.15%","0","0","-"],["446","33","92.60%","25","0","100.00%","486","35","92.80%","0","0","-"]],
["<comparison>/candidate/product/src/cli/search_all/dispatch.rs",["548","227","58.58%","28","17","39.29%","375","164","56.27%","0","0","-"],["548","432","21.17%","28","17","39.29%","375","283","24.53%","0","0","-"],["548","190","65.33%","28","14","50.00%","375","136","63.73%","0","0","-"]],
["<comparison>/candidate/product/src/cli/search_all/format.rs",["500","293","41.40%","34","16","52.94%","303","174","42.57%","0","0","-"],["500","218","56.40%","34","8","76.47%","303","116","61.72%","0","0","-"],["500","150","70.00%","34","7","79.41%","303","86","71.62%","0","0","-"]],
["<comparison>/candidate/product/src/cli/search_all/links.rs",["533","130","75.61%","25","8","68.00%","301","64","78.74%","0","0","-"],["533","315","40.90%","25","14","44.00%","301","177","41.20%","0","0","-"],["533","126","76.36%","25","6","76.00%","301","62","79.40%","0","0","-"]],
["<comparison>/candidate/product/src/cli/search_all/mod.rs",["232","65","71.98%","19","5","73.68%","144","33","77.08%","0","0","-"],["232","51","78.02%","19","3","84.21%","144","25","82.64%","0","0","-"],["232","44","81.03%","19","3","84.21%","144","18","87.50%","0","0","-"]],
["<comparison>/candidate/product/src/cli/search_all/plan.rs",["746","521","30.16%","39","24","38.46%","473","331","30.02%","0","0","-"],["746","229","69.30%","39","6","84.62%","473","119","74.84%","0","0","-"],["746","229","69.30%","39","6","84.62%","473","119","74.84%","0","0","-"]],
["<comparison>/candidate/product/src/cli/search_all_command.rs",["94","94","0.00%","3","3","0.00%","74","74","0.00%","0","0","-"],["94","3","96.81%","3","0","100.00%","74","3","95.95%","0","0","-"],["94","3","96.81%","3","0","100.00%","74","3","95.95%","0","0","-"]],
["<comparison>/candidate/product/src/cli/shared.rs",["551","71","87.11%","47","4","91.49%","387","66","82.95%","0","0","-"],["551","128","76.77%","47","8","82.98%","387","69","82.17%","0","0","-"],["551","19","96.55%","47","1","97.87%","387","12","96.90%","0","0","-"]],
["<comparison>/candidate/product/src/cli/shared/search_payloads.rs",["203","59","70.94%","20","5","75.00%","202","61","69.80%","0","0","-"],["203","42","79.31%","20","2","90.00%","202","42","79.21%","0","0","-"],["203","42","79.31%","20","2","90.00%","202","41","79.70%","0","0","-"]],
["<comparison>/candidate/product/src/cli/skill/assets.rs",["69","11","84.06%","5","1","80.00%","44","10","77.27%","0","0","-"],["69","11","84.06%","5","1","80.00%","44","10","77.27%","0","0","-"],["69","11","84.06%","5","1","80.00%","44","10","77.27%","0","0","-"]],
["<comparison>/candidate/product/src/cli/skill/atomic_swap.rs",["55","21","61.82%","6","3","50.00%","33","14","57.58%","0","0","-"],["55","7","87.27%","6","1","83.33%","33","2","93.94%","0","0","-"],["55","7","87.27%","6","1","83.33%","33","2","93.94%","0","0","-"]],
["<comparison>/candidate/product/src/cli/skill/catalog.rs",["165","28","83.03%","13","2","84.62%","103","19","81.55%","0","0","-"],["165","7","95.76%","13","0","100.00%","103","2","98.06%","0","0","-"],["165","7","95.76%","13","0","100.00%","103","2","98.06%","0","0","-"]],
["<comparison>/candidate/product/src/cli/skill/install.rs",["793","415","47.67%","45","23","48.89%","470","261","44.47%","0","0","-"],["793","248","68.73%","45","10","77.78%","470","140","70.21%","0","0","-"],["793","220","72.26%","45","7","84.44%","470","122","74.04%","0","0","-"]],
["<comparison>/candidate/product/src/cli/skill/mod.rs",["8","1","87.50%","1","0","100.00%","13","0","100.00%","0","0","-"],["8","8","0.00%","1","1","0.00%","13","13","0.00%","0","0","-"],["8","1","87.50%","1","0","100.00%","13","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/skill/status.rs",["365","103","71.78%","22","6","72.73%","218","72","66.97%","0","0","-"],["365","78","78.63%","22","5","77.27%","218","60","72.48%","0","0","-"],["365","78","78.63%","22","5","77.27%","218","60","72.48%","0","0","-"]],
["<comparison>/candidate/product/src/cli/study/dispatch.rs",["534","534","0.00%","10","10","0.00%","331","331","0.00%","0","0","-"],["534","332","37.83%","10","0","100.00%","331","171","48.34%","0","0","-"],["534","332","37.83%","10","0","100.00%","331","171","48.34%","0","0","-"]],
["<comparison>/candidate/product/src/cli/system/batch.rs",["984","686","30.28%","42","23","45.24%","681","433","36.42%","0","0","-"],["984","82","91.67%","42","4","90.48%","681","55","91.92%","0","0","-"],["984","29","97.05%","42","3","92.86%","681","12","98.24%","0","0","-"]],
["<comparison>/candidate/product/src/cli/system/dispatch.rs",["770","599","22.21%","76","47","38.16%","570","427","25.09%","0","0","-"],["770","565","26.62%","76","55","27.63%","570","416","27.02%","0","0","-"],["770","454","41.04%","76","37","51.32%","570","332","41.75%","0","0","-"]],
["<comparison>/candidate/product/src/cli/system/mod.rs",["45","18","60.00%","5","1","80.00%","39","12","69.23%","0","0","-"],["45","35","22.22%","5","4","20.00%","39","29","25.64%","0","0","-"],["45","18","60.00%","5","1","80.00%","39","12","69.23%","0","0","-"]],
["<comparison>/candidate/product/src/cli/system/who_sync.rs",["61","54","11.48%","5","3","40.00%","47","41","12.77%","0","0","-"],["61","12","80.33%","5","2","60.00%","47","8","82.98%","0","0","-"],["61","5","91.80%","5","0","100.00%","47","2","95.74%","0","0","-"]],
["<comparison>/candidate/product/src/cli/trial/dispatch.rs",["761","344","54.80%","57","36","36.84%","552","247","55.25%","0","0","-"],["761","226","70.30%","57","22","61.40%","552","133","75.91%","0","0","-"],["761","176","76.87%","57","20","64.91%","552","108","80.43%","0","0","-"]],
["<comparison>/candidate/product/src/cli/trial/documents.rs",["162","82","49.38%","14","7","50.00%","159","95","40.25%","0","0","-"],["162","46","71.60%","14","3","78.57%","159","31","80.50%","0","0","-"],["162","17","89.51%","14","2","85.71%","159","19","88.05%","0","0","-"]],
["<comparison>/candidate/product/src/cli/trial/zero_result.rs",["380","380","0.00%","15","15","0.00%","182","182","0.00%","0","0","-"],["380","96","74.74%","15","1","93.33%","182","44","75.82%","0","0","-"],["380","96","74.74%","15","1","93.33%","182","44","75.82%","0","0","-"]],
["<comparison>/candidate/product/src/cli/types.rs",["159","101","36.48%","24","15","37.50%","183","98","46.45%","0","0","-"],["159","44","72.33%","24","9","62.50%","183","51","72.13%","0","0","-"],["159","26","83.65%","24","6","75.00%","183","26","85.79%","0","0","-"]],
["<comparison>/candidate/product/src/cli/update.rs",["677","677","0.00%","50","50","0.00%","435","435","0.00%","0","0","-"],["677","320","52.73%","50","25","50.00%","435","202","53.56%","0","0","-"],["677","320","52.73%","50","25","50.00%","435","202","53.56%","0","0","-"]],
["<comparison>/candidate/product/src/cli/variant/articles.rs",["226","97","57.08%","17","9","47.06%","222","89","59.91%","0","0","-"],["226","214","5.31%","17","15","11.76%","222","198","10.81%","0","0","-"],["226","97","57.08%","17","9","47.06%","222","89","59.91%","0","0","-"]],
["<comparison>/candidate/product/src/cli/variant/car.rs",["114","99","13.16%","13","11","15.38%","86","75","12.79%","0","0","-"],["114","68","40.35%","13","9","30.77%","86","56","34.88%","0","0","-"],["114","53","53.51%","13","7","46.15%","86","45","47.67%","0","0","-"]],
["<comparison>/candidate/product/src/cli/variant/dispatch.rs",["632","282","55.38%","22","6","72.73%","467","202","56.75%","0","0","-"],["632","196","68.99%","22","1","95.45%","467","120","74.30%","0","0","-"],["632","137","78.32%","22","1","95.45%","467","75","83.94%","0","0","-"]],
["<comparison>/candidate/product/src/cli/variant/erepo.rs",["385","188","51.17%","18","7","61.11%","305","164","46.23%","0","0","-"],["385","228","40.78%","18","11","38.89%","305","157","48.52%","0","0","-"],["385","96","75.06%","18","3","83.33%","305","75","75.41%","0","0","-"]],
["<comparison>/candidate/product/src/cli/variant/guidance.rs",["19","19","0.00%","2","2","0.00%","21","21","0.00%","0","0","-"],["19","13","31.58%","2","1","50.00%","21","13","38.10%","0","0","-"],["19","13","31.58%","2","1","50.00%","21","13","38.10%","0","0","-"]],
["<comparison>/candidate/product/src/cli/variant/mod.rs",["62","16","74.19%","9","2","77.78%","42","10","76.19%","0","0","-"],["62","3","95.16%","9","1","88.89%","42","1","97.62%","0","0","-"],["62","3","95.16%","9","1","88.89%","42","1","97.62%","0","0","-"]],
["<comparison>/candidate/product/src/cli/variant/normalization_json.rs",["103","63","38.83%","4","2","50.00%","81","47","41.98%","0","0","-"],["103","8","92.23%","4","0","100.00%","81","4","95.06%","0","0","-"],["103","3","97.09%","4","0","100.00%","81","2","97.53%","0","0","-"]],
["<comparison>/candidate/product/src/cli/variant/query.rs",["492","462","6.10%","23","22","4.35%","418","391","6.46%","0","0","-"],["492","25","94.92%","23","0","100.00%","418","36","91.39%","0","0","-"],["492","25","94.92%","23","0","100.00%","418","36","91.39%","0","0","-"]],
["<comparison>/candidate/product/src/cli/variant/trial.rs",["50","50","0.00%","5","5","0.00%","33","33","0.00%","0","0","-"],["50","50","0.00%","5","5","0.00%","33","33","0.00%","0","0","-"],["50","50","0.00%","5","5","0.00%","33","33","0.00%","0","0","-"]],
["<comparison>/candidate/product/src/cli/worker.rs",["44","24","45.45%","5","3","40.00%","30","14","53.33%","0","0","-"],["44","4","90.91%","5","0","100.00%","30","2","93.33%","0","0","-"],["44","4","90.91%","5","0","100.00%","30","2","93.33%","0","0","-"]],
["<comparison>/candidate/product/src/entities/adverse_event.rs",["2534","1903","24.90%","202","146","27.72%","1930","1378","28.60%","0","0","-"],["2534","852","66.38%","202","76","62.38%","1930","617","68.03%","0","0","-"],["2534","611","75.89%","202","50","75.25%","1930","419","78.29%","0","0","-"]],
["<comparison>/candidate/product/src/entities/adverse_event/device.rs",["193","66","65.80%","13","4","69.23%","146","37","74.66%","0","0","-"],["193","48","75.13%","13","4","69.23%","146","33","77.40%","0","0","-"],["193","23","88.08%","13","1","92.31%","146","11","92.47%","0","0","-"]],
["<comparison>/candidate/product/src/entities/adverse_event/sections.rs",["85","58","31.76%","4","1","75.00%","74","36","51.35%","0","0","-"],["85","0","100.00%","4","0","100.00%","74","0","100.00%","0","0","-"],["85","0","100.00%","4","0","100.00%","74","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/assets.rs",["4011","1876","53.23%","234","73","68.80%","3054","1466","52.00%","0","0","-"],["4011","1243","69.01%","234","46","80.34%","3054","938","69.29%","0","0","-"],["4011","574","85.69%","234","16","93.16%","3054","454","85.13%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/backends.rs",["1194","575","51.84%","83","38","54.22%","845","416","50.77%","0","0","-"],["1194","206","82.75%","83","14","83.13%","845","145","82.84%","0","0","-"],["1194","179","85.01%","83","10","87.95%","845","128","84.85%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/backends/provider_unit.rs",["123","43","65.04%","12","0","100.00%","101","18","82.18%","0","0","-"],["123","8","93.50%","12","0","100.00%","101","4","96.04%","0","0","-"],["123","8","93.50%","12","0","100.00%","101","4","96.04%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/batch.rs",["247","91","63.16%","24","9","62.50%","165","57","65.45%","0","0","-"],["247","78","68.42%","24","5","79.17%","165","40","75.76%","0","0","-"],["247","65","73.68%","24","5","79.17%","165","34","79.39%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/candidates.rs",["469","222","52.67%","39","19","51.28%","309","142","54.05%","0","0","-"],["469","39","91.68%","39","6","84.62%","309","15","95.15%","0","0","-"],["469","39","91.68%","39","6","84.62%","309","15","95.15%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/detail.rs",["928","531","42.78%","71","32","54.93%","687","367","46.58%","0","0","-"],["928","291","68.64%","71","9","87.32%","687","177","74.24%","0","0","-"],["928","248","73.28%","71","8","88.73%","687","154","77.58%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/enrichment.rs",["616","260","57.79%","36","10","72.22%","471","189","59.87%","0","0","-"],["616","107","82.63%","36","4","88.89%","471","83","82.38%","0","0","-"],["616","107","82.63%","36","4","88.89%","471","83","82.38%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/filters.rs",["339","106","68.73%","34","11","67.65%","252","68","73.02%","0","0","-"],["339","17","94.99%","34","1","97.06%","252","10","96.03%","0","0","-"],["339","17","94.99%","34","1","97.06%","252","10","96.03%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/fulltext.rs",["1553","814","47.59%","119","58","51.26%","1281","679","46.99%","0","0","-"],["1553","374","75.92%","119","16","86.55%","1281","309","75.88%","0","0","-"],["1553","263","83.07%","119","10","91.60%","1281","205","84.00%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/graph.rs",["571","417","26.97%","60","43","28.33%","455","321","29.45%","0","0","-"],["571","364","36.25%","60","36","40.00%","455","248","45.49%","0","0","-"],["571","317","44.48%","60","32","46.67%","455","217","52.31%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/graph/citation_evidence.rs",["1134","1080","4.76%","114","104","8.77%","902","854","5.32%","0","0","-"],["1134","83","92.68%","114","16","85.96%","902","49","94.57%","0","0","-"],["1134","83","92.68%","114","16","85.96%","902","49","94.57%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/identity_verification.rs",["1736","1736","0.00%","126","126","0.00%","1233","1233","0.00%","0","0","-"],["1736","116","93.32%","126","10","92.06%","1233","61","95.05%","0","0","-"],["1736","116","93.32%","126","10","92.06%","1233","61","95.05%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/mod.rs",["492","407","17.28%","35","25","28.57%","378","303","19.84%","0","0","-"],["492","13","97.36%","35","0","100.00%","378","10","97.35%","0","0","-"],["492","13","97.36%","35","0","100.00%","378","10","97.35%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/planner.rs",["283","188","33.57%","19","12","36.84%","246","163","33.74%","0","0","-"],["283","24","91.52%","19","1","94.74%","246","21","91.46%","0","0","-"],["283","24","91.52%","19","1","94.74%","246","21","91.46%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/query.rs",["833","466","44.06%","80","51","36.25%","596","325","45.47%","0","0","-"],["833","128","84.63%","80","15","81.25%","596","72","87.92%","0","0","-"],["833","117","85.95%","80","11","86.25%","596","67","88.76%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/ranking.rs",["670","133","80.15%","59","16","72.88%","496","90","81.85%","0","0","-"],["670","19","97.16%","59","2","96.61%","496","9","98.19%","0","0","-"],["670","18","97.31%","59","2","96.61%","496","8","98.39%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/search.rs",["1053","534","49.29%","46","15","67.39%","759","378","50.20%","0","0","-"],["1053","348","66.95%","46","11","76.09%","759","234","69.17%","0","0","-"],["1053","286","72.84%","46","9","80.43%","759","195","74.31%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/search/deadline.rs",["117","68","41.88%","14","6","57.14%","118","56","52.54%","0","0","-"],["117","7","94.02%","14","1","92.86%","118","6","94.92%","0","0","-"],["117","7","94.02%","14","1","92.86%","118","6","94.92%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/search/status.rs",["56","27","51.79%","6","4","33.33%","58","34","41.38%","0","0","-"],["56","2","96.43%","6","0","100.00%","58","2","96.55%","0","0","-"],["56","2","96.43%","6","0","100.00%","58","2","96.55%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/supplement_search.rs",["681","263","61.38%","42","16","61.90%","458","164","64.19%","0","0","-"],["681","485","28.78%","42","29","30.95%","458","324","29.26%","0","0","-"],["681","165","75.77%","42","12","71.43%","458","105","77.07%","0","0","-"]],
["<comparison>/candidate/product/src/entities/article/variant_search.rs",["7896","6452","18.29%","490","388","20.82%","6599","5427","17.76%","0","0","-"],["7896","686","91.31%","490","37","92.45%","6599","558","91.54%","0","0","-"],["7896","673","91.48%","490","34","93.06%","6599","534","91.91%","0","0","-"]],
["<comparison>/candidate/product/src/entities/author/mod.rs",["532","345","35.15%","46","22","52.17%","410","268","34.63%","0","0","-"],["532","97","81.77%","46","11","76.09%","410","81","80.24%","0","0","-"],["532","38","92.86%","46","4","91.30%","410","23","94.39%","0","0","-"]],
["<comparison>/candidate/product/src/entities/author/papers.rs",["1957","1398","28.56%","112","63","43.75%","1193","790","33.78%","0","0","-"],["1957","182","90.70%","112","8","92.86%","1193","95","92.04%","0","0","-"],["1957","125","93.61%","112","5","95.54%","1193","48","95.98%","0","0","-"]],
["<comparison>/candidate/product/src/entities/author/search.rs",["252","252","0.00%","14","14","0.00%","240","240","0.00%","0","0","-"],["252","36","85.71%","14","2","85.71%","240","31","87.08%","0","0","-"],["252","36","85.71%","14","2","85.71%","240","31","87.08%","0","0","-"]],
["<comparison>/candidate/product/src/entities/cell_line.rs",["693","581","16.16%","59","47","20.34%","476","411","13.66%","0","0","-"],["693","325","53.10%","59","20","66.10%","476","200","57.98%","0","0","-"],["693","240","65.37%","59","12","79.66%","476","156","67.23%","0","0","-"]],
["<comparison>/candidate/product/src/entities/cell_line/chembl.rs",["99","99","0.00%","9","9","0.00%","82","82","0.00%","0","0","-"],["99","33","66.67%","9","2","77.78%","82","15","81.71%","0","0","-"],["99","33","66.67%","9","2","77.78%","82","15","81.71%","0","0","-"]],
["<comparison>/candidate/product/src/entities/cell_line/pharmacodb.rs",["120","111","7.50%","8","6","25.00%","106","96","9.43%","0","0","-"],["120","95","20.83%","8","6","25.00%","106","84","20.75%","0","0","-"],["120","86","28.33%","8","4","50.00%","106","74","30.19%","0","0","-"]],
["<comparison>/candidate/product/src/entities/diagnostic/get.rs",["880","537","38.98%","73","43","41.10%","586","353","39.76%","0","0","-"],["880","382","56.59%","73","33","54.79%","586","225","61.60%","0","0","-"],["880","228","74.09%","73","19","73.97%","586","130","77.82%","0","0","-"]],
["<comparison>/candidate/product/src/entities/diagnostic/mod.rs",["749","705","5.87%","59","50","15.25%","555","523","5.77%","0","0","-"],["749","16","97.86%","59","5","91.53%","555","10","98.20%","0","0","-"],["749","16","97.86%","59","5","91.53%","555","10","98.20%","0","0","-"]],
["<comparison>/candidate/product/src/entities/diagnostic/search.rs",["823","823","0.00%","67","67","0.00%","588","588","0.00%","0","0","-"],["823","86","89.55%","67","3","95.52%","588","52","91.16%","0","0","-"],["823","86","89.55%","67","3","95.52%","588","52","91.16%","0","0","-"]],
["<comparison>/candidate/product/src/entities/discover.rs",["4421","3740","15.40%","273","216","20.88%","3238","2702","16.55%","0","0","-"],["4421","435","90.16%","273","17","93.77%","3238","271","91.63%","0","0","-"],["4421","432","90.23%","273","17","93.77%","3238","269","91.69%","0","0","-"]],
["<comparison>/candidate/product/src/entities/disease/article_links.rs",["191","191","0.00%","16","16","0.00%","113","113","0.00%","0","0","-"],["191","13","93.19%","16","0","100.00%","113","8","92.92%","0","0","-"],["191","13","93.19%","16","0","100.00%","113","8","92.92%","0","0","-"]],
["<comparison>/candidate/product/src/entities/disease/associations.rs",["861","682","20.79%","63","38","39.68%","582","473","18.73%","0","0","-"],["861","571","33.68%","63","37","41.27%","582","386","33.68%","0","0","-"],["861","489","43.21%","63","22","65.08%","582","339","41.75%","0","0","-"]],
["<comparison>/candidate/product/src/entities/disease/enrichment.rs",["997","612","38.62%","66","45","31.82%","712","444","37.64%","0","0","-"],["997","598","40.02%","66","35","46.97%","712","389","45.37%","0","0","-"],["997","391","60.78%","66","27","59.09%","712","282","60.39%","0","0","-"]],
["<comparison>/candidate/product/src/entities/disease/fallback.rs",["584","584","0.00%","38","38","0.00%","413","413","0.00%","0","0","-"],["584","131","77.57%","38","5","86.84%","413","89","78.45%","0","0","-"],["584","131","77.57%","38","5","86.84%","413","89","78.45%","0","0","-"]],
["<comparison>/candidate/product/src/entities/disease/get.rs",["372","228","38.71%","22","13","40.91%","227","131","42.29%","0","0","-"],["372","116","68.82%","22","3","86.36%","227","63","72.25%","0","0","-"],["372","116","68.82%","22","3","86.36%","227","63","72.25%","0","0","-"]],
["<comparison>/candidate/product/src/entities/disease/mod.rs",["16","12","25.00%","2","1","50.00%","14","11","21.43%","0","0","-"],["16","2","87.50%","2","0","100.00%","14","0","100.00%","0","0","-"],["16","2","87.50%","2","0","100.00%","14","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/entities/disease/resolution.rs",["1126","320","71.58%","103","21","79.61%","749","210","71.96%","0","0","-"],["1126","59","94.76%","103","7","93.20%","749","32","95.73%","0","0","-"],["1126","55","95.12%","103","6","94.17%","749","30","95.99%","0","0","-"]],
["<comparison>/candidate/product/src/entities/disease/resolution/ambiguity.rs",["109","48","55.96%","16","3","81.25%","80","27","66.25%","0","0","-"],["109","6","94.50%","16","0","100.00%","80","2","97.50%","0","0","-"],["109","6","94.50%","16","0","100.00%","80","2","97.50%","0","0","-"]],
["<comparison>/candidate/product/src/entities/disease/search.rs",["899","506","43.72%","91","59","35.16%","690","399","42.17%","0","0","-"],["899","252","71.97%","91","32","64.84%","690","185","73.19%","0","0","-"],["899","216","75.97%","91","27","70.33%","690","160","76.81%","0","0","-"]],
["<comparison>/candidate/product/src/entities/drug/cell_lines.rs",["165","162","1.82%","10","9","10.00%","149","146","2.01%","0","0","-"],["165","131","20.61%","10","7","30.00%","149","113","24.16%","0","0","-"],["165","131","20.61%","10","7","30.00%","149","113","24.16%","0","0","-"]],
["<comparison>/candidate/product/src/entities/drug/get.rs",["1711","1070","37.46%","121","61","49.59%","1238","728","41.20%","0","0","-"],["1711","370","78.38%","121","24","80.17%","1238","258","79.16%","0","0","-"],["1711","279","83.69%","121","13","89.26%","1238","191","84.57%","0","0","-"]],
["<comparison>/candidate/product/src/entities/drug/interactions.rs",["520","514","1.15%","37","35","5.41%","342","334","2.34%","0","0","-"],["520","127","75.58%","37","8","78.38%","342","79","76.90%","0","0","-"],["520","121","76.73%","37","6","83.78%","342","71","79.24%","0","0","-"]],
["<comparison>/candidate/product/src/entities/drug/label.rs",["1638","1638","0.00%","164","164","0.00%","1051","1051","0.00%","0","0","-"],["1638","80","95.12%","164","3","98.17%","1051","45","95.72%","0","0","-"],["1638","80","95.12%","164","3","98.17%","1051","45","95.72%","0","0","-"]],
["<comparison>/candidate/product/src/entities/drug/metadata.rs",["769","612","20.42%","70","55","21.43%","474","392","17.30%","0","0","-"],["769","143","81.40%","70","15","78.57%","474","95","79.96%","0","0","-"],["769","139","81.92%","70","13","81.43%","474","93","80.38%","0","0","-"]],
["<comparison>/candidate/product/src/entities/drug/mod.rs",["210","106","49.52%","28","11","60.71%","147","82","44.22%","0","0","-"],["210","22","89.52%","28","2","92.86%","147","14","90.48%","0","0","-"],["210","15","92.86%","28","1","96.43%","147","11","92.52%","0","0","-"]],
["<comparison>/candidate/product/src/entities/drug/query.rs",["300","229","23.67%","28","24","14.29%","251","178","29.08%","0","0","-"],["300","121","59.67%","28","13","53.57%","251","102","59.36%","0","0","-"],["300","121","59.67%","28","13","53.57%","251","102","59.36%","0","0","-"]],
["<comparison>/candidate/product/src/entities/drug/search.rs",["1215","1006","17.20%","100","85","15.00%","840","679","19.17%","0","0","-"],["1215","507","58.27%","100","33","67.00%","840","334","60.24%","0","0","-"],["1215","448","63.13%","100","30","70.00%","840","295","64.88%","0","0","-"]],
["<comparison>/candidate/product/src/entities/drug/targets.rs",["593","593","0.00%","44","44","0.00%","385","385","0.00%","0","0","-"],["593","130","78.08%","44","4","90.91%","385","81","78.96%","0","0","-"],["593","130","78.08%","44","4","90.91%","385","81","78.96%","0","0","-"]],
["<comparison>/candidate/product/src/entities/gene.rs",["4592","2774","39.59%","336","196","41.67%","3069","1818","40.76%","0","0","-"],["4592","2570","44.03%","336","190","43.45%","3069","1647","46.33%","0","0","-"],["4592","1587","65.44%","336","127","62.20%","3069","1030","66.44%","0","0","-"]],
["<comparison>/candidate/product/src/entities/gene/cell_lines.rs",["171","171","0.00%","17","17","0.00%","113","113","0.00%","0","0","-"],["171","120","29.82%","17","10","41.18%","113","74","34.51%","0","0","-"],["171","120","29.82%","17","10","41.18%","113","74","34.51%","0","0","-"]],
["<comparison>/candidate/product/src/entities/gene/clingen.rs",["289","211","26.99%","30","18","40.00%","218","155","28.90%","0","0","-"],["289","44","84.78%","30","4","86.67%","218","26","88.07%","0","0","-"],["289","44","84.78%","30","4","86.67%","218","26","88.07%","0","0","-"]],
["<comparison>/candidate/product/src/entities/gene/cspec.rs",["1418","1393","1.76%","84","79","5.95%","903","884","2.10%","0","0","-"],["1418","326","77.01%","84","27","67.86%","903","189","79.07%","0","0","-"],["1418","301","78.77%","84","22","73.81%","903","170","81.17%","0","0","-"]],
["<comparison>/candidate/product/src/entities/gene/gencc.rs",["309","309","0.00%","17","17","0.00%","219","219","0.00%","0","0","-"],["309","90","70.87%","17","4","76.47%","219","44","79.91%","0","0","-"],["309","90","70.87%","17","4","76.47%","219","44","79.91%","0","0","-"]],
["<comparison>/candidate/product/src/entities/mod.rs",["18","0","100.00%","3","0","100.00%","36","0","100.00%","0","0","-"],["18","6","66.67%","3","1","66.67%","36","13","63.89%","0","0","-"],["18","0","100.00%","3","0","100.00%","36","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/entities/pathway.rs",["1450","1163","19.79%","93","79","15.05%","957","757","20.90%","0","0","-"],["1450","410","71.72%","93","14","84.95%","957","255","73.35%","0","0","-"],["1450","311","78.55%","93","11","88.17%","957","193","79.83%","0","0","-"]],
["<comparison>/candidate/product/src/entities/pgx.rs",["1239","1092","11.86%","90","80","11.11%","853","734","13.95%","0","0","-"],["1239","682","44.96%","90","45","50.00%","853","460","46.07%","0","0","-"],["1239","589","52.46%","90","40","55.56%","853","395","53.69%","0","0","-"]],
["<comparison>/candidate/product/src/entities/pharmacodb.rs",["231","217","6.06%","26","23","11.54%","181","171","5.52%","0","0","-"],["231","9","96.10%","26","2","92.31%","181","5","97.24%","0","0","-"],["231","9","96.10%","26","2","92.31%","181","5","97.24%","0","0","-"]],
["<comparison>/candidate/product/src/entities/protein.rs",["967","808","16.44%","50","36","28.00%","688","572","16.86%","0","0","-"],["967","454","53.05%","50","20","60.00%","688","310","54.94%","0","0","-"],["967","361","62.67%","50","13","74.00%","688","240","65.12%","0","0","-"]],
["<comparison>/candidate/product/src/entities/section_outcome.rs",["402","264","34.33%","43","19","55.81%","306","174","43.14%","0","0","-"],["402","9","97.76%","43","0","100.00%","306","2","99.35%","0","0","-"],["402","9","97.76%","43","0","100.00%","306","2","99.35%","0","0","-"]],
["<comparison>/candidate/product/src/entities/source_state_registry.rs",["155","78","49.68%","16","7","56.25%","118","65","44.92%","0","0","-"],["155","22","85.81%","16","5","68.75%","118","37","68.64%","0","0","-"],["155","22","85.81%","16","5","68.75%","118","37","68.64%","0","0","-"]],
["<comparison>/candidate/product/src/entities/study.rs",["1770","1770","0.00%","161","161","0.00%","1325","1325","0.00%","0","0","-"],["1770","354","80.00%","161","36","77.64%","1325","235","82.26%","0","0","-"],["1770","354","80.00%","161","36","77.64%","1325","235","82.26%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/design.rs",["683","242","64.57%","93","20","78.49%","597","179","70.02%","0","0","-"],["683","39","94.29%","93","4","95.70%","597","19","96.82%","0","0","-"],["683","39","94.29%","93","4","95.70%","597","19","96.82%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/documents.rs",["621","357","42.51%","60","31","48.33%","397","217","45.34%","0","0","-"],["621","140","77.46%","60","15","75.00%","397","92","76.83%","0","0","-"],["621","43","93.08%","60","4","93.33%","397","35","91.18%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/eligibility.rs",["821","605","26.31%","66","44","33.33%","538","374","30.48%","0","0","-"],["821","55","93.30%","66","11","83.33%","538","42","92.19%","0","0","-"],["821","55","93.30%","66","11","83.33%","538","42","92.19%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/get.rs",["314","40","87.26%","16","1","93.75%","234","29","87.61%","0","0","-"],["314","22","92.99%","16","0","100.00%","234","15","93.59%","0","0","-"],["314","22","92.99%","16","0","100.00%","234","15","93.59%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/mod.rs",["728","579","20.47%","46","29","36.96%","447","351","21.48%","0","0","-"],["728","29","96.02%","46","4","91.30%","447","13","97.09%","0","0","-"],["728","29","96.02%","46","4","91.30%","447","13","97.09%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/search/ctgov.rs",["1215","333","72.59%","76","23","69.74%","919","235","74.43%","0","0","-"],["1215","441","63.70%","76","15","80.26%","919","273","70.29%","0","0","-"],["1215","175","85.60%","76","7","90.79%","919","122","86.72%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/search/eligibility.rs",["515","136","73.59%","59","16","72.88%","378","92","75.66%","0","0","-"],["515","44","91.46%","59","2","96.61%","378","22","94.18%","0","0","-"],["515","32","93.79%","59","0","100.00%","378","18","95.24%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/search/eligibility/keep_paths.rs",["198","198","0.00%","16","16","0.00%","101","101","0.00%","0","0","-"],["198","3","98.48%","16","0","100.00%","101","2","98.02%","0","0","-"],["198","3","98.48%","16","0","100.00%","101","2","98.02%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/search/essie.rs",["314","203","35.35%","27","17","37.04%","237","156","34.18%","0","0","-"],["314","38","87.90%","27","5","81.48%","237","27","88.61%","0","0","-"],["314","38","87.90%","27","5","81.48%","237","27","88.61%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/search/mod.rs",["405","87","78.52%","44","21","52.27%","363","74","79.61%","0","0","-"],["405","28","93.09%","44","2","95.45%","363","20","94.49%","0","0","-"],["405","27","93.33%","44","2","95.45%","363","20","94.49%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/search/nci.rs",["227","130","42.73%","18","6","66.67%","183","107","41.53%","0","0","-"],["227","24","89.43%","18","0","100.00%","183","22","87.98%","0","0","-"],["227","24","89.43%","18","0","100.00%","183","22","87.98%","0","0","-"]],
["<comparison>/candidate/product/src/entities/trial/search/normalization.rs",["286","155","45.80%","24","10","58.33%","196","95","51.53%","0","0","-"],["286","23","91.96%","24","2","91.67%","196","22","88.78%","0","0","-"],["286","14","95.10%","24","0","100.00%","196","16","91.84%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/clinvar.rs",["881","698","20.77%","47","28","40.43%","537","404","24.77%","0","0","-"],["881","18","97.96%","47","2","95.74%","537","7","98.70%","0","0","-"],["881","15","98.30%","47","1","97.87%","537","6","98.88%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/erepo.rs",["1221","433","64.54%","90","34","62.22%","813","263","67.65%","0","0","-"],["1221","137","88.78%","90","14","84.44%","813","92","88.68%","0","0","-"],["1221","80","93.45%","90","9","90.00%","813","62","92.37%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/get.rs",["1973","1275","35.38%","149","103","30.87%","1532","986","35.64%","0","0","-"],["1973","448","77.29%","149","21","85.91%","1532","326","78.72%","0","0","-"],["1973","396","79.93%","149","16","89.26%","1532","290","81.07%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/gwas.rs",["793","377","52.46%","84","51","39.29%","569","275","51.67%","0","0","-"],["793","319","59.77%","84","45","46.43%","569","234","58.88%","0","0","-"],["793","315","60.28%","84","45","46.43%","569","232","59.23%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/mod.rs",["234","93","60.26%","18","9","50.00%","136","52","61.76%","0","0","-"],["234","20","91.45%","18","0","100.00%","136","10","92.65%","0","0","-"],["234","19","91.88%","18","0","100.00%","136","9","93.38%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/normalization.rs",["334","198","40.72%","35","20","42.86%","248","150","39.52%","0","0","-"],["334","99","70.36%","35","7","80.00%","248","69","72.18%","0","0","-"],["334","71","78.74%","35","7","80.00%","248","51","79.44%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/resolution.rs",["1638","776","52.63%","156","70","55.13%","1134","567","50.00%","0","0","-"],["1638","107","93.47%","156","6","96.15%","1134","90","92.06%","0","0","-"],["1638","107","93.47%","156","6","96.15%","1134","90","92.06%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/search/diagnostics.rs",["414","377","8.94%","14","13","7.14%","306","275","10.13%","0","0","-"],["414","293","29.23%","14","9","35.71%","306","200","34.64%","0","0","-"],["414","293","29.23%","14","9","35.71%","306","200","34.64%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/search/mod.rs",["1196","636","46.82%","84","49","41.67%","967","460","52.43%","0","0","-"],["1196","303","74.67%","84","25","70.24%","967","212","78.08%","0","0","-"],["1196","269","77.51%","84","19","77.38%","967","155","83.97%","0","0","-"]],
["<comparison>/candidate/product/src/entities/variant/structure.rs",["473","143","69.77%","38","10","73.68%","348","123","64.66%","0","0","-"],["473","60","87.32%","38","4","89.47%","348","37","89.37%","0","0","-"],["473","60","87.32%","38","4","89.47%","348","37","89.37%","0","0","-"]],
["<comparison>/candidate/product/src/error.rs",["1265","802","36.60%","59","30","49.15%","926","596","35.64%","0","0","-"],["1265","220","82.61%","59","4","93.22%","926","117","87.37%","0","0","-"],["1265","220","82.61%","59","4","93.22%","926","117","87.37%","0","0","-"]],
["<comparison>/candidate/product/src/lib.rs",["18","18","0.00%","7","7","0.00%","26","26","0.00%","0","0","-"],["18","0","100.00%","7","0","100.00%","26","0","100.00%","0","0","-"],["18","0","100.00%","7","0","100.00%","26","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/main.rs",["225","54","76.00%","12","3","75.00%","151","33","78.15%","0","0","-"],["225","225","0.00%","12","12","0.00%","151","151","0.00%","0","0","-"],["225","54","76.00%","12","3","75.00%","151","33","78.15%","0","0","-"]],
["<comparison>/candidate/product/src/mcp/catalog.rs",["79","12","84.81%","10","3","70.00%","65","10","84.62%","0","0","-"],["79","49","37.97%","10","8","20.00%","65","42","35.38%","0","0","-"],["79","12","84.81%","10","3","70.00%","65","10","84.62%","0","0","-"]],
["<comparison>/candidate/product/src/mcp/mod.rs",["23","2","91.30%","5","0","100.00%","19","0","100.00%","0","0","-"],["23","23","0.00%","5","5","0.00%","19","19","0.00%","0","0","-"],["23","2","91.30%","5","0","100.00%","19","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/mcp/shell.rs",["3295","2716","17.57%","174","135","22.41%","2077","1668","19.69%","0","0","-"],["3295","735","77.69%","174","54","68.97%","2077","502","75.83%","0","0","-"],["3295","658","80.03%","174","45","74.14%","2077","445","78.57%","0","0","-"]],
["<comparison>/candidate/product/src/mcp/shell/http_server.rs",["719","664","7.65%","73","66","9.59%","504","458","9.13%","0","0","-"],["719","345","52.02%","73","33","54.79%","504","235","53.37%","0","0","-"],["719","326","54.66%","73","31","57.53%","504","221","56.15%","0","0","-"]],
["<comparison>/candidate/product/src/mcp/shell/modern.rs",["371","371","0.00%","23","23","0.00%","212","212","0.00%","0","0","-"],["371","371","0.00%","23","23","0.00%","212","212","0.00%","0","0","-"],["371","371","0.00%","23","23","0.00%","212","212","0.00%","0","0","-"]],
["<comparison>/candidate/product/src/mcp/shell/pre_session.rs",["172","111","35.47%","11","7","36.36%","112","78","30.36%","0","0","-"],["172","172","0.00%","11","11","0.00%","112","112","0.00%","0","0","-"],["172","111","35.47%","11","7","36.36%","112","78","30.36%","0","0","-"]],
["<comparison>/candidate/product/src/mcp/shell/trial_phase.rs",["9","5","44.44%","2","1","50.00%","8","5","37.50%","0","0","-"],["9","0","100.00%","2","0","100.00%","8","0","100.00%","0","0","-"],["9","0","100.00%","2","0","100.00%","8","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/mcp/shell/typed_get.rs",["143","32","77.62%","9","3","66.67%","94","24","74.47%","0","0","-"],["143","3","97.90%","9","0","100.00%","94","2","97.87%","0","0","-"],["143","3","97.90%","9","0","100.00%","94","2","97.87%","0","0","-"]],
["<comparison>/candidate/product/src/next_command.rs",["89","15","83.15%","11","1","90.91%","61","10","83.61%","0","0","-"],["89","0","100.00%","11","0","100.00%","61","0","100.00%","0","0","-"],["89","0","100.00%","11","0","100.00%","61","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/render/chart.rs",["2112","2112","0.00%","83","83","0.00%","1730","1730","0.00%","0","0","-"],["2112","310","85.32%","83","5","93.98%","1730","198","88.55%","0","0","-"],["2112","310","85.32%","83","5","93.98%","1730","198","88.55%","0","0","-"]],
["<comparison>/candidate/product/src/render/chart/bounds.rs",["80","80","0.00%","6","6","0.00%","51","51","0.00%","0","0","-"],["80","10","87.50%","6","1","83.33%","51","9","82.35%","0","0","-"],["80","10","87.50%","6","1","83.33%","51","9","82.35%","0","0","-"]],
["<comparison>/candidate/product/src/render/human.rs",["371","241","35.04%","22","12","45.45%","256","173","32.42%","0","0","-"],["371","1","99.73%","22","0","100.00%","256","0","100.00%","0","0","-"],["371","1","99.73%","22","0","100.00%","256","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/render/json.rs",["1487","1288","13.38%","53","36","32.08%","1177","951","19.20%","0","0","-"],["1487","38","97.44%","53","0","100.00%","1177","21","98.22%","0","0","-"],["1487","37","97.51%","53","0","100.00%","1177","20","98.30%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/adverse_event.rs",["418","305","27.03%","19","14","26.32%","309","224","27.51%","0","0","-"],["418","38","90.91%","19","0","100.00%","309","20","93.53%","0","0","-"],["418","38","90.91%","19","0","100.00%","309","20","93.53%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/article.rs",["967","492","49.12%","49","25","48.98%","635","322","49.29%","0","0","-"],["967","167","82.73%","49","5","89.80%","635","85","86.61%","0","0","-"],["967","167","82.73%","49","5","89.80%","635","85","86.61%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/author.rs",["985","621","36.95%","55","29","47.27%","717","499","30.40%","0","0","-"],["985","149","84.87%","55","9","83.64%","717","98","86.33%","0","0","-"],["985","149","84.87%","55","9","83.64%","717","98","86.33%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/cell_line.rs",["126","126","0.00%","7","7","0.00%","80","80","0.00%","0","0","-"],["126","8","93.65%","7","0","100.00%","80","3","96.25%","0","0","-"],["126","8","93.65%","7","0","100.00%","80","3","96.25%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/diagnostic.rs",["291","169","41.92%","19","11","42.11%","194","103","46.91%","0","0","-"],["291","14","95.19%","19","1","94.74%","194","7","96.39%","0","0","-"],["291","14","95.19%","19","1","94.74%","194","7","96.39%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/discovery.rs",["152","97","36.18%","9","6","33.33%","124","84","32.26%","0","0","-"],["152","16","89.47%","9","0","100.00%","124","7","94.35%","0","0","-"],["152","12","92.11%","9","0","100.00%","124","4","96.77%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/disease.rs",["450","450","0.00%","33","33","0.00%","282","282","0.00%","0","0","-"],["450","24","94.67%","33","2","93.94%","282","13","95.39%","0","0","-"],["450","24","94.67%","33","2","93.94%","282","13","95.39%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/drug.rs",["908","674","25.77%","55","41","25.45%","621","474","23.67%","0","0","-"],["908","169","81.39%","55","17","69.09%","621","96","84.54%","0","0","-"],["908","168","81.50%","55","16","70.91%","621","95","84.70%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/drug_regulatory.rs",["958","745","22.23%","76","63","17.11%","681","534","21.59%","0","0","-"],["958","212","77.87%","76","37","51.32%","681","158","76.80%","0","0","-"],["958","212","77.87%","76","37","51.32%","681","158","76.80%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/evidence.rs",["606","370","38.94%","57","36","36.84%","495","260","47.47%","0","0","-"],["606","53","91.25%","57","3","94.74%","495","28","94.34%","0","0","-"],["606","53","91.25%","57","3","94.74%","495","28","94.34%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/funding.rs",["91","82","9.89%","10","8","20.00%","65","56","13.85%","0","0","-"],["91","11","87.91%","10","2","80.00%","65","4","93.85%","0","0","-"],["91","11","87.91%","10","2","80.00%","65","4","93.85%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/gene.rs",["172","43","75.00%","8","5","37.50%","101","31","69.31%","0","0","-"],["172","9","94.77%","8","0","100.00%","101","3","97.03%","0","0","-"],["172","9","94.77%","8","0","100.00%","101","3","97.03%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/mod.rs",["840","274","67.38%","96","37","61.46%","655","220","66.41%","0","0","-"],["840","107","87.26%","96","12","87.50%","655","97","85.19%","0","0","-"],["840","74","91.19%","96","5","94.79%","655","67","89.77%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/pathway.rs",["92","92","0.00%","4","4","0.00%","46","46","0.00%","0","0","-"],["92","19","79.35%","4","2","50.00%","46","5","89.13%","0","0","-"],["92","19","79.35%","4","2","50.00%","46","5","89.13%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/pgx.rs",["118","118","0.00%","6","6","0.00%","62","62","0.00%","0","0","-"],["118","20","83.05%","6","1","83.33%","62","12","80.65%","0","0","-"],["118","20","83.05%","6","1","83.33%","62","12","80.65%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/pharmacodb.rs",["67","67","0.00%","3","3","0.00%","39","39","0.00%","0","0","-"],["67","67","0.00%","3","3","0.00%","39","39","0.00%","0","0","-"],["67","67","0.00%","3","3","0.00%","39","39","0.00%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/protein.rs",["190","190","0.00%","13","13","0.00%","113","113","0.00%","0","0","-"],["190","29","84.74%","13","2","84.62%","113","19","83.19%","0","0","-"],["190","29","84.74%","13","2","84.62%","113","19","83.19%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/related.rs",["1595","854","46.46%","166","105","36.75%","1084","577","46.77%","0","0","-"],["1595","264","83.45%","166","28","83.13%","1084","181","83.30%","0","0","-"],["1595","167","89.53%","166","16","90.36%","1084","116","89.30%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/related/article_support.rs",["555","236","57.48%","58","27","53.45%","368","163","55.71%","0","0","-"],["555","26","95.32%","58","3","94.83%","368","14","96.20%","0","0","-"],["555","26","95.32%","58","3","94.83%","368","14","96.20%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/sections.rs",["791","235","70.29%","64","12","81.25%","532","161","69.74%","0","0","-"],["791","54","93.17%","64","1","98.44%","532","32","93.98%","0","0","-"],["791","54","93.17%","64","1","98.44%","532","32","93.98%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/study.rs",["622","622","0.00%","25","25","0.00%","431","431","0.00%","0","0","-"],["622","76","87.78%","25","4","84.00%","431","55","87.24%","0","0","-"],["622","76","87.78%","25","4","84.00%","431","55","87.24%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/support.rs",["190","92","51.58%","14","5","64.29%","121","67","44.63%","0","0","-"],["190","40","78.95%","14","2","85.71%","121","30","75.21%","0","0","-"],["190","40","78.95%","14","2","85.71%","121","30","75.21%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/trial.rs",["633","131","79.30%","49","8","83.67%","402","91","77.36%","0","0","-"],["633","22","96.52%","49","0","100.00%","402","9","97.76%","0","0","-"],["633","20","96.84%","49","0","100.00%","402","7","98.26%","0","0","-"]],
["<comparison>/candidate/product/src/render/markdown/variant.rs",["873","680","22.11%","51","46","9.80%","593","469","20.91%","0","0","-"],["873","136","84.42%","51","4","92.16%","593","77","87.02%","0","0","-"],["873","136","84.42%","51","4","92.16%","593","77","87.02%","0","0","-"]],
["<comparison>/candidate/product/src/render/provenance.rs",["1998","1565","21.67%","76","56","26.32%","1713","1311","23.47%","0","0","-"],["1998","172","91.39%","76","3","96.05%","1713","96","94.40%","0","0","-"],["1998","168","91.59%","76","2","97.37%","1713","92","94.63%","0","0","-"]],
["<comparison>/candidate/product/src/render/provenance/adverse_event.rs",["106","46","56.60%","6","1","83.33%","85","29","65.88%","0","0","-"],["106","28","73.58%","6","1","83.33%","85","19","77.65%","0","0","-"],["106","22","79.25%","6","0","100.00%","85","11","87.06%","0","0","-"]],
["<comparison>/candidate/product/src/skill_assets.rs",["27","11","59.26%","5","2","60.00%","18","7","61.11%","0","0","-"],["27","11","59.26%","5","2","60.00%","18","7","61.11%","0","0","-"],["27","11","59.26%","5","2","60.00%","18","7","61.11%","0","0","-"]],
["<comparison>/candidate/product/src/sources/archive_budget.rs",["631","631","0.00%","20","20","0.00%","378","378","0.00%","0","0","-"],["631","43","93.19%","20","0","100.00%","378","13","96.56%","0","0","-"],["631","43","93.19%","20","0","100.00%","378","13","96.56%","0","0","-"]],
["<comparison>/candidate/product/src/sources/ca_bundle.rs",["527","361","31.50%","30","15","50.00%","316","201","36.39%","0","0","-"],["527","60","88.61%","30","7","76.67%","316","43","86.39%","0","0","-"],["527","40","92.41%","30","4","86.67%","316","30","90.51%","0","0","-"]],
["<comparison>/candidate/product/src/sources/cancerhotspots.rs",["185","111","40.00%","19","12","36.84%","126","77","38.89%","0","0","-"],["185","12","93.51%","19","1","94.74%","126","6","95.24%","0","0","-"],["185","11","94.05%","19","1","94.74%","126","6","95.24%","0","0","-"]],
["<comparison>/candidate/product/src/sources/cbioportal.rs",["318","226","28.93%","32","18","43.75%","228","152","33.33%","0","0","-"],["318","197","38.05%","32","15","53.12%","228","126","44.74%","0","0","-"],["318","196","38.36%","32","15","53.12%","228","126","44.74%","0","0","-"]],
["<comparison>/candidate/product/src/sources/cbioportal_download.rs",["572","572","0.00%","34","34","0.00%","446","446","0.00%","0","0","-"],["572","215","62.41%","34","11","67.65%","446","168","62.33%","0","0","-"],["572","215","62.41%","34","11","67.65%","446","168","62.33%","0","0","-"]],
["<comparison>/candidate/product/src/sources/cbioportal_study.rs",["4088","4063","0.61%","184","181","1.63%","2463","2444","0.77%","0","0","-"],["4088","336","91.78%","184","14","92.39%","2463","213","91.35%","0","0","-"],["4088","336","91.78%","184","14","92.39%","2463","213","91.35%","0","0","-"]],
["<comparison>/candidate/product/src/sources/cellosaurus.rs",["362","244","32.60%","49","35","28.57%","293","213","27.30%","0","0","-"],["362","179","50.55%","49","27","44.90%","293","151","48.46%","0","0","-"],["362","106","70.72%","49","16","67.35%","293","104","64.51%","0","0","-"]],
["<comparison>/candidate/product/src/sources/chembl.rs",["289","289","0.00%","33","33","0.00%","226","226","0.00%","0","0","-"],["289","75","74.05%","33","9","72.73%","226","40","82.30%","0","0","-"],["289","75","74.05%","33","9","72.73%","226","40","82.30%","0","0","-"]],
["<comparison>/candidate/product/src/sources/civic.rs",["288","169","41.32%","29","16","44.83%","240","144","40.00%","0","0","-"],["288","25","91.32%","29","3","89.66%","240","24","90.00%","0","0","-"],["288","12","95.83%","29","0","100.00%","240","12","95.00%","0","0","-"]],
["<comparison>/candidate/product/src/sources/clingen.rs",["830","614","26.02%","75","54","28.00%","618","423","31.55%","0","0","-"],["830","61","92.65%","75","9","88.00%","618","67","89.16%","0","0","-"],["830","60","92.77%","75","9","88.00%","618","67","89.16%","0","0","-"]],
["<comparison>/candidate/product/src/sources/clingen_allele_registry.rs",["911","722","20.75%","57","40","29.82%","594","453","23.74%","0","0","-"],["911","209","77.06%","57","16","71.93%","594","144","75.76%","0","0","-"],["911","118","87.05%","57","7","87.72%","594","90","84.85%","0","0","-"]],
["<comparison>/candidate/product/src/sources/clingen_cspec.rs",["406","273","32.76%","26","16","38.46%","281","201","28.47%","0","0","-"],["406","42","89.66%","26","1","96.15%","281","21","92.53%","0","0","-"],["406","31","92.36%","26","1","96.15%","281","14","95.02%","0","0","-"]],
["<comparison>/candidate/product/src/sources/clingen_erepo.rs",["374","112","70.05%","32","7","78.12%","270","74","72.59%","0","0","-"],["374","55","85.29%","32","4","87.50%","270","46","82.96%","0","0","-"],["374","29","92.25%","32","2","93.75%","270","21","92.22%","0","0","-"]],
["<comparison>/candidate/product/src/sources/clingen_ldh.rs",["131","131","0.00%","17","17","0.00%","101","101","0.00%","0","0","-"],["131","23","82.44%","17","4","76.47%","101","16","84.16%","0","0","-"],["131","23","82.44%","17","4","76.47%","101","16","84.16%","0","0","-"]],
["<comparison>/candidate/product/src/sources/clinicaltrials.rs",["779","99","87.29%","70","8","88.57%","574","38","93.38%","0","0","-"],["779","45","94.22%","70","3","95.71%","574","9","98.43%","0","0","-"],["779","38","95.12%","70","2","97.14%","574","6","98.95%","0","0","-"]],
["<comparison>/candidate/product/src/sources/complexportal.rs",["182","106","41.76%","20","12","40.00%","154","89","42.21%","0","0","-"],["182","80","56.04%","20","8","60.00%","154","53","65.58%","0","0","-"],["182","25","86.26%","20","2","90.00%","154","15","90.26%","0","0","-"]],
["<comparison>/candidate/product/src/sources/cpic.rs",["402","289","28.11%","39","27","30.77%","328","238","27.44%","0","0","-"],["402","104","74.13%","39","12","69.23%","328","80","75.61%","0","0","-"],["402","80","80.10%","39","8","79.49%","328","62","81.10%","0","0","-"]],
["<comparison>/candidate/product/src/sources/cvx.rs",["1028","793","22.86%","88","62","29.55%","665","490","26.32%","0","0","-"],["1028","351","65.86%","88","30","65.91%","665","218","67.22%","0","0","-"],["1028","228","77.82%","88","22","75.00%","665","127","80.90%","0","0","-"]],
["<comparison>/candidate/product/src/sources/dbsnp.rs",["179","15","91.62%","20","0","100.00%","129","10","92.25%","0","0","-"],["179","14","92.18%","20","0","100.00%","129","9","93.02%","0","0","-"],["179","13","92.74%","20","0","100.00%","129","9","93.02%","0","0","-"]],
["<comparison>/candidate/product/src/sources/ddinter.rs",["549","542","1.28%","47","45","4.26%","359","352","1.95%","0","0","-"],["549","235","57.19%","47","17","63.83%","359","146","59.33%","0","0","-"],["549","228","58.47%","47","15","68.09%","359","139","61.28%","0","0","-"]],
["<comparison>/candidate/product/src/sources/dgidb.rs",["270","188","30.37%","23","14","39.13%","194","127","34.54%","0","0","-"],["270","74","72.59%","23","7","69.57%","194","51","73.71%","0","0","-"],["270","23","91.48%","23","1","95.65%","194","15","92.27%","0","0","-"]],
["<comparison>/candidate/product/src/sources/disable.rs",["535","273","48.97%","55","20","63.64%","314","131","58.28%","0","0","-"],["535","95","82.24%","55","8","85.45%","314","63","79.94%","0","0","-"],["535","19","96.45%","55","0","100.00%","314","12","96.18%","0","0","-"]],
["<comparison>/candidate/product/src/sources/disgenet.rs",["451","310","31.26%","41","26","36.59%","323","218","32.51%","0","0","-"],["451","191","57.65%","41","15","63.41%","323","107","66.87%","0","0","-"],["451","120","73.39%","41","7","82.93%","323","61","81.11%","0","0","-"]],
["<comparison>/candidate/product/src/sources/ema.rs",["1042","471","54.80%","79","26","67.09%","684","307","55.12%","0","0","-"],["1042","332","68.14%","79","21","73.42%","684","208","69.59%","0","0","-"],["1042","153","85.32%","79","8","89.87%","684","97","85.82%","0","0","-"]],
["<comparison>/candidate/product/src/sources/ema/identity.rs",["251","186","25.90%","32","25","21.88%","191","144","24.61%","0","0","-"],["251","3","98.80%","32","0","100.00%","191","2","98.95%","0","0","-"],["251","3","98.80%","32","0","100.00%","191","2","98.95%","0","0","-"]],
["<comparison>/candidate/product/src/sources/enrichr.rs",["237","41","82.70%","21","3","85.71%","184","34","81.52%","0","0","-"],["237","173","27.00%","21","17","19.05%","184","128","30.43%","0","0","-"],["237","32","86.50%","21","3","85.71%","184","21","88.59%","0","0","-"]],
["<comparison>/candidate/product/src/sources/europepmc.rs",["606","187","69.14%","53","22","58.49%","464","157","66.16%","0","0","-"],["606","105","82.67%","53","15","71.70%","464","80","82.76%","0","0","-"],["606","85","85.97%","53","13","75.47%","464","63","86.42%","0","0","-"]],
["<comparison>/candidate/product/src/sources/fda_orphan.rs",["1284","984","23.36%","103","72","30.10%","819","602","26.50%","0","0","-"],["1284","35","97.27%","103","3","97.09%","819","21","97.44%","0","0","-"],["1284","35","97.27%","103","3","97.09%","819","21","97.44%","0","0","-"]],
["<comparison>/candidate/product/src/sources/figshare.rs",["507","507","0.00%","50","50","0.00%","370","370","0.00%","0","0","-"],["507","124","75.54%","50","12","76.00%","370","86","76.76%","0","0","-"],["507","124","75.54%","50","12","76.00%","370","86","76.76%","0","0","-"]],
["<comparison>/candidate/product/src/sources/gencc.rs",["1416","1332","5.93%","107","95","11.21%","802","747","6.86%","0","0","-"],["1416","206","85.45%","107","13","87.85%","802","70","91.27%","0","0","-"],["1416","204","85.59%","107","12","88.79%","802","68","91.52%","0","0","-"]],
["<comparison>/candidate/product/src/sources/gencc/model.rs",["529","529","0.00%","35","35","0.00%","288","288","0.00%","0","0","-"],["529","39","92.63%","35","1","97.14%","288","11","96.18%","0","0","-"],["529","39","92.63%","35","1","97.14%","288","11","96.18%","0","0","-"]],
["<comparison>/candidate/product/src/sources/gencc/store.rs",["1845","1732","6.12%","88","80","9.09%","752","712","5.32%","0","0","-"],["1845","214","88.40%","88","6","93.18%","752","44","94.15%","0","0","-"],["1845","214","88.40%","88","6","93.18%","752","44","94.15%","0","0","-"]],
["<comparison>/candidate/product/src/sources/gnomad.rs",["268","205","23.51%","24","19","20.83%","200","151","24.50%","0","0","-"],["268","69","74.25%","24","6","75.00%","200","38","81.00%","0","0","-"],["268","68","74.63%","24","6","75.00%","200","38","81.00%","0","0","-"]],
["<comparison>/candidate/product/src/sources/gprofiler.rs",["194","52","73.20%","18","3","83.33%","147","33","77.55%","0","0","-"],["194","117","39.69%","18","10","44.44%","147","77","47.62%","0","0","-"],["194","36","81.44%","18","1","94.44%","147","13","91.16%","0","0","-"]],
["<comparison>/candidate/product/src/sources/gtex.rs",["275","167","39.27%","29","15","48.28%","188","108","42.55%","0","0","-"],["275","112","59.27%","29","14","51.72%","188","69","63.30%","0","0","-"],["275","42","84.73%","29","4","86.21%","188","18","90.43%","0","0","-"]],
["<comparison>/candidate/product/src/sources/gtr.rs",["913","329","63.96%","81","28","65.43%","623","244","60.83%","0","0","-"],["913","341","62.65%","81","32","60.49%","623","242","61.16%","0","0","-"],["913","201","77.98%","81","14","82.72%","623","133","78.65%","0","0","-"]],
["<comparison>/candidate/product/src/sources/gwas.rs",["363","163","55.10%","32","10","68.75%","286","122","57.34%","0","0","-"],["363","46","87.33%","32","1","96.88%","286","34","88.11%","0","0","-"],["363","45","87.60%","32","1","96.88%","286","34","88.11%","0","0","-"]],
["<comparison>/candidate/product/src/sources/hpa.rs",["663","591","10.86%","71","64","9.86%","472","422","10.59%","0","0","-"],["663","240","63.80%","71","23","67.61%","472","154","67.37%","0","0","-"],["663","200","69.83%","71","20","71.83%","472","128","72.88%","0","0","-"]],
["<comparison>/candidate/product/src/sources/hpo.rs",["342","201","41.23%","34","19","44.12%","244","141","42.21%","0","0","-"],["342","103","69.88%","34","11","67.65%","244","68","72.13%","0","0","-"],["342","102","70.18%","34","11","67.65%","244","68","72.13%","0","0","-"]],
["<comparison>/candidate/product/src/sources/interpro.rs",["147","90","38.78%","15","9","40.00%","111","57","48.65%","0","0","-"],["147","22","85.03%","15","2","86.67%","111","16","85.59%","0","0","-"],["147","20","86.39%","15","2","86.67%","111","12","89.19%","0","0","-"]],
["<comparison>/candidate/product/src/sources/kegg.rs",["441","266","39.68%","31","18","41.94%","256","150","41.41%","0","0","-"],["441","145","67.12%","31","14","54.84%","256","82","67.97%","0","0","-"],["441","74","83.22%","31","7","77.42%","256","36","85.94%","0","0","-"]],
["<comparison>/candidate/product/src/sources/litsense2.rs",["134","59","55.97%","16","7","56.25%","111","52","53.15%","0","0","-"],["134","66","50.75%","16","8","50.00%","111","51","54.05%","0","0","-"],["134","9","93.28%","16","0","100.00%","111","4","96.40%","0","0","-"]],
["<comparison>/candidate/product/src/sources/litvar.rs",["186","126","32.26%","22","15","31.82%","139","92","33.81%","0","0","-"],["186","6","96.77%","22","0","100.00%","139","3","97.84%","0","0","-"],["186","6","96.77%","22","0","100.00%","139","3","97.84%","0","0","-"]],
["<comparison>/candidate/product/src/sources/medlineplus.rs",["244","168","31.15%","21","13","38.10%","177","116","34.46%","0","0","-"],["244","36","85.25%","21","3","85.71%","177","26","85.31%","0","0","-"],["244","35","85.66%","21","3","85.71%","177","26","85.31%","0","0","-"]],
["<comparison>/candidate/product/src/sources/mod.rs",["2743","1461","46.74%","281","132","53.02%","2035","1041","48.85%","0","0","-"],["2743","185","93.26%","281","7","97.51%","2035","100","95.09%","0","0","-"],["2743","153","94.42%","281","5","98.22%","2035","79","96.12%","0","0","-"]],
["<comparison>/candidate/product/src/sources/monarch.rs",["679","534","21.35%","53","38","28.30%","515","390","24.27%","0","0","-"],["679","181","73.34%","53","18","66.04%","515","140","72.82%","0","0","-"],["679","100","85.27%","53","8","84.91%","515","70","86.41%","0","0","-"]],
["<comparison>/candidate/product/src/sources/mutalyzer.rs",["276","90","67.39%","30","15","50.00%","216","68","68.52%","0","0","-"],["276","37","86.59%","30","6","80.00%","216","23","89.35%","0","0","-"],["276","34","87.68%","30","5","83.33%","216","22","89.81%","0","0","-"]],
["<comparison>/candidate/product/src/sources/mychem.rs",["181","62","65.75%","18","6","66.67%","124","35","71.77%","0","0","-"],["181","15","91.71%","18","1","94.44%","124","8","93.55%","0","0","-"],["181","14","92.27%","18","1","94.44%","124","8","93.55%","0","0","-"]],
["<comparison>/candidate/product/src/sources/mydisease.rs",["384","194","49.48%","30","17","43.33%","297","160","46.13%","0","0","-"],["384","23","94.01%","30","0","100.00%","297","13","95.62%","0","0","-"],["384","22","94.27%","30","0","100.00%","297","13","95.62%","0","0","-"]],
["<comparison>/candidate/product/src/sources/mygene.rs",["1003","732","27.02%","84","53","36.90%","579","389","32.82%","0","0","-"],["1003","70","93.02%","84","3","96.43%","579","25","95.68%","0","0","-"],["1003","68","93.22%","84","3","96.43%","579","25","95.68%","0","0","-"]],
["<comparison>/candidate/product/src/sources/myvariant.rs",["1315","859","34.68%","121","86","28.93%","949","580","38.88%","0","0","-"],["1315","226","82.81%","121","38","68.60%","949","145","84.72%","0","0","-"],["1315","223","83.04%","121","38","68.60%","949","145","84.72%","0","0","-"]],
["<comparison>/candidate/product/src/sources/myvariant/filter_conflicts.rs",["77","33","57.14%","4","0","100.00%","40","11","72.50%","0","0","-"],["77","0","100.00%","4","0","100.00%","40","0","100.00%","0","0","-"],["77","0","100.00%","4","0","100.00%","40","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/sources/myvariant/filter_keys.rs",["31","12","61.29%","1","0","100.00%","18","8","55.56%","0","0","-"],["31","1","96.77%","1","0","100.00%","18","0","100.00%","0","0","-"],["31","1","96.77%","1","0","100.00%","18","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/sources/ncbi_efetch.rs",["1853","1669","9.93%","120","103","14.17%","1067","928","13.03%","0","0","-"],["1853","84","95.47%","120","2","98.33%","1067","35","96.72%","0","0","-"],["1853","83","95.52%","120","2","98.33%","1067","35","96.72%","0","0","-"]],
["<comparison>/candidate/product/src/sources/ncbi_idconv.rs",["160","58","63.75%","19","6","68.42%","113","41","63.72%","0","0","-"],["160","33","79.38%","19","4","78.95%","113","19","83.19%","0","0","-"],["160","33","79.38%","19","4","78.95%","113","19","83.19%","0","0","-"]],
["<comparison>/candidate/product/src/sources/nci_cts.rs",["844","204","75.83%","58","10","82.76%","520","95","81.73%","0","0","-"],["844","71","91.59%","58","1","98.28%","520","11","97.88%","0","0","-"],["844","70","91.71%","58","1","98.28%","520","11","97.88%","0","0","-"]],
["<comparison>/candidate/product/src/sources/nih_reporter.rs",["275","163","40.73%","31","18","41.94%","201","115","42.79%","0","0","-"],["275","80","70.91%","31","10","67.74%","201","51","74.63%","0","0","-"],["275","22","92.00%","31","3","90.32%","201","12","94.03%","0","0","-"]],
["<comparison>/candidate/product/src/sources/ols4.rs",["123","15","87.80%","12","2","83.33%","82","11","86.59%","0","0","-"],["123","10","91.87%","12","1","91.67%","82","5","93.90%","0","0","-"],["123","9","92.68%","12","1","91.67%","82","5","93.90%","0","0","-"]],
["<comparison>/candidate/product/src/sources/oncokb.rs",["204","31","84.80%","22","3","86.36%","145","23","84.14%","0","0","-"],["204","131","35.78%","22","17","22.73%","145","90","37.93%","0","0","-"],["204","23","88.73%","22","3","86.36%","145","14","90.34%","0","0","-"]],
["<comparison>/candidate/product/src/sources/opencitations.rs",["383","383","0.00%","25","25","0.00%","242","242","0.00%","0","0","-"],["383","10","97.39%","25","0","100.00%","242","2","99.17%","0","0","-"],["383","10","97.39%","25","0","100.00%","242","2","99.17%","0","0","-"]],
["<comparison>/candidate/product/src/sources/openfda.rs",["598","340","43.14%","54","26","51.85%","489","269","44.99%","0","0","-"],["598","128","78.60%","54","12","77.78%","489","86","82.41%","0","0","-"],["598","57","90.47%","54","3","94.44%","489","38","92.23%","0","0","-"]],
["<comparison>/candidate/product/src/sources/opentargets.rs",["1822","1503","17.51%","137","106","22.63%","1238","1010","18.42%","0","0","-"],["1822","255","86.00%","137","25","81.75%","1238","199","83.93%","0","0","-"],["1822","220","87.93%","137","20","85.40%","1238","174","85.95%","0","0","-"]],
["<comparison>/candidate/product/src/sources/orcid.rs",["695","148","78.71%","65","14","78.46%","467","100","78.59%","0","0","-"],["695","68","90.22%","65","5","92.31%","467","37","92.08%","0","0","-"],["695","53","92.37%","65","4","93.85%","467","28","94.00%","0","0","-"]],
["<comparison>/candidate/product/src/sources/ordinary_url_policy.rs",["409","153","62.59%","42","14","66.67%","304","116","61.84%","0","0","-"],["409","32","92.18%","42","6","85.71%","304","19","93.75%","0","0","-"],["409","32","92.18%","42","6","85.71%","304","19","93.75%","0","0","-"]],
["<comparison>/candidate/product/src/sources/pharmacodb.rs",["469","469","0.00%","50","50","0.00%","327","327","0.00%","0","0","-"],["469","161","65.67%","50","18","64.00%","327","95","70.95%","0","0","-"],["469","161","65.67%","50","18","64.00%","327","95","70.95%","0","0","-"]],
["<comparison>/candidate/product/src/sources/pharmgkb.rs",["343","343","0.00%","37","37","0.00%","299","299","0.00%","0","0","-"],["343","119","65.31%","37","14","62.16%","299","81","72.91%","0","0","-"],["343","119","65.31%","37","14","62.16%","299","81","72.91%","0","0","-"]],
["<comparison>/candidate/product/src/sources/pmc_article.rs",["1057","693","34.44%","71","34","52.11%","663","415","37.41%","0","0","-"],["1057","92","91.30%","71","6","91.55%","663","62","90.65%","0","0","-"],["1057","92","91.30%","71","6","91.55%","663","61","90.80%","0","0","-"]],
["<comparison>/candidate/product/src/sources/pmc_oa.rs",["654","219","66.51%","60","24","60.00%","450","135","70.00%","0","0","-"],["654","247","62.23%","60","26","56.67%","450","156","65.33%","0","0","-"],["654","111","83.03%","60","17","71.67%","450","59","86.89%","0","0","-"]],
["<comparison>/candidate/product/src/sources/provider_url_policy.rs",["1305","746","42.84%","86","43","50.00%","875","473","45.94%","0","0","-"],["1305","180","86.21%","86","18","79.07%","875","91","89.60%","0","0","-"],["1305","138","89.43%","86","16","81.40%","875","66","92.46%","0","0","-"]],
["<comparison>/candidate/product/src/sources/pubmed.rs",["819","506","38.22%","83","57","31.33%","582","360","38.14%","0","0","-"],["819","111","86.45%","83","10","87.95%","582","68","88.32%","0","0","-"],["819","110","86.57%","83","10","87.95%","582","68","88.32%","0","0","-"]],
["<comparison>/candidate/product/src/sources/pubtator.rs",["368","173","52.99%","30","13","56.67%","247","115","53.44%","0","0","-"],["368","25","93.21%","30","0","100.00%","247","13","94.74%","0","0","-"],["368","24","93.48%","30","0","100.00%","247","13","94.74%","0","0","-"]],
["<comparison>/candidate/product/src/sources/quickgo.rs",["123","55","55.28%","14","6","57.14%","94","35","62.77%","0","0","-"],["123","76","38.21%","14","9","35.71%","94","54","42.55%","0","0","-"],["123","30","75.61%","14","3","78.57%","94","15","84.04%","0","0","-"]],
["<comparison>/candidate/product/src/sources/rate_limit.rs",["771","523","32.17%","62","36","41.94%","503","317","36.98%","0","0","-"],["771","8","98.96%","62","0","100.00%","503","3","99.40%","0","0","-"],["771","7","99.09%","62","0","100.00%","503","2","99.60%","0","0","-"]],
["<comparison>/candidate/product/src/sources/reactome.rs",["368","237","35.60%","45","31","31.11%","259","161","37.84%","0","0","-"],["368","220","40.22%","45","29","35.56%","259","148","42.86%","0","0","-"],["368","171","53.53%","45","23","48.89%","259","111","57.14%","0","0","-"]],
["<comparison>/candidate/product/src/sources/seer.rs",["654","586","10.40%","55","46","16.36%","453","399","11.92%","0","0","-"],["654","228","65.14%","55","23","58.18%","453","155","65.78%","0","0","-"],["654","174","73.39%","55","17","69.09%","453","121","73.29%","0","0","-"]],
["<comparison>/candidate/product/src/sources/semantic_scholar.rs",["891","422","52.64%","82","38","53.66%","718","343","52.23%","0","0","-"],["891","147","83.50%","82","14","82.93%","718","100","86.07%","0","0","-"],["891","117","86.87%","82","11","86.59%","718","74","89.69%","0","0","-"]],
["<comparison>/candidate/product/src/sources/string.rs",["85","12","85.88%","9","1","88.89%","70","8","88.57%","0","0","-"],["85","57","32.94%","9","7","22.22%","70","44","37.14%","0","0","-"],["85","11","87.06%","9","1","88.89%","70","5","92.86%","0","0","-"]],
["<comparison>/candidate/product/src/sources/trust_failure_tests/mod.rs",["87","87","0.00%","11","11","0.00%","83","83","0.00%","0","0","-"],["87","0","100.00%","11","0","100.00%","83","0","100.00%","0","0","-"],["87","0","100.00%","11","0","100.00%","83","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/sources/umls.rs",["213","115","46.01%","20","11","45.00%","158","80","49.37%","0","0","-"],["213","64","69.95%","20","7","65.00%","158","37","76.58%","0","0","-"],["213","63","70.42%","20","7","65.00%","158","37","76.58%","0","0","-"]],
["<comparison>/candidate/product/src/sources/uniprot.rs",["869","585","32.68%","90","54","40.00%","581","390","32.87%","0","0","-"],["869","145","83.31%","90","13","85.56%","581","95","83.65%","0","0","-"],["869","144","83.43%","90","13","85.56%","581","95","83.65%","0","0","-"]],
["<comparison>/candidate/product/src/sources/vaers.rs",["469","469","0.00%","52","52","0.00%","359","359","0.00%","0","0","-"],["469","211","55.01%","52","27","48.08%","359","165","54.04%","0","0","-"],["469","211","55.01%","52","27","48.08%","359","165","54.04%","0","0","-"]],
["<comparison>/candidate/product/src/sources/variantvalidator.rs",["291","151","48.11%","30","18","40.00%","228","106","53.51%","0","0","-"],["291","38","86.94%","30","5","83.33%","228","24","89.47%","0","0","-"],["291","37","87.29%","30","5","83.33%","228","24","89.47%","0","0","-"]],
["<comparison>/candidate/product/src/sources/who_ivd.rs",["477","318","33.33%","48","28","41.67%","322","205","36.34%","0","0","-"],["477","241","49.48%","48","23","52.08%","322","147","54.35%","0","0","-"],["477","127","73.38%","48","10","79.17%","322","67","79.19%","0","0","-"]],
["<comparison>/candidate/product/src/sources/who_pq.rs",["1392","365","73.78%","115","31","73.04%","980","259","73.57%","0","0","-"],["1392","239","82.83%","115","24","79.13%","980","179","81.73%","0","0","-"],["1392","218","84.34%","115","22","80.87%","980","169","82.76%","0","0","-"]],
["<comparison>/candidate/product/src/sources/wikipathways.rs",["464","220","52.59%","50","26","48.00%","316","137","56.65%","0","0","-"],["464","179","61.42%","50","20","60.00%","316","106","66.46%","0","0","-"],["464","107","76.94%","50","11","78.00%","316","53","83.23%","0","0","-"]],
["<comparison>/candidate/product/src/transform/adverse_event.rs",["769","472","38.62%","68","49","27.94%","527","326","38.14%","0","0","-"],["769","442","42.52%","68","44","35.29%","527","274","48.01%","0","0","-"],["769","323","58.00%","68","35","48.53%","527","197","62.62%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article.rs",["41","19","53.66%","2","1","50.00%","45","30","33.33%","0","0","-"],["41","0","100.00%","2","0","100.00%","45","0","100.00%","0","0","-"],["41","0","100.00%","2","0","100.00%","45","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/anchors.rs",["100","13","87.00%","12","2","83.33%","66","10","84.85%","0","0","-"],["100","2","98.00%","12","0","100.00%","66","3","95.45%","0","0","-"],["100","2","98.00%","12","0","100.00%","66","3","95.45%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/annotations.rs",["533","434","18.57%","41","37","9.76%","373","308","17.43%","0","0","-"],["533","19","96.44%","41","3","92.68%","373","13","96.51%","0","0","-"],["533","19","96.44%","41","3","92.68%","373","13","96.51%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/federation.rs",["784","300","61.73%","76","30","60.53%","515","178","65.44%","0","0","-"],["784","77","90.18%","76","8","89.47%","515","36","93.01%","0","0","-"],["784","73","90.69%","76","6","92.11%","515","34","93.40%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/html.rs",["643","643","0.00%","43","43","0.00%","422","422","0.00%","0","0","-"],["643","32","95.02%","43","2","95.35%","422","19","95.50%","0","0","-"],["643","32","95.02%","43","2","95.35%","422","19","95.50%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/jats.rs",["952","307","67.75%","72","11","84.72%","545","168","69.17%","0","0","-"],["952","80","91.60%","72","2","97.22%","545","41","92.48%","0","0","-"],["952","80","91.60%","72","2","97.22%","545","41","92.48%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/jats/refs.rs",["825","699","15.27%","54","40","25.93%","449","374","16.70%","0","0","-"],["825","280","66.06%","54","11","79.63%","449","129","71.27%","0","0","-"],["825","280","66.06%","54","11","79.63%","449","129","71.27%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/jats/refs/citation.rs",["687","687","0.00%","49","49","0.00%","453","453","0.00%","0","0","-"],["687","33","95.20%","49","1","97.96%","453","24","94.70%","0","0","-"],["687","33","95.20%","49","1","97.96%","453","24","94.70%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/jats/supplements.rs",["222","52","76.58%","20","2","90.00%","155","34","78.06%","0","0","-"],["222","10","95.50%","20","1","95.00%","155","4","97.42%","0","0","-"],["222","8","96.40%","20","0","100.00%","155","3","98.06%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/jats/tables.rs",["205","13","93.66%","17","2","88.24%","109","6","94.50%","0","0","-"],["205","6","97.07%","17","0","100.00%","109","2","98.17%","0","0","-"],["205","6","97.07%","17","0","100.00%","109","2","98.17%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/pdf.rs",["64","36","43.75%","6","4","33.33%","46","30","34.78%","0","0","-"],["64","9","85.94%","6","2","66.67%","46","5","89.13%","0","0","-"],["64","9","85.94%","6","2","66.67%","46","5","89.13%","0","0","-"]],
["<comparison>/candidate/product/src/transform/article/supplement_text.rs",["685","543","20.73%","26","16","38.46%","506","395","21.94%","0","0","-"],["685","204","70.22%","26","6","76.92%","506","160","68.38%","0","0","-"],["685","204","70.22%","26","6","76.92%","506","160","68.38%","0","0","-"]],
["<comparison>/candidate/product/src/transform/disease.rs",["1565","987","36.93%","115","72","37.39%","995","593","40.40%","0","0","-"],["1565","169","89.20%","115","11","90.43%","995","82","91.76%","0","0","-"],["1565","166","89.39%","115","10","91.30%","995","81","91.86%","0","0","-"]],
["<comparison>/candidate/product/src/transform/drug.rs",["2163","953","55.94%","188","63","66.49%","1343","577","57.04%","0","0","-"],["2163","117","94.59%","188","11","94.15%","1343","62","95.38%","0","0","-"],["2163","114","94.73%","188","10","94.68%","1343","61","95.46%","0","0","-"]],
["<comparison>/candidate/product/src/transform/gene.rs",["569","242","57.47%","38","9","76.32%","318","115","63.84%","0","0","-"],["569","78","86.29%","38","3","92.11%","318","39","87.74%","0","0","-"],["569","31","94.55%","38","0","100.00%","318","10","96.86%","0","0","-"]],
["<comparison>/candidate/product/src/transform/pathway.rs",["207","192","7.25%","14","11","21.43%","166","145","12.65%","0","0","-"],["207","0","100.00%","14","0","100.00%","166","0","100.00%","0","0","-"],["207","0","100.00%","14","0","100.00%","166","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/transform/protein.rs",["220","179","18.64%","17","16","5.88%","192","160","16.67%","0","0","-"],["220","2","99.09%","17","1","94.12%","192","1","99.48%","0","0","-"],["220","2","99.09%","17","1","94.12%","192","1","99.48%","0","0","-"]],
["<comparison>/candidate/product/src/transform/trial.rs",["720","276","61.67%","100","24","76.00%","478","152","68.20%","0","0","-"],["720","23","96.81%","100","0","100.00%","478","11","97.70%","0","0","-"],["720","23","96.81%","100","0","100.00%","478","11","97.70%","0","0","-"]],
["<comparison>/candidate/product/src/transform/variant.rs",["1630","255","84.36%","171","27","84.21%","1149","145","87.38%","0","0","-"],["1630","130","92.02%","171","13","92.40%","1149","65","94.34%","0","0","-"],["1630","122","92.52%","171","10","94.15%","1149","61","94.69%","0","0","-"]],
["<comparison>/candidate/product/src/utils/date.rs",["257","137","46.69%","22","14","36.36%","164","90","45.12%","0","0","-"],["257","12","95.33%","22","3","86.36%","164","8","95.12%","0","0","-"],["257","12","95.33%","22","3","86.36%","164","8","95.12%","0","0","-"]],
["<comparison>/candidate/product/src/utils/download.rs",["620","466","24.84%","47","28","40.43%","376","280","25.53%","0","0","-"],["620","90","85.48%","47","2","95.74%","376","49","86.97%","0","0","-"],["620","90","85.48%","47","2","95.74%","376","49","86.97%","0","0","-"]],
["<comparison>/candidate/product/src/utils/fs_lock.rs",["17","2","88.24%","3","0","100.00%","11","1","90.91%","0","0","-"],["17","2","88.24%","3","0","100.00%","11","1","90.91%","0","0","-"],["17","2","88.24%","3","0","100.00%","11","1","90.91%","0","0","-"]],
["<comparison>/candidate/product/src/utils/mod.rs",["29","29","0.00%","3","3","0.00%","17","17","0.00%","0","0","-"],["29","0","100.00%","3","0","100.00%","17","0","100.00%","0","0","-"],["29","0","100.00%","3","0","100.00%","17","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/utils/query.rs",["24","6","75.00%","2","1","50.00%","15","4","73.33%","0","0","-"],["24","0","100.00%","2","0","100.00%","15","0","100.00%","0","0","-"],["24","0","100.00%","2","0","100.00%","15","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/utils/serde.rs",["48","29","39.58%","4","1","75.00%","23","11","52.17%","0","0","-"],["48","0","100.00%","4","0","100.00%","23","0","100.00%","0","0","-"],["48","0","100.00%","4","0","100.00%","23","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/utils/sync.rs",["50","46","8.00%","4","3","25.00%","24","21","12.50%","0","0","-"],["50","0","100.00%","4","0","100.00%","24","0","100.00%","0","0","-"],["50","0","100.00%","4","0","100.00%","24","0","100.00%","0","0","-"]],
["<comparison>/candidate/product/src/workflow_ladders.rs",["106","51","51.89%","7","3","57.14%","82","43","47.56%","0","0","-"],["106","18","83.02%","7","1","85.71%","82","25","69.51%","0","0","-"],["106","18","83.02%","7","1","85.71%","82","25","69.51%","0","0","-"]],
["<comparison>/candidate/product/src/xml.rs",["246","151","38.62%","17","14","17.65%","162","94","41.98%","0","0","-"],["246","5","97.97%","17","0","100.00%","162","1","99.38%","0","0","-"],["246","5","97.97%","17","0","100.00%","162","1","99.38%","0","0","-"]],
["<rust-stdlib>/std/src/sys/thread_local/native/mod.rs",["7","0","100.00%","2","0","100.00%","6","0","100.00%","0","0","-"],["7","0","100.00%","2","0","100.00%","6","0","100.00%","0","0","-"],["7","0","100.00%","2","0","100.00%","6","0","100.00%","0","0","-"]],
["TOTAL",["206102","138485","32.81%","15196","9379","38.28%","145197","95691","34.10%","0","0","-"],["206102","45341","78.00%","15196","3067","79.82%","145197","29540","79.66%","0","0","-"],["206102","34754","83.14%","15196","2250","85.19%","145197","22281","84.65%","0","0","-"]]
]
```

## Historical attempts — superseded

Everything below is historical evidence from earlier attempts. Its cross-host timing columns, failed-suite percentages and prior conclusions are not the final before/after comparison. Use the same-host measurements above for this release.

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
