---
base: 122546ac383509157c1098bdcf34fda34f5747e2
head: 777a72fe8956d1fdbb66fb881a98686c645f80ad
---

Registered the five supported release targets and built each target's canonical
executable once before deriving its native archive and platform wheel. The
workflow uses pinned Linux build images and explicit macOS and Windows runners.

Artifact inspection proves target identity, hashes, package membership,
platform floors, signing evidence, deterministic SBOM data, CLI identity, MCP
initialization, seven-tool catalog, and PNG support.
