---
flow: build
priority: 7
---
# Unify shell quoting across emitted suggested commands

biomcp emits suggested commands as shell strings throughout the rendered output (`See also:` blocks, `More:` follow-ups, error recovery hints). Some of these strings include drug/disease names with spaces, parens, or quotes. A copy-paste workflow can break when shell-active characters land unquoted. This is a cross-cutting hygiene issue with security-flavored implications (command injection if an upstream label contained `;` or backticks).

Completed under March on 2026-04-26, as March ticket 313. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/313-unify-shell-quoting-across-emitted-suggested-commands
