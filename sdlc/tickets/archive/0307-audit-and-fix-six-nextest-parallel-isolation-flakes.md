---
flow: build
priority: 7
---
# Audit and fix six nextest parallel-isolation flakes

Six nextest tests fail intermittently under parallel execution, all with the same shape: shared mock-server port, shared HTTP cache directory, or env-var mutation racing across test threads. They individually pass under `--test-threads=1` but block CI under default parallelism. Each one was filed separately; the underlying pattern is one bug.

Completed under March on 2026-04-25, as March ticket 307. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/307-audit-and-fix-six-nextest-parallel-isolation-flakes
