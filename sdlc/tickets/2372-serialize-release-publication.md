# 2372 — Serialize release publication across versions

Status: OPEN.

Milestone: 0.9.4

Priority: 1

- Changes: `.github/workflows/release.yml`, `.github/actionlint.yaml`, `tests/test_release_workflow_provenance.py`, `docs/reference/release-process.md`, `CHANGELOG.md`.

## Outcome

Concurrent releases cannot move the container latest pointer backward after a newer release has published.

## Starts from

Fresh release review confirmed the defect on main `01eccd84ff53cc97c684906d95c730610d51feae`. Workflow concurrency separates each tag or candidate. The publish-release job reads public releases, moves latest and publishes its draft in separate steps. An older publisher can make its decision before a newer publisher finishes, then overwrite latest afterward. The structural cause is shared state without common exclusion.

## Keeps

Preserve concurrent builds, the existing version comparison, failed-publication behavior, permissions, tag-only publishers and nonpublishing untagged rehearsal. No actual publication or external pointer changes are authorized by this ticket.

## Design

Design review accepted job-level concurrency on the existing publish-release job with fixed group `release-publication`, `queue: max` and `cancel-in-progress: false`. Retain workflow-level per-tag concurrency. The protected job already spans the release-list query, version decision, latest move and draft publication. It is the only latest writer; container-only dispatch writes versioned images and cannot enter this publisher. Keep builds outside the shared exclusion.

GitHub's current concurrency documentation says the default replaces pending jobs; queue max retains multiple waiting publishers within its platform limit. Retain version comparison regardless of queue order. Source: https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency

Pinned actionlint 1.7.12 does not parse queue. Add a release.yml-specific configuration exception for only that exact unsupported-key diagnostic, preserving all other diagnostics. Existing parsed workflow tests must enforce the queue value and cancellation setting. Sources: https://raw.githubusercontent.com/rhysd/actionlint/v1.7.12/parse.go and https://raw.githubusercontent.com/rhysd/actionlint/v1.7.12/docs/config.md

- Proof: Extend the existing workflow contract to fail on the missing shared publication exclusion. Assert the fixed group, queue and cancellation values and bind the query, decision, latest move and draft publication to that protected job. Preserve dispatch/rehearsal refusal and concurrent builds. Run existing version-decision tests, affected workflow checks and workflow lint through build jobs, then the existing complete untagged rehearsal. Do not add a simulated scheduler; rehearsal does not prove live scheduling or pointer mutation.
