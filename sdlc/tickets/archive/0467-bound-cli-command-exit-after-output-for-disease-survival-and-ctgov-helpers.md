---
flow: build
priority: 8
---
# Bound CLI command exit after output for disease survival and CTGov helpers

An issue report (436) describes a command that prints the expected answer but does not exit within a practical timeout: `get disease ... survival`. For agents, producing the answer is not enough; the process must terminate promptly or callers waste turns and hit wrapper timeouts.

Completed under March on 2026-06-30, as March ticket 467. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/467-bound-cli-command-exit-after-output-for-disease-survival-and-ctgov-helpers
