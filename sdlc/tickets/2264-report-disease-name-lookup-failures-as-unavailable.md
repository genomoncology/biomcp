---
priority: 1
---

# 2264 — Report disease name lookup failures as unavailable

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 37ff9be, accept

Reviews: revision 47117ca, accept

Reviews: revision 11a005c, accept

## Outcome

Changes: `CHANGELOG.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`, `src/entities/disease/fallback.rs`, `src/entities/disease/fallback/tests.rs`, `src/entities/drug/get.rs`, `tests/test_disease_name_outcomes.py`.

A network or provider failure during `get disease --name` returns an unavailable or upstream-error outcome naming the source, never not_found, so a failure is never read as absence.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R1: `HTTPS_PROXY=http://127.0.0.1:9 biomcp --json get disease --name "Turcot syndrome" phenotypes` exits 1 with a 203-byte not_found and empty stderr, byte-identical to a failed live run; a normal run resolves MONDO:0010159.

## Keeps

A real no-match stays not_found. Check the other `--name` resolvers for the same fault and fix them in the same way.

## Proof

Replay tests: a failing provider gives the unavailable outcome; a true no-match gives not_found.

## Implementation evidence

Fresh developer stopped clean at 8cbb348f555186b3ec984e76f6bed336e9ee5091. Baseline 413f0a3f7 reproduced OLS4 failure becoming not_found, and separately MyChem GET-empty followed by SEARCH503 becoming not_found. Genuine empty disease and drug controls remained empty; direct MyDisease503 already preserved its source. Final job 2264/1792437200528-93936 passed all five CLI controls, seventy-five focused library tests and make lint including formatting, warnings-denied Clippy, Ruff, shell/workflow checks and quality boundaries. An earlier --bin selection found no tests and failed; final verification corrected it to --lib. The source patch preserves disease discovery, row resolution and canonical-fetch errors, plus drug initial empty-hit fallback search and requery errors. Local authored responses establish behavior; no live-provider result is claimed.

Carried records and milestone declarations are approved PM ancestry.

Keep: experiments/2080-biomcp-disease-name-outcomes — retain primary or authored inputs, baseline, red and green results, commands, tested patches and source hashes.
