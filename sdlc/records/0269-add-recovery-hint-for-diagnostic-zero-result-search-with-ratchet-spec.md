---
base: 240804781383d98658461c0680bf9a8b27da68cd
head: ccca04813c8524201df07fda8ea7ad0238524e63
---
`biomcp search diagnostic --disease "nonexistent xyz"` currently outputs `No diagnostic tests found. Showing 0 of 0 results.` with no recovery suggestion. Gene and disease zero-result paths emit actionable `Try searching:` or `See also:` hints. The diagnostic zero-result path is the weakest operational path on the new surface, and no spec pins the expected recovery contract.

Imported from March ticket 269. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/269-add-recovery-hint-for-diagnostic-zero-result-search-with-ratchet-spec
