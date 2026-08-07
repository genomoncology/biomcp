---
base: e62b45066d931480b8d4fd38df09ab4216af266b
head: 46f714c1392de82ef51081f2be70de8b61e84b40
---
Ticket 505 proved that ticket 502 did not meet its 60% performance target. Across three cold article-only samples, the pre-502 median was 335,081 ms and the shared-fixture candidate median was 332,757 ms, or only 0.69% faster (0.63% including the candidate's separately timed setup and cleanup). Fixture lifecycle was under 1% of wall time; BioMCP commands consumed about 99%.

Imported from March ticket 507. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/507-stop-live-pacing-on-explicit-fixtures-and-contain-article-overrides
