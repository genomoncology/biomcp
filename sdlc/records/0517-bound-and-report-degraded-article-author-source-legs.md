---
base: 1e3ef8ff1a5c544419b90f58de9c71c21afafe01
head: 9c3c9420e4af3dfb191bec188e2424b74d6966eb
---
Survey issue 4 found that `search_type_capable_page()` bypasses the bounded federated source-outcome path: one failed PubMed or Europe PMC author leg is silently omitted and a slow leg can consume the generic HTTP timeout. This existing reliability defect must be fixed before an author publication corpus can claim honest coverage.

Imported from March ticket 517. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/517-bound-and-report-degraded-article-author-source-legs
