---
base: 017c2c4ab2d56fe915c99feb7a6c1ddd9d9635aa
head: 25081e2ab82abd10632808fe60274b91bcc18d88
---
BioMCP's blocking `make spec-pr` lane is red because `biomcp get disease 'Marfan syndrome' funding` cannot resolve the disease by name. Direct upstream checks show OLS4 returns the correct MONDO concept but exposes it as `short_form: MONDO_0007947` with an empty `obo_id`; BioMCP currently treats the empty `obo_id` as present and only normalized underscore short forms for HP IDs. This prevents the discover fallback from converting OLS4's Marfan result into the usable `MONDO:0007947` ID, so build-flow kickoff for unrelated tickets fails before agents can work.

Imported from March ticket 345. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/345-fix-ols4-underscore-ids-in-disease-discover-fallback
