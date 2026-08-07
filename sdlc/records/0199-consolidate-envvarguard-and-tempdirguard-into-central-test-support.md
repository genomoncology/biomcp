---
base: 8d846d56dd167f08245644408ef89a93110fae6f
head: f34f9af97160bfc8569db17ae58a27277b699a0c
---
`EnvVarGuard`, `TempDirGuard`, and `set_env_var` are each defined in roughly 20 copies across the crate — `src/sources/{mod,europepmc, wikipathways,rate_limit,who_pq,ema}.rs`, `src/utils/download.rs`, `src/cache/{clear,config,planner,migration,limits,manager,clean}.rs`, `src/cli/{search_all,health,cache}.rs`, `src/cli/benchmark/run.rs`, `src/entities/{variant,pathway,trial}.rs`, and two sub-crate `test_support.rs` files under `src/entities/{disease,article}`. Canonical versions already exist in `src/cli/test_support.rs` but are not reused. The copies are drifting — some have `// Safety:` comments on the unsafe env-mutation blocks, others don't; some respect `TMPDIR` via `std::env::temp_dir()`, others hardcode `/tmp`; pid+nanos suffix formatting differs across copies. If a Rust edition update changes the rules around unsafe env mutation, the fix has to be replicated in 20 places.

Imported from March ticket 199. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/199-consolidate-envvarguard-and-tempdirguard-into-central-test-support
