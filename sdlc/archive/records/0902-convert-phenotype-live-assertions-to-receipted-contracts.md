---
base: 173bd553
head: 4f414ae1
---

Phenotype phrase, direct-HPO, rendered follow-up, and JSON follow-up checks now
run routinely through the shared supervised HTTP fixture. Fresh
production-path captures cover the HPO seizure lookup and the exact Monarch
term sets for limits one and three. The committed responses retain only the
identifiers, names, and scores consumed by BioMCP.

The fixture rejects unknown routes, the request log asserts the exact paths
and limits, and no phenotype live duplicate remains in the verify registry.
All five focused spec blocks and 52 receipt, lifecycle, and registry tests
passed. No source lines were added against the 120-line ceiling.
