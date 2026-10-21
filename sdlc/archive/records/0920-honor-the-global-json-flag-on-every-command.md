---
base: 9259bc2c4ab8379602f666322ebb82dbe4a9692b
head: da390d02cb7aa3e15f7f60def1e83c7549a27374
---

Every finite command now honors the global JSON flag with a typed JSON
document. Commands that start long-running servers reject JSON mode before
startup, and an unexpected successful plain-text projection is converted to a
structured internal error rather than emitted as invalid JSON.

The command matrix covers the formerly text-only system surfaces and both
global flag positions. Focused JSON tests and the complete repository gates
passed.
