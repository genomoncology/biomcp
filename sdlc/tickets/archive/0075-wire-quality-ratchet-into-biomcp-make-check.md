---
flow: build
priority: 7
---
# Wire quality ratchet into BioMCP make check

Research 006 proved quality ratchet tooling at 0% false-positive rate across NLP, G2, and ATB. BioMCP is a Rust/Python project with no Zig code, so the zig-scanner doesn't apply. But analysis of 17 recent BioMCP code reviews found three recurring defect patterns that are deterministically checkable:

Completed under March on 2026-03-28, as March ticket 075. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/075-wire-quality-ratchet-into-biomcp-make-check
