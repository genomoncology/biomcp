# Release workflow gating and permissions

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`.

## Homebrew and the container skip the crash smoke

`homebrew-tap` (`release.yml:276`) and `container-publish` (`release.yml:336`) need only `[build, docs-live]`. The tarballs use the same release profile as the wheel, so a crash the wheel smoke catches still ships through the tap and GHCR, and `latest` moves. The container smoke runs only `--version` (lines 410 and 429). Add `wheel-smoke` to both jobs' `needs`, or run the four overflow commands against the Linux tarball binary.

## A full dispatch can roll Homebrew back

`homebrew-tap` has no event gate. A dispatch without `container_only` uploads nothing and publishes nothing to PyPI, so its only effect is rewriting the formula, possibly to an older tag. `docs/reference/release-process.md:86-90` documents this. Add `if: github.event_name == 'release'`.

## Every job gets write permissions

Top-level `contents: write`, `id-token: write`, and `packages: write` (`release.yml:18-21`) reach `build`, `pypi-build`, `wheel-smoke`, and `homebrew-tap`. Those jobs run floating-tag actions such as `PyO3/maturin-action@v1` and `arduino/setup-protoc@v3`. Unless the PyPI trusted publisher is pinned to the `pypi` environment, any of them can mint a publishing token. Set top-level `permissions: {}`, grant each job what it uses, and pin actions by SHA as `docs-edge.yml` does. Confirm the PyPI trusted publisher names the environment.

## Smaller gaps

- The wheel smoke covers only Linux x86_64. Add a `macos-14` leg. The smoke accepts exit 1, so an upstream outage can hide a crash; require at least one command to exit 0.
- `pypi-publish` has `skip_existing: false`, so every full re-run ends red after the wheels land (for example run 35182821002). Set `skip-existing: true` or document "re-run failed jobs only".
- `container-publish` uses `always()` (line 337) and starts after a cancel. Use `!cancelled()`.
- The "Resolve the tag commit" step (line 224) has no retry.
- No workflow-level `concurrency`, so a dispatch can race a release on the tap push.

## Resolved

Ticket 1234 moves release validation ahead of a draft-last GitHub release, checks versions and changelog coverage with behavior-tested scripts, pins the complete publish-job dependency graph with mutation tests, smokes every built wheel, narrows permissions, and preserves the explicit container-backfill dispatch. See `sdlc/records/1234-rework-the-release-gates-so-v0-9-1-can-pass-them.md` after verification.
