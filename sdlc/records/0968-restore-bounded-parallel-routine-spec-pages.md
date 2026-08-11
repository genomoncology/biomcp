---
base: bcd8ed3b1fe46502501e715c3bb3477f459a4a25
head: 87e1af574753b05034d9bdec993c73a5307582de
---

Routine `spec`, `spec-pr`, and `spec-contracts` now run independent Markdown
pages through a bounded four-worker pool. Article/author and section outcomes
retain explicit serial ownership boundaries; static and live verification
modes are unchanged. Set `BIOMCP_SPEC_WORKERS=1` for serial diagnosis.

Synthetic tests prove the four-worker default, one-worker mode, deterministic
and aggregate failure output, no later batch after failure, invalid-setting
rejection, and interrupt cleanup of each page process group. The complete
runner/fixture/isolation selection passed 106 tests, the 32-test isolation
contract passed three consecutive runs, full lint passed, and two complete
four-worker `make spec` runs passed.

The complete runs took 216.47 and 194.37 seconds after prewarming. Their
205.42-second median is 1.36x faster than the post-0967 serial comparison and
2.88x faster than the 592.1-second intervention baseline. The prewarm took
65.26 seconds solely after Git `HEAD` moved, reinforcing ticket 0970 as the
next intervention target.
