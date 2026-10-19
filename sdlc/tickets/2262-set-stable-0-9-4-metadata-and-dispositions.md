---
priority: 1
---

# 2262 — Set stable 0.9.4 metadata and dispositions

Status: OPEN.

Milestone: 0.9.4

Depends on: 2244, 2259, 2260, 2261, 2247, 2251, 2252, 2253, 2254, 2256, 2111, 2263, 2264, 2265, 2266, 2267, 2268, 2269, 2270, 2271, 2273, 2274

## Outcome

One atomic commit on main carries everything step 1 of 2258 needs: stable 0.9.4 in every public version file, the dated changelog section `## 0.9.4 — YYYY-MM-DD` citing every ticket landed since v0.9.3 (including the twelve found missing: 2191, 2202, 2203, 2204, 2205, 2210, 2211, 2213, 2214, 2228, 2231, 2232), and an explicit disposition line for each finding in the `sdlc/issues/` files this release touches.

## Changes

- Version files and `CHANGELOG.md`, as `scripts/check-version-sync.sh` and `scripts/check-release-versions.py` define them.
- Dispositions for both 2026-10-07 review files as mapped in the PM's answer of 2026-10-19 (fixed, moot, process or historical, each with evidence; the history-rewrite finding records Ian's ruling).
- Stale status lines: the disable-sources toggle issue file (shipped in 0.9.3, reply posted and closed) and the GTR diagnostic-synonym file (resolved by 2242).
- Remove every machine name from `sdlc/` in this public repository, starting with `sdlc/planning/onboarding.md`. Role words stay.

## Keeps

No product code. Describe each landed outcome honestly, including studies that changed nothing.

## Proof

`check-version-sync.sh`, `check-release-versions.py --tag v0.9.4` and `check-changelog-coverage.py --tag v0.9.4 --revision <sha>` pass without creating a tag. A grep of `sdlc/` for the workspace's machine names finds none.
