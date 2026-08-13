---
base: 19c35aade626d45a98e7232e11c93c8b5ad34657
head: 3d5950358653d35e1296d13056f0490609beb130
---

Added protected promotion as a separate mode that consumes only one successful,
sealed prior stage run. It reconciles all 13 candidate artifacts and protected
signing policy, publishes immutable versioned objects, and installs or downloads
only public GitHub, PyPI, GHCR, Homebrew, and MCPB bytes for platform proof.

Mutable pointers move only after the public matrix, installer, live-provider,
updater-transition, and Ian-recorded Windows Claude Desktop checks pass. Safe
replay, conflicting bytes, missing or stale public data, identity drift,
provider limits, partial records, and pointer ordering have local fixture tests.
The operator guide makes version approval, credential/signing provisioning, and
separate official MCP Registry submission explicit. No release was performed.
