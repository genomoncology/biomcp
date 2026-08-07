---
base: ba4d160af6a5a0916fbb694aaec7a94e5da09317
head: a220bcae691bd80d1cca5d50226e4ef71add1058
---
126 commits have landed on `main` since the last release (`v0.8.24`, 2026-06-24), including several features that shipped in code and docs but were never published because no release has been cut to fire the release workflow. The most visible consequence is a user-reported bug: `docker pull ghcr.io/genomoncology/biomcp:latest` returns 404 because the `docker-publish` job (added after v0.8.24) has never run — the docs already tell users to run that image. Cutting a new release publishes the Docker image, the Homebrew tap update, the PyPI package, and the MCP registry metadata that have all accumulated behind the last tag.

Imported from March ticket 479. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/479-release-prep-v0-8-25-version-bump-curated-changelog
