# 2308 — Security lock updates

Update only the vulnerable locked versions identified by the PM: certifi, urllib3, mkdocs-material and xxhash-rust. Keep dependency and manifest changes to the smallest supported set and preserve application behavior.

Before code, the developer read GitHub alerts 80, 87, 88, 89, 91 and 93. Their first patched versions are certifi 2026.10.19, urllib3 2.8.0, mkdocs-material 9.7.7 and xxhash-rust 0.8.16. Planned changes are uv.lock, Cargo.lock and the changelog. The focused job compares the current lock pins with retained authoritative alert JSON before generating the narrow updates. Initial M5 job 2308/1792530249031-74931 stopped because its Python lacked tomllib; this is a setup failure, not a reproduced advisory failure or a passing proof.

Raw alert inputs, commands, before and after locks and job results belong under M5 experiment 2102-biomcp-2258-final-live-replay/security-pins/2308. Focused red and green results, routine qualification, fresh review and automatic post-landing alert closure remain required. No GitHub alert is closed manually.

## Focused pin proof and expanded quinn scope

Corrected M5 job 2308/1792530290770-80985 passed: current locks matched all six retained GitHub alerts (FLAGGED_LOCK_PINS=6, expected exit 1); the minimal four-package update matched none (FLAGGED_LOCK_PINS=0, exit 0). Candidate 4f0ef3031 changes only those four locked versions and associated artifacts; dependency edges and manifests remain unchanged. Its routine and Linux installed-wheel runs are still pending and cannot qualify subsequent quinn changes.

The official OSV scan in 2307 additionally identified quinn-proto 0.11.15 under RUSTSEC-2026-0338 through 0342. The PM requires their minimal compatible update in this ticket: 0.11.18 clears all five. Split only if a direct dependency needs a major-version move. Preserve the vulnerable scanner JSON and bind the new green scan and qualification to the expanded candidate; no advisory suppression is authorized.

## Expanded compatible lock candidate

Quinn generation job 2308/1792530653783-9052 passed. Candidate d0b775be9 adds quinn-proto 0.11.18 while preserving both manifests and direct dependency requirements. Its resolver removes the unused rand 0.9 chain, adds rand_pcg 0.10.2 and moves JavaScript getrandom edges to the existing 0.4 package. The original four security pins remain at their minimum patched versions. The earlier dependency-edge equality statement applies only to 4f0ef3031, not this expanded lock.

Superseded pre-quinn routine and wheel runs ended with exit 143 after deliberate cancellation, and their raw partial results remain retained. They are not product failures or final qualification. Expanded-source routine 2308/1792530688392-10588 and installed Linux wheel 2308/1792530689487-1311653 are running here; final results and actual live scanner findings remain pending.
