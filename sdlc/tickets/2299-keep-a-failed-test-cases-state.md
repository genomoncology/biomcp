---
priority: 2
---

# 2299 — Keep a failed test case's state

Status: OPEN.

Milestone: 0.9.4

## Outcome

When a replay-backed test fails, its case directory, cache database and replay logs survive for diagnosis. tests/golden/replay_env.py borrows a pytest-owned case directory, following the 2285 pattern, so teardown no longer deletes the evidence.

- Proof: A deliberately failing replay test leaves its database and logs under the pytest case directory, and a passing test cleans up as before.

## Evidence

Four lanes (2275, 2283, 2285, 2298) ended with the original cause unknown because fixture teardown deleted the failed case's database before anyone could read it. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.

## Implementation contract

Start from `3819a97abb142b7d57f71eada2e8a4031b9c65a9`, which contains main `91118b5881ccbf4186875e445173b61ec4dbc86d`. Given a replay-backed pytest case, when its assertion fails, pytest owns and retains its command home, cache database and case-specific replay log after the shared server exits. Given a passing case, pytest removes that case directory. Standalone replay owners keep their existing cleanup.

Changes: `tests/golden/replay_env.py` borrows the active pytest case directory and snapshots its request log; `tests/conftest.py` supplies that owner; `pyproject.toml` selects failed-case temporary retention; `tests/test_replay_case_state.py` exercises an actual nested pytest failure and passing cleanup; `CHANGELOG.md` records the internal change. Implementation evidence appends to `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.

Outside-in check: `uv run --no-sync pytest tests/test_replay_case_state.py`. The nested test invokes the prepared CLI against replay, creates a real cache database, then deliberately fails one case and passes another. On unchanged main, the outer check must fail because replay teardown deletes the failed case's command directory. Exact command and observed output will be recorded from the admitted build job before implementation.

## Main reproduction

Before implementation, admitted job `2299/1792523971430-33961` ran the new outside-in check against seed `3819a97ab`, containing main `91118b5881ccbf4186875e445173b61ec4dbc86d`, with the current-main prepared Python and an immutable copy of its CLI. Command: `<prepared-python> -m pytest tests/test_replay_case_state.py --basetemp <evidence>/red-cases -q`. The outer test failed at `tests/test_replay_case_state.py:57`: `AssertionError: failed replay command state was deleted`; summary: `1 failed in 1.76s`. The nested runner had one deliberate assertion failure and one passing case. Replay teardown had deleted the failed `run-1` directory. Raw command, output and CLI identity are retained with the implementation evidence.
