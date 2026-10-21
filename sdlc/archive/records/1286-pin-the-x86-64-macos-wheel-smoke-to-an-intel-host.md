---
ticket: 1286
head: 64183a17
merge: (this record's merge)
---

# 1286 — pin the x86_64 macOS wheel smoke to an Intel host

The rehearsal runs failed the x86_64 macOS wheel-smoke leg at install on every attempt from the first (36872650687, 14:30 UTC, both attempts) onward: GitHub's macos-latest label resolved to the macos-26-arm64 image, and pip refuses a macosx_x86_64 wheel on an arm64 host. No mid-day switch occurred, and the record's earlier claim that the leg had installed on an x86_64 host was inference from another leg's log, corrected 2026-10-01. The build legs cross-compile with explicit cargo targets, so their artifacts stay correctly tagged; only the smoke needs a matching host. The fix pins that leg to macos-15-intel; the matrix pin test is updated in the same commit. Code review: ACCEPT 2026-10-01 dispatch 944e8046-1a7c-48ea-9719-6295869163a3. GitHub's runner-images announcements retire the Intel macOS images in August 2027 (macos-15-intel among them); when that lands, this leg moves to Rosetta 2 on an arm64 image. Recorded 2026-10-02 from the sixth review's reading of the runner-images announcements.
