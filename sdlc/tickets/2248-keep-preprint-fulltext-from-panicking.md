---
priority: 1
---

# 2248 — Keep preprint full text from panicking

Status: OPEN.

Milestone: 0.9.4

## Outcome

`get article` full text for a preprint DOI returns a structured answer, the text or a named reason, and never panics. A batch keeps every unrelated item when one fails.

## Evidence

Reproduced on 0.9.3: `biomcp --json --no-cache get article 10.1101/2022.07.18.22276644 fulltext` panics at a provenance invariant (`provenance.rs:115`) and writes zero JSON bytes. `batch article 14735200,25922291,18178632,18509008,12752119,20301425,10.1101/2022.07.18.22276644 --mode detail --sections fulltext` loses all seven items. The preprint server's API has the JATS body, and `--pdf` returns real text.

## Keeps

Fix the invariant's cause, not just the symptom. Keep provenance accurate for every source.

## Proof

Reproduce on current main first. A replay test returns the preprint's text with its provenance, and the batch returns all seven items with the preprint's outcome named.

## Coordinator study brief

Restate the approved outcome, reproduce on current main before source edits, and report the exact command/output, proposed precise source/test paths and outside-in failing case. Reuse qualified unchanged current-main CLI when possible; all execution goes through the configured build job. Preserve exact raw source responses and invocation in the next numbered three-digit experiment on the executing host, with OWNER biomcp/2248. No live rerun if an existing deterministic replay already reproduces the same failure; any authored controls must be explicitly identified, not called captured source. Read source redistribution standards before publishing new captures. No product or test claim is granted yet. No source edits, WIP pushes, child agents, mail or status changes during this study. Stop after bounded reproduction and proposal; do not widen the ticket.

## Granted implementation brief

- Changes: `src/entities/source_state_registry.rs` `src/entities/article/fulltext.rs` `tests/test_article_preprint_fulltext.py` `tests/fixtures/article/preprint-fulltext/**` `CHANGELOG.md` `sdlc/tickets/2248-keep-preprint-fulltext-from-panicking.md`. `src/sources/provider_url_policy.rs` `docs/reference/sources.json` `docs/reference/source-licensing.md` `src/error.rs`

Restated outcome: preprint DOI full text returns actual available JATS text with accurate provenance, or a named structured failure when unavailable; one preprint outcome must not erase unrelated ordered batch items. PM explicitly approved the JATS route within this ticket. Fix the provenance-registry mismatch rather than weakening its invariant.

Current-main8cf9c3be7 job1792424078528-68450 reproduced exact preprint10.1101/2022.07.18.22276644 single lookup and seven-item batch both exit1/stdout0 at provenance.rs:115. Captured EuropePMC PPR521666 lacks PMID/PMCID; healthy ID Converter Identifier not found in PMC was credited but absent from article/fulltext registry. Evidence789-biomcp-2248-preprint-panic retains31rawbodies/invocations/providerbytes and hashes. PM supplied an existing114486byte advertisedJATS reply and preprint details metadata; reuse this evidence rather than another live chain. Preserve article source attribution, license facts and existing URL/trust boundaries. Review redistribution before publishing captures; retain full raw privately and mark authored controls or projections truthfully. Proposed source/test claim is intentionally small; report any additional exact file before editing it.

Write failing outside-in tests first, apply the minimal repair, run affected tests and mechanical checks through the configured build job. Preserve private patch and per-file hashes; verify published source matches the tested manifest. No WIP pushes, merges, ticket/status edits, mail or child agents. Push once review-ready and stop after reporting.

Keep: experiments/789-biomcp-2248-preprint-panic — Retain cited raw proof until release evidence is explicitly released.

Claim expansion: Register the approved preprint source, preserve provider identity and generated license facts, and constrain the recorded advertised-JATS origin through the existing URL policy. Inspect actual source/licensing evidence before implementing its route; no weakened generic fetch or provenance invariant.
