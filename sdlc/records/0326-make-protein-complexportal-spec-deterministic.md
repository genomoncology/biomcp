---
base: 9df9e57e308c69d2a6349e02e151a28d17c82b38
head: ed9a536a245697b56d93fc088b61ce6b7b52b680
---
The blocking `make spec-pr` lane currently proves the protein ComplexPortal section by calling the live EBI ComplexPortal API for P15056. March kickoff worktrees do not restore the CI `.cache/biomcp-specs/` cache, so unrelated refactor tickets can be blocked by transient live API empties, rate limits, or degraded 200 responses. The spec should prove CLI request translation and rendering against a deterministic fixture, while live ComplexPortal availability remains an operator-health concern.

Imported from March ticket 326. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/326-make-protein-complexportal-spec-deterministic
