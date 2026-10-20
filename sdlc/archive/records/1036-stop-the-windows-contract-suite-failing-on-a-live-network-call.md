---
base: d31d48a0a0ce97da2fce1207108da6c6ad8fc56c
head: 8e44f5fa83472312e54c6286bd5b79ce380dafb5
---

# Stop the Windows contract suite failing on a live network call

The Windows cache epoch contract now uses `cache stats` to create, repair, and
reject managed epoch files without issuing a provider request. Cache stats
initializes and secures the epoch before cleanup and snapshotting, so the
permission and hard-link assertions exercise their named behavior directly.
