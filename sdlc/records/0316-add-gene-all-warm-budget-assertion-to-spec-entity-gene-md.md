---
base: 405ebaea06c512f832ab5b86804d63ceb1655fd3
head: 87702047578ad6787e8551484f568331eb24a698
---
The `biomcp get gene <symbol> all` runtime was already optimized to ~2–3s warm by the ParallelTop fanout (artifact `225-reduce-biomcp-get-gene-symbol-all-runtime-under-30s-budget`), but no executable spec assertion was added. There is currently no spec-only ratchet that would catch a regression back toward the historical ~8s baseline or the prior ~44s serial path. This ticket lands the missing contract only — no runtime work.

Imported from March ticket 316. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/316-add-gene-all-warm-budget-assertion-to-spec-entity-gene-md
