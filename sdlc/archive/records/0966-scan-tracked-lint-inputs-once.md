---
base: 28bc5158b09c23988d90221499853c81bddccc7c
head: b408f23ca8944960ffd420a95e44178c8fa4635e
---

Replaced the per-file and per-line shell subprocess loops with one tracked-text
checker. It collects Git paths once, reads each text file at most once, and
retains the credential, TBD, pycache, public-doc, and documentation-code-leak
contracts and diagnostics.

All 16 lint-contract tests pass. On the same warm loaded machine, `bin/lint`
fell from 40.22 seconds to 8.82 seconds (4.6x), and complete `make lint` fell
from 84.35 seconds to 26.35 seconds (3.2x).
