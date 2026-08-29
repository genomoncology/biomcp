---
base: 3fa4f0b3ceb6a5d4c1a69d4c272423f199a650bb
head: ac7b6b20907a29e79d600b4e8572ee2602405985
---

BioMCP documentation now publishes byte-exact Markdown twins and serves them directly or through `Accept: text/markdown` negotiation. HTML pages advertise their twin while preserving ordinary page and static-asset behavior.

A strict deployment workflow builds the source-derived twins before deploying the Cloudflare edge worker, and `llms.txt` documents every supported agent path.
