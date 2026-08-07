---
flow: quickfix
priority: 5
---
# Fix release.yml validate: install ripgrep + sync dev deps before make spec

The `Release` workflow (`.github/workflows/release.yml`) never publishes because its `validate` job fails at `make spec`, and every publishing job (`build`, `homebrew-tap`, `docker-publish`, `pypi-build`, `pypi-publish`, `deploy-docs`) gates on `validate`. Two environment gaps in the validate job cause the failure — both masked locally and in March worktrees because those environments already provide the tools:

Completed under March on 2026-07-08, as March ticket 481. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/481-fix-release-yml-validate-install-ripgrep-sync-dev-deps-before-make-spec
