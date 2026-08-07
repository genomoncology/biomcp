---
base: 61d4c6e34dfa7a59d8f60c164d2012d44fc980cf
head: 636d8f0ab6ceeb52eb46d1641dc3b1d9fb9260ec
---
The structure already supports this: `release.yml` has a `validate` job that every publish job depends on (`needs: validate`) — so a failing `validate` blocks asset upload, PyPI publish, and docs deploy. But `validate` currently runs fmt/clippy/test + pytest + mkdocs only; it does **not** run `make spec`, `contract-smoke.sh`, or `release-smoke.sh`.

Imported from March ticket 448. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/448-run-contract-release-smokes-only-at-tag-release-time-gatekeep-release-remove-daily-cron-fix-stale-wikipathways-probe
