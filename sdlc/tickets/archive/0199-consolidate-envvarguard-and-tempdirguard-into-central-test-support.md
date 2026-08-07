---
flow: build
priority: 4
---
# Consolidate EnvVarGuard and TempDirGuard into central test support

`EnvVarGuard`, `TempDirGuard`, and `set_env_var` are each defined in roughly 20 copies across the crate — `src/sources/{mod,europepmc, wikipathways,rate_limit,who_pq,ema}.rs`, `src/utils/download.rs`, `src/cache/{clear,config,planner,migration,limits,manager,clean}.rs`, `src/cli/{search_all,health,cache}.rs`, `src/cli/benchmark/run.rs`, `src/entities/{variant,pathway,trial}.rs`, and two sub-crate `test_support.rs` files under `src/entities/{disease,article}`. Canonical versions already exist in `src/cli/test_support.rs` but are not reused. The copies are drifting — some have `// Safety:` comments on the unsafe env-mutation blocks, others don't; some respect `TMPDIR` via `std::env::temp_dir()`, others hardcode `/tmp`; pid+nanos suffix formatting differs across copies. If a Rust edition update changes the rules around unsafe env mutation, the fix has to be replicated in 20 places.

Completed under March on 2026-04-16, as March ticket 199. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/199-consolidate-envvarguard-and-tempdirguard-into-central-test-support
