---
flow: build
priority: 5
---
# Release prep v0.8.25: version bump + curated changelog

126 commits have landed on `main` since the last release (`v0.8.24`, 2026-06-24), including several features that shipped in code and docs but were never published because no release has been cut to fire the release workflow. The most visible consequence is a user-reported bug: `docker pull ghcr.io/genomoncology/biomcp:latest` returns 404 because the `docker-publish` job (added after v0.8.24) has never run — the docs already tell users to run that image. Cutting a new release publishes the Docker image, the Homebrew tap update, the PyPI package, and the MCP registry metadata that have all accumulated behind the last tag.

Completed under March on 2026-07-07, as March ticket 479. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/479-release-prep-v0-8-25-version-bump-curated-changelog
