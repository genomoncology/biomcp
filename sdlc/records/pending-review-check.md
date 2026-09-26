---
base: 8cebc07c
head: 7d032d4d
---

The pending-review check and the conflict-marker guard, from the
2026-09-26 review's paperwork section.

A ticket with a record file can no longer carry a stale pending
review line: tests/test_sdlc_review_status_contract.py intersects the
four-digit ticket and record sets and fails naming every pending
line, with a scoped allowance only for slices that name a batch or
item — never an open-ended "future". The matcher accepts the scope in
both house positions (before and after the review kind), after the
first review round caught that the original shape let the real
scoped line pass by regex miss rather than by exemption. Run red
first, the check caught seven stale tickets — the review's six plus
1224, which nobody had listed — then the seven lines were rewritten
from each record's evidence (1249's record gained one honest sentence:
the fix was verified by tests and the gate, not a fresh reviewer
accept). The conflict-marker guard fails on any tracked conflict
marker; a scan of the exact bytes merge 7206240b shipped finds all
three marker lines, so that class of broken merge cannot land
silently again.

Evidence: the red-green run is recorded in the commit; code review
REJECT once (the scoped allowance was a no-op for the real line
shape; a vacuous records test; a dead regex) — all fixed and
re-verified; yellow gate at 7d032d4d — lint, test, and spec OK after
one boundary-count cycle for the new packaged test file (1,363).
