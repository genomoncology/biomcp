---
flow: build
priority: 3
deps: []
---

# 1229: Close the dispatch publish path and harden the runtime image parse

## Goal

A `workflow_dispatch` can no longer publish wheels for a ref that was never released, and the recorded container base stops accepting whatever text follows the Dockerfile argument.

## Current Facts

- `pypi-publish` has no event gate (`release.yml` `needs: [pypi-build, wheel-smoke, docs-live]` only). On a full dispatch whose `inputs.tag` names an unpublished ref, `pypi-build` builds wheels from that ref and `pypi-publish` would upload them; only `homebrew-tap` and `container-publish` fail closed through `gh release download`. The reachability predates ticket 1222, but the new runbook sentence (`docs/reference/release-process.md:88-90`) asserts the safe outcome as if it always held.
- `release/container.py:31-37` `runtime_image()` takes the `ARG RUNTIME_IMAGE=` line verbatim, so a trailing inline comment or an empty value lands in `provenance.base`, and a missing Dockerfile raises an uncaught `FileNotFoundError` instead of `ContainerError`. `tests/test_release_container.py` parses the same line the same way, so that class cannot fail the test.

## Design

- Add `if: github.event_name == 'release'` to `pypi-publish` and pin it in `tests/test_release_workflow_provenance.py`; update the runbook so a full dispatch is described as never reaching PyPI rather than failing there.
- In `runtime_image()`, strip an inline `#` comment, reject an empty or whitespace value, and map a missing or unreadable Dockerfile to `ContainerError`; pin the expected literal from `Dockerfile:1` in the test instead of re-parsing the line.

## Acceptance

1. A dispatch cannot reach `pypi-publish` on any input; the provenance guard fails if the event gate is dropped.
2. The runbook states the dispatch publish behavior accurately.
3. An empty, commented, or missing `RUNTIME_IMAGE` value fails `runtime_image()` with `ContainerError`, and the test pins the real trixie literal.

## Out of scope

- The `homebrew-tap` older-tag rewrite, which the runbook already warns about.
- Any change to the container backfill path.

## Review

- Design review: the 2026-09-22 full review of the release wave (gpt-5.6-sol, medium) found the ungated dispatch publish path and the loose runtime image parse
- Code review: ACCEPT 2026-09-22 (gpt-5.6-sol, medium) — one report-only note on the substring pin
- Verification: yellow gate at 3fba56e3 lint/test/spec OK (986 Python passed, 3 skipped); mutation checks on the gate and the parse; see `sdlc/records/1229-close-the-dispatch-publish-path-and-harden-the-runtime-image-parse.md`
