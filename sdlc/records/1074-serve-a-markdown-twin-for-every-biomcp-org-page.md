---
base: 3fa4f0b3ceb6a5d4c1a69d4c272423f199a650bb
head: ac7b6b20907a29e79d600b4e8572ee2602405985
---

BioMCP documentation now publishes byte-exact Markdown twins and serves them directly or through `Accept: text/markdown` negotiation. HTML pages advertise their twin while preserving ordinary page and static-asset behavior.

A strict deployment workflow builds the source-derived twins before deploying the Cloudflare edge worker, and `llms.txt` documents every supported agent path.

Superseded by ticket 1096 on 2026-09-04. The byte-exact explicit `.md` twins
remain and hosted run 33845621279 verified them from exact revision
`fce136f3df2ff0fbc720ef8d168b9f41d7681b4a`. The former alternative-content
header and `Accept: text/markdown` negotiation requirements were deliberately
retired with the Cloudflare Worker; agents use the explicit paths advertised by
`llms.txt`. The contradictory duplicate source ticket was removed.
