---
base: 25994919212f885b613d8746b5a2a4c632d9207e
head: bfaf29d4a22c55487798104b126f62cbd2b9ad38
---
The Seven-Variant Recall canary returns PMID 11805335 in results while debug_plan.candidate_trace has no row for it, so a returned article carries no route-stage receipt.

Imported from March ticket 638. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/638-account-for-every-returned-article-in-the-candidate-trace-atm-p-c2464r-returns-pmid-11805335-with-no-route-provenance
