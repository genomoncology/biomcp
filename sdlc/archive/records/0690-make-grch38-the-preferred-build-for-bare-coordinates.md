---
base: 19be5c56
head: 3561fd61
---

Bare genomic coordinates now probe both assemblies and prefer GRCh38. If both
records differ, the GRCh38 record is primary and the GRCh37 record remains a
labeled competing candidate. A one-build hit still returns the assembly that
actually answered.

Assembly preference is resolved only when a bare variant coordinate needs it:
explicit `--assembly`, then `BIOMCP_DEFAULT_ASSEMBLY`, then GRCh38. The four
documented spellings are accepted, invalid environment values fail before the
provider client is built, and explicit input wins even over an invalid
environment value. Typed MCP adds assembly only to its variant get branch and
maps it through the same CLI path.

Focused entity, renderer, MCP-schema, and precedence tests passed, as did
no-feature Clippy. The real seven-tool response remains under its byte cap at
15,871 bytes. The implementation added 63 net `src` lines under the 140-line
ceiling.
