---
base: 4c638da5
head: 3231f49b
---

The typed MCP search and get tools now publish discriminated `oneOf` branches
instead of generic entity unions. Search has exactly the eight intended
entities and named fields; get has exactly twelve branches whose sections come
from each entity's production section constants. Author exposes no sections,
and an entity cannot accept another entity's section or filter.

Runtime validation mirrors the schemas: bounded trimmed strings and arrays,
enum and numeric checks, required identity choices, GWAS's 50-row window, NCI
mutual exclusion, and a final production Clap parse all happen before command
execution. Tests cover every typed search entity, cross-entity mismatches,
boolean and pagination errors, and section isolation. Focused MCP tests and
no-feature Clippy passed.

The real seven-tool response is 15,817 UTF-8 bytes under the 16,000-byte cap.
The implementation is exactly +260 net `src` lines, matching the ticket
ceiling.
