---
base: 5303c228bf4c07254e13e3185952e7292e1be3c7
head: f090ca0914fcd01b72df4c0b7c613ee77e8e5336
---
Six nextest tests fail intermittently under parallel execution, all with the same shape: shared mock-server port, shared HTTP cache directory, or env-var mutation racing across test threads. They individually pass under `--test-threads=1` but block CI under default parallelism. Each one was filed separately; the underlying pattern is one bug.

Imported from March ticket 307. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/307-audit-and-fix-six-nextest-parallel-isolation-flakes
