---
flow: quickfix
priority: 5
---
# Pin every release job and packaged binary to the requested tag commit

The published `v0.8.25` tag points to commit `b5337826`, while the installed Linux `0.8.25` release binary reports embedded git SHA `a6694289`, a later commit. The manual `workflow_dispatch` path accepts `inputs.tag` but the validate, binary build, PyPI build, Homebrew, and docs jobs use `actions/checkout@v4` without pinning that tag. They can validate and package the selected branch while uploading artifacts to an older tag. Docker already checks out the requested tag, so one version can differ by distribution channel.

Completed under March on 2026-07-10, as March ticket 495. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/495-pin-every-release-job-and-packaged-binary-to-the-requested-tag-commit
