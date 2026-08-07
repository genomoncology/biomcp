---
flow: build
priority: 10
---
# Stop the strict route from starving the exact route and stop labelling an internal work-budget stop as a provider outage

The 50-unit per-item work budget is spent by the strict route before exact_lexical binds, and --verify-identity can never return complete; both are reported to the caller as provider degradation and outage, including for a provider that was never called.

Completed under March on 2026-07-31, as March ticket 634. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/634-stop-the-strict-route-from-starving-the-exact-route-and-stop-labelling-an-internal-work-budget-stop-as-a-provider-outage
