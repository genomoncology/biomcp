---
priority: 3
---

# 2184 — Route an rsID in plain article search through the PubTator entity

Status: OPEN.

Milestone: 0.9.4

## Outcome

`search article <rsID> --source pubtator` uses the PubTator variant entity, as `variant articles` already does.

- Evidence: research's spot check: `search article rs113488022 --source pubtator` returns the 47 text matches, while the variant-articles annotation route finds about 1,250 candidates.
- Changes: claims the PubTator article search path and its tests, `CHANGELOG.md`.
- Proof: recorded replies show the plain search resolving the rsID to its entity; current main uses text search.
