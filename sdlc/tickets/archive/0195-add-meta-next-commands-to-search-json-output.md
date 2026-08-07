---
flow: build
priority: 7
---
# Add meta next commands to search JSON output

`biomcp get drug daraxonrasib --json` returns a structured `_meta.next_commands` array with four follow-up commands — exactly what an agent needs to auto-chain research without parsing markdown. Every `search` command's `--json` output, however, returns no `_meta` block at all. `biomcp search article --json`, `biomcp search trial --json`, `biomcp search variant --json`, and the rest have top-level keys like `query`, `pagination`, `count`, `results` but no `next_commands`. The markdown renderer already emits context-aware follow-up suggestions for search results (the filters hint line, the `get <entity> <id>` hint) but those strings never get threaded into the JSON response.

Completed under March on 2026-04-15, as March ticket 195. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/195-add-meta-next-commands-to-search-json-output
