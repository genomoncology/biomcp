---
base: 4fb32f6e727140cbc7844d4ebe68e1e99526fd13
head: 9f620333c4fa23c43e88f9cbee9078de5da2612d
---
Human-facing Markdown escapes line breaks and table pipes but preserves ANSI/CSI, OSC hyperlinks, NUL, and other terminal controls from provider titles/venues/citation text and rejected user identifiers. A crafted upstream value can corrupt terminal or log presentation. This is output integrity rather than code execution, and it should be prevented by one reusable boundary plus a monotonic table-driven ratchet.

Imported from March ticket 574. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/574-sanitize-terminal-controls-at-human-output-boundaries
