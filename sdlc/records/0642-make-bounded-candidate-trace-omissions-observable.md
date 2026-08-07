---
base: d508ccd03c59a1a4cb4a1001b1afa029a14e963a
head: 7c93b6a3bfee4f0a56da46267f14c0722699e55b
---
candidate_trace.bounded is a hardcoded true, so an operator cannot tell a full trace from one that silently dropped rows at the ITEM_WORK_LIMIT cap.

Imported from March ticket 642. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/642-make-bounded-candidate-trace-omissions-observable
