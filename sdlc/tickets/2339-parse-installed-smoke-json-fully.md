# 2339 — Parse installed-smoke JSON fully

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 7f8e8fc3bd8375b9849f297112b3cc22f5c39b83, accept

Reviews: revision 327ea87767718fb1702279fe6a981db65cfd8f63, accept

## Outcome

scripts/installed-wheel-smoke.sh parses each command's complete stdout as JSON and requires an object where the release runbook promises one. Malformed output such as `{garbage`, trailing text, or an array where an object is required fails the smoke with a message naming the command.

- Changes: `scripts/installed-wheel-smoke.sh`, `tests/test_installed_wheel_smoke.py`, `tests/test_release_workflow_provenance.py`, `docs/reference/release-process.md`, `CHANGELOG.md`.
- Proof: negative regressions feed `{garbage`, valid JSON followed by text, and a JSON array to the smoke's check and see each fail; the existing installed-wheel smoke still passes on a real wheel.

## Evidence

- Coordinator restatement: The installed-wheel smoke must parse each complete JSON stdout document and require an object, naming the command when output fails. It also exercises embedded skill render/install from the installed package, as coordinated with 2338.
- Reproduction and planned proof: The reviewed current-main script accepts an opening brace without parsing the remaining document. The developer published tests-only RED candidate 1302695d6 before product edits; exact-job outcome will follow. Negative controls cover malformed opening-brace output, trailing text and arrays.
- Planned files and scope: scripts/installed-wheel-smoke.sh, tests/test_installed_wheel_smoke.py and any maintained production smoke body pin required by the existing contract.

- On 2026-10-21, a fresh independent release review of integration 3c8aee87a found that scripts/installed-wheel-smoke.sh accepts any stdout starting with `{` as a JSON object, including malformed `{garbage`. The release runbook promises that malformed JSON fails.

- Actual Gator j2705 at tests-only 1302695d681008c6821d42db434bbb3c088891c4 failed the new negative controls: malformed/trailing output falsely passed, while arrays already failed with incomplete command/object diagnostics. The first affected run j2721 remained failed for an old prefix-only static linkage assertion and two direct-Python PATH harness errors; corrected production uv invocation and linkage passed in j2733. No failure was relabelled.
- Fresh review found Bash command substitution silently strips NUL bytes before parsing. Actual j2735 reproduced acceptance of raw object-plus-NUL output. The repair preserves each JSON command's raw stdout in a temporary file, then one shared validator strictly decodes UTF-8, parses the entire JSON document, rejects invalid constants, and requires an object with command-specific diagnostics. Focused j2741 passed all affected controls including the NUL negatives. Installed smoke also checks rendered embedded skill content and exact installed content, satisfying the coordinated 2338 proof.
- PM m3223 assigns this lane the text-only removal of the unsupported workflow long-suite claim. The release-process workflow paragraph is corrected; required local long checks and actual workflow mechanisms remain unchanged.
- Direct M5 make landing-gate j2751 passed exit zero at clean published 24df0c27d18513e9c5d462bd212ddcd658f907c7. Separate actual production make installed-wheel-smoke Gator j2749 passed at that same source: optimized manylinux wheel build, glibc 2.28 floor and the installed smoke including render/install succeeded. Focused controls do not substitute for this real artifact proof. Yellow data/biomcp/ticket-evidence/2339 retains the failed and passing raw receipts; j2751 log SHA256 1421d649aef506bf5db4966765e50100f1edc981b1e3840e21c6689d5d5f291c; j2749 log SHA256 00a0b730231630ede0f2bb76c1b33ceda34c97de6a4b2fc512b6419cd764225a.
- Fresh reviewer accepts 24df0c27d18513e9c5d462bd212ddcd658f907c7 after independently reading both terminal passes and the hostile-output proof. The coordinator independently verified the actual job states. This Evidence-only update changes no gate input; land with the other accepted corrections through PM m3262's approved one-candidate, one-gate method.
