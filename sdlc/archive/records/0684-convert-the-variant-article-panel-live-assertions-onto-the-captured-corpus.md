---
base: f3bc4220
head: 8b510b41
---

The seven-variant recall gate is now routine, deterministic, credential-free,
and public-network-free. It preserves the existing 9-of-12 landmark and 6-of-7
variant thresholds, the two MLH1 family papers, route attribution, and the
captured positive, empty, degraded, and not-attempted states.

Exact Europe PMC requests run through the production CLI in JSON and Markdown;
the PubMed request plans and captures run through the production decoder test.
The fixture deliberately probes an unknown route and requires a 404. The new
routine mustmatch page and focused Python/Rust checks passed.
