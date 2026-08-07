---
base: c496a4203210a8e6db82c6ffee0d6b93234ae2bb
head: 9c963ad49307dbabbece7bed5610134ee4c14f8c
---
`variant normalize all ... --json` can exit 0 with no stdout. For scripts and agents, a successful JSON command must emit parseable JSON or a clear nonzero error; empty success is not honest output.

Imported from March ticket 464. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/464-fix-variant-normalize-json-silent-success
