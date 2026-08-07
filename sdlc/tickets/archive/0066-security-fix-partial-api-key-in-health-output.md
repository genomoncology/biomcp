---
flow: build
priority: 9
---
# Security fix — remove partial API key from health output

`biomcp health` and `biomcp health --json` embed the first 3 characters of every configured API key in the `status` column/field:

Completed under March on 2026-03-27, as March ticket 066. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/066-security-fix-partial-api-key-in-health-output
