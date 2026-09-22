---
flow: build
priority: 4
deps: []
---

# 1222: Harden the release path and retire stale release facts

## Goal

The release workflow loses its unmaintained pieces and its stale claims. Nothing here changes published artifacts; it removes the next failures before they happen and the drift that hid the dropped container job.

## Current Facts

- `release.yml` uploads assets with `actions/upload-release-asset@v1`, which is archived and only works on `release: published` because it reads `github.event.release.upload_url`.
- `homebrew-tap` computes `VERSION="${GITHUB_REF_NAME#v}"` (`release.yml:181`). On `workflow_dispatch`, `GITHUB_REF_NAME` is the branch, so a manual run could write a formula versioned `main` into the tap.
- A full `workflow_dispatch` fails at the asset upload for the same missing URL; the container-only path avoids it and the release page documents that.
- `docs/reference/mcp-server.md:17` says the metadata is truthful for "the already published v0.8.25 release".
- `architecture/technical/overview.md:67-69` still describes the retired two-step stage/promote workflow.
- `tests/test_docs_changelog_refresh.py:824-825` contains `... or True` and `assert True`, which cannot fail.
- `tests/test_source_package_boundary.py` counts `cargo package` output exactly; untracked files in the yellow gate clone inflated it to 1,347 against the correct 1,342 and produced a false failure there.
- `release/container.py:117` records the retired `debian:bookworm-slim` base, which the Dockerfile no longer uses.
- `cache::migration::tests::async_io_crossing_expiry_settles_without_admitting_a_mutation` (`src/cache/migration.rs:889`) failed once in a full nextest run and passed on rerun and 20/20 in isolation.

## Design

- `build`: upload with `gh release upload "$TAG" <assets> --clobber` and guard the step on `github.event_name == 'release'`; keep the existing packaging and checksums.
- `homebrew-tap`: resolve the tag once as `github.event.release.tag_name || inputs.tag` and use it for the download and the formula version, the way `container-publish` does.
- `release.yml`: verify the live documentation witness for the tag commit after the build (`https://biomcp.org/__biomcp_revision__/<sha>.txt` must equal the tag commit, with bounded retries), so a release fails when biomcp.org is behind instead of shipping while the site is stale.
- Correct the stale architecture document and remove the vacuous test lines, keeping the assertions that can fail.
- Make the package-boundary count compare tracked files only, or document and enforce a clean-tree requirement so the gate clone cannot inflate it.
- Update or delete the `release/container.py` base constant together with a decision on the retired staged tooling.
- Stabilize the cache-expiry test with deterministic IO completion instead of real file IO racing a paused clock.

## Acceptance

1. No workflow uses `actions/upload-release-asset@v1`; `actionlint` and the provenance tests pass.
2. The Homebrew job uses the resolved tag on both triggers; a dispatch cannot write a branch-named formula.
3. The architecture document names v0.9.0 and the single workflow; the vacuous assertions are gone and the surviving ones still fail on a wrong sentence.
4. The package-boundary test passes on a clean checkout and does not depend on untracked files.
5. A release whose site revision is not live fails the documentation check instead of publishing silently.
6. The cache-expiry test passes repeatedly under the full nextest run.
7. CI `canonical-gates` is green on main at the pushed SHA.

## Out of scope

- Deleting the retired `release/` package; that decision is recorded but not taken here.
- Other CI jobs.

## Review

- Design review: pending
- Code review: pending
