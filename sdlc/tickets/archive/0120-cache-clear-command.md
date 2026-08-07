---
flow: build
priority: 5
---
# Cache clear command

The cache family CLI (116) and the non-destructive `cache clean` command (143) establish the CLI surface and cleanup patterns. This ticket adds the destructive `biomcp cache clear` subcommand — a full wipe of the managed HTTP cache with TTY confirmation safety.

Completed under March on 2026-04-04, as March ticket 120. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/120-cache-clear-command
