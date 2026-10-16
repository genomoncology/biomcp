# 2126 — Find why an instant article search spends its whole deadline

Status: OPEN.

Milestone: 0.9.3

## Outcome

The article-query contract test that 2102 bounded at 600 s answers in a few seconds under the product's own 60 s deadline, because the cause of the wait is found and fixed, and its budget returns to the product deadline.

## Evidence

Found by the 0.9.3 adversarial review on 2026-10-16, against 2102 (landed 16f0a1320) and 2119.

- The test sends one Semantic Scholar search, limit 1, to a loopback fixture that answers at once, and took 60.198 s on a quiet host: the full 60 s article-search deadline.
- 2102 raised the budget tenfold to 600 s (`tests/rmcp_client_contract.rs:1179-1188`) without finding why the search uses the whole deadline. 2119 then proposed raising nextest's 240 s limit for it too.
- 2103 found the shared cause for the sibling tests, child processes waiting on the user's default cache, and gave them fresh cache folders; this test never got one.
- A search that runs to its deadline against an instant provider is a product failure a clinician would feel.

- Starts from: the test above, 2103's fix, the article search fan-out and its per-source deadlines.
- Decisions:
  - Find the waiting step first and record it in this ticket. If it is test isolation, give the test the same fresh cache 2103 gave its siblings. If it is the product, fix the product path so an instant provider answers instantly.
  - Restore the test's budget to the product deadline; do not raise any test runner limit.
- Changes: claims `tests/rmcp_client_contract.rs` and the source files the cause names; the lane adds them to this ticket before its first push. `CHANGELOG.md` if the product changes.
- Proof: the test passes under its product deadline in a few seconds, alone and beside a second closing suite on the Linux gate host; the cause and its evidence are recorded here.
- Order: replaces 2119, which is withdrawn.
