# Spec fixture lifecycle runner tests time out under host pressure

Observed 2026-09-12 during full `make test` gate runs on both hosts.

The runner-termination lifecycle tests spawn `bash scripts/run-specs.sh` with
a 10-second subprocess timeout and assert cleanup after signaling:

- `tests/test_article_spec_fixture_lifecycle.py::test_runner_signal_cleans_article_fixture`
- `tests/test_article_spec_fixture_lifecycle.py::test_interrupted_routine_fixture_owns_a_separate_process_group_and_reruns`
- `tests/test_ctgov_spec_fixture_lifecycle.py::test_runner_termination_cleans_ctgov_process_group_env_and_port`
- `tests/test_disease_survival_fixture_lifecycle.py::test_real_bounded_runner_timeout_reaps_disease_server_and_root`

All fail with `subprocess.TimeoutExpired` at the 10-second budget on the
4-core gate host (9 of them per run) and on the 16-core dev host (4 of them
per run at 93 percent disk usage, 7 percent free). Evidence they are
independent of the 1187/1188/1189 changes: the pass/fail set is byte-identical
between yellow runs at 857f5deb (before 1189) and 34dc58c1 (after), and the
dev host's last fully green `make test` on main was verified 2026-08-27/28
(release-0.9-runbook Phase 0) on a main that has since changed by
documentation commits only, while its disk filled from about 10 percent free
to 7 percent free during 2026-09-12 build work.

Worth considering: whether the 10-second budget reflects a realistic runner
startup under fixture standup on loaded or disk-pressed hosts, whether the
timeout should scale with host capability, and whether the runner's fixture
standup slowed for an environmental reason worth finding on its own. These
tests are the only remaining red in an otherwise fully passing `make test`
on the gate host.

Root causes identified 2026-09-12 by live capture of stuck runners during
reproduction:

1. Deterministic, gate-host-only: Ubuntu 25.10's uutils coreutils 0.2.2
   `timeout --signal=KILL` never delivers the kill (`timeout --signal=KILL
   2s sleep 5` runs the full five seconds and exits 124 there; GNU kills at
   two seconds, exit 137). The disease-survival bounded-runner test wraps its
   runner in this construct, so the wrapper never fires on the gate host and
   the runner bash orphans in the hold loop indefinitely. Fixed in ticket
   1190 amendment by enforcing the three-second bound in Python with a direct
   SIGKILL of the Popen pid. Recorded here per policy; no external issue
   filed against uutils.
2. Intermittent, both hosts: CPU-starvation sensitivity of the fork-heavy
   EXIT-trap teardown. All affected tests pass solo on both hosts, pass the
   full trio on an idle gate host, and passed the complete pytest lane on an
   idle gate host. Every captured runner waits in `do_wait` on a one-second
   sleep; no locks or D-states are involved. Mitigated by ticket 1190's
   sixty-second waits plus the operational rule that the gate host runs
   nothing else during gates.
