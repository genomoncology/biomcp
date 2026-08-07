---
base: 1de9def75c2df4907fa8a248be4401fcc84a20b6
head: 5fbca44606e74a8d8fdafc561b5c477e49d7e098
---
Dead and orphaned code is accumulating in the source layer. An orphaned ORCID network client (and its rate-limit wiring) remains in `src/sources/orcid.rs` / `src/sources/orcid/` even though the product boundary is **no ORCID API calls** (ORCID is citation-supplied evidence only) — so this is executable code that contradicts a decided boundary. Separately, superseded parallel code paths and unjustified `#[allow(dead_code)]` suppressions are scattered across entity and render modules, where an internal seam awaiting a ticket is indistinguishable from abandoned code.

Imported from March ticket 581. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/581-retire-orphaned-orcid-client-and-superseded-dead-code-paths
