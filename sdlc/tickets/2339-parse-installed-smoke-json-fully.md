# 2339 — Parse installed-smoke JSON fully

Status: OPEN.

Milestone: 0.9.4

## Outcome

scripts/installed-wheel-smoke.sh parses each command's complete stdout as JSON and requires an object where the release runbook promises one. Malformed output such as `{garbage`, trailing text, or an array where an object is required fails the smoke with a message naming the command.

- Proof: negative regressions feed `{garbage`, valid JSON followed by text, and a JSON array to the smoke's check and see each fail; the existing installed-wheel smoke still passes on a real wheel.

## Evidence

- On 2026-10-21, a fresh independent release review of integration 3c8aee87a found that scripts/installed-wheel-smoke.sh accepts any stdout starting with `{` as a JSON object, including malformed `{garbage`. The release runbook promises that malformed JSON fails.
