---
base: a659054ea85ec55d8c02161695362cf25e663395
head: 76f7da16385430bed86f0dc845087eef6be9911d
---
release-smoke.sh defaults to an existing target/release binary and only\ \ rebuilds if absent, so it can validate a stale build (it did \u2014 a dab68f67-stamped\ \ binary gave a misleading 444 FAIL). Assert binary SHA == HEAD; rebuild if missing/stale.\ \ Low-priority tooling hygiene.

Imported from March ticket 446. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/446-harden-release-smoke-sh-assert-binary-sha-matches-head-never-smoke-a-stale-build
