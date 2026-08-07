---
base: 04ba8c30fd478f0cc2fdcdd50859960689ddcb33
head: 11b9e019074cb143d58fa5afab62ad31c32ab59a
---
`biomcp suggest "<question>"` is a 100% offline, zero-backend regex/keyword router over a fixed in-binary catalog of ~15 "playbook" routes (`src/cli/suggest/`, ~1400 lines). It matches by ordered substring-keyword gates: the first route whose hardcoded keywords appear in the question wins. It is brittle in a way that defeats its own purpose:

Imported from March ticket 488. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/488-retire-biomcp-suggest-command-and-its-offline-regex-router-point-agents-at-skill-list
