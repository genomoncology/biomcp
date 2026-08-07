---
base: ce1bae9a2baf0f9fb7e2684a17346d1a948eb261
head: 199b47971ff5cff3db571cc3e0fcc119f28e3c0e
---
Ticket 514's `get article <pmid> indexing` surface is unavailable for every tested live PMID, including the documented PMID 22663011, even though PubMed returns HTTP 200 `text/xml` with authors, affiliations, and MeSH. The defect is deterministic: live PubMed XML begins with a `DOCTYPE`, while `parse_citation_xml` uses `roxmltree::Document::parse`, whose default `allow_dtd: false` rejects any DTD. The synthetic fixture omitted the `DOCTYPE`, and `detail.rs` discards the resulting error, so routine gates stayed green and operators saw only `status: unavailable`.

Imported from March ticket 529. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/529-parse-live-pubmed-citation-xml-and-report-indexing-degradation
