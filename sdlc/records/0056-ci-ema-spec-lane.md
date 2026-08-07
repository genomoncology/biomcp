---
base: d3da24d6686341424d44d34e8cf251f603c80386
head: 869aa4cdc2b07d041134a2bb84b425d0b450267a
---
The v0.8.18 release CI (`spec-stable` job) failed because GitHub Actions runners don't have EMA local data at `~/.local/share/biomcp/ema/`. The drug search spec added for EMA `--region` support runs in the stable gate and hard-fails on any runner without the local JSON files.

Imported from March ticket 056. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/056-ci-ema-spec-lane
