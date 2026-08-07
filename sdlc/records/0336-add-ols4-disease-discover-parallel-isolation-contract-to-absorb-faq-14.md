---
base: 443a4d3b3a8ca230d5e06c688bbe184f47ca6c2b
head: 6c3c46f6ee7f3f25e9ce2f790441ddc515a668c6
---
FAQ watching entry #14 says OLS4 discover fallbacks are slow enough to flake under `-n auto --dist loadfile` and should be absorbed by serial or fixture-backed lanes. The Makefile only carves out `spec/entity/protein.md` today. The 327 code review found that `spec/entity/disease.md` and `spec/surface/discover.md` still call OLS4-backed discover headings (`Resolved via discover + crosswalk`, synonym rescue, alias routing, symptom mapping), and there is no automated check that prevents new OLS4-heavy headings from being added back into the parallel pool.

Imported from March ticket 336. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/336-add-ols4-disease-discover-parallel-isolation-contract-to-absorb-faq-14
