---
base: 2e2b4798
head: 3fba56e3
---

Closed the dispatch publish path and hardened the recorded runtime image parse,
from the 2026-09-22 full review of the release wave.

`pypi-publish` now carries `if: github.event_name == 'release'`, so a
`workflow_dispatch` cannot publish wheels for an unpublished ref on any input
combination, including re-runs; the runbook states that a full dispatch never
reaches PyPI instead of claiming it fails there. `release/container.py`'s
`runtime_image()` strips an inline `#` comment from the `ARG RUNTIME_IMAGE=`
value, rejects an empty value, and maps an unreadable or missing Dockerfile to
`ContainerError`; the test now pins the real trixie literal from `Dockerfile:1`
instead of re-parsing the argument line, with failure cases for the empty,
commented, and missing-Dockerfile paths.

Evidence: actionlint clean; the provenance and container tests pass with
mutation checks (deleting the gate fails `test_pypi_publish_requires_a_release_event`;
reverting the parse fails six tests); the `cli_surface_contract` ratchet audit
passes; the full gate on yellow at 3fba56e3 passes `make lint`, `make test`
(986 passed, 3 skipped), and `make spec`.

Reviews: the ticket came straight from the full review's findings, so the
design review is that review; the code review accepted with one report-only
note (the gate pin is a substring match that a step-level relocation would
satisfy).

Also from the full review: the three independent passes over the release path,
the CA bundle, and the documentation and test changes returned SHIP with no
P0 or P1; the remaining notes are recorded as residuals in the 1221 and 1226
records and here. The release binary at `d2e180ec` (0.9.1-dev.1, `.text`
24,444,368) runs the four overflow commands, the issue #250 command, search,
health, and the MCP tool list with no crash, all CA-bundle failure modes exit 1
naming the path, and the release-profile wheel (14,906,067 bytes) runs the same
commands without a stack overflow.

Residual: a step-level `if` relocation inside `pypi-publish` would still satisfy
the provenance pin's substring match, and the full review's dispatch-publish
note about `homebrew-tap` rewriting an older tag remains documented in the
runbook rather than gated.
