---
flow: build
priority: 7
---

# Continue citation and reference pages without losing coverage

`biomcp article citations` and `biomcp article references` return only the first requested page. They expose no offset, continuation, or completion state. A researcher knowledge-base run stopped after a fixed number of edges because it could not request later pages or tell whether the provider had more results. A live one-edge request returned no continuation even though the anchor had more citations. The current behavior and provider evidence appear in `sdlc/issues/feature-make-citation-pages-complete-and-continuable.md`.

## Required behavior

Both article graph commands accept an offset and return the provider-reported offset and next offset. Each response tells the caller whether the provider exposed another page. A response with another page includes a runnable command that preserves the article identifier, graph direction, page size, and next offset.

Semantic Scholar does not report a total on these endpoints. BioMCP therefore reports no exact total. BioMCP describes coverage from the provider’s continuation signal and marks exhaustion only when the provider omits the next offset.

Done, observably:

- A caller can walk citation pages and reference pages until the provider reports no next page.
- Consecutive offsets return consecutive provider pages without silently repeating the first page.
- JSON exposes offset, page size, next offset, and provider-relative completion without inventing a total.
- Markdown prints a copyable continuation command when another page exists.
- The continuation command parses and requests the next page.
- Existing calls without an offset keep their current first-page behavior and default page size.

Boundary: this ticket adds offset continuation and honest page coverage to the existing Semantic Scholar citation and reference traversals. It does not add date-based incremental refresh, merge graph providers, calculate a global total, fetch every page automatically, or change citation-context retrieval.
