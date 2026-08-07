---
base: 123a3591266c047dcb941d7391fce27f0705d711
head: 5d0a4209f13ec8563b19e0a261771617244ce963
---
Routine `make spec` is non-deterministic: it executes specs that call **live external biomedical APIs**, so every ticket's gate (design baseline AND verify) randomly fails on whichever upstream is flaky that minute. This has blocked a string of unrelated tickets — pathway (Reactome/WikiPathways), the MCP serve-http race, and most recently `phenotype.md` with a **Monarch HTTP 502** that killed an unrelated quickfix mid-flight. Patching one spec at a time is futile (the patch's own `make spec` trips on the next live upstream). This ticket makes routine `make spec` **fully offline/deterministic** by moving every live-upstream spec into an operator-run `make verify` lane (the four-lane model), permanently unblocking biomcp development. It also folds in the MCP serve-http readiness fix (superseding ticket 394).

Imported from March ticket 395. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/395-make-routine-make-spec-fully-offline-move-all-live-upstream-specs-to-make-verify
