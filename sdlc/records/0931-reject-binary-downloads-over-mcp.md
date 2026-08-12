---
base: 0e546a44
head: cce59339
---

The shared MCP entrypoint now rejects trial document and article asset byte
downloads before dispatch for raw and typed tools. The bounded error identifies
the download as CLI-only and gives the corresponding terminal command, while
the plural `documents` and `assets` manifest forms remain allowed.

The lower execution boundary independently refuses any binary command outcome.
It no longer has a lossy UTF-8 conversion path, so an allowlist regression
becomes a normal MCP tool error instead of corrupt output.

The stdio and HTTP MCP core contracts cover raw and typed rejection without a
provider, and focused unit tests cover manifest allowance and non-UTF-8 bytes.
No-feature Clippy passed with warnings denied. The implementation added 75 net
`src` lines against the ticket's 100-line ceiling.
