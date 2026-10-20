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

## Independent live advisory scope

Official scanner job 2307/1792530898689-1331650 scanned exact d0b775be9 lock inputs under the unchanged six reviewed deny.toml exceptions. The six original GitHub alerts and all five additional quinn advisories are absent. The aggregate scanner still exits 1 for the four distinct rmcp advisories assigned to 2309; this is scoped security success for 2308, not a passing aggregate dependency scan. Canonical JSON is retained under advisory-gating/2307/osv-live-policy/pins-d0b775be9/results.json. Routine and installed Linux wheel results remain pending; fresh review may read this source while they finish.

## Qualification and independent review

M5 routine job 2308/1792530688392-10588 passed on exact source d0b775be9 with exit 0. It ran the declared landing gate, including the strict MkDocs build. The final locks remain byte-identical in the reviewed metadata head: Cargo.lock SHA256 6070ca9929f0e4f441cc19ade37ecb97d7337ecedcc23e73488f5520e61a1469 and uv.lock SHA256 d08fbccf14705bfc7db693f122415333e9bdf35c540153a9850aaeeb40be558f.

Gator job 2308/1792530689487-1311653 built the Linux wheel and completed its installed-wheel smoke through the existing pinned release recipe. Both checked subprocess calls returned normally. The aggregate command then FAILED with exit 2 because TemporaryDirectory cleanup could not remove the container-owned target/CACHEDIR.TAG. This is not a passing aggregate wheel job. Cleanup-only continuation 2308/1792531418898-1375131 passed and verified removal of that exact scratch directory. The partial original cleanup had already deleted the wheel, so the continuation proves neither artifact equivalence nor another smoke. The retained native build binary has SHA256 e0ca59e300967a2b379204a2291fe1d084b7af83a693b830fc9f178adbbbcc10. A separate shared-recipe repair remains a release blocker.

The owning security-pins/2308 experiment retains the final runners, routine-final.log, wheel-final-gator.log, cleanup-receipt.json, native binary, and complete job log/state/result directories for all three runs. Its osv-scoped/results.json and the advisory-gating/2307 canonical result have SHA256 73615f218c0d3adaba4e3adbc832bdf511ddb51462027a6d7983bea9def59bb0.

Independent review read the complete lock diff, package manifests, resolver output, advisory inputs and raw qualification results. The five repaired package versions are the first patched versions required by this ticket. The quinn resolver changes follow its published dependency declarations; quinn-proto and rand_pcg declare Rust 1.85, below the project's declared 1.89 minimum. This is a declaration check, not execution on Rust 1.89. Python dependencies remain in the existing development graph and the strict docs build uses the patched lock. No manifest or advisory exception changed. No source correction was required. Automatic closure of GitHub alerts 80, 87, 88, 89, 91 and 93 remains a required post-landing observation. Final release admission and rehearsal remain held for the separate shared-recipe repair.
