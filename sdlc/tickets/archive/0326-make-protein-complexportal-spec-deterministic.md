---
flow: quickfix
priority: 8
---
# Make protein ComplexPortal spec deterministic

The blocking `make spec-pr` lane currently proves the protein ComplexPortal section by calling the live EBI ComplexPortal API for P15056. March kickoff worktrees do not restore the CI `.cache/biomcp-specs/` cache, so unrelated refactor tickets can be blocked by transient live API empties, rate limits, or degraded 200 responses. The spec should prove CLI request translation and rendering against a deterministic fixture, while live ComplexPortal availability remains an operator-health concern.

Completed under March on 2026-04-27, as March ticket 326. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/326-make-protein-complexportal-spec-deterministic
