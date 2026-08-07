---
flow: build
priority: 5
---
# Move EMA-backed specs out of CI stable gate

The v0.8.18 release CI (`spec-stable` job) failed because GitHub Actions runners don't have EMA local data at `~/.local/share/biomcp/ema/`. The drug search spec added for EMA `--region` support runs in the stable gate and hard-fails on any runner without the local JSON files.

Completed under March on 2026-03-26, as March ticket 056. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/056-ci-ema-spec-lane
