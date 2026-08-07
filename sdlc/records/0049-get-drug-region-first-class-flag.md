---
base: 3984b070083ab458027e31c1604bae8e9c3c7ae7
head: f58a6e4d28473b3d19f94e97383a9f5c8eca01bf
---
The v0.8.18 review found that `biomcp get drug` accepts `--region` at runtime but does not declare it in the clap option block. The EMA path works, yet the flag is effectively hidden from operators and scripts.

Imported from March ticket 049. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/049-get-drug-region-first-class-flag
