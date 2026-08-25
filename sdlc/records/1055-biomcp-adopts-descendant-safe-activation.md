---
base: b7b76fb692ed5abc105cabe2824c460ce3a25588
head: 08c4b4001bc821d54aa651e5cfcbe61350f6e970
---

# BioMCP adopts descendant-safe activation

BioMCP now carries the canonical descendant-safe lifecycle activation. A stored
landing can activate a clean current main descendant without rolling back later
work, while its hook and receipt preserve the original landed identities.

A consumer contract covers the descendant activation path and exact receipt.
