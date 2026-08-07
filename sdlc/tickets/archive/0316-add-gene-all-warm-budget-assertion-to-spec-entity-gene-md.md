---
flow: quickfix
priority: 5
---
# Add gene-all warm-budget assertion to spec/entity/gene.md

The `biomcp get gene <symbol> all` runtime was already optimized to ~2–3s warm by the ParallelTop fanout (artifact `225-reduce-biomcp-get-gene-symbol-all-runtime-under-30s-budget`), but no executable spec assertion was added. There is currently no spec-only ratchet that would catch a regression back toward the historical ~8s baseline or the prior ~44s serial path. This ticket lands the missing contract only — no runtime work.

Completed under March on 2026-04-26, as March ticket 316. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/316-add-gene-all-warm-budget-assertion-to-spec-entity-gene-md
