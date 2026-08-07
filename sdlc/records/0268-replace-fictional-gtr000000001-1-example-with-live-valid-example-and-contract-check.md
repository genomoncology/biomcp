---
base: 09a6259dc97981a9132493cb8e7111cf0c13e8d1
head: c04e7d6ca982e1b46e4377259818ec9f40a1c5fd
---
`GTR000000001.1` is used as the example diagnostic accession in seven locations across `README.md` and `docs/user-guide/diagnostic.md`, but that accession does not exist in the live GTR bundle — running `biomcp get diagnostic GTR000000001.1` returns "not found". First-time users following docs hit a dead end. No contract check protects public example accessions from drifting.

Imported from March ticket 268. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/268-replace-fictional-gtr000000001-1-example-with-live-valid-example-and-contract-check
