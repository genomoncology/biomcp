---
flow: quickfix
priority: 5
---
# Make the discover shell-quoting ratchet fully deterministic

The routine shell-quoting contract in `spec/surface/cli-contract-ratchet.md` currently runs `biomcp --json discover 'NM_000248.3:c.1799T>A'`. That command contacts public OLS4 with an 8-second timeout even though the assertion only checks that a generated next command quotes `>` safely. Ticket 501 verification observed the local formatting contract fail when OLS4 timed out, then pass on retry.

Completed under March on 2026-07-12, as March ticket 504. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/504-make-the-discover-shell-quoting-ratchet-fully-deterministic
