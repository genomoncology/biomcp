---
base: d97a0d9394e0dda497c3e1f949da1eae893ce896
head: 01c32d86afbf4f92bd30e6c75834736905e1cde1
---
mustmatch is now a single Rust binary (mustmatch team ticket 11) — the pytest plugin biomcp uses (`pytest spec/ --mustmatch-lang bash …`) is deleted. biomcp is the only repo on that plugin (it is pinned to the last plugin release to stay working). This ticket moves biomcp onto the new `mustmatch test` binary, replaces the `--deselect`/`-n auto` pytest mechanics the binary lacks, formalizes the routine-vs-live lane split the `--deselect` was hiding, extracts standup into the standard `scripts/run-specs.sh`, and unpins mustmatch. After this, biomcp runs its spec corpus on the same runner as every other repo (Gen 2).

Imported from March ticket 393. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/393-migrate-biomcp-spec-runner-to-the-mustmatch-binary-and-the-spec-verify-lane-model
