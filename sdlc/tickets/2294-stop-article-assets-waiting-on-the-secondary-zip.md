---
priority: 2
---

# 2294 — Stop article assets waiting on the secondary ZIP

Status: OPEN.

Milestone: 0.9.4

Depends on: 2236

Changes: `tests/golden/cases/*`, `src/entities/article/assets.rs`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/primary_archive.rs`, `sdlc/records/2294-primary-article-asset-resolution.md`, `CHANGELOG.md`.

Reviews: revision 3d9472e, accept

Reviews: revision 148474f, accept

Reviews: revision 98152f6, accept

## Outcome

An article assets or asset-bytes request does not call or wait on the optional Europe PMC supplementaryFiles ZIP when the primary PMC archive answer is complete and sufficient. Existing XML, HTML and Figshare discovery can add assets and retain their existing budgets. When the primary fails or lacks the asset, the secondary sources still serve as fallback with their existing budgets and unavailable labels.

- Proof: An outside-in replay case where the primary archive succeeds and the ZIP route hangs returns the primary assets without waiting out the ZIP budget, and its output names the same assets as today. A case where the primary fails still reaches the ZIP and other fallbacks. Existing availability and fallback controls pass unchanged.

## Evidence

The 2236 coverage lane read the assets flow: resolution starts the PMC archive, the Europe PMC ZIP and XML, NCBI XML and official HTML together with `tokio::join!`. The ZIP call runs even after the primary archive succeeds, is NoStore, and has a 10-second optional-source budget, so repeated asset and byte requests can pay that wait each time. A live capture of PMC3467229 saw the ZIP time out twice while the primary assets succeeded. Healthy full text did not request the ZIP.

Keep the change local to assets resolution. Keep source labels and unavailable reporting truthful. Add no new cache layer unless the measured case requires one.

## Reproduction and implementation

Return the usable primary assets without making an unnecessary supplementary ZIP request. Preserve the existing fallback for missing primary bytes and failed primary archives. Starting source: 63e786985. Gator job 1792514074258-861653 ran the real CLI control against unchanged production source plus test-only overlay 6a3b6bf13; it failed after 10.759 seconds because the complete-primary request contacted the hanging ZIP. The missing-byte fallback control passed. Exact commands and retained evidence are in `sdlc/records/2294-primary-article-asset-resolution.md`.

PM approved updating the two superseded primary-success empty-ZIP observation assertions. The explicit historical refusal handle, failed/missing-primary fallbacks, existing budgets and unavailable labels remain covered. No new cache or public provider calls are involved.
