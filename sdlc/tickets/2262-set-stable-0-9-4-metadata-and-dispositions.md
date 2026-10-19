---
priority: 1
---

# 2262 — Set stable 0.9.4 metadata and dispositions

Status: OPEN.

Milestone: 0.9.4

Depends on: 2244, 2259, 2260, 2261, 2247, 2251, 2252, 2253, 2254, 2256, 2111, 2263, 2264, 2265, 2266, 2267, 2268, 2269, 2270, 2271, 2273, 2274, 2275, 2276

Reviews: revision b18f3e8, accept

Reviews: revision 09fa4b5, accept

## Outcome

One atomic commit on main carries everything step 1 of 2258 needs: stable 0.9.4 in every public version file, the dated changelog section `## 0.9.4 — YYYY-MM-DD` citing every ticket landed since v0.9.3 (including the twelve found missing: 2191, 2202, 2203, 2204, 2205, 2210, 2211, 2213, 2214, 2228, 2231, 2232), and an explicit disposition line for each finding in the `sdlc/issues/` files this release touches.

## Changes

- Version files and `CHANGELOG.md`, as `scripts/check-version-sync.sh` and `scripts/check-release-versions.py` define them.
- Dispositions for both 2026-10-07 review files as mapped in the PM's answer of 2026-10-19 (fixed, moot, process or historical, each with evidence; the history-rewrite finding records “not rewritten by Ian’s decision 2026-10-19; current tree clean”).
- Dispositions in the sixteen historical issue files touched by this release, retaining original evidence and citing landed residual repairs.
- Current release-process version wording.
- Stale status lines: the disable-sources toggle issue file (shipped in 0.9.3, reply posted and closed) and the GTR diagnostic-synonym file (resolved by 2242).

## Keeps

No product code. Describe each landed outcome honestly, including studies that changed nothing.

## Proof

`check-version-sync.sh`, `check-release-versions.py --tag v0.9.4` and `check-changelog-coverage.py --tag v0.9.4 --revision <sha>` pass without creating a tag.

## Milestone configuration

Carry the approved milestone-only sdlc/pm.json update with this standalone metadata landing: settle 0.9.3 and activate 0.9.4 and 0.9.5. This update had arrived as inherited PM ancestry on runtime lanes; pm classifies its configuration file as a separate landing, so the runtime batch retains current main configuration and this ticket owns the unchanged update. No gate command changes.

## Final metadata proof

Developer 295f6dcc84b5958432af6767001e3826c7baaadb finalizes exactly the approved twenty-seven metadata paths directly atop actual main 17eb1e637c7a5883e70946c7f1db58ac738bbc06. All current runtime and changelog outcomes remain. The final MCP dispositions cite landed 2263 df2319982; historical prose and the approved history-rewrite disposition remain intact. Only milestone values change in pm.json; no runtime, tests, scripts, workflow or gate commands change.

M5 job 2262/1792444864948-83331 reconstructs the private candidate from published main and verifies exact Git tree and commit parity before checks. Version sync, stable v0.9.4 equality, exact-candidate changelog coverage for all seventy-four tickets since v0.9.3, 115 focused metadata tests and whitespace checks pass; no release tag was created. This is metadata proof, not full release qualification.

Keep: experiments/2095-biomcp-2262-stable-metadata — retain raw job 1792444864948-83331, exact private patch and reproducible commit/parity script.

## Deadline repair integration

Integrate actual main b7d6f83acc14eab10b832eecb133ca9d3c9a4672 while preserving the accepted metadata, dispositions, historical evidence and milestone configuration. Retain the 2276 fixture repair and changelog entry, retain the 2258 dependency on 2276, and cite this metadata landing in the release changelog. The existing 115 focused metadata tests remain applicable because their inputs and implementation remain unchanged; refresh only version sync, stable version equality, exact-candidate changelog coverage and whitespace proof.

Post-repair proof: M5 job 2262/1792446171402-28234 reconstructs candidate 10b3cdc7f5105aafca9accb07ee2d20985abff3f and tree a8f94604b726ae699e702b59690c1dd545d53f56 from published repaired main b7d6f83acc14eab10b832eecb133ca9d3c9a4672. Version sync, stable v0.9.4 equality, exact-candidate coverage for all seventy-five landed tickets, explicit 2276 and own 2262 citations, whitespace and clean-tree checks pass. The same twenty-seven approved metadata paths differ from main; twenty-five are identical to the earlier accepted metadata. Only CHANGELOG and the own record integrate the new repair/citation. No source, test, script, workflow or gate command changes. Earlier 115-test proof remains evidence for its original candidate and was not repeated. Raw new job and reproducible helper are retained under 2095-biomcp-2262-stable-metadata/1792444864948-83331/integration-1792446171402-28234. No tag or release qualification is claimed.

## Runbook wording repair

Standalone gate 2262/1792446600260-85353 stopped before tests on the existing release-process version fact audit: the runbook combined Cargo and Python labels after one shared version value, while the audit requires each label beside its package-derived version. Developer f75cc9f971366bc76d6fdea4020c79dfa6750a9e changes only that sentence to name Cargo 0.9.4 and Python 0.9.4 separately, preserving stable-candidate and development-cycle meaning. No checker, source, version metadata, changelog, milestone or disposition changed.

Focused M5 job 2262/1792446798109-18318 reconstructs the exact private commit/tree from published c0494ff5f before push. Version sync, stable equality, seventy-five-ticket changelog coverage, the existing runbook regression, fifteen release-version cases, Ruff, full quality ratchet and whitespace checks pass. Raw proof, all quality artifacts and reproducible helper remain in 2095-biomcp-2262-stable-metadata/runbook-repair/1792446798109-18318; the failed gate remains in its landing-gate/1792446600260-85353 folder. Main b7d6f83 remains fully green; no new full gate or tag is claimed for this sentence repair.
