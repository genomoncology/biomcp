---
base: a1c8d07d
head: d67c5ccd
---

External provider failures now cross one structured logging boundary with a
provider, operation, stable class, optional HTTP status, and bounded safe
message. Nested request URLs and credentials never reach tracing, including
debug logging in article federation, enrichment, PMC HTML, and full-text PDF
paths.

Tests cover a recognizable fake secret, exact and plus-one 512-byte UTF-8
boundaries, and the source audit that rejects raw Debug logging of caught
external errors. All 279 focused article tests and the new quality-ratchet
contract passed. Source growth finished exactly at the authorized 160 lines.
