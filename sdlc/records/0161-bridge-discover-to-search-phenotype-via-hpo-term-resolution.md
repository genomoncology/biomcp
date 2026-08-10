---
base: f68a2589043cd3b97cf825b60f524548751d21b7
head: 7bca6b8163716d23b70937f4947c8f5f1e6a2033
---
`discover` queries OLS4 which already searches the HPO ontology, but when HPO terms are found they are not surfaced prominently and `search phenotype` is never suggested as a follow-up command. This means an agent that starts from free-text symptoms ("febrile seizures, developmental delay") cannot reach the phenotype-to-disease matching pipeline without already knowing HPO codes.

Imported from March ticket 161. The range was recovered after the fact, then
corrected by operator review on 2026-08-10 to the main-reachable landed commit
`7bca6b8163716d23b70937f4947c8f5f1e6a2033`. Ticket-owned patches are
byte-identical after excluding unrelated branch-wide tracing changes under
`src/sources/**` and an unrelated landed change in `spec/06-article.md`; the
normalized patch SHA-256 is
`d5b33e4c3abdfe873b7aede5959e8a6ea17467c4782a525469a757897d566c53`.
Both commit objects exist, the recorded base is the landed head's parent, and
the landed head is an ancestor of current main. This note deliberately does
not claim whole-tree equivalence for the excluded paths.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/161-bridge-discover-to-search-phenotype-via-hpo-term-resolution
