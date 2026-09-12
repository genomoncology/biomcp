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
