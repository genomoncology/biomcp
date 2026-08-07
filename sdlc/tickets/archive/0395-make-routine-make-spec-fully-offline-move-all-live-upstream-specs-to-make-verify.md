---
flow: quickfix
priority: 5
---
# Make routine make spec fully offline; move all live-upstream specs to make verify

Routine `make spec` is non-deterministic: it executes specs that call **live external biomedical APIs**, so every ticket's gate (design baseline AND verify) randomly fails on whichever upstream is flaky that minute. This has blocked a string of unrelated tickets — pathway (Reactome/WikiPathways), the MCP serve-http race, and most recently `phenotype.md` with a **Monarch HTTP 502** that killed an unrelated quickfix mid-flight. Patching one spec at a time is futile (the patch's own `make spec` trips on the next live upstream). This ticket makes routine `make spec` **fully offline/deterministic** by moving every live-upstream spec into an operator-run `make verify` lane (the four-lane model), permanently unblocking biomcp development. It also folds in the MCP serve-http readiness fix (superseding ticket 394).

Completed under March on 2026-06-04, as March ticket 395. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/395-make-routine-make-spec-fully-offline-move-all-live-upstream-specs-to-make-verify
