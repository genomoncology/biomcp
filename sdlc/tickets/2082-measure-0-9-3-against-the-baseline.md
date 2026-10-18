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
Depends on: 2207
Depends on: 2208
Depends on: 2209

Reviews: revision 4160f2f, accept

Reviews: revision ff6ce34, accept

Reopened: 2026-10-17 — Refresh the measurement rows the five research fixes affect, on current main, keeping unchanged rows and their source identities.

Depends on: 2179
Depends on: 2180
Depends on: 2181
Depends on: 2182
Depends on: 2183

Reviews: revision dff1bfc, accept

Reopened: 2026-10-18 — Refresh affected runtime and coverage measurements after additional release fixes; retain unaffected evidence under its measured revision.

Reviews: revision ddc1729, accept

Reopened: 2026-10-18 — The rejected final rehearsal requires two repairs; refresh only measurement rows affected by any production change and keep the prior measurements under their original source identity. Land the final record after both repairs, then rehearse the replacement candidate.

Reviews: revision 075ab15, accept

## Outcome

Fresh v0.9.2 and release-candidate measurements run on the same build machine with the same corrected harness, toolchain, workloads, provider settings and cache preparation, and the baseline record reports raw samples, honest comparisons and remaining failures.

- Starts from: `tools/measure-0-9-3-baseline` and `sdlc/records/2069-0-9-3-baseline.md`. Earlier cross-host, timed-out and failed-suite values are historical and superseded, not a valid before column.
- Keeps: exact immutable product revisions, the exact corrected harness revision, tool versions, command arguments, environment variable names without secret values, suite exit results and raw samples. Do not modify either measured product revision to make the harness run; distinguish harness compatibility work from product changes.
- Changes: `sdlc/records/2069-0-9-3-baseline.md` `sdlc/tickets/2082-measure-0-9-3-against-the-baseline.md`. The completed comparable measurement collection remains at its original immutable sources. This final delta adds only the reviewed parallel-startup behavior proof and native/Linux qualification results under the latest PM decision; no fresh serial runtime, coverage, census or benchmark collection is authorized.
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

PM decision: 2026-10-17. No numeric test-code-share target was ever set; the October14 decision deferred it and no later ruling supplied one. Report measured share against the v0.9.2 baseline without a met-or-missed verdict. The recorded local test and specification timing targets still apply.

- 2026-10-17 next: Sequential same-host collection1792269937662-71934 uses reviewed harnessdedbc1307 and immutable products4049666/234d2b3 with matching Python3.12.13 and Rust tools. Old arm completed valid builds and87 runtime samples. Old Linux-only gate/spec/lint prerequisites make those rows unavailable; no old coverage suite ran. Candidate collection now runs. Two aborted setup attempts are excluded; shared host load is retained as a timing limitation.


Final measurement proof: Same-host collection2082/1792269937662-71934 and source/export jobs1792271866922-64213 and1792271923818-66165 passed. Both products retained their immutable identities, with matching actual Python3.12.13 and Rust/Cargo/uv identities before measurements. The canonical2069record now supersedes the earlier cross-host columns and retains every raw runtime sample, status, checksum, safe error class, section command/load, per-file source count and per-file coverage report. Root verified all runtime medians and source sums. All174samples exited0 without outer timeouts; measured GWAS sections had no errors. Candidate routine test118.2seconds and specification24.4seconds both meet the recorded200second targets; lint84.6seconds passed. The32.190second Python warning reported load and did not fail the gate. Candidate fresh-build and Rust-touch observations were slower, while documentation-touch and several live runtime medians were faster; shared load prevents strict causal speedup claims. No optimization or passing-warm-cache requirement was added.

Measurement limits and share: Old unchanged Linux-only gate prerequisites make its test/spec/lint/census rows unavailable on this host. Old coverage was stopped at entry: no suites, exit files or profiles, and no percentage. Candidate coverage collected all suites successfully using the existing145197-line filtered instrumented-source denominator: outside-in34.10percent, unit79.66percent, union84.65percent. It includes inline tests and some standard-library instrumentation, and covers routine features; individual report exits were not retained although wrappers returned0 and valid fresh TOTAL rows had no diagnostic errors. Primary nongenerated root-crate Rust test share, including external integration tests, changed from139914/301538=46.4001percent to149869/314914=47.5905percent; separate Python test lines changed37979to35606. No share target was set, so no met/missed verdict applies. Warm drug output hash variation is solely the cache-age metadata line, with all other lines unchanged; original hashes remain intact. Mixed-provider drug timing alone does not prove zero OpenFDA requests. The two aborted Python setup attempts contribute no performance values. The version_large_cache label uses the existing default cache without a recorded entry-count claim.

Release preparation evidence: Fresh bounded read-only dispatch review_release_readiness_through_repaired_main accepted product234d2b32d, reusing existing fresh ticket reviews and reading release/source/cache/fixture and relevant record diffs. It did not claim a new line-by-line rereview of every historical intermediate commit. The actual product full-main job2156/1792269541418-30247 passed207.724seconds with shipped-feature and PNG/SVG/terminal checks. The harness component at5fb552375 had fresh acceptance and eleven entry/summary contracts passed, including clean and forced-failure2117 behavior. The complete final measurement diff now needs its own fresh review and normal landing/full-main checks. After this final measurement landing, run one full untagged nonpublishing release workflow on the final candidate; record the actual run in pm and chat without a follow-up commit solely for that run. Keep2175and2176 open through that rehearsal under the PM's explicit exception, then close them on non-recurrence with the clean-check counts. No tag or publication has been created. Ian alone cuts the release.


Landing-gate RED before the selection correction:2082/1792272548440-79852 failed the configured make landing-gate in235.0seconds. Python test_the_binary_dependent_modules_carry_the_marker rejected tests/test_measurement_summary_contract.py;1256other Python cases passed and9skipped. The conservative guard matches the scratch fake CLI path target/release before its fake-Cargo exemption and the module lacks pytestmark. The eleven entry/summary cases use fake executables, not a real product artifact. Claim only that contract module: add its conventional needs_binary selection marker with a truthful explanation of the conservative artifact-path classification. Do not weaken the guard, change the measurement tool, or repeat formal measurements. Run the actual failing guard and eleven affected contracts, plus module Ruff checks, through one Python-only build job before fresh review. Main234d2b32d remains green and unchanged.


Selection correction GREEN: Sourcecc6a4b4f090511a4258bbb59e0b7cf1e6522c423 adds only the conventional module marker and its truthful fake-artifact-path comment. Job2082/1792272865499-11576 ran uv sync --extra dev --no-install-project, then uv run --no-sync pytest -q tests/test_ci_workflow_contract.py::test_the_binary_dependent_modules_carry_the_marker tests/test_measurement_summary_contract.py: all12cases passed. Ruff format/check passed on the module. No guard, measurement tool, product source, formal measurements or coverage method changed. This test-selection correction does not invalidate the recorded immutable product comparison. The accepted4160f2f6f data review remains evidence for the unchanged raw data; a fresh review checks this final delta and record before normal landing.


Coordinator refresh scope: PM added five research findings after the comparison landed. Preserve the completed immutable old arm and all unaffected runtime rows. After the product fixes land and main passes, refresh only affected candidate build/test/spec/lint, release size/IR, source-share and coverage observations using the existing reviewed harness and collection method. Determine which runtime rows exercise changed paths before rerunning any. No new harness feature or optimization is authorized. Read the existing record and report the minimal execution plan before collection; no build or timing while current product lanes run. The supported API proof corrections and unavailable PDF evidence are not reasons to invent performance values. Final rehearsal still follows the final measurement landing.
- 2026-10-17 next: Refresh affected candidate build, suite, size, source-share and coverage rows after the accepted five-fix batch passes actual main; preserve old arm and all unchanged runtime samples at their original revisions.

Final candidate refresh authorization: The five research fixes landed in reviewed carrierbf9916b7265437367a9a50a86a6e4ca8afb317ac, with completion records through actual main518a0e689659500bec8cbc33d195a6cbd1da78b8. Its full main-gate job2156/1792275455029-64826 passed377.096seconds, including shipped-feature tests and PNG/SVG/terminal output checks. This is the tenth clean full-main check since the startup diagnostics, and sixth after the reviewed no-store correction; no recurrence was observed. Ordinary source landings remain serial with their full checks.

Refresh only affected candidate rows using unchanged reviewed harnessdedbc1307e65c21c6f79bdb49025f209def074ac and immutable product518a0e689659500bec8cbc33d195a6cbd1da78b8. One fresh candidate-only collection selects existing clone/build/Rust-touch/docs-touch/preparation/test/census/spec/lint/release/size/IR sections, then one qualified coverage collection and the original source-count/export methods. No coverage preflight or new harness feature. Retain actual Python3.12.13 and Rust1.95 identities and ordinary compiler cache/linker settings. Complete build cleanup before timing; no other BioMCP build runs during collection. Shared host load remains a limitation.

Preserve every old-arm result at4049666f5cce2794f16cdbdf0ded49878e5bb147 and all174 runtime observations under their original candidate234d2b32d3b5068dc27b2543d676997b75ad5baa identity. The existing runtime commands never invoke asset output, supplement search, exact variant article search, article full text or PDF extraction; do not relabel them as fresh timings of518a0e6. Dependency/advisory rows are unchanged. The collector records each selected command, exit, raw result and load. Keep the existing coverage denominator and its inline-test/standard-library/routine-feature limits; report missing or failed collections separately. The final measurement record will be reviewed before normal landing; final rehearsal follows.

Candidate refresh GREEN:2082/1792275937080-64055 passed all31 sections in898.428seconds, using immutable product518a0e689 and unchanged harnessdedbc1307. The canonical2069record now contains the candidate-only raw results, per-file source counts and all3 coverage reports while retaining every old-arm/runtime observation at its original source. Routine test111.2seconds and spec24.5seconds meet200second targets; lint75.1seconds passed. Rust4364/Python1259 passed, with34/9 skips; the31.185second routine warning passed under the warning policy. Release235.4seconds, stripped binary33882992bytes. Fresh/Rust-touch build observations remain slower than the old release and shared load limits causal claims. No optimization work follows from these observations.

Coverage collected all4 suites successfully once:162 integration,4202unit,1267Python and7specification pages, with their skips retained. The existing145220-line filtered denominator includes inline tests and some standard-library instrumentation: outside34.65percent, unit79.63percent, union84.69percent. Individual report tool exits remain unrecorded; wrappers returned0 and reports had no diagnostics. Primary Rust test share149989/315068=47.6053percent, with no numeric target. Source-only and Python counts remain separate. Product HEAD matched518a0e689 and tracked source stayed unchanged;21generated untracked coverage profile files remained in the product tree. They are build output, not source modifications.

All raw logs, setup, source counts and safe reports were retrieved before cleanup; the safe export checksum is5d7cdc9215c485cce99e4a5c76f017154151d99840469ec0e746d589a671e32c. The complete raw metrics/reports are durable in2069; the scratch copy can be removed after review. Main95bdc07c1 changes only2082metadata relative to measured/qualified518a0e689. Merge that ruling and observer progress before ending acceptance. No code, workflow or harness changed in this refresh. Fresh review checks the numeric record and reuse boundaries; the existing source qualification remains applicable. Final untagged nonpublishing rehearsal still follows measurement landing and its actual-main check.
- 2026-10-18 next: Measure old v0.9.2 and final product c4281daa7304c0dd2875aa3e0d253ae9d2010eed with the frozen harness on one build host: refresh runtime, candidate four-suite coverage/source counts and stripped sizes only. Preserve historical build and gate timings at their measured revisions. Main check 2156/1792327112413-3132 passed; collection launched after confirming no other build processes.

PM decision: 2026-10-18. Jarvis declined a build-machine hold. Run the existing runtime cases now in alternating old/new pairs under the same conditions, compare pairs and record host load as a limitation. Quiet activity is recorded and cannot block this collection. The first scoped attempt, `2082/1792327543488-12063`, built the candidate successfully but stopped at quiet admission before any runtime sample; retain both binaries and resume without rebuilding. The temporary ordering adapter preserves frozen `run_once`/`report`, cases, counts, per-arm working directories, cache preparation and 180-second limits. Its SHA-256 is `267ba2aa25c778fe0ad0b309885ce4e4b36da32b243f8d16735532b6c5da74c5`. This explicitly changes version scheduling rather than claiming the frozen harness already paired versions.

PM decision: 2026-10-18. Land approved rehearsal repairs2207 and2208 in one batch. Before the next rehearsal, run the full Rust and Python suites once offline from a fresh Linux build-job clone, matching rehearsal isolation. Dispatch the next full rehearsal only after that and the native main check pass. The final measurement record remains last and refreshes only rows affected by production changes.
- 2026-10-18 next: Refresh affected runtime, coverage and source-count rows at f7c08d7c09479a3cf6c7336f779f4e46e1ce861b after both native main and fresh-clone offline Linux Rust/Python qualification pass. Optimized candidate preparation runs independently now; old artifact and earlier evidence remain retained. Collect paired runtime only after own qualification jobs finish, then land this record last and rehearse the replacement final main.
- 2026-10-18 next: Follow the latest PM limit: update only parallel-startup proof after native and fresh-clone offline Linux qualification pass. The existing harness has no parallel-startup benchmark, so retain all serial runtime, coverage and census figures at their original measured source; add the reviewed 2207/2208 behavior proof without a new benchmark or timing claim. Stop the superseded optimized build and land this minimal record last.

PM decision: 2026-10-18. Keep the refresh limited to the parallel-startup rows affected by2208. The existing helper and record contain no parallel-startup benchmark, so add only its reviewed real-process behavior proof and actual native/Linux qualification results. Retain every serial runtime, coverage and census figure at its original measured source; do not create a benchmark or claim a new startup time. PM explicitly agreed to this implementation. The superseded candidate optimized-build prerequisite `2082/1792332066970-57223` was deliberately stopped through its verified owned process group, exiting143 before any collection; this was a scope change, not a product or measurement failure.

Final scoped qualification: native main `f7c08d7c09479a3cf6c7336f779f4e46e1ce861b` passed `2156/1792331956596-45568` in387.871s. Fresh offline Linux job `2208/1792331957777-4022455` passed the entire unchanged Rust lane (4373 passed,34 skipped) but found one fake-CLI fixture HOME timeout in Python; reviewed2209 isolates only that fixture. Full Linux Python from a new clean clone of `ea810da6ce3c0af5b0aeeda120918718a667189f` now passes in `2209/1792333399989-4061171`:1286 passed,4 skipped in222.08s, strict docs passed,407.152s total including fresh build. Preserve historical measurements and source identities. This final record-only delta lands last in the accepted2209+2082 dependency batch; actual-main qualification and a new full nonpublishing release rehearsal follow.
