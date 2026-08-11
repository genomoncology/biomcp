---
base: bfd9488e7933f009a2c2b66ae5311135890157f2
head: df0dce065a525cfb89dfe9261b109f7fc841b73a
---

Removed the nested CLI-surface pytest invocation from the routine executable
spec. The contract remains directly owned by `make test`.

Added an executable-block and helper-script ratchet for pytest, nextest,
non-dry-run `make test`/`make lint`, and the complete quality wrapper. It parses
shell tokens so prose, quoted expectations, arguments, and dry-run Makefile
inspection do not become false positives. All 49 focused quality and CLI-surface
tests pass, and the repository-wide named spec-lint audit passes.
