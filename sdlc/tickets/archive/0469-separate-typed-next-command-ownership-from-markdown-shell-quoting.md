---
flow: build
priority: 6
---
# Separate typed next-command ownership from markdown shell quoting

The architecture review found next-command ownership leaking across layers: some entity code imports markdown quoting helpers to construct semantic guidance, while JSON envelopes are built unevenly across dispatchers. This makes shell safety and JSON follow-up behavior inconsistent and hard to ratchet.

Completed under March on 2026-06-30, as March ticket 469. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/469-separate-typed-next-command-ownership-from-markdown-shell-quoting
