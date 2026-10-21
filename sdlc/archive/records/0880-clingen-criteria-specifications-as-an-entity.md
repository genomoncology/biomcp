---
base: 4693d157
head: c6af99b7
---

`gene cspec --version ... --files` and `--capture-id ... --files` now expose a
bounded metadata-only attachment manifest, while normal criteria responses
report the exact linked attachment count. Capture selection reuses the stored
bytes and retains capture ID, digest, resource IRI, and binding provenance.

Only public File entities linked from selected specification RuleSets are
accepted. The parser enforces the 100-file count, all 512-byte field limits,
the 4,096-byte URL limit, same-origin HTTPS resolution, stable identifiers,
and duplicate rejection before returning any rows. It never downloads an
attachment. A byte-faithful PTEN GN003 capture plus synthetic exact/plus-one
fixtures prove parsing, no-refetch reuse, safe URL handling, rendering, and
typed MCP parity.

The complete lint, routine test, executable specification, and all-feature
gates passed as part of the three-ticket ClinGen batch.
