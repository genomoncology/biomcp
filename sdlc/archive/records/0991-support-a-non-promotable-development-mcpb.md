---
base: 2f401f38
head: 271ed09d
---

Private development staging can now use an unsigned outer MCPB only when its
macOS and Windows executables carry real platform signatures. The exception is
explicitly development-only and non-promotable; stable candidates still require
a real signed outer MCPB. The attestation consumes the exact candidate manifest,
archive hash, protected policy, tool identity, source, version pair, stage run,
and complete GitHub job context without copying or mutating the archive.

Candidate registration deeply validates the complete nested stable or
development evidence form, including archive bytes, signing identity, policy,
run, source, versions, fixture status, and canonical filename. A reproduced
development-to-stable relabel attempt now fails at registration and promotion.
Workflow smokes validate the recorded exception and the bundled native
signatures; documentation distinguishes runner smoke from manual Claude Desktop
installation. Focused release tests, canonical lint, independent review, and
the later exact-head hosted gates passed.

External provisioning remains intentionally outside this record: staging still
requires a protected `biomcp-release-signing` environment, real Apple signing
and notarization material, real Windows Authenticode material, and a predecessor
commit that enables those identities in the protected policy.
