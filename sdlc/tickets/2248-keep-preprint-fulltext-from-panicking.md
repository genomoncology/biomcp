---
priority: 1
---

# 2248 — Keep preprint full text from panicking

Status: OPEN.

Milestone: 0.9.4

Reviews: revision ff159bb, accept

## Outcome

`get article` full text for a preprint DOI returns a structured answer, the text or a named reason, and never panics. A batch keeps every unrelated item when one fails.

- Changes: `CHANGELOG.md` `architecture/technical/source-integration.md` `docs/reference/source-licensing.md` `docs/reference/sources.json` `sdlc/tickets/2245-name-the-cgi-sections-source.md` `sdlc/tickets/2246-query-every-coding-alias-or-report-dropped-forms.md` `sdlc/tickets/2247-search-previous-gene-symbols-in-variant-articles.md` `sdlc/tickets/2248-keep-preprint-fulltext-from-panicking.md` `sdlc/tickets/2249-show-the-match-in-supplement-search-context.md` `sdlc/tickets/2250-mark-supplement-search-incomplete-when-files-are-unreachable.md` `src/entities/article/fulltext.rs` `src/entities/source_state_registry.rs` `src/error.rs` `src/sources/provider_url_policy.rs` `tests/fixtures/article/preprint-fulltext/README.md` `tests/fixtures/article/preprint-fulltext/article.xml` `tests/test_article_preprint_fulltext.py`.

## Evidence

Reproduced on 0.9.3: `biomcp --json --no-cache get article 10.1101/2022.07.18.22276644 fulltext` panics at a provenance invariant (`provenance.rs:115`) and writes zero JSON bytes. `batch article 14735200,25922291,18178632,18509008,12752119,20301425,10.1101/2022.07.18.22276644 --mode detail --sections fulltext` loses all seven items. The preprint server's API has the JATS body, and `--pdf` returns real text.

## Keeps

Fix the invariant's cause, not just the symptom. Keep provenance accurate for every source.

## Proof

Reproduce on current main first. A replay test returns the preprint's text with its provenance, and the batch returns all seven items with the preprint's outcome named.

## Coordinator study brief

Restate the approved outcome, reproduce on current main before source edits, and report the exact command/output, proposed precise source/test paths and outside-in failing case. Reuse qualified unchanged current-main CLI when possible; all execution goes through the configured build job. Preserve exact raw source responses and invocation in the next numbered three-digit experiment on the executing host, with OWNER biomcp/2248. No live rerun if an existing deterministic replay already reproduces the same failure; any authored controls must be explicitly identified, not called captured source. Read source redistribution standards before publishing new captures. No product or test claim is granted yet. No source edits, WIP pushes, child agents, mail or status changes during this study. Stop after bounded reproduction and proposal; do not widen the ticket.

## Granted implementation brief



Restated outcome: preprint DOI full text returns actual available JATS text with accurate provenance, or a named structured failure when unavailable; one preprint outcome must not erase unrelated ordered batch items. PM explicitly approved the JATS route within this ticket. Fix the provenance-registry mismatch rather than weakening its invariant.

Current-main8cf9c3be7 job1792424078528-68450 reproduced exact preprint10.1101/2022.07.18.22276644 single lookup and seven-item batch both exit1/stdout0 at provenance.rs:115. Captured EuropePMC PPR521666 lacks PMID/PMCID; healthy ID Converter Identifier not found in PMC was credited but absent from article/fulltext registry. Evidence789-biomcp-2248-preprint-panic retains31rawbodies/invocations/providerbytes and hashes. PM supplied an existing114486byte advertisedJATS reply and preprint details metadata; reuse this evidence rather than another live chain. Preserve article source attribution, license facts and existing URL/trust boundaries. Review redistribution before publishing captures; retain full raw privately and mark authored controls or projections truthfully. Proposed source/test claim is intentionally small; report any additional exact file before editing it.

Write failing outside-in tests first, apply the minimal repair, run affected tests and mechanical checks through the configured build job. Preserve private patch and per-file hashes; verify published source matches the tested manifest. No WIP pushes, merges, ticket/status edits, mail or child agents. Push once review-ready and stop after reporting.

Keep: experiments/789-biomcp-2248-preprint-panic — Retain cited raw proof until release evidence is explicitly released.

Claim expansion: Register the approved preprint source, preserve provider identity and generated license facts, and constrain the recorded advertised-JATS origin through the existing URL policy. Inspect actual source/licensing evidence before implementing its route; no weakened generic fetch or provenance invariant.

Claim expansion: update only the article/fulltext provider cell in the existing architecture source-integration mirror to match the approved source-state registry. No architecture redesign.

## Focused preprint repair proof

Source21284308b22a075e2bdacaf1839fe754ca852bee corrects actual fulltext provider registration and adds the approved API-advertised JATS rung with restricted origin policy, existing parser/body limits/shared client, source URL and article license. The provenance invariant remains enforced. Source inventory and generated licensing state that article licenses vary; captured article-specificCCBY4.0 is not a blanket provider reuse claim. Architecture provider-cell mirror matches the source-state registry.

Historical source evidence was unavailable, so PM authorized a single bounded public details/JATS recapture.1792424571441-77240 retained the114486byteadvertisedXML, exactURLs/date/hashes privately in789. Sourcefixture is unmodified and attributed to its authors/title/DOI/article-specificembeddedCCBY4.0. Red1792424666541-5016 reproduced the provenance invariant. Final1792424960570-12098 passed six command cases, including actual single preprint text, seven ordered batch outcomes, unavailabletext404/503, rejectedforeignadvertisedhost and healthyidentity-onlynonmatch.56affected Rust cases and finalmake lint passed. An initial mechanical check required updating the architecture source-provider mirror; finalproofincludes that fix. All eleven committed product/test/fixture/docs blobs match the retained tested private manifest and commit comparison in experiments/789-biomcp-2248-preprint-panic. Only this medRxiv article was live-tested; no broad preprint live-coverage claim.
