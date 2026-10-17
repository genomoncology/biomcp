# 2155 — Stop automatic CI and documentation runs

Status: OPEN.

Milestone: 0.9.3

## Outcome

Branch pushes, main pushes and pull requests start neither CI nor documentation publication; manual runs remain usable and tag-triggered releases remain unchanged.

Depends on: 2154

- Starts from: `.github/workflows/ci.yml`, `.github/workflows/docs-edge.yml`, `tests/test_ci_workflow_contract.py`, and the existing documentation-publication contracts.
- Keeps: manual CI runs exercise the relevant checks rather than skipping them behind old push-only conditions; manual documentation runs retain the publication revision check; release tag triggers and all publication safeguards remain unchanged.
- Changes: claims `.github/workflows/ci.yml` `.github/workflows/docs-edge.yml` `tests/test_ci_workflow_contract.py` `tests/test_docs_publication_contract.py` `tests/test_release_workflow_provenance.py` `CHANGELOG.md`.
- Proof: on current main, demonstrate the automatic push and pull-request triggers through a failing workflow contract check; run the updated contracts through the build job and prove the workflows admit manual runs while automatic push and pull-request triggers are absent. Check the release workflow trigger is unchanged. Run the existing mechanical checks before reporting.
- Defers: callable release qualification, untagged full rehearsal and moving documentation publication into the release workflow belong to the subsequent release-workflow ticket.

PM decision: 2026-10-17. Ian approved disabling automatic triggers immediately after main's repair, before the historical failure investigation. Keep this change small. Until it lands, push this branch only when ready for review; restore checkpoint pushes afterward. The coordinator records reproduction and review evidence. Do not add machine names or private team details to public text.
