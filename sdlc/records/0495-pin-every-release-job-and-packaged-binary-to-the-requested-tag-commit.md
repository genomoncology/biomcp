---
base: 0a1f4cd202911e67c2107885e137e93188876799
head: 9ab7d711e7fca2270d37a425609254bee1f2ae0e
---
The published `v0.8.25` tag points to commit `b5337826`, while the installed Linux `0.8.25` release binary reports embedded git SHA `a6694289`, a later commit. The manual `workflow_dispatch` path accepts `inputs.tag` but the validate, binary build, PyPI build, Homebrew, and docs jobs use `actions/checkout@v4` without pinning that tag. They can validate and package the selected branch while uploading artifacts to an older tag. Docker already checks out the requested tag, so one version can differ by distribution channel.

Imported from March ticket 495. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/495-pin-every-release-job-and-packaged-binary-to-the-requested-tag-commit
