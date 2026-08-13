---
base: a42eada2
head: aea7c696
---

Raw MCP command execution now rejects every shipped caller-selected file or
stdin input, including variant articles, ClinGen CAR, and ERepo spellings. The
ordinary terminal CLI and bounded typed MCP tools retain their existing input
behavior.

An exhaustive structural test walks the Clap command model and requires every
file-reading argument to have an explicit raw-MCP decision. Focused command and
transport tests passed, and the complete release gate passed after the batch.
