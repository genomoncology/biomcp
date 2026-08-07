---
flow: quickfix
priority: 4
---
# Warn when biomcp-ci falls back to system biomcp binary

`tools/biomcp-ci` (the spec invocation wrapper from ticket 298) silently falls back to the system `biomcp` binary when `BIOMCP_BIN` is unset, instead of failing loudly. This means a stale system binary can quietly poison spec runs that the developer thought were testing the worktree. Add an explicit warning (or hard error) when `BIOMCP_BIN` is unset and the fallback path is taken.

Completed under March on 2026-04-26, as March ticket 315. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/315-warn-when-biomcp-ci-falls-back-to-system-biomcp-binary
