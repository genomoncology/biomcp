---
base: e39601386908c4c8eaca866f24c0bc7a09bcc6fa
head: c3092c884663cde32f6e4368106ede61680518a9
---
variant erepo --detail fails for every record because we require the provider's @id to equal a URL we build on our own host, and we report that self-inflicted failure as a generic ClinGen ERepo API error.

Imported from March ticket 650. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/650-stop-comparing-the-erepo-detail-id-host-and-stop-reporting-our-own-identity-check-as-a-clingen-outage
