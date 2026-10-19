---
flow: build
priority: 3
---

# 2111: Move the remaining fixture consumers onto the replay server

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 503afdf, accept

Reviews: revision 1b4efc1, accept

## Outcome

The named remaining HTTP fixture consumers use the existing replay server, and an obsolete setup/cleanup script is removed only after its final retained consumer moves.

Depends on: 2159, 2229

- Starts from: the existing replay infrastructure and named article-fulltext, CTGov intervention-alias, provider-contract and ComplexPortal conversion groups.
- Keeps: CLI/MCP behavior, refusals, source receipts, live canaries and their needed helpers, local study datasets and separately approved frozen-capture runners.
- Changes: `tests/test_ctgov_trial_search_detail_reuse.py` `CHANGELOG.md` and this ticket for this partial batch. The parent remains open for the other named consumers; no server, launcher or script is removed in this batch.
- Proof: through the build job, affected existing command/MCP cases pass against recorded replay without missing requests, preserving counts, failures and refusals. No retained consumer refers to each deleted script; script-specific tests disappear with obsolete scripts. Run the routine merged-candidate gate without live canaries.
- Defers: local study-dataset provisioning and broad shared fixture identifiers to the next release.

PM decision: 2026-10-17. Study setup provisions files rather than an HTTP server and is outside this migration. Do not require deletion of every fixture outside live canaries. Reuse existing replay infrastructure and depend only on a particular recording batch when a named consumer actually needs it.

PM decision: 2026-10-17. Moved to 0.9.4. Ian ruled that 0.9.3 ships once every bug is fixed; this ticket improves tests, tooling or dependencies and fixes no user-facing bug. Rebase onto main before resuming; earlier reviews cover the old base only.

## Current partial development brief

Restated outcome: retire only the duplicated references-only branch now owned by four recorded replay goldens, retaining its distinct all-section behavior and every remaining fixture consumer.

Baseline job `2111/1792412021715-15952` at `5dc83f71deb668810a8fe99307a8df159f03be00` passed nine tests: the three existing CTGov search/detail-reuse cases, both old references/all cases, and four trial-reference goldens. It used the qualified CLI from main `1faffd0eb3e85561e941baee8acd6dad07d77f92`, banner `0.9.4-dev+g1faffd0e`, SHA256 `6b0b8861838f3c8574a69c03f9be1393d3fb8c35b2062c93989018e4f36097bc`. No build; Python setup was `make sync-python-dev` then `uv run --no-sync`. The exact invocation is retained by the job. Owned temporary binary was removed.

Exact edits: remove the references parameter, its conditional projection assertion branch and unused REFERENCE_FIELDS; keep and rename the all-section case. The four existing golden cases for populated NCT02576665 and empty NCT06131398 own JSON/Markdown output, ordered PMID/citation/source types, links and explicit empty output. Their manifest projections and failed-request assertions preserve the exact references request. Existing 2079 mutation proof failed both populated formats when references were removed. This is coverage-backed test retirement, not a product defect; no new failing test or repeated mutation is warranted. Run the eight surviving focused cases and mechanical checks, without a Rust rebuild.

No helper deletion is valid: three CTGov shell cases and the custom server's other route/error/all-section cases remain, as do the named article, provider and ComplexPortal consumers. Keep this parent open after landing and use a later narrow batch for those consumers.

## Progress

- 2026-10-19 started
