---
base: 8014e0927199a6abc8f46cb6501d841a241bc920
head: 11295038848e5292e10a36414c3ef6fd40c10b56
---

Routine test and specification gates now run fail-closed in a Linux network
namespace that blocks public DNS and direct public TCP while retaining local
TCP and Unix sockets. Build and dependency preparation happens before
isolation; Rust tests execute from a prepared nextest archive. Remaining
routine live dependencies and stale gate contracts were replaced with local
proofs. The complete test and specification suites passed inside isolation.
