---
priority: 2
---

# 2246 — Query every coding alias or report the dropped forms

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 02c08e8, accept

## Outcome

`biomcp variant articles` sends every protein and coding form it resolved for the variant to the literature sources, or names in its output and debug plan each form it did not send and why. Other-transcript forms follow the transcript-alias decision.

- Changes: `CHANGELOG.md` `sdlc/tickets/2245-name-the-cgi-sections-source.md` `sdlc/tickets/2246-query-every-coding-alias-or-report-dropped-forms.md` `sdlc/tickets/2247-search-previous-gene-symbols-in-variant-articles.md` `sdlc/tickets/2248-keep-preprint-fulltext-from-panicking.md` `sdlc/tickets/2249-show-the-match-in-supplement-search-context.md` `sdlc/tickets/2250-mark-supplement-search-incomplete-when-files-are-unreachable.md` `src/cli/variant/articles.rs` `src/entities/article/backends/provider_unit.rs` `src/entities/article/variant_search.rs` `tests/fixtures/article/coding-aliases/authored.json` `tests/test_variant_article_alias_coverage.py`.

## Evidence

Reproduced on 0.9.3: `biomcp --json variant articles 'ABCC8 R1437Q' --limit 5 --debug-plan --verify-identity` resolves twelve protein and coding aliases but queries ten. `c.4310G>A` and `c.4373G>A` are never sent and the omission is not reported. Europe PMC `ABCC8 AND "c.4310G>A"` returns PMID 41948559, an open-access paper that names the variant only that way. Several query slots go to other-transcript protein forms (R1353Q, R1425Q, R1458Q).

## Keeps

Follow `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`: query every supplied coding form; count an other-transcript alias as evidence only when the paper pins it to the same genomic change, otherwise label it an unverified alias match. Keep the request deadline. Clinical questions go to a clinical reviewer.

## Proof

A replay test for ABCC8 R1437Q shows both coding forms sent and PMID 41948559 returned, or the dropped forms named in the output. The debug plan lists every resolved form with sent or dropped.

## Coordinator study brief

Restate the approved outcome, reproduce on current main before source edits, and report the exact command/output, proposed precise source/test paths and outside-in failing case. Reuse qualified unchanged current-main CLI when possible; all execution goes through the configured build job. Preserve exact raw source responses and invocation in the next numbered three-digit experiment on the executing host, with OWNER biomcp/2246. No live rerun if an existing deterministic replay already reproduces the same failure; any authored controls must be explicitly identified, not called captured source. Read source redistribution standards before publishing new captures. No product or test claim is granted yet. No source edits, WIP pushes, child agents, mail or status changes during this study. Stop after bounded reproduction and proposal; do not widen the ticket.

## Granted implementation brief



Restated outcome: every resolved protein/coding form is queried or explicitly reported as omitted with its reason in ordinary and debug output. Choose the approved disclosure outcome to preserve existing request caps and deadline. No new identity fallback or query expansion.

Current-main8cf9c3be7 job1792424100327-68113 live+replay ABCC8 R1437Q --limit5 --debug-plan --verify-identity resolved12forms but exact_aliases selected10, omitting c.4310G>A and c.4373G>A without per-form reasons. Both exit0/same5PMIDs, replay99hits/0misses. Exact mechanism is exact_aliases, not fallback_aliases for this resolved case; truncation was ignored as _alias_budget_stopped. Evidence791-biomcp-2246-coding-alias-study retains allraw/private. Do not claim recovery of another paper. Preserve source-provided gene/DNA identity ruling. Distinguish planned or selected forms from actually attempted queries and deadline omissions; never label unexecuted plans sent. One shared coverage value should drive output/debug, not duplicated inferred counts. Clearly identify authored controls or minimized source projections; inspect capture rights before public fixtures.2247 follows this candidate for separate gene-symbol restriction disclosure.

Write failing outside-in tests first, apply the minimal repair, run affected tests and mechanical checks through the configured build job. Preserve private patch and per-file hashes; verify published source matches the tested manifest. No WIP pushes, merges, ticket/status edits, mail or child agents. Push once review-ready and stop after reporting.

Keep: experiments/791-biomcp-2246-coding-alias-study — Retain cited raw proof until release evidence is explicitly released.

Claim expansion: Track actual request polling after provider setup and admission, so disabled/deadline-blocked plans are not called attempted. No changes to scheduling, caps, identity or provider selection.

## Focused alias coverage proof

Source1b777757706d149180ea1ce6108a08dbb1ffb448 changes six product/test/fixture files. Shared per-form coverage appears in ordinaryJSON, per-batch items and debug plans, and in Markdown including hard errors. The provider-unit hook marks an attempt immediately before an admitted query future is polled, without changing alias selection, query limits, deadlines or identity rules. Strict PubTator entity lookup resolves the original request and does not count as sending alternate text spellings; literal lexical PubTator queries do count. The final qualification includes that attribution correction and a small shared terminal-error helper extraction required by the existing function-length ratchet.

Red1792424562961-75975 reproduced missing per-form omission disclosure. Final1792425146818-45854 passed six CLI cases,13focused Rust cases and make lint. Earlier focused checks also exercised existing exact-name/transcript safeguards and CLI/MCP behavior; finalclaims rely on the retained exactsource manifests. Both published commit and all six changed blobs match the testedSHA256manifest. Public fixture is explicitly authored MyVariant-shaped data with empty literature controls; private patch, source manifest and verification script remain in experiments/791-biomcp-2246-coding-alias-study. No newqueries, paper-recovery claim, gene-symbol expansion or identity change.
