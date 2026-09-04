---
flow: build
priority: 5
---

# Return an opt-in rich page of papers for an exact author

`biomcp author papers` returns compact identifiers, titles, venues, and years. A researcher knowledge-base run needed abstracts, full publication dates, citation data, open-access facts, research fields, publication types, and complete bylines. The run called Semantic Scholar outside BioMCP and rebuilt BioMCP’s pagination, retry, provenance, and normalization work. The current behavior and provider evidence appear in `sdlc/issues/feature-add-a-full-record-mode-to-author-paper-pages.md` at commit `995fa87e`.

Semantic Scholar’s author-papers endpoint returns these fields in one caller-selected response and already supplies offset continuation. A live request confirmed that support on 2026-09-04.

## Required behavior

`biomcp --json author papers <exact-provider-id> --full` returns a rich source record for each paper. The result includes the provider-supplied details needed for local search, citation ranking, identity checks, paper pages, and related-author discovery. Missing source fields remain missing and do not become inferred facts.

The existing command remains compact when `--full` is absent. Rich pages use the same author identity, ordering, page size, offset, continuation, provenance, and source-status contracts as compact pages.

Done, observably:

- A rich paper page includes available abstracts, publication dates, citation and reference counts, open-access facts, research fields, publication types, stable identifiers, and every author supplied by Semantic Scholar.
- The response distinguishes an absent source field from an empty value.
- A rich page returns the same papers in the same order as the matching compact page.
- The continuation from a rich page requests the next rich page.
- The compact default keeps its current response size and fields.
- Help and command references explain the compact default and the opt-in rich response.

Boundary: this ticket enriches one page from an exact author corpus. It does not export a whole corpus, create a local database, fetch article full text, infer missing metadata, resolve author identity across providers, or change ordinary `search article --full` behavior.
