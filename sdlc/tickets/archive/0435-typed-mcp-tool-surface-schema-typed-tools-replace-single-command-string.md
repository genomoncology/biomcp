---
flow: build
priority: 5
---
# Typed MCP tool surface: schema-typed tools (replace single command string)

A benchmark pilot measured the cost: **18% of 638 MCP calls failed (112)**, almost all because the agent guessed the CLI wrong — 38 nonexistent flags, 33 `--limit` out-of-bounds, 11 wrong-section. A typed surface (real input schemas enumerating valid entities/sections/flags and stating limit bounds) makes most of those un-guessable: the agent picks from the schema instead of probing by trial and error. This is the highest-leverage **usability** fix in the feedback note (`planning/feedback/biomcp/2026-06-22-biomcp-bench-pilot-rough-edges.md`, #1) and a benchmark-validity issue (it's why the pilot added an RQ4 interface ablation — to separate "BioMCP's data helps" from "BioMCP's interface hurts").

Completed under March on 2026-06-23, as March ticket 435. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/435-typed-mcp-tool-surface-schema-typed-tools-replace-single-command-string
