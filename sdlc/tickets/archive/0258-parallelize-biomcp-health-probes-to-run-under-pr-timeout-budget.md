---
flow: build
priority: 6
---
# Parallelize biomcp health probes and consolidate health spec coverage

`biomcp health` probes 52 external services serially. A fully healthy run takes ~20s cold; when even one upstream retries or hangs on a socket timeout, wall time jumps to 120-130s. Serial dispatch means one slow probe dominates the total rather than running alongside the others. Cold-CI `spec-pr` runs regularly blow the 60-second per-heading timeout because of this retry amplification, and operators running `biomcp health` interactively wait two minutes for what should be a fast diagnostic.

Completed under March on 2026-04-20, as March ticket 258. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/258-parallelize-biomcp-health-probes-to-run-under-pr-timeout-budget
