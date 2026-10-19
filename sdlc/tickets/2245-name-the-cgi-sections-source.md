---
priority: 2
---

# 2245 — Name the CGI section's source

Status: OPEN.

Milestone: 0.9.4

Reviews: revision d8a56bc, accept

Reviews: revision 38aa6c0, accept

Reviews: revision 3606278, accept

## Outcome

The CGI section of variant output names its source as MyVariant's variant-level CGI subset and its load date, so an empty result reads as "no record in that subset", not "CGI has nothing on this gene".

- Changes: `CHANGELOG.md` `sdlc/tickets/2245-name-the-cgi-sections-source.md` `sdlc/tickets/2246-query-every-coding-alias-or-report-dropped-forms.md` `sdlc/tickets/2247-search-previous-gene-symbols-in-variant-articles.md` `sdlc/tickets/2248-keep-preprint-fulltext-from-panicking.md` `sdlc/tickets/2249-show-the-match-in-supplement-search-context.md` `sdlc/tickets/2250-mark-supplement-search-incomplete-when-files-are-unreachable.md` `src/cli/variant/dispatch.rs` `src/render/markdown/variant.rs` `src/render/provenance.rs` `templates/variant.md.j2` `tests/test_variant_cgi_source.py`.

## Evidence

Research on 0.9.3: KRAS G12C returns "No CGI drug associations". BioMCP reads CGI through MyVariant, whose CGI load is dated 2023-02-27 and holds 323 variant-level records; KRAS G12C has none, while BRAF V600E returns 32. A reader can mistake the empty section for CGI having no data on the gene.

## Keeps

Only the section label and empty-state wording change, in Markdown and as a source note in JSON. No data or routing change.

## Proof

Replay tests for a variant with CGI rows and one without show the source label and the subset-specific empty wording. Changelog bullet.

## Coordinator implementation brief



Restated outcome: identify the CGI data as MyVariant variant-level CGI subset, loaded 2023-02-27; an empty response says no record in that subset, not no CGI evidence for the gene. Change labels and source notes only in Markdown/JSON, preserving row content and routing.

Unchanged-main reproduction on08d721f8525a35267bcd0a307248c69b1de210fc: job2245/1792423332921-20221 failed the intended source-scope/load-date contract. Four CLI calls succeeded without replay misses. Recorded BRAF response contains32rows and the existing transform emits10; the clearly authored CGI-removed copy emits0. Empty Markdown says No CGI drug associations reported, and neither JSON result names the subset/date. Exact invocation, outputs and executable identity are retained in experiments/788-biomcp-2245-cgi-label. The empty input is synthetic, not a recorded KRAS response. No new live acquisition is needed.

Write four failing outside-in replay cases for populated/empty JSON/Markdown using existing replay helper; keep authored projection provenance explicit. Use one shared note rather than duplicated source facts. No new entity-model fields are needed. 1260 shares provenance/render/template files for matched-transcript additions; changes are semantically separate and the coordinator will resolve them at landing. Run focused red/green and mechanical checks through configured build job, retain patch/manifests/raw in788, publish once review-ready without WIP pushes.

Keep: experiments/788-biomcp-2245-cgi-label — Retain cited raw proof until release evidence is explicitly released.

## Focused repair proof

Source76131e979d090dd3cd2bdff4740f67efa73137bc changes the six declared files. Red1792423524173-65627 failed the missing source-note contract; green1792423552211-69922 passed all four real CLI replay cases and make lint including warnings-denied Clippy, formatting and quality checks, with zero replay misses. Empty CGI input is a clearly authored projection of the recorded BRAF reply, not a captured KRAS response. No live acquisition, data or routing change. All six final blobs and full source diff match the tested private patchSHA256 c7c28a1acffac0554e307424e0747aa9c862df82408ab58ef5ba46a0e467ce4e. Patch, comparison, hashes and raw results remain in experiments/788-biomcp-2245-cgi-label.

PM authorized carrying bc2b82d3b five unchanged literature bug records with this landing. Their source work runs in separate lanes; this ticket makes no claim that those outcomes are complete.

## Binary marker integration repair

Merged-candidate gate2237/1792425706706-70958 failed the existing binary-dependent-module contract on tests/test_variant_cgi_source.py;1373Pythoncasespassed and10skipped. The source-qualified CLI tests passed, but the new module lacks the required needs_binary declaration for binary-free test selection. Repair only that existing claimed test path, preserve assertions, and run the owning marker contract plus Ruff through the configured build job. No new test or product rebuild is needed. Retain integration proof in788.
