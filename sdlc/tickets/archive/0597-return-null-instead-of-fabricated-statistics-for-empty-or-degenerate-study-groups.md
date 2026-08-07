---
flow: build
priority: 5
---
# Return null instead of fabricated statistics for empty or degenerate study groups

The 2026-07-18 fuzz sweep (`experiments/161-biomcp-adversarial-input-fuzz/FINDINGS.md`) found two `study` analytics commands emit confident-looking statistics on empty or structurally-degenerate groups, where `study survival` already does the right thing (returns `null`):

Completed under March on 2026-07-21, as March ticket 597. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/597-return-null-instead-of-fabricated-statistics-for-empty-or-degenerate-study-groups
