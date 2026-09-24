# Make the disease-survival reap test tolerate suite load

Filed from the 2026-09-24 yellow gate failure of ticket 1247's branch
(17ac88cc; the failure is unrelated to that branch's gencc change).
Same family as tickets 1239 and 1247.

## Problem

`tests/test_disease_survival_fixture_lifecycle.py::test_disease_survival_setup_reaps_ppid_one_marker_orphan`
failed once in a full `make test` run under load with "a PPID-1
process with only a similarly named path is not an authenticated
disease-survival fixture", and passed three consecutive focused runs
immediately after on the same host. The test's authentication scan
races the suite's transient processes.

## Design

Read the test and its fixture setup script; identify the scan (PPID-1
processes with similarly named paths) and the race window. Fix by the
1239 pattern: either bound the wait with a generous deadline and
assert with a diagnosable message, or make the fixture's marker
authentication independent of transient sibling processes. Record
which. No production code changes expected.

## Acceptance

- The mechanism is identified and the fix lands with a named pattern.
- Three consecutive full-suite runs on the gate host stay green.

## Review

- Design review: pending
- Code review: pending
