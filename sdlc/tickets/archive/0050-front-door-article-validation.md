---
flow: build
priority: 7
---
# Reject invalid article input before backend work and classify usage failures cleanly

The v0.8.18 review found that invalid article dates can emit backend-looking warnings before the CLI reports a usage error. That makes a simple operator typo look like a network problem and is hard for wrappers to classify.

Completed under March on 2026-03-25, as March ticket 050. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/050-front-door-article-validation
