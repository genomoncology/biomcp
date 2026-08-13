---
base: b98862e6b1e5ac16ad5f66e0c8f75061c5cc52aa
head: 19c35aade626d45a98e7232e11c93c8b5ad34657
---

Built the declared MCPB bundle from the two signed macOS executables and signed
Windows executable without recompiling. The generated manifest selects the
universal macOS member and the exact Windows executable while retaining the
truthful seven-tool catalog.

The official MCPB tool is integrity-pinned. Packing, certificate signing,
verification, exact-member and executable-hash inspection, and three-runner
smokes all bind the bundle to its candidate; fixture signatures are rejected.
