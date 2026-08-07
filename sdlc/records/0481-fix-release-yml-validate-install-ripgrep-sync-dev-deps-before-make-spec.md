---
base: b5337826dbf06db6d6409f36ead7a4d6a70c710e
head: 9cefbdf1fabaf40ac8c16f6bbc4a0e3cf88260d6
---
The `Release` workflow (`.github/workflows/release.yml`) never publishes because its `validate` job fails at `make spec`, and every publishing job (`build`, `homebrew-tap`, `docker-publish`, `pypi-build`, `pypi-publish`, `deploy-docs`) gates on `validate`. Two environment gaps in the validate job cause the failure — both masked locally and in March worktrees because those environments already provide the tools:

Imported from March ticket 481. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/481-fix-release-yml-validate-install-ripgrep-sync-dev-deps-before-make-spec
