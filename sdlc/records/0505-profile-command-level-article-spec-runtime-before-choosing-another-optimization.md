---
base: a47f371bdf7c70b8d1ee613b02d15b813fe88fcd
head: 7936fed3f23136a4e03a9b882196dd735df44754
---
Ticket 502 reduced the routine article fixture from 15 server launches to one but merged without its required same-binary before/after measurement. A post-merge review measured current warm `make spec` at 739.08 seconds with a 0.30-second Cargo freshness check. The ticket-501 timing record reports the pre-502 full gate at 704.28 seconds including 114 seconds of compilation, so fixture startup was not demonstrated to be the dominant bottleneck.

Imported from March ticket 505. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/505-profile-command-level-article-spec-runtime-before-choosing-another-optimization
