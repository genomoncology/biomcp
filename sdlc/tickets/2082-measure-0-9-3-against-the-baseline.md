---
priority: 4
---

# 2082 — Measure both releases under comparable conditions

Status: OPEN.

Milestone: 0.9.3
Depends on: 2073
Depends on: 2081
Depends on: 2094
Depends on: 2122

Depends on: 2088
Depends on: 2095
Depends on: 2120
Depends on: 2128
Depends on: 2129
Depends on: 2139
Depends on: 2149
Depends on: 2151
Depends on: 2152
Depends on: 2158
Depends on: 2159
Depends on: 2160
Depends on: 2161
Depends on: 2162
Depends on: 2164
Depends on: 2165

Depends on: 2166

Depends on: 2167

Depends on: 2168

Depends on: 2169
Depends on: 2172
Depends on: 2173
Depends on: 2174
Depends on: 2177

## Outcome

Fresh v0.9.2 and release-candidate measurements run on the same build machine with the same corrected harness, toolchain, workloads, provider settings and cache preparation, and the baseline record reports raw samples, honest comparisons and remaining failures.

- Starts from: `tools/measure-0-9-3-baseline` and `sdlc/records/2069-0-9-3-baseline.md`. Earlier cross-host, timed-out and failed-suite values are historical and superseded, not a valid before column.
- Keeps: exact immutable product revisions, the exact corrected harness revision, tool versions, command arguments, environment variable names without secret values, suite exit results and raw samples. Do not modify either measured product revision to make the harness run; distinguish harness compatibility work from product changes.
- Changes: claims `tools/measure-0-9-3-baseline` `tests/test_measurement_summary_contract.py` `sdlc/records/2069-0-9-3-baseline.md` `sdlc/release-checklist.md` `docs/reference/release-process.md` `CHANGELOG.md`. Run both products anew through the build job under matching settings. Alternate warm and cache-off pairs, prefetch consistently and isolate each cache-off sample so it cannot destroy the warm cache. Compute a true median, including the even-sized case.
- Proof: report routine lint/test/spec timings, build and release measurements, product test-code share, and comparable coverage using 2120's corrected collection; a failed or missing collection is a separate completeness result. Record 2117's clean and forced-failure behavior. Include five live search-all samples with GWAS errors, ten-run drug-card comparisons with output checksums, and day-cached GWAS/openFDA comparisons. Separate local product cost from live-provider timing and timeouts. Mark each applicable approved target met or missed with its measured number; retain the local `make test` and `make spec` targets and the approved test-code-share target, replace obsolete CI timing with local measurements, and report thirty-second slow-test warnings without failing solely on time. Warm-cache wins are observations, never an acceptance precondition. Explain any unavailable row rather than inventing a value.
- Release exit: after this ticket and every other release ticket land, run the entire manual untagged, nonpublishing release workflow on final main using 2161's rehearsal mode. Record the actual run and immutable revision in pm and report the result in chat; do not create a follow-up commit solely to record that final run. Repair any failure before repeating the final rehearsal. Ian alone creates the release tag.
- Defers: publication to Ian; no first-attempt success requirement and no automatic branch CI requirement.

PM decision: 2026-10-17. Measure both versions on one build machine. This ticket lands last; the final rehearsal follows all landings, avoiding a self-referential requirement to commit proof for its own final commit.

PM decision: 2026-10-17. Coverage uses the existing groups. The test deletion, parallel lanes and audit move land before this measurement.

PM decision: recurrence tickets 2175 and 2176 remain open through the final release rehearsal. Non-recurrence across the remaining full checks and rehearsal is sufficient for closure, with clean-check counts recorded; 2176 must include checks after the cache changes. They therefore do not block final measurement. All remaining product bugs, including 2172, 2173 and 2174, still precede this measurement. If a recurrence occurs, its fix precedes release.

Coordinator initial outcome before the subsequent main recurrence: Replace the unusable cross-host before/after column with fresh, comparable measurements of immutable v0.9.2 and the then-candidate0.9.3 product atda3be5638db5f6b21b395998e412e6bbe9587b0c on the same build host. The fifty isolated MCP passes did not establish stability under the full concurrent workload; that candidate subsequently failed and is not the final measured candidate. Only2175/2176 observation remains open by the PM's explicit exception through final rehearsal. Existing code supplies the corrected measurement and coverage harness; prefer it unchanged. No product fix is planned. Before any necessary harness edit, report the actual command failure, exact correction and claimed files. Preserve both measured product revisions and record the separate immutable harness revision. Final measurements must wait for the current mandatory full check of main to finish so this repository does not load the host while timing itself. Run comparisons through build jobs, retain raw samples and honest completeness/target results, and separate provider variability from local cost. No new optimization or passing-warm-cache requirement is authorized by a missed target. Reuse qualified2120 coverage tooling and settings; do not run an extra coverage preflight before the final collections. The final untagged nonpublishing rehearsal follows this ticket's landing, not this record.

## Progress

- 2026-10-17 started

Harness correction scope before code: On the measured host GNU time is absent; the current fallback appends a parenthetical notice to executable Cargo commands, producing invalid shell syntax. The runtime block also references unimported sys, groups warm samples before cache-off samples, shares the cache directory and discards needed sample output/status detail. Claim the existing measurement contract module as well as the harness. First add an outside-in runtime entry case using a clean fixture clone and fake prebuilt CLI; run the actual `--only runtime` entry through the build job and capture its failure before changing the harness. Cover alternating warm/off pairs, independent off caches, retained stdout/stderr/status/checksum/provider errors, ten drug samples and the true even median. Keep the correction in these two existing files; preserve qualified2120 coverage. Use the native time command where available and keep missing-tool annotations separate from executable commands. Ordinary compiler/cache/linker settings remain intact; final coverage retains its already qualified instrumentation settings and restores them before other phases. No formal timing runs until the current main check finishes.

Actual harness RED before implementation: In build job1792268262446-20740, `uv run --no-sync pytest -q .cache/measurement-harness-proof/tests/test_measurement_summary_contract.py -k 'runtime_retains or build_measurement'` failed both outside-in cases, with seven deselected. The real runtime entry exited1 with `NameError: name 'sys' is not defined`. The cold-dev entry exited127 with `bash: -f: command not found`: when no GNU time is available, the finder returns success with an empty TIME_BIN, incorrectly selecting its GNU branch. The parenthetical shell fallback is a separate compatibility defect. Fix only the claimed harness/contract module: truthful capability failure/native time fallback, import sys, paired alternating independent caches and required raw sample/error/checksum retention. Keep qualified coverage unchanged. Earlier missing-pytest setup ran no tests and is not RED. Main da3be56's concurrent MCP recurrence now holds formal measurements; finish only the independent harness correction until repaired main is green and the measured candidate is updated.

Qualified harness correction: Sourcee1425db386682c25615b7552229d1cf91474199b changes only the existing measurement tool and contract module. Job1792268262446-20740 recorded the two genuine entry failures above, then passed all nine contracts after correction, including the seven existing summary/clean-and-forced-failure cases. Mechanical follow-up1792268442145-35445 passed Ruff, formatting and shell syntax. Qualified2120 coverage is unchanged byte-for-byte. Runtime observations retain raw sample values, output, error, exit and checksum data; warm/off pairs alternate with independent off caches, drug comparisons have ten samples, search-all has five pairs, default day-cache rows are separate, and statistics.median handles even samples. Ordinary dependency job1792268522897-42157 installed the recorded cargo-llvm-lines0.4.48 version without configuration changes; it measures LLVM IR size separately from source test-code share. Anonymous runtime comparisons will remove only OPENFDA_API_KEY from both child process environments, preserving global settings and the same provider conditions; report anonymous quota/variability and do not infer zero requests from latency alone. Review this harness before formal collection. Formal data remains absent until the MCP repair establishes a new green immutable product candidate.

Completeness boundary before final harness review: Static inspection found the existing census section masks its nextest command failure with `|| true`. The approved measurement requires failed or missing collections to remain visibly incomplete. Exercise the real census entry with a deliberately failing fake nextest command before changing it, then preserve its actual exit status and retained output in the same existing harness. This small correction belongs to the already claimed two files; no external interception framework or separate census rerun should substitute for truthful status reporting. Preserve successful census behavior and the existing2117 summary contracts.

Census entry RED before correction: The Python-only build-job phase running `uv run --no-sync pytest -q .cache/measurement-harness-proof/tests/test_measurement_summary_contract.py -k census_retains` failed both cases, with nine deselected. Both the successful fake runner and exit42 fake runner left census.exit absent while the harness returned0; raw events remained, but explicit status and complete/incomplete labeling were missing because the command used `|| true`. Reuse the existing timedsh_exit helper and a census.exit artifact, remove that masking and emit explicit collection completeness from the retained status. Preserve raw output, parser and duration samples; a failed duration must not be described as a successful collection. No new helper framework or formal product run is needed for this correction.

Census correction GREEN: Python-only job1792268711571-66185 recorded the two entry failures above, then passed all eleven contracts in6.53seconds, plus Ruff formatting/lint and shell syntax. Local harness sourcebdd86c3087dbf5e3fa9a851ed31f7a0eed09b429 adds only the existing-helper census status correction and its success/failure cases to the previously qualified runtime/native-time changes. The real runner status remains in census.exit with an explicit complete or incomplete result while the harness can continue collecting later sections. Failed raw durations remain visible and must not become successful timing claims. All coverage code remains unchanged. Source writes are stopped for a fresh review of this harness component; the complete measurement ticket is not yet ready for acceptance because no formal product comparison has run.

Fresh harness review probe: Review suspected that the native release fallback might reject its time-command flags. Actual release-entry probe1792269007090-91668 passed unchanged on the build host, including separate build output, timing and native RSS assertions. No RED occurred and no harness correction was warranted. The temporary extra test was removed, leaving the two harness files identical to5fb552375. Fresh component review accepts5fb5523758e312716684a3fda4d8694cc8dbfa66 for pairing, cache isolation, raw observations, median and collection completeness. This is harness-component acceptance only; formal measurements and final ticket acceptance remain pending.

Coordinator candidate update before formal collection: Reopened2174 captured two real TCP fixture defects with failing split-header probes, preserved all product assertions, and passed its concurrent Rust repair proof. Fresh review accepted81142aaac7ed805bda94310431ef746e3ebe17ce; the local landing gate passed and the repair landed as234d2b32d3b5068dc27b2543d676997b75ad5baa. Required actual-main job2156/1792269541418-30247 passed207.724seconds on that exact revision, including shipped-feature tests and PNG/SVG/terminal output checks. Formal comparison is now authorized with no competing BioMCP build job. Compare immutable v0.9.2 product4049666f5cce2794f16cdbdf0ded49878e5bb147 with repaired product234d2b32d3b5068dc27b2543d676997b75ad5baa using the same reviewed harness snapshotdedbc1307e65c21c6f79bdb49025f209def074ac; harness code was accepted at5fb5523758e312716684a3fda4d8694cc8dbfa66 and the later snapshot changes only its ticket record. These separate identities remain fixed during collection. The exact historical HTTP transport category remains unconfirmed. No formal product comparison has run yet.
