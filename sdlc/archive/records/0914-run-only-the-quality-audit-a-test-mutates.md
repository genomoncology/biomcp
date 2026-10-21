---
base: 78368d9d64052865e46d2de954bc63b495ec07ff
head: 5175790cc94e35c79a620f70f39d1b38b4436164
---

Added stable named quality-ratchet audits while retaining one complete wrapper
integration. Negative fixtures now invoke only the audit they mutate, and the
two full Rust-tree audits share one read-and-mask snapshot.

On the same loaded machine, the complete sequential Python contract lane fell
from 593.65 seconds before the change to 93.77 seconds after it: 6.3x faster and
499.88 seconds removed from every `make test` invocation. All 465 Python tests
passed. The focused quality-ratchet file passed all 41 tests, and the separately
owned CLI-surface and ticket-401 groups passed all 10 tests.
