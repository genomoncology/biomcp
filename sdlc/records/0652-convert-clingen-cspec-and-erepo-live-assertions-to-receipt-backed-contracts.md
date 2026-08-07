---
base: 0ae3f4c0fe056be75afb7a8d6614053c2960e4dd
head: f899503236d897681b03da15e0dc05b9233cbbae
---
What is IN scope: - `src/entities/gene/cspec.rs`, `src/entities/variant/erepo.rs`, their source tests/captures, and the two live pages. - The corresponding entries in `scripts/run-specs.sh::SPEC_LIVE_PATHS` only after their replacements are green.

Imported from March ticket 652. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/652-convert-clingen-cspec-and-erepo-live-assertions-to-receipt-backed-contracts
