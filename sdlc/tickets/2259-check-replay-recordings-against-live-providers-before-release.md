---
priority: 2
---

# 2259 — Check replay recordings against live providers before release

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 0e1bb2a, reject

Reviews: revision 9822256, accept

Reviews: revision ea1000c, accept

Reviews: revision 814d1ba, accept

## Outcome

Changes: `sdlc/tickets/2267-enforce-or-document-the-article-disease-filter.md`, `sdlc/tickets/2274-resolve-common-disease-names-past-the-first-page.md`, `sdlc/decisions/2026-10-19-clinical-rulings-for-the-0-9-4-review-dispositions.md`, `sdlc/tickets/2261-accept-other-transcript-protein-changes-only-for-the-same-allele.md`, `sdlc/tickets/2262-set-stable-0-9-4-metadata-and-dispositions.md`, `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`, `sdlc/tickets/2256-bound-only-the-mcp-handshake-wait-by-the-startup-timer.md`, `sdlc/tickets/2257-route-every-test-cache-root-through-the-shared-guard.md`, `sdlc/tickets/2258-admit-the-0-9-4-release-candidate.md`, `sdlc/tickets/2260-share-one-replay-server-per-test-run.md`, `tools/check-replay-live`, `scripts/replay_drift.py`, `tests/test_replay_live_drift.py`, `docs/reference/release-process.md`, `sdlc/release-checklist.md`, `tools/replay-server`, `testdata/replay/manifest.json`, `CHANGELOG.md`, `sdlc/tickets/2259-check-replay-recordings-against-live-providers-before-release.md`.

Provide the accepted opt-in provider-recording drift checker with complete inventory, generic response-structure comparison, safe requests, pacing, explicit source restrictions and release instructions. Ticket 2258 owns its mandatory observed collection on the final candidate and every resulting refresh or fix before go. Implementation acceptance does not claim live coverage passed.

## Evidence

Routine tests and specs run offline against recorded replies. `make release-live-smoke` runs a handful of live commands and the live verify spec lane, but nothing compares the recordings themselves with today's providers, nothing says when recordings are refreshed, and the release checklist names no live step.

## Keeps

Opt-in only, never in routine gates. Pace every provider and respect each provider's terms and the disabled-source groups; no licensed source is called without its key. Compare structure without response values: JSON key paths with their observed value types, or XML element paths. Ignore timestamps, other value changes and ordering. Added paths are informational; removed or retyped paths block until the recording is refreshed or its test is fixed. Record no credentials.

## Proof

The check runs once on the 0.9.4 candidate and its report is retained in a numbered experiment folder. A recording deliberately edited to a stale shape is reported as drifted. The release checklist gains the step: run the drift check, then re-record or fix every drifted recording before the go request. Document when and how to re-record in the testing docs.

## Progress

- 2026-10-19 started

## Coordinator implementation grant

Approved implementation: opt-in serial live checker with complete inventory and checked/blocked/disabled/drift accounting; source-derived JSON consumed-key/type projection with explicit wildcard handling, narrow KEGG text adapter, and blocked unsupported or missing-metadata cases. Hash-match any reconstructed POST body before send. Never infer missing historical content-type as observed fact. Replay-server changes only normalize response-header lookup and retain reconstructable nonsecret request metadata; lifecycle is owned separately by 2260. Actual final-candidate live collection follows admission and requires a complete report; development uses controlled local replies.

## Implementation evidence

Developer stopped clean at 2361055f0023aeb2eb24e50d287acae86a36e826. Controlled job 2259/1792432476405-60645 passed eighteen cases, Ruff formatting and whitespace; no live calls or full gate. The four recovered nonsecret POST bodies match the existing recorded request hashes. Inventory accounts for all forty-four entries: thirteen ready and thirty-one blocked; twenty-eight original entries lack observed historical content-type, with restricted-source and undeclared-upstream cases separately identified. This is implementation proof, not final collection or complete live coverage.

Keep: experiments/805-biomcp-2259-live-recording-drift — retain red and green controlled results, inventory, tested patch and hashes.

The carried sibling ticket records are unchanged approved PM briefs; their declaration permits the shared ancestry to land without claiming their implementation. Final live collection remains required on the frozen candidate and the parent remains open until it is complete.

## PM design ruling

The PM rejected source-model projection after fresh review demonstrated false passes for null/mixed type unions and dynamic raw JSON consumers. Replace that approach with one generic response-structure comparison; no source parser scanning or provider-specific consumed-field adapters. Report added, removed and retyped paths per recording. The original eighteen controlled cases do not prove this replacement. Preserve request safety, complete inventory, source/credential/terms restrictions, pacing and observed-only metadata refresh; no live collection during implementation. The final-candidate run remains outstanding.

## Replacement implementation proof

Fresh developer stopped clean at 6217e7f8c0cf67c9231e9b4b2a9f266de0322798. Generic JSON key-path/type unions and XML element paths replace source-model scanning and provider-specific projections. Job 2259/1792433264101-34336 retained nine meaningful red controls on 8a277bd96, then twenty-five green checks, Ruff, formatting and whitespace. The committed replacement matches retained green patch SHA256 518e9cfe0ac30fb71f5d6d83e1edafabac749dbb7b5bd91c1500a6a26d3547ea under archive 805/structure-replacement. No live calls ran.

The complete inventory still has forty-four recordings: thirteen ready, thirty-one blocked and zero checked live. Forty-one payloads are JSON; three restricted KEGG payloads are plaintext and unsupported by the generic JSON/XML comparison. Twenty-eight recordings lack observed historical content types. Parent remains open for final observed collection, unsupported plaintext disposition and any demonstrated stale-route or provider changes before go.

## Final collection dispositions

PM ruling: do not call KEGG live from these runs because its commercial restrictions apply. List each of the three KEGG plaintext recordings by name in the drift and release records as not checked live: commercial-restricted source, and never count them as checked. No plaintext adapter is required for this approved disposition; every other unresolved coverage gap blocks release.

Read-only endpoint triage found no product dbSNP endpoint bug: current DbSnpClient calls api.ncbi.nlm.nih.gov/variation/v0/refsnp/113488022. Only the replay declaration incorrectly names E-utilities. The replay bytes and current source fixture both hash to 98dd1bc691723fce26da3202455f9b1857510490fbc09912d444638833c518aa; capture-receipts.json binds those bytes to the current endpoint at 2026-10-15T15:57:39Z. Refresh the declared upstream from an observed current-endpoint response during final collection, as PM ordered; do not invent provenance or file an unsupported product defect.

## Final PM ownership ruling

The PM approved closing 2259 with the accepted checker implementation through its own landing because the checker changes a gate-definition input. This supersedes earlier parent-open notes. Ticket 2258 owns the observed final collection and report, the twenty-eight header refreshes, the dbSNP observed upstream refresh, the three named KEGG exemptions and every follow-up fix before go. The release bar is unchanged: partial coverage is never a release pass.
