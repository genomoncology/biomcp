# A citation graph cannot say whether it captured every edge

Severity: should-fix

A durable knowledge base needs to ask: did this capture include every paper that cites the anchor, and how can the next refresh continue after the stored page? `article citations` and `article references` cannot answer either part. Both commands accept a limit up to 100. Neither accepts an offset. Their JSON responses contain the anchor and returned edges without a total, next offset, or completeness status.

A downstream researcher-corpus exercise deliberately stopped at 20 edges per direction because it could not walk later pages. The local policy discloses that bound, but BioMCP does not disclose whether a result exhausted the provider or only filled the requested page. A live `article citations 22237106 --limit 1` call returned one edge with no continuation even though the anchor has more citations.

The cheapest useful addition would add `--offset`, retain the provider's offset and next values, and emit a runnable next command. JSON should include the same pagination object used by `author papers`. Markdown should print the continuation. If the provider does not report enough information to prove completion, BioMCP should label the page as source-limited instead of complete.

A later `--since` mode could support incremental refreshes. Offset pagination and honest completion metadata should land first.

The negative was verified with `biomcp 0.9.0-dev.6` and the current help and JSON output on 2026-09-04. `src/cli/article/mod.rs` exposes only `--limit`. `src/sources/semantic_scholar.rs::paper_subresource_plan` sends only `fields` and `limit`. `SemanticScholarGraphResponse` keeps only `data`. This issue is related to ticket 1103. Ticket 1103 adds recovery commands to capped sections. Citation traversal first needs a continuation capability.

## Provider verification

Semantic Scholar supports this change directly. Its citation and reference endpoints accept `offset` and a limit up to 1,000. Their documented response shape contains `offset`, optional `next`, and `data`. It does not contain a total. A live three-page check against PMID 22237106 returned offsets 0, 1, and 2 with next values 1, 2, and 3 and a different paper on each page.

BioMCP can therefore provide continuation and can mark the provider result complete when `next` is absent. BioMCP should not promise an exact total because the provider does not return one on this surface.

Provider documentation: <https://api.semanticscholar.org/api-docs/graph>
