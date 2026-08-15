---
base: b6084b6a
head: 9247b6bd
---

BioMCP now has an honest private development identity: Rust and native artifacts
use `0.9.0-dev.1`, while Python wheels use canonical PEP 440
`0.9.0.dev1`. Candidate schema 2 owns both identities and the candidate kind;
wheel evidence is bound to the Python identity and native evidence to Rust
SemVer. Leading-zero, malformed, mismatched, and schema-1 identities fail.

Every promotion, publication, updater, public-snapshot, fixture, and
release-record boundary rejects development candidates before side effects.
Public MCP registry, citation, directory, Homebrew, and latest-release metadata
truthfully remain at `0.8.25`. The breaking-change contract prevents a false
0.8 patch release. Version-sync tests, release suites, executable specs,
independent review, and exact-head hosted gates passed.
