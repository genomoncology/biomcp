---
base: 81e4f4816be5412c62506b85cf715042ee1d8246
head: f52672f8d77f80b560f7f51818cb290512af6a23
---
`make spec-pr` is currently red under normal network conditions because several live-network spec tests exceed the 60s `--mustmatch-timeout`. Issues 182 and 223 enumerate six specific test headings. The PR quality-bar command is unreliable, so the team either waits on retries or silently accepts noise. Consolidating these into a separate `make spec-smoke` lane with a longer timeout pins the PR lane as a fast deterministic gate and absorbs both `watching` issues.

Imported from March ticket 270. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/270-consolidate-live-network-spec-tests-into-make-spec-smoke-lane
