---
flow: build
priority: 8
---
# Surface working typed paths in help and add debug plan output

Agents rely heavily on inline help, `list`, and query echoes to choose the next BioMCP command. Today some working typed paths are not discoverable from the command help surface, and multi-leg searches do not expose the retrieval plan they actually ran. That causes failed commands and extra tool calls even when the needed path already exists.

Completed under March on 2026-03-20, as March ticket 032. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/032-help-surface-and-debug-plan

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
