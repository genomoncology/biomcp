---
flow: quickfix
priority: 9
---
# Make BioMCP spec-pr self-contained for March kickoff

March build tickets use the repo-owned `spec-only` validation profile during kickoff. In BioMCP, that profile runs `make spec-pr`, but `make spec-pr` invokes specs with `BIOMCP_BIN=$(CURDIR)/target/release/biomcp` without first building that binary. Fresh March worktrees therefore fail kickoff with `target/release/biomcp: No such file or directory` before any agent can work.

Completed under March on 2026-04-27, as March ticket 344. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/344-make-biomcp-spec-pr-self-contained-for-march-kickoff
