---
flow: quickfix
priority: 7
---
# Reorder install instructions: curl first, then uv, then pip

All install documentation lists PyPI (`uv tool install`) as the primary install method and the curl binary installer as a secondary option. The preferred order should be curl first (zero dependencies, fastest), then `uv tool install`, then `pip install` as fallback. This applies across README, installation docs, and blog article "Try It" sections.

Completed under March on 2026-04-10, as March ticket 168. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/168-reorder-install-instructions-curl-first-then-uv-then-pip

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
