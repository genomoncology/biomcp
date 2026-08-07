---
base: 417841de2d3815afdd9294e8fb0ec16939e08bce
head: 7c1119e889777433f2a3e7364d8b7a3dc8aa585e
---
Ticket 089 expanded disease free-text search to include synonym fields (\`disease_ontology.synonyms\`, \`mondo.synonym\`). This improved recall for diseases that exist in MONDO/DOID under alternate names. However, many clinically important diseases are absent from the MONDO/DOID-backed MyDisease.info index entirely.

Imported from March ticket 091. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/091-fallback-to-discover-when-search-disease-returns-zero-results
