---
base: 49523709554d55e353ab9ff6f668d08560221b4d
head: 0b5de1ba5dec02b8aa511803881b8c85a88d276e
---
`src/cli/suggest.rs` is 1,654 lines and currently mixes the route table, 15 route matcher functions, entity/anchor extraction helpers, a large `OnceLock` regex catalog, and inline tests. `biomcp suggest` is an agent-facing first-move surface, so the command must stay behavior-identical while the implementation is split into ownership zones that can evolve independently.

Imported from March ticket 321. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/321-decompose-suggest-rs-into-src-cli-suggest-submodules
