---
flow: build
priority: 7
---
# Consolidate live-network spec tests into make spec-smoke lane

`make spec-pr` is currently red under normal network conditions because several live-network spec tests exceed the 60s `--mustmatch-timeout`. Issues 182 and 223 enumerate six specific test headings. The PR quality-bar command is unreliable, so the team either waits on retries or silently accepts noise. Consolidating these into a separate `make spec-smoke` lane with a longer timeout pins the PR lane as a fast deterministic gate and absorbs both `watching` issues.

Completed under March on 2026-04-21, as March ticket 270. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/270-consolidate-live-network-spec-tests-into-make-spec-smoke-lane
