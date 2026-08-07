---
base: 2aba9af672783d05771e3bf060fb450688c28bab
head: b70416fc3dd533ffacdcb3ce1a332c363b15ab76
---
`biomcp get drug daraxonrasib --json` returns a structured `_meta.next_commands` array with four follow-up commands — exactly what an agent needs to auto-chain research without parsing markdown. Every `search` command's `--json` output, however, returns no `_meta` block at all. `biomcp search article --json`, `biomcp search trial --json`, `biomcp search variant --json`, and the rest have top-level keys like `query`, `pagination`, `count`, `results` but no `next_commands`. The markdown renderer already emits context-aware follow-up suggestions for search results (the filters hint line, the `get <entity> <id>` hint) but those strings never get threaded into the JSON response.

Imported from March ticket 195. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/195-add-meta-next-commands-to-search-json-output
