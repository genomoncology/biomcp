---
flow: build
priority: 7
---
# Fallback to discover when search disease returns zero results

Ticket 089 expanded disease free-text search to include synonym fields (\`disease_ontology.synonyms\`, \`mondo.synonym\`). This improved recall for diseases that exist in MONDO/DOID under alternate names. However, many clinically important diseases are absent from the MONDO/DOID-backed MyDisease.info index entirely.

Completed under March on 2026-03-31, as March ticket 091. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/091-fallback-to-discover-when-search-disease-returns-zero-results
