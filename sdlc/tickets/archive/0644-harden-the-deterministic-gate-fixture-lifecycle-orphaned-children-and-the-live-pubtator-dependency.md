---
flow: build
priority: 7
---
# Harden the deterministic gate: fixture lifecycle, orphaned children, and the live PubTator dependency

Three known ways the routine gate stops being deterministic \u2014 cold-cache\ \ fixture-lifecycle timeouts, orphaned fixture children holding the routine lock,\ \ and a make test contract that still calls live PubTator.

Completed under March on 2026-08-02, as March ticket 644. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/644-harden-the-deterministic-gate-fixture-lifecycle-orphaned-children-and-the-live-pubtator-dependency
