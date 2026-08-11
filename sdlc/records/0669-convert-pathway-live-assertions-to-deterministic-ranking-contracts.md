---
base: e3654c09
head: 972b6063
---

All seven pathway checks now run routinely through the shared provider
server. Fresh production-path captures cover KEGG search/detail, Reactome
search/detail/participants/events, and the current WikiPathways unavailable
response. Large responses were reduced to the fields consumed by BioMCP, and
unknown routes fail closed.

The page retains alias normalization, exact-title ranking, the KEGG row and
default card, typed Reactome optional-section outcomes, and source-aware
rejection. It passed all seven focused blocks; 49 fixture, receipt, and
registry tests passed. No source lines were added against the 120-line
ceiling.
