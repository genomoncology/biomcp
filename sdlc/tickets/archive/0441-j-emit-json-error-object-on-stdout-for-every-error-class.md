---
flow: quickfix
priority: 7
---
# -j: emit JSON error object on stdout for every error class

Make -j emit a JSON error object on stdout for every error class (not just some); today not_found/InvalidArgument go to stderr with empty stdout, breaking jq piping. Low severity (MCP unaffected).

Completed under March on 2026-06-23, as March ticket 441. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/441-j-emit-json-error-object-on-stdout-for-every-error-class
