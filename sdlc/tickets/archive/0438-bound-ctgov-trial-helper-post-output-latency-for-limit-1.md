---
flow: quickfix
priority: 5
---
# Bound CTGov trial-helper post-output latency for --limit 1

Root cause (verified in-tree): after the requested results are collected and rendered, `src/cli/disease/dispatch.rs:64` calls `disease_search_workflow(results.first())`, which probes follow-up workflows — including `Workflow::TrialRecruitment` (`:101-110`) → `disease_has_recruiting_trials()` (`:148-154`) → `crate::entities::trial::search_page(...)` (`:152`). That probe **re-queries CTGov with the full condition-/alias-expansion fan-out**, even when the user asked for `--limit 1`. The post-output probe, not the primary query, is what blows the latency budget.

Completed under March on 2026-06-23, as March ticket 438. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/438-bound-ctgov-trial-helper-post-output-latency-for-limit-1
