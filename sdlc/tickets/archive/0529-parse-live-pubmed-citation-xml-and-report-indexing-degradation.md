---
flow: build
priority: 10
---
# Parse live PubMed citation XML and report indexing degradation

Ticket 514's `get article <pmid> indexing` surface is unavailable for every tested live PMID, including the documented PMID 22663011, even though PubMed returns HTTP 200 `text/xml` with authors, affiliations, and MeSH. The defect is deterministic: live PubMed XML begins with a `DOCTYPE`, while `parse_citation_xml` uses `roxmltree::Document::parse`, whose default `allow_dtd: false` rejects any DTD. The synthetic fixture omitted the `DOCTYPE`, and `detail.rs` discards the resulting error, so routine gates stayed green and operators saw only `status: unavailable`.

Completed under March on 2026-07-14, as March ticket 529. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/529-parse-live-pubmed-citation-xml-and-report-indexing-degradation
