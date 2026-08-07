---
base: 69387c5439069275168eafb2c7b2dee0ea25b8df
head: a8d6e098a21a044e124abe7ab25b53a14294283f
---
BioMCP's object-shaped JSON commands do not preserve their collection key across outcomes. On current merged main, `article recommendations 23450558 --limit 5 --json` exits 0 with only `positive_seeds` and no `recommendations`; `search article -q test --limit 999 --json` exits 2 with `_meta` and `error` but no `results`. Natural agent pipelines such as `jq '.results[]'` or `jq '.recommendations[]'` therefore crash and hide BioMCP's actionable error. This caused 102 lost calls across 24 measured agent runs.

Imported from March ticket 533. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/533-keep-json-collection-keys-stable-on-empty-and-error-outcomes
