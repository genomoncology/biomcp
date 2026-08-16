---
flow: quickfix
priority: 9
---

# Remove cold-start timing from the Windows epoch test

Exact-head CI run 31935518546 compiled BioMCP and reached the Windows managed-state suite, but `windows_cache_epoch_files_are_user_only_and_reject_hard_links` failed before inspecting either ACL. Its local MyGene fixture gives the newly built process only about two seconds to connect (`200` nonblocking polls separated by `10 ms`). A cold hosted Windows process did not connect inside that window, the fixture exited, and the command exhausted its retries with `HTTP request to MyGene.info failed`. The test therefore depends on process startup speed even though it is intended to prove cache epoch permissions.

Keep the production cache code, Windows ACL assertion, real cached fast-path repair, and hard-link rejection unchanged. In `tests/managed_state_permissions.rs`, make the local fixture remain available until it receives the child request, subject to a 30-second monotonic deadline, and do not contact a public provider. A timeout or malformed request must be returned explicitly to the test rather than hidden behind a successful fixture-thread join and a later generic CLI failure. Accept only `GET /query` with BRAF in the query and `species=human`, `size=1`, and `from=0`; query parameter order must not matter. Do not use an arbitrary unconditional sleep in the normal probe.

Use two explicit probe modes. The initial and repaired probes must each finish successfully and require exactly one valid MyGene request before the deadline. The hard-link probe must fail specifically with `managed file has 2 links` and no `fsutil` text, require zero fixture contact, and stop the waiting fixture when the child exits rather than waiting out the deadline. Any request in the hard-link mode is a failure because cache validation must reject the linked marker before provider contact.

Add a focused fixture-only Windows regression whose client connects after at least 2.5 seconds, receives the valid response, and proves the expected request was observed. This delay belongs only to that focused regression, not the normal probes, so shortening the fixture back to the former two-second timing fails red without slowing the production-shaped ACL test. The exact hosted `windows-contracts` job, focused local checks, `make lint`, and `git diff --check` must pass.
