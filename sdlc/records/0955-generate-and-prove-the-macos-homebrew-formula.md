---
base: e96cb5f98b69f3d4cd3a1d60b07a99d967a230ee
head: b98862e6b1e5ac16ad5f66e0c8f75061c5cc52aa
---

The Homebrew formula is generated from the exact signed Intel and Apple Silicon
candidate records and their immutable GitHub release URLs. It verifies both the
downloaded archive and installed executable, installs the compatibility command
as a symlink, and binds output to the canonical version and revision.

Private stage tests preseed Homebrew's final-URL cache and install offline on
both supported Mac runners, including quarantine and Gatekeeper assessment,
without mutating the public tap.
