# Author paper pages cannot supply a searchable researcher corpus

Severity: should-fix

A researcher knowledge base needs to ask: what abstracts, citation counts, publication dates, open-access facts, fields of study, and complete bylines belong to the papers on an exact author record? `biomcp author papers` cannot answer that question. It returns compact rows with identifiers, title, venue, and year.

A downstream researcher-corpus exercise needed those richer fields for local search, citation ranking, paper pages, identity checks, and related-person discovery. It used a custom Semantic Scholar client with a larger field list and batch lookups. The custom client added retry, rate-limit, pagination, normalization, and provenance code that BioMCP already owns for its other Semantic Scholar surfaces.

The cheapest useful addition would mirror article search: `biomcp author papers semanticscholar:<id> --full`. The default response would remain compact. Full JSON rows would add the source-supplied abstract, publication date, citation and reference counts, influential citation count, open-access fields, fields of study, publication types, and author list. Existing offset pagination would remain unchanged.

A bulk corpus export and a local library remain separate ideas. A full page mode would let ordinary JSON composition handle those jobs.

The missing fields were verified with `biomcp 0.9.0-dev.6`, `biomcp list author`, `biomcp author papers --help`, and a live two-paper JSON page on 2026-09-04. `src/sources/semantic_scholar.rs` requests `AUTHOR_PAPER_FIELDS`, and `src/entities/author/papers.rs` converts each result into the compact `ArticleRelatedPaper` shape.

## Provider verification

Semantic Scholar supports this change directly. Its `GET /graph/v1/author/{author_id}/papers` endpoint accepts a caller-selected `fields` list, an offset, and a limit up to 1,000. A live request for one exercise author returned `abstract`, `publicationDate`, `citationCount`, `referenceCount`, `influentialCitationCount`, `isOpenAccess`, `openAccessPdf`, `fieldsOfStudy`, `publicationTypes`, `authors`, and `externalIds` in one row. The response also returned `offset` and `next`.

BioMCP only needs to request and model fields that the provider already returns. Larger responses and the Semantic Scholar rate budget are the costs. The compact default should remain unchanged for that reason.

Provider documentation: <https://api.semanticscholar.org/api-docs/graph>
