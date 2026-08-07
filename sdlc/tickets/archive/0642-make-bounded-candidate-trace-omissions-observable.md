---
flow: build
priority: 7
---
# Make bounded candidate-trace omissions observable

candidate_trace.bounded is a hardcoded true, so an operator cannot tell a full trace from one that silently dropped rows at the ITEM_WORK_LIMIT cap.

Completed under March on 2026-08-02, as March ticket 642. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/642-make-bounded-candidate-trace-omissions-observable
