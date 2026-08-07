---
flow: quickfix
priority: 9
---
# Stop comparing the ERepo detail @id host and stop reporting our own identity check as a ClinGen outage

variant erepo --detail fails for every record because we require the provider's @id to equal a URL we build on our own host, and we report that self-inflicted failure as a generic ClinGen ERepo API error.

Completed under March on 2026-08-02, as March ticket 650. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/650-stop-comparing-the-erepo-detail-id-host-and-stop-reporting-our-own-identity-check-as-a-clingen-outage
