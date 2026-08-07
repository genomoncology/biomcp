---
flow: build
priority: 5
---
# Expose PubMed affiliations and MeSH as article indexing metadata

A real researcher-profile task needed author affiliations and MeSH headings, but BioMCP exposes neither. The fallback was hand-written NCBI E-utilities XML parsing. This is a valid data gap, but it is not part of the silent author-truncation bug: BioMCP currently uses PubMed `esearch`/`esummary` and does not fetch PubMed citation XML at all. Treating the fields as already available would hide a new network and parsing path.

Completed under March on 2026-07-14, as March ticket 514. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/514-expose-pubmed-affiliations-and-mesh-as-article-indexing-metadata
