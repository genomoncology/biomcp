---
flow: build
priority: 9
---
# Align NCI trial search to current CTS contract

The current Rust NCI trial path still serializes ClinicalTrials.gov-shaped filters into the NCI CTS client, and authenticated repro against the live NCI API now proves those mappings return zero where the current NCI contract returns real results. `search trial --source nci -c melanoma`, `-s recruiting`, geo-filtered NCI search, and `-p 2` all fail end-to-end today even though the repo already has disease crosswalk data that can resolve NCI concept IDs for condition searches.

Completed under March on 2026-04-07, as March ticket 152. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/152-align-nci-trial-search-to-current-cts-contract
