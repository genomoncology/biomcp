---
base: aea7c696
head: ff863da6
---

Partially failed article batches now preserve their structured report on
stdout while returning a nonzero process status. Human and JSON callers no
longer receive the whole useful report wrapped as one stderr `Error:` message,
and MCP continues to receive the report through the shared command outcome.

Focused unit and real-process tests covered successful, partial, and failed
settlement. The complete release gate passed after the batch.
