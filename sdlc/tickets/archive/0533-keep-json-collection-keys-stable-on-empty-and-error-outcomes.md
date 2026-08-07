---
flow: build
priority: 10
---
# Keep JSON collection keys stable on empty and error outcomes

BioMCP's object-shaped JSON commands do not preserve their collection key across outcomes. On current merged main, `article recommendations 23450558 --limit 5 --json` exits 0 with only `positive_seeds` and no `recommendations`; `search article -q test --limit 999 --json` exits 2 with `_meta` and `error` but no `results`. Natural agent pipelines such as `jq '.results[]'` or `jq '.recommendations[]'` therefore crash and hide BioMCP's actionable error. This caused 102 lost calls across 24 measured agent runs.

Completed under March on 2026-07-15, as March ticket 533. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/533-keep-json-collection-keys-stable-on-empty-and-error-outcomes
