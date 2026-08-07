---
flow: quickfix
priority: 5
---
# Pin build.rs git SHA stamp to --short=8 (deterministic across clones)

The `Release` workflow's `validate` job now gets past `make spec` but fails in `scripts/release-smoke.sh`, which asserts the release binary's stamped git SHA matches `HEAD`. The binary is stamped with an **adaptive-length** short SHA while release-smoke compares against a **fixed 8-char** short SHA, so they diverge by environment:

Completed under March on 2026-07-08, as March ticket 482. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/482-pin-build-rs-git-sha-stamp-to-short-8-deterministic-across-clones
