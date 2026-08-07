---
flow: build
priority: 7
---
# Auto-download EMA data on first use and refresh stale files

EMA data requires a manual multi-step curl download before any EU drug command works. Users hit a wall on first use with a "Missing required EMA file(s)" error and a URL to go figure it out. The data is public, small (~11 MB), and has no auth — BioMCP should just fetch it automatically.

Completed under March on 2026-03-26, as March ticket 057. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/057-ema-auto-download-and-sync
