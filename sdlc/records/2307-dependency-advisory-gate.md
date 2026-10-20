# 2307 — Scope routine advisory checks to dependency inputs

The routine landing entry point selects conditional advisory work. One selector in existing `bin/lint` compares Cargo.lock, Cargo.toml, uv.lock and pyproject.toml against a verified origin/main that must strictly precede the candidate. It includes tracked working-tree changes. Missing, equal, divergent, ahead or errored comparison context runs the check. Ordinary lint, full main and release gates always require live advisories. Licenses and remaining lint checks retain their existing behavior; no audit implementation or suppression is added. No pip or uv advisory audit existed to change.

Before production edits, Gator job 2307/1792527808874-1239817 ran the existing lint command control on repaired main 7503f6fd plus test overlay 439a8a26ab5f27d7a405dde6fd258ae5f21a066b. The unchanged-input landing expected only `check licenses`, but observed `check licenses` followed by `check advisories`. That assertion failed as required.

Definitive focused green job 2307/1792528018599-1246655 exercised committed source 93bcd37133bd12f6085c6eaef6b7c9204e4c265e, tree f27e1c8546b9e1a7f40a6f593c9610082afac4f6. Ruff, Bash syntax, ShellCheck and 89 Python controls passed. Two pre-existing gate-order table rows skipped because those stages do not belong to the landing entry point; no new advisory control skipped. The controls cover all four dependency inputs, ambiguous base contexts, staged and other tracked changes, actual Make entry-point propagation, retained lint/license behavior and required advisory failure. Reusable release qualification invokes default `make lint`; the later marker-only subset remains unchanged rather than duplicating advisory work.

Three setup-only failures remain separate: 1792527768978-1239138 used a nonexistent bundle ref, 1792527793435-1239447 lacked pytest in the fresh job environment, and 1792527914259-1241717 lacked Ruff. Corrected runs use the bundle HEAD, existing Python preparation and job-local pinned Ruff. None of these setup failures is a product or final control failure.

The red bundle SHA256 is 5e8b5051f3b2cf81668bdc18eaf6417f50b0ea1c9cfa40ce13dc885dd68d9b5b; the definitive green bundle SHA256 is 4d05485eefdf3d55ed00def575856170c6cb8a22463b36a598202cf0a8f03196. The owning experiment 2102-biomcp-2258-final-live-replay/advisory-gating/2307 retains the bundles, scripts and full raw job logs, state and results before job cleanup. Final routine qualification on a clean committed branch merged with current main remains required before fresh review and landing. No final routine pass is claimed here.

## Expanded advisory coverage and portability correction

The original-scope M5 routine 2307/1792529870260-3002 failed with exit 2. Its Make version rejects the target-specific override export assignment; three Python command controls failed. This is not a passing routine proof. The developer owns a portable target-specific assignment correction.

The PM requires GitHub advisories and uv.lock coverage because RustSec alone missed the rmcp and xxhash alerts and no Python lock audit existed. The expanded design keeps one selector in bin/lint and adds pinned official OSV-Scanner alongside cargo-deny. It must support the actual Linux and macOS qualification hosts without requiring another compiler, preserve install and scan failures, and verify vulnerable retained Cargo.lock and uv.lock against live advisory data. A mocked executable boundary proves invocation only. The earlier focused RustSec-selection proof does not qualify the expanded scanner outcome. Fresh focused controls, actual advisory detection and routine qualification remain required.

## Authoritative coverage and reviewed policy

The executable-boundary red reproduced in Gator 2307/1792530419053-1303359: the old lint never invoked the GitHub/Python scanner for either lock. Separate official live scanner job 2307/1792530432247-1303711 parsed the original 267f583 Cargo.lock and uv.lock and detected all ten distinct advisory IDs behind 2308 and 2309, plus the newer quinn advisories and existing policy exceptions. Mocked controls do not establish this live coverage.

The PM approves deriving OSV exceptions from the exact reviewed deny.toml list as the only policy source, without broadening exceptions. Linux focused job 2307/1792530839725-1319295 passed 103 controls and mechanical checks on ddb7f1f04. These cover selector and failure propagation, both complete lock inputs, exact policy derivation and bootstrap digest failures. Two existing gate-table rows skip. Mac Make portability and actual Darwin bootstrap controls remain pending.

Live policy job 2307/1792530898689-1331650 confirms the original locks remain blocked; exact 2308 d0b775be9 locks clear their six original and five quinn alerts while four rmcp advisories remain. Approved zip aliases are filtered; adding unrelated vulnerable xxhash still fails. The aggregate dependency check is not green until 2309 lands. Canonical osv-live-vulnerable and osv-live-policy folders retain official release input, binary digest, JSON, stderr, temporary policy and fixture lock inputs. No new exception or package override was added.

## Mac portability and durable scoped scan evidence

M5 2307/1792530932038-98015 passed the same 103 focused controls, mechanical checks and actual pinned Darwin-arm64 scanner bootstrap with digest verification. The old Make release-scope failure is corrected. These controls bind source ddb7f1f04. Latest-main composition f2de0d095 additionally merges reviewed main a5314c569, preserving both changelog entries; it still requires its routine gate before fresh review.

The policy evidence mirror was initially deferred and absent on M5. It is now copied and verified: 34 files, including osv-live-policy/pins-d0b775be9/results.json with SHA256 73615f218c0d3adaba4e3adbc832bdf511ddb51462027a6d7983bea9def59bb0. The manifest policy-evidence.SHA256SUMS has SHA256 e7bcd94bc5267fed48073f326aab7cfb14c433422ceb979610050236481b372f. This correction changes neither the scan result nor its tested lock inputs.

## Final routine classification correction

Routine 2307/1792531221869-74743 failed one Python CI-classification contract while 1781 cases passed. Its lexical detector interpreted a fixture label ending in the quoted word cargo as a real Cargo caller. Candidate a8bf9f19e changes only malformed and missing fixture case names to their actual Cargo.lock and uv.lock names. It changes neither the detector nor test markers or dependency requirements. The failed aggregate routine remains failed, with full raw retained under routine-fe8093832/jobs. Focused helper, classifier and Ruff checks precede the next exact routine attempt.

## Forced-path interpreter correction

Conditional routine 2307/1792531757999-79181 passed on 5a9a0e76e. It skips unchanged-input advisory execution and therefore does not prove that path uses a compatible interpreter. Actual M5 preimage 2307/1792532156514-8292 reproduced system Python 3.9.6 failing to import tomllib. Candidate fdbebabe24b1c70440832fef152aea2829ce50c6 invokes the helper through uv run --no-sync python, honoring the existing .python-version 3.12 development pin without installing the project. Project Python support starts at 3.10; it was incorrect to call 3.12 the package minimum.

M5 focused 2307/1792532243153-20795 passed Ruff and 79 affected controls, including classifier coverage. Its actual host preflight used declared Python 3.12.13 and reached the real scanner, which correctly reported pending dependency findings instead of an interpreter error. This is not a forced-main or release advisory pass. The final corrected candidate still requires its complete routine before acceptance. Canonical python-runtime-preflight.log and job records retain the actual interpreter and scanner outputs.
