---
flow: build
priority: 10
---
# Sanitize terminal controls at human output boundaries

Human-facing Markdown escapes line breaks and table pipes but preserves ANSI/CSI, OSC hyperlinks, NUL, and other terminal controls from provider titles/venues/citation text and rejected user identifiers. A crafted upstream value can corrupt terminal or log presentation. This is output integrity rather than code execution, and it should be prevented by one reusable boundary plus a monotonic table-driven ratchet.

Completed under March on 2026-07-15, as March ticket 574. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/574-sanitize-terminal-controls-at-human-output-boundaries
