---
flow: quickfix
priority: 9
---
# Fix OLS4 underscore IDs in disease discover fallback

BioMCP's blocking `make spec-pr` lane is red because `biomcp get disease 'Marfan syndrome' funding` cannot resolve the disease by name. Direct upstream checks show OLS4 returns the correct MONDO concept but exposes it as `short_form: MONDO_0007947` with an empty `obo_id`; BioMCP currently treats the empty `obo_id` as present and only normalized underscore short forms for HP IDs. This prevents the discover fallback from converting OLS4's Marfan result into the usable `MONDO:0007947` ID, so build-flow kickoff for unrelated tickets fails before agents can work.

Completed under March on 2026-04-28, as March ticket 345. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/345-fix-ols4-underscore-ids-in-disease-discover-fallback
