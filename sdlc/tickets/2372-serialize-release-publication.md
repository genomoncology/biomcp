# 2372 — Serialize release publication across versions

Status: OPEN.

Milestone: 0.9.4

Priority: 1

- Changes: `.github/workflows/release.yml`, `tests/test_release_promotion.py`, `tests/test_release_container.py`, `CHANGELOG.md`.

## Outcome

Concurrent releases cannot move the container latest pointer backward after a newer release has published.

## Starts from

Fresh release review confirmed the defect on main `01eccd84ff53cc97c684906d95c730610d51feae`. Workflow concurrency separates each tag or candidate. The publish-release job reads public releases, moves latest and publishes its draft in separate steps. An older publisher can make its decision before a newer publisher finishes, then overwrite latest afterward. The structural cause is shared state without common exclusion.

## Keeps

Preserve concurrent builds, the existing version comparison, failed-publication behavior, permissions, tag-only publishers and nonpublishing untagged rehearsal. No actual publication or external pointer changes are authorized by this ticket.

## Design

Use one shared publication exclusion across versions around the release-list query, latest decision and pointer move, and draft publication. A fresh design review verifies workflow queue and cancellation semantics from primary documentation and checks every publisher that can affect the same pointer. Do not serialize all build work or add another release path.

- Proof: Extend the existing release promotion/workflow checks so the current per-tag publication design fails and the repaired shared exclusion covers the decision through publication. Preserve older/newer version decisions and prove rehearsal cannot enter publishing. Run affected checks and workflow lint through build jobs; the final complete untagged release rehearsal exercises the repaired workflow without publishing.
