---
base: 3124d59c865f3678a6015d883eedf817be1bec72
head: d9f43fc1cae975493afebf9cbc70d4fc575f58ac
---
Repair CSpec version paging, which fails after a successful fetch and capture, and stop attributing the internal failure to ClinGen

Imported from March ticket 633. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/633-repair-clingen-cspec-version-paging-and-stop-reporting-internal-failures-as-provider-failures
