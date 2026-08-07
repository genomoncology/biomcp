---
flow: build
priority: 5
---
# Decompose skill.rs into src/cli/skill/ submodules

`src/cli/skill.rs` is 1,032 lines and currently mixes embedded-asset loading, read-only skill catalog rendering, install-path discovery, filesystem install orchestration, and inline tests. The same module also backs MCP resource reads via `src/mcp/shell.rs`, so the read-only catalog surface must stay stable while installation logic moves into its own ownership zone.

Completed under March on 2026-04-26, as March ticket 322. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/322-decompose-skill-rs-into-src-cli-skill-submodules
