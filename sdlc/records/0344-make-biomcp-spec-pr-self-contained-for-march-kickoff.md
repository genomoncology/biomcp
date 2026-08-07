---
base: 867c8ba3e20977edc83ea142ef1e19cea4b48839
head: a05bf36d1b9385c9092d4e895d6e35d0110ee32e
---
March build tickets use the repo-owned `spec-only` validation profile during kickoff. In BioMCP, that profile runs `make spec-pr`, but `make spec-pr` invokes specs with `BIOMCP_BIN=$(CURDIR)/target/release/biomcp` without first building that binary. Fresh March worktrees therefore fail kickoff with `target/release/biomcp: No such file or directory` before any agent can work.

Imported from March ticket 344. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/344-make-biomcp-spec-pr-self-contained-for-march-kickoff
