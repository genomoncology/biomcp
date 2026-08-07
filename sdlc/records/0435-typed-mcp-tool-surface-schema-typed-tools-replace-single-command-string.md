---
base: 12498d1eb107e54a0c2167d59330637891286a0b
head: 99c25a4d45914667800a9f534305c186068d537b
---
A benchmark pilot measured the cost: **18% of 638 MCP calls failed (112)**, almost all because the agent guessed the CLI wrong — 38 nonexistent flags, 33 `--limit` out-of-bounds, 11 wrong-section. A typed surface (real input schemas enumerating valid entities/sections/flags and stating limit bounds) makes most of those un-guessable: the agent picks from the schema instead of probing by trial and error. This is the highest-leverage **usability** fix in the feedback note (`planning/feedback/biomcp/2026-06-22-biomcp-bench-pilot-rough-edges.md`, #1) and a benchmark-validity issue (it's why the pilot added an RQ4 interface ablation — to separate "BioMCP's data helps" from "BioMCP's interface hurts").

Imported from March ticket 435. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/435-typed-mcp-tool-surface-schema-typed-tools-replace-single-command-string
