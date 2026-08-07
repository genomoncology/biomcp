---
base: 6e4a9fae3c8252c5d33ffd0b3059f89b803da0a3
head: 1265d0ae3289c9b7659aa310be27bf0674b244f4
---
Root cause (verified in-tree): after the requested results are collected and rendered, `src/cli/disease/dispatch.rs:64` calls `disease_search_workflow(results.first())`, which probes follow-up workflows — including `Workflow::TrialRecruitment` (`:101-110`) → `disease_has_recruiting_trials()` (`:148-154`) → `crate::entities::trial::search_page(...)` (`:152`). That probe **re-queries CTGov with the full condition-/alias-expansion fan-out**, even when the user asked for `--limit 1`. The post-output probe, not the primary query, is what blows the latency budget.

Imported from March ticket 438. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/438-bound-ctgov-trial-helper-post-output-latency-for-limit-1
