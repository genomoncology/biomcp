---
flow: build
priority: 8
---
# Bound remote waits and stabilize residual reliability flakes

Several remaining issues are reliability/performance boundaries where the right outcome is an explicit policy plus an automated check: extreme `Retry-After` headers can stall CLI commands, study downloads need a no-stall timeout contract, warm/performance canaries have intermittent outliers, and live-source flakes should not silently weaken release confidence.

Completed under March on 2026-06-09, as March ticket 403. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/403-bound-remote-waits-and-stabilize-residual-reliability-flakes
