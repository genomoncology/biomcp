---
flow: quickfix
priority: 5
---
# Fix spec17 bash backtick quoting so mustmatch assertion runs

`spec/17-cross-entity-pivots.md` line 23's mustmatch assertion contains backticks that bash evaluates as command substitution, producing `bash: line 8: get: command not found` and masking the real assertion. The test has been broken for weeks; pre-existing on main, not introduced by ticket 248.

Completed under March on 2026-04-22, as March ticket 276. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/276-fix-spec17-bash-backtick-quoting-so-mustmatch-assertion-runs
