---
flow: build
priority: 7
---
# Accept long-form protein notation in typed variant queries

BioMCP currently rejects valid typed variant inputs such as `AKT2 p.Pro50Thr` before query execution, even though the underlying variant source can resolve the long-form protein notation. This creates unnecessary parser failures on otherwise answerable typed queries.

Completed under March on 2026-03-21, as March ticket 039. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/039-long-form-protein-notation-parser

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
