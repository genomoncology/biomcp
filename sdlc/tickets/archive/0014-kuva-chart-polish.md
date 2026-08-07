---
flow: build
priority: 6
---
# Polish kuva chart output: human labels and KM curves

BioMCP's study charts pass raw cBioPortal identifiers as chart labels (e.g., `Missense_Mutation`, `Frame_Shift_Del`). These snake_case strings are long, don't word-wrap naturally, and cause x-axis label collision and truncation in both terminal and SVG output. Mapping them to short human-readable display names before passing to kuva would fix most of the reported rendering issues without requiring kuva changes.

Completed under March on 2026-03-18, as March ticket 014. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/014-kuva-chart-polish

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
