---
flow: quickfix
priority: 8
---
# Fix variant normalize JSON silent success

`variant normalize all ... --json` can exit 0 with no stdout. For scripts and agents, a successful JSON command must emit parseable JSON or a clear nonzero error; empty success is not honest output.

Completed under March on 2026-06-29, as March ticket 464. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/464-fix-variant-normalize-json-silent-success
