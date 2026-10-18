---
priority: 2
---

# 2223 — Document legacy cache reconciliation at daily maintenance

Status: OPEN.

Milestone: 0.9.4

Depends on: 2218

Reviews: revision a40fd13, accept

Reviews: revision 8a34b9a, accept

## Outcome

Document the approved mixed-version boundary: sharing a cache with v0.9.2 may temporarily exceed the configured size limit until the next command-triggered daily clean reconciles it. Preserve fast startup and hit paths, and name explicit cleanup as the earlier remedy. No production cache-code change.
- Evidence:2203 job1792355882950-20969 used exact87d53c current and verified publishedv0.9.2 with a50000-byte limit. Currentseed left17440actual/17440marker; oldwrite left34882actual/17440marker; nextcurrentcommand returned validBRAF but left52332actual/34890marker, exceeding the limit. Daily marker remained fresh. This is more than an expected intermediate legacy stale marker.
- Changes: `docs/reference/configuration.md` `CHANGELOG.md` `sdlc/tickets/2223-account-for-legacy-writes-within-cache-size-limit.md`. Add the narrow explanation beside existing daily-clean guidance; no new framework, policy parser, or test duplicating prose.
- Proof: compare documentation with the recorded PM ruling and2203's two controlled daily-clean observations. Reuse2218 explicit-clean proof for the manual remedy. The earlier immediate-cap RED records the superseded premise; remove that uncommitted assertion and preserve the raw experiment. No new build or behavioral test is required for this wording-only change.

PM ruling: Option1 supersedes immediate mixed-version enforcement. Keep fast paths; reconcile v0.9.2 writes at the daily maintenance boundary, document the temporary overage, and finish2203's two controlled daily cleans. Maintenance runs when a command opens the cache, not from a background timer. SQLite storage compatibility will be considered separately.
Actual RED: job2223/1792356413928-86704 on publishede334c5965 plus deterministic existing-module test, copied qualified20de CLI SHA2569291dc4583a51a2d156b9da720684b6d186b7fceeefc84c3baab8d923bb81ebf. Three real CLI commands return valid gene data; restoring the prior saved marker reproduces a legacy untracked write, then the next command leaves52494physical bytes against50000 and fails the size assertion in0.65s. A first fresh-venv setup invocation lacked pytest; corrected invocation reused prepared2156 Python without rebuilding. Product code remains unchanged.

PM decision: resolved as Option1 above. No per-operation full-tree walk, namespace change or production edit was made. The standalone13-line strict-cap regression was removed because its assertion conflicts with the revised boundary.
