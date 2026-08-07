---
base: 88fc9934e8df541ec15a44177064974392b87fac
head: f670cb69fbbcd15a265e9cff57bed05b08d295de
---
`src/cli/skill.rs` is 1,032 lines and currently mixes embedded-asset loading, read-only skill catalog rendering, install-path discovery, filesystem install orchestration, and inline tests. The same module also backs MCP resource reads via `src/mcp/shell.rs`, so the read-only catalog surface must stay stable while installation logic moves into its own ownership zone.

Imported from March ticket 322. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/322-decompose-skill-rs-into-src-cli-skill-submodules
