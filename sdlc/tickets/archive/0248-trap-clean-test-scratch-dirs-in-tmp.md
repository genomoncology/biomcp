---
flow: build
priority: 4
---
# Trap-clean test scratch dirs in /tmp

biomcp test/check runs leak named scratch dirs in /tmp; add EXIT trap cleanup

Completed under March on 2026-04-19, as March ticket 248. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/248-trap-clean-test-scratch-dirs-in-tmp
