---
flow: build
priority: 19
---
# The survival fixture stops reading /proc after the process exits

`tests/test_disease_survival_fixture_lifecycle.py` inspects a
child process's `/proc` entry after the process has already exited.
Whether the entry still exists at that moment is a race against the
kernel reaping the process, so the test passes or fails on timing
that has nothing to do with the behavior under test. On 2026-08-20
it failed inside an unrelated attempt and refused ticket 1027 —
the refusal was correct, the test was the defect.

Done, observably: the lifecycle test proves what it means to prove
without ever depending on a `/proc` entry outliving its process —
either by collecting what it needs while the process is verifiably
alive, or by asserting on evidence that survives exit. The suite
then passes repeatedly under retry (the repository's own test gate
run in a loop is the witness), and no assertion in the file can be
made to fail by the process being reaped quickly.

The behavior being replaced is only the fixture's read-after-exit
timing; what the test guards — the fixture's lifecycle contract —
must still be guarded just as strongly afterward. Ticket 1027 is
amended to wait on this one via `deps`, so it re-flies against a
deterministic suite.
