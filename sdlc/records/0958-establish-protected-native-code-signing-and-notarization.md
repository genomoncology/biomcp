---
base: 5ec8a2ba11b68d287ed7d26e6227263f6df9b332
head: 122546ac383509157c1098bdcf34fda34f5747e2
---

Established a protected, candidate-bound signing policy and fail-closed native
finalization seam. macOS signing and notarization and Windows Authenticode
verification must match committed policy bytes whose hash is also supplied by
the protected environment.

The committed policy intentionally remains disabled until Ian provisions and
reviews real Apple, Windows, and MCPB identities. Fixture signing evidence is
useful for local tests but cannot enter a final candidate.
