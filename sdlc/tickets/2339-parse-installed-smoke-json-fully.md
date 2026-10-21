# 2339 — Parse installed-smoke JSON fully

Status: OPEN.

Milestone: 0.9.4

## Outcome

scripts/installed-wheel-smoke.sh parses each command's complete stdout as JSON and requires an object where the release runbook promises one. Malformed output such as `{garbage`, trailing text, or an array where an object is required fails the smoke with a message naming the command.

- Changes: `scripts/installed-wheel-smoke.sh`, `tests/test_installed_wheel_smoke.py`, `tests/test_release_workflow_provenance.py`, `docs/reference/release-process.md`, `CHANGELOG.md`.
- Proof: negative regressions feed `{garbage`, valid JSON followed by text, and a JSON array to the smoke's check and see each fail; the existing installed-wheel smoke still passes on a real wheel.

## Evidence

- Coordinator restatement: The installed-wheel smoke must parse each complete JSON stdout document and require an object, naming the command when output fails. It also exercises embedded skill render/install from the installed package, as coordinated with 2338.
- Reproduction and planned proof: The reviewed current-main script accepts an opening brace without parsing the remaining document. The developer published tests-only RED candidate 1302695d6 before product edits; exact-job outcome will follow. Negative controls cover malformed opening-brace output, trailing text and arrays.
- Planned files and scope: scripts/installed-wheel-smoke.sh, tests/test_installed_wheel_smoke.py and any maintained production smoke body pin required by the existing contract.

- On 2026-10-21, a fresh independent release review of integration 3c8aee87a found that scripts/installed-wheel-smoke.sh accepts any stdout starting with `{` as a JSON object, including malformed `{garbage`. The release runbook promises that malformed JSON fails.
