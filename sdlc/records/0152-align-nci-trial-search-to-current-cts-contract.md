---
base: 475254d779ad622d8cf802a28e2668c943f1e126
head: 9bab4c1d9f517267f3110bc4ad3d7304711e1e54
---
The current Rust NCI trial path still serializes ClinicalTrials.gov-shaped filters into the NCI CTS client, and authenticated repro against the live NCI API now proves those mappings return zero where the current NCI contract returns real results. `search trial --source nci -c melanoma`, `-s recruiting`, geo-filtered NCI search, and `-p 2` all fail end-to-end today even though the repo already has disease crosswalk data that can resolve NCI concept IDs for condition searches.

Imported from March ticket 152. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/152-align-nci-trial-search-to-current-cts-contract
