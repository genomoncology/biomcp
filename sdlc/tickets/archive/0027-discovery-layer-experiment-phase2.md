---
flow: experiment
priority: 6
---
# Discovery layer experiment phase 2: UMLS + MedlinePlus + unified design

Experiment 035 (ticket 026) validated OLS4 as the v1 discovery backbone but left two open questions: (1) UMLS wasn't tested live because the API key wasn't in the environment, and (2) MedlinePlus showed promise for patient- facing context but its integration model with OLS4 wasn't explored. This follow-up experiment runs UMLS live, tests all three APIs in parallel on the same query set, and produces a concrete integration design for a `biomcp discover` command that unifies OLS4 + UMLS + MedlinePlus.

Completed under March on 2026-03-20, as March ticket 027. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/027-discovery-layer-experiment-phase2

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
