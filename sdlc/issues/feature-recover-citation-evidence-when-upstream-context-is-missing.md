# A citation edge often cannot show how the citing paper used the earlier work

Severity: should-fix

A literature-review agent needs to ask: how did this later paper use the earlier paper? `article citations` returns Semantic Scholar citation contexts when the provider has them. The edge proves citation when the context is empty. It does not answer the research question.

A downstream researcher-corpus exercise captured 61 incoming citation edges across four anchors. Only 23 edges carried any context. One selected impact trail had no context. Another returned only `7,13 We also performed pathway analysis...`. The agent fetched open full text, found the cited reference, and read the surrounding paragraph before it could establish direct cohort reuse. One paper with no available full text remained unusable because BioMCP also returned no citation context.

This gap comes from upstream coverage. BioMCP should treat it as a retrieval fallback instead of presenting invented meaning.

The cheapest improvement would add an explicit context status to every edge and provide the exact full-text command for the citing paper when context is empty. A more useful follow-up would add `article citation-evidence <citing-id> <cited-id>`. That command would return Semantic Scholar contexts first. When open full text exists, it would locate the cited reference marker and return a bounded surrounding passage with the source and locator. It would return an explicit reason when no passage can be recovered. The agent would still judge what the passage means.

This request differs from `2026-08-27-a-citation-sidecar-would-make-synthesis-mechanical.md`. That issue maps an existing BioMCP result to citable source URLs. This issue retrieves the missing passage that supports the relationship between two papers.

The counts and workarounds were verified against stored `biomcp 0.9.0-dev.6` JSON captures and open full-text retrievals on 2026-09-04.
