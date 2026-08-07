---
base: 34ce91e2fe2619c77431e97b452b4e81282029e2
head: 716f9d7370521b63e56cfa7caf268d898c359738
---
`tools/biomcp-ci` (the spec invocation wrapper from ticket 298) silently falls back to the system `biomcp` binary when `BIOMCP_BIN` is unset, instead of failing loudly. This means a stale system binary can quietly poison spec runs that the developer thought were testing the worktree. Add an explicit warning (or hard error) when `BIOMCP_BIN` is unset and the fallback path is taken.

Imported from March ticket 315. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/315-warn-when-biomcp-ci-falls-back-to-system-biomcp-binary
