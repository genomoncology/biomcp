---
flow: build
priority: 5
---
# Migrate biomcp spec runner to the mustmatch binary and the spec/verify lane model

mustmatch is now a single Rust binary (mustmatch team ticket 11) — the pytest plugin biomcp uses (`pytest spec/ --mustmatch-lang bash …`) is deleted. biomcp is the only repo on that plugin (it is pinned to the last plugin release to stay working). This ticket moves biomcp onto the new `mustmatch test` binary, replaces the `--deselect`/`-n auto` pytest mechanics the binary lacks, formalizes the routine-vs-live lane split the `--deselect` was hiding, extracts standup into the standard `scripts/run-specs.sh`, and unpins mustmatch. After this, biomcp runs its spec corpus on the same runner as every other repo (Gen 2).

Completed under March on 2026-06-04, as March ticket 393. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/393-migrate-biomcp-spec-runner-to-the-mustmatch-binary-and-the-spec-verify-lane-model
