---
base: 9d5349fc1ca8cc763cffe53fe9be3ed8e5a9f77a
head: 44bab11d09628056ecb3173ea64392655181945d
---
A real researcher-profile task needed author affiliations and MeSH headings, but BioMCP exposes neither. The fallback was hand-written NCBI E-utilities XML parsing. This is a valid data gap, but it is not part of the silent author-truncation bug: BioMCP currently uses PubMed `esearch`/`esummary` and does not fetch PubMed citation XML at all. Treating the fields as already available would hide a new network and parsing path.

Imported from March ticket 514. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/514-expose-pubmed-affiliations-and-mesh-as-article-indexing-metadata
