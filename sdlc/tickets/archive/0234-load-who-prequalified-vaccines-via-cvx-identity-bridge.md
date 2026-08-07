---
flow: build
priority: 6
---
# Load WHO prequalified vaccines via CVX identity bridge

WHO publishes 284 prequalified vaccines covering the diseases that determine drug access for hundreds of millions of patients (HIV, TB, malaria, hepatitis, measles, HPV, COVID-19, yellow fever, polio). Spike 231 proved these can't load through the MyChem/INN pipeline (57% match), but with the CVX identity layer (ticket 233), vaccine names can map through CVX codes instead.

Completed under March on 2026-04-18, as March ticket 234. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/234-load-who-prequalified-vaccines-via-cvx-identity-bridge
