# 2071 — Take slow Python tests off the gate path

Status: OPEN.

Milestone: 0.9.3

## Outcome

The Python lane finishes in about two minutes, no test reads or cleans the developer's own cache, and the cold packaging check runs only in the release gate.

## Evidence

- `make test` takes 635 s, and the Python lane 473 s of it. Its floor is three files: the cold `cargo package` and check test (426 s), a trial search stuck in the HTTP cache's start-up clean because 22 test files set no `BIOMCP_CACHE_DIR` (381 s), and the ratchet self-tests (427 s) (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 3).

- Starts from: `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 3.
- Keeps: every product behavior those tests prove.
- Changes: one shared pytest fixture gives every binary-running test an empty `BIOMCP_CACHE_DIR` in its temporary folder; move the packaging test to `make release-gate`; run the short-socket lib test from the nextest archive; skip the start-up cache clean when the cache is off and run it at most once a day otherwise; run the ratchet self-tests against a small fixture repository, or leave them to 2076.
- Proof: `pytest --durations=20` before and after; `make test` wall time before and after; a test proves a binary run under the fixture never touches the real cache folder.
- Defers: deleting the ratchets (2076).
