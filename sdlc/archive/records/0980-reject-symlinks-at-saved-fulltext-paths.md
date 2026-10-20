---
base: 4560607b
head: f051668b
---

Saved full-text reads now open one verified regular file without following
symlinks and reject files with multiple hard links. The read remains attached
to that opened handle, so replacing the path after validation cannot redirect
the operation to another file.

Focused symlink, hard-link, replacement-race, size, line-range, and outline
tests passed. The complete release gate passed after the batch.
