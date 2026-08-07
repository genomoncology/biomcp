---
flow: quickfix
priority: 5
---
# Run contract+release smokes only at tag/release time (gatekeep release); remove daily cron; fix stale WikiPathways probe

The structure already supports this: `release.yml` has a `validate` job that every publish job depends on (`needs: validate`) — so a failing `validate` blocks asset upload, PyPI publish, and docs deploy. But `validate` currently runs fmt/clippy/test + pytest + mkdocs only; it does **not** run `make spec`, `contract-smoke.sh`, or `release-smoke.sh`.

Completed under March on 2026-06-24, as March ticket 448. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/448-run-contract-release-smokes-only-at-tag-release-time-gatekeep-release-remove-daily-cron-fix-stale-wikipathways-probe
