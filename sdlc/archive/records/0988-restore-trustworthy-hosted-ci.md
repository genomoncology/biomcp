---
base: 349abee9
head: f9a113c9
---

Hosted CI now gives trustworthy evidence for Linux, Windows, the full feature
set, generated sources, repository contracts, and the canonical lint, test,
and specification gates. Windows managed-state hard-link checks use the opened
file handle and fail closed. Ubuntu offline gates run in scoped user and network
namespaces with zero capabilities, NoNewPrivs, blocked public networking,
working loopback and Unix sockets, and verified host ownership mapping.

Repeated clean-run failures removed the local-state assumptions that CI was
supposed to expose: cache statistics use isolated roots, fixture reaping checks
are bounded and fail closed, the published changelog boundary no longer needs a
local tag, MCP catalog measurement uses a verified committed tokenizer, and
specs declare pinned ripgrep. Version checks receive the full commit/tag graph
through blob-filtered partial clones instead of downloading unrelated history.

Independent review accepted each remediation. Exact-head GitHub run
31899002554 passed all five jobs at `f9a113c9`.
