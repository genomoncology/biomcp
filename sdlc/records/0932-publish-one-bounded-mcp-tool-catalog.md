---
base: cdba3e9a
head: 1f8fc94d
---

One typed Rust catalog now owns the intentional seven-tool MCP inventory,
stable ordering, compact descriptions, titles, and read-only annotations. The
router supplies each derived input schema through that catalog, and startup
fails immediately if a router method and catalog entry drift apart. Server
instructions name all seven tools and favor bounded typed routes before the raw
escape hatch.

The raw `biomcp` description now points to bounded `biomcp list` discovery
instead of embedding the Markdown command reference. A reproducible local
measurement reports 6,707 UTF-8 bytes and 1,628 `cl100k_base` tokens for the
full `tools/list`, with a 425-byte raw description. Tests ratchet the 16,000-byte,
4,000-token, and 4,000-byte ceilings.

Claude setup, MCP reference, MCPB manifest, active design blog, release smoke,
and quality audits now use or check the same seven-tool inventory. The focused
stdio/HTTP contracts, 72 catalog/packaging/documentation/quality tests, 74
broader documentation tests, release-smoke tests, and no-feature Clippy passed.
The change removed 172 net `src` lines against the ticket's +280-line ceiling.
