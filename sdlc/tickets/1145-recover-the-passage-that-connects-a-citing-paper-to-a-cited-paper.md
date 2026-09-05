---
flow: build
priority: 6
deps: [1144]
---

# Recover the passage that connects a citing paper to a cited paper

`biomcp article citations` proves that one paper cites another, but many Semantic Scholar edges contain no useful context. A researcher knowledge-base run found context on only 23 of 61 incoming edges. The run had to fetch open full text, match the cited work to a reference, and locate the surrounding paragraph before it could explain how later research used the earlier work. The current behavior, measured coverage, and verified full-text examples appear in `sdlc/issues/feature-recover-citation-evidence-when-upstream-context-is-missing.md` at commit `995fa87e`.

Europe PMC’s open-access JATS XML preserves links between inline citation markers and reference-list entries. Verification found three linked paragraphs in `PMC12923956` for cited DOI `10.1038/nature10725`. Verification also found one linked paragraph in `PMC13200738` for cited DOI `10.1016/j.artmed.2020.101822`. These examples prove the fallback path. They do not prove universal coverage.

## Required behavior

`biomcp article citation-evidence <citing-id> <cited-id>` returns source text that supports the citation relationship when BioMCP can recover it. BioMCP returns each nonblank Semantic Scholar context as provider context. When Semantic Scholar returns no nonblank context and structured open full text is available, BioMCP resolves the cited reference and returns a bounded passage around a matching in-text citation marker. A caller can still request the full-text recovery path when provider context exists but lacks enough surrounding text for review.

The response identifies the source paper, cited paper, evidence source, and a locator that lets a caller inspect the passage. It distinguishes provider context, recovered full-text context, unavailable full text, an unresolved reference, and an unlinked citation marker in plain language and structured output. It never invents a passage or interprets what the passage means.

Citation edges with no useful provider context offer the exact evidence command as a next step.

Done, observably:

- Captured JATS fixtures for the two verified open-access examples recover passages linked to the requested cited references.
- Existing useful Semantic Scholar context remains available without an unnecessary full-text dependency.
- A recovered passage carries enough provenance and location information for a caller to inspect its source.
- Missing or unusable evidence returns a specific unavailable outcome instead of an empty string or inferred explanation.
- Grouped markers and ambiguous references do not silently select one cited work.
- Human-readable and JSON responses expose the same evidence status and next action.

Boundary: this ticket retrieves evidence for one directed citation pair. It does not summarize the passage, judge the relationship, claim that a citation supports a scientific conclusion, bypass paywalls, parse PDF-only articles, or promise evidence for every citation. The existing citation-sidecar issue covers mechanical citation formatting and remains separate.
