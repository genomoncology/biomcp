---
priority: 2
---

# 2271 — Name the real source for article asset misses

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `CHANGELOG.md`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `src/entities/article/assets.rs`, `src/error.rs`, `src/main.rs`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/source_outcomes.rs`, `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`, `sdlc/tickets/2256-bound-only-the-mcp-handshake-wait-by-the-startup-timer.md`, `sdlc/tickets/2258-admit-the-0-9-4-release-candidate.md`, `sdlc/tickets/2259-check-replay-recordings-against-live-providers-before-release.md`, `sdlc/tickets/2260-share-one-replay-server-per-test-run.md`, `sdlc/tickets/2261-accept-other-transcript-protein-changes-only-for-the-same-allele.md`, `sdlc/tickets/2262-set-stable-0-9-4-metadata-and-dispositions.md`, `sdlc/tickets/2263-surface-mcp-argument-parse-errors-to-the-model.md`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2267-enforce-or-document-the-article-disease-filter.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`.

`get article ... assets` for a paper with no open-access asset package returns a clear outcome naming the real source, never "BioMCP source" with "Review source configuration and retry", and expected misses during a successful full-text fetch stay out of stderr or move into the JSON coverage.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R12: `biomcp --json get article 26617739 assets --asset-view retrievable --asset-limit 100` returns source_unavailable from "BioMCP source" with that recovery text, while full text (PMC4637554, open_access false) succeeds from PMC HTML; 31342592 gets source_unavailable too. R13: `biomcp --json get article 26617739 fulltext --pdf --out DIR` succeeds but prints "PMC OA manifest/archive request failed", "Europe PMC supplementary request failed" and an html5ever "node with weird namespace" on stderr for every non-open-access PMC fetch.

## Keeps

Real provider failures still warn and name the provider. Same wording family as 2250's unavailable files.

## Proof

Replay tests: the 26617739 assets call names the source and the no-open-access reason; the full-text call prints no expected-miss warnings and records them in coverage.

## Implementation evidence

Fresh developer stopped clean at 58c71d115cd8a67b92a3e0fbf1e880b81da4158a. Red job 2271/1792436733897-80829 reproduced generic source/configuration wording and serializer noise. Final job 2271/1792437183970-87601 passed four outside-in controls and make lint. Healthy misses remain quiet, and a real authored Europe PMC outage still warns during successful PDF retrieval. The PM ruled that unknown availability stays unknown and names the actual source; package absence may be stated only from a healthy reply. The retained research lacks original provider HTTP replies for 26617739 and 31342592, so their historical package availability remains unknown. No provider failure classification changed. The coordinator moved the completed owned archive from its initial 809 folder into the unique retained 2085 folder without altering evidence.

Carried ticket records are approved PM ancestry.

Keep: experiments/2085-biomcp-2271-asset-outcomes — retain baseline, red and green results, commands, tested patches, source hashes and permitted private research evidence.
