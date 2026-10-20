"""Exercise measurement summaries through the coverage command entry point."""

from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

import pytest

# Fake binaries use built-artifact paths, so the canonical lane owns this harness.
pytestmark = [pytest.mark.needs_binary]

ROOT = Path(__file__).resolve().parents[1]


def executable(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("#!/usr/bin/env bash\nset -eu\n" + body)
    path.chmod(0o755)


@pytest.fixture(scope="session")
def bash() -> str:
    for directory in os.get_exec_path():
        for name in ("bash", "bash5", "bash4"):
            candidate = shutil.which(name, path=directory)
            if candidate is None:
                continue
            try:
                version = subprocess.run(
                    [candidate, "-c", '[[ "${BASH_VERSINFO[0]}" -ge 4 ]]'],
                    check=False,
                    capture_output=True,
                )
            except OSError:
                continue
            if version.returncode == 0:
                return candidate
    pytest.skip("coverage measurement requires Bash 4 or newer on PATH")


@pytest.mark.parametrize(
    ("nextest_summary", "spec_summary", "status", "expected", "incomplete"),
    [
        (
            "Summary [ 12.0s] 4183 tests run: 4183 passed, 2 skipped",
            "=== spec pages: spec/a.md spec/b.md spec/c.md ===",
            "0",
            ("4183 run: 4183 passed", "3 pages, 0 failed"),
            False,
        ),
        (
            "Summary [  24.364s] 1841/4183 tests run: 1840 passed, 1 failed, 29 skipped",
            "=== spec pages: spec/a.md spec/b.md spec/c.md ===",
            "100",
            ("1841 run: 1840 passed, 1 failed", "3 pages, 0 failed"),
            True,
        ),
        (
            "Summary [ 1.0s] 3 tests run: 3 passed",
            "=== spec page: spec/a.md ===\n=== spec page: spec/b.md ===",
            "0",
            ("3 run: 3 passed", "2 pages, 0 failed"),
            False,
        ),
        (
            "Summary [ 1.0s] 3 tests run: 3 passed",
            "=== spec pages: spec/a.md spec/b.md ===\nspec run failed (1)",
            "0",
            ("3 run: 3 passed", "2 pages, 1 failed"),
            True,
        ),
        (
            "Summary [ 1.0s] unknown runner output",
            "=== spec pages: spec/a.md ===",
            "0",
            ("summary unparsed", "1 pages, 0 failed"),
            True,
        ),
        (
            "",
            "=== spec pages: spec/a.md ===",
            "0",
            ("summary unparsed", "1 pages, 0 failed"),
            True,
        ),
        (
            "Summary [ 1.0s] 3 tests run: 3 passed",
            "",
            "0",
            ("3 run: 3 passed", "summary unparsed"),
            True,
        ),
    ],
)
def test_coverage_summary_counts_executed_work(
    tmp_path: Path,
    bash: str,
    nextest_summary: str,
    spec_summary: str,
    status: str,
    expected: tuple[str, str],
    incomplete: bool,
) -> None:
    clone = tmp_path / "clone"
    clone.mkdir()
    fake_home = tmp_path / "home"
    executable(
        fake_home / ".cargo/bin/cargo",
        """case "$*" in
  "llvm-cov report"*) echo 'TOTAL 0 0 0 0 0 0 0 0 72.5%' ;;
  "nextest run"*) printf '%s\\n' "$TEST_NEXTEST_SUMMARY"; if [ "$TEST_SUITE_STATUS" != 0 ]; then echo "coverage command error" >&2; fi; exit "$TEST_SUITE_STATUS" ;;
esac
""",
    )
    executable(
        fake_home / ".local/bin/uv",
        "echo '================ 5 passed in 0.1s ================'\n",
    )
    executable(clone / "tools/run-offline", 'shift; exec "$@"\n')
    executable(
        clone / "scripts/run-specs.sh",
        'printf "%s\\n" "$TEST_SPEC_SUMMARY"\n',
    )
    env = dict(os.environ)
    env.update(
        HOME=str(fake_home),
        TEST_NEXTEST_SUMMARY=nextest_summary,
        TEST_SPEC_SUMMARY=spec_summary,
        TEST_SUITE_STATUS=status,
    )
    proc = subprocess.run(
        [
            bash,
            str(ROOT / "tools/measure-0-9-3-baseline"),
            str(clone),
            "--only",
            "coverage",
        ],
        env=env,
        text=True,
        capture_output=True,
        timeout=20,
    )
    assert proc.returncode == (1 if status != "0" else 0), proc.stdout + proc.stderr
    if status != "0":
        assert "coverage command error" in proc.stderr
    lines = dict(
        line.removeprefix("RESULT ").split(" = ", 1)
        for line in proc.stdout.splitlines()
        if line.startswith("RESULT ")
    )
    assert expected[0] in lines["coverage_unit_tests"], proc.stdout
    assert expected[1] in lines["coverage_spec_lane"], proc.stdout
    if incomplete:
        assert lines["coverage_combined_lines_pct"].startswith("incomplete ("), (
            proc.stdout
        )
    else:
        assert lines["coverage_combined_lines_pct"] == "72.5", proc.stdout


def test_runtime_retains_alternating_isolated_samples(
    tmp_path: Path, bash: str
) -> None:
    """Failed commands retain observations and cache-off cannot erase warm data."""
    import hashlib
    import json
    import statistics

    clone = tmp_path / "clone"
    clone.mkdir()
    fake_home = tmp_path / "home"
    fake_home.mkdir()
    env = dict(os.environ, HOME=str(fake_home))
    executable(
        clone / "target/release/biomcp",
        """python3 - <<'PYFAKE'
import json
import os
from pathlib import Path
cache = Path(os.environ['BIOMCP_CACHE_DIR'])
cache.mkdir(parents=True, exist_ok=True)
(cache / 'retained').write_text('warm data')
print(json.dumps({'sections': [{'entity': 'gwas', 'error': 'provider unavailable'}]}))
raise SystemExit(7 if os.environ.get('BIOMCP_CACHE_MODE') == 'off' else 0)
PYFAKE
""",
    )
    proc = subprocess.run(
        [
            bash,
            str(ROOT / "tools/measure-0-9-3-baseline"),
            str(clone),
            "--only",
            "runtime",
        ],
        env=env,
        text=True,
        capture_output=True,
        timeout=60,
    )
    assert proc.returncode != 0, proc.stdout + proc.stderr
    assert "provider unavailable" in proc.stdout + proc.stderr
    runtime = clone / ".cache/measure/runtime"
    samples = json.loads((runtime / "samples.json").read_text())
    measured = [s for s in samples if s["phase"] == "measure"]
    for case, count in (("get_drug_imatinib", 10), ("search_all_BRAF_melanoma", 5)):
        pairs = [
            s
            for s in measured
            if s["case"] == case and s["mode"] in ("infinite", "off")
        ]
        assert [s["mode"] for s in pairs] == ["infinite", "off"] * count
        warm = {s["cache_dir"] for s in pairs if s["mode"] == "infinite"}
        off = {s["cache_dir"] for s in pairs if s["mode"] == "off"}
        assert len(warm) == 1 and len(off) == count and not warm & off
        assert all((runtime / p / "retained").exists() for p in warm)
        assert all(s["exit_code"] == 7 for s in pairs if s["mode"] == "off")
        for sample in pairs:
            output = (runtime / sample["stdout_file"]).read_bytes()
            assert sample["stdout_sha256"] == hashlib.sha256(output).hexdigest()
            assert (runtime / sample["stderr_file"]).exists()
        if case.startswith("search_all"):
            assert all(s["gwas_errors"] == ["provider unavailable"] for s in pairs)
    assert any(s["mode"] == "default" and "gwas" in s["case"].lower() for s in measured)
    summary = json.loads((runtime / "summary.json").read_text())
    values = [
        s["elapsed_s"]
        for s in measured
        if s["case"] == "get_drug_imatinib" and s["mode"] == "infinite"
    ]
    assert len(values) == 10
    assert summary["warm_get_drug_imatinib"]["median_s"] == statistics.median(values)


def test_build_measurement_fallback_is_executable(tmp_path: Path, bash: str) -> None:
    """Missing GNU time never turns the build command into shell syntax."""
    clone = tmp_path / "clone"
    clone.mkdir()
    fake_home = tmp_path / "home"
    executable(clone / "tools/with-build-identity", 'exec "$@"\n')
    executable(fake_home / ".cargo/bin/cargo", "exit 0\n")
    env = dict(os.environ, HOME=str(fake_home))
    proc = subprocess.run(
        [
            bash,
            str(ROOT / "tools/measure-0-9-3-baseline"),
            str(clone),
            "--only",
            "cold-dev-build",
        ],
        env=env,
        text=True,
        capture_output=True,
        timeout=10,
    )
    assert proc.returncode == 0, proc.stdout + proc.stderr


@pytest.mark.parametrize("status", [0, 42])
def test_census_retains_the_runner_exit_status(
    tmp_path: Path, bash: str, status: int
) -> None:
    """A failed census retains its samples without claiming complete collection."""
    clone = tmp_path / "clone"
    clone.mkdir()
    fake_home = tmp_path / "home"
    executable(clone / "tools/run-offline", 'shift; exec "$@"\n')
    executable(
        fake_home / ".cargo/bin/cargo",
        "printf '%s\\n' "
        '\'{"type":"test","event":"ok","name":"sample","exec_time":1.5}\'\n'
        'if [ "$CENSUS_EXIT" != 0 ]; then echo "census command error" >&2; fi\n'
        'exit "$CENSUS_EXIT"\n',
    )
    env = dict(os.environ, HOME=str(fake_home), CENSUS_EXIT=str(status))
    proc = subprocess.run(
        [
            bash,
            str(ROOT / "tools/measure-0-9-3-baseline"),
            str(clone),
            "--only",
            "rust-test-census",
        ],
        env=env,
        text=True,
        capture_output=True,
        timeout=10,
    )
    assert proc.returncode == (0 if status == 0 else 1), proc.stdout + proc.stderr
    if status != 0:
        assert "census command error" in proc.stderr
    assert (clone / ".cache/measure/census.exit").read_text().strip() == str(status)
    expected = "complete" if status == 0 else "incomplete (exit 42)"
    assert f"RESULT rust_test_census_collection = {expected}" in proc.stdout
    assert (clone / ".cache/measure/census.jsonl").read_text()
    assert "RESULT rust_tests_with_duration = 1" in proc.stdout


@pytest.mark.parametrize("retained", [False, True])
def test_checklist_rows_report_values_or_explicit_unavailability(
    tmp_path: Path, bash: str, retained: bool
) -> None:
    clone = tmp_path / "clone"
    (clone / "tests").mkdir(parents=True)
    (clone / "src/render/markdown").mkdir(parents=True)
    (clone / "src/render/markdown/gene.rs").write_text("// fixture\n")
    (clone / "target").mkdir()
    (clone / "target/built").write_bytes(b"built output")
    (clone / "tests/test_runtime.py").write_text(
        "import pytest\n@pytest.mark.parametrize('x', [1, 2])\n"
        "def test_card(x): pass\nclass TestCard:\n"
        "    async def test_other(self): pass\n"
    )
    (clone / "tests/test_tool.py").write_text(
        "TOOL = 'tools/measure-0-9-3-baseline'\n"
        "def test_output(): pass\ndef helper(): pass\n"
    )
    home = tmp_path / "home"
    executable(
        home / ".cargo/bin/cargo", "echo 'Summary [0.1s] 2 tests run: 2 passed'\n"
    )
    executable(home / ".local/bin/uv", "echo '4 tests collected in 0.01s'\n")
    executable(clone / "tools/with-build-identity", 'exec "$@"\n')
    executable(clone / "tools/run-offline", 'shift; exec "$@"\n')
    args = []
    if retained:
        log = tmp_path / "linux.log"
        log.write_text(
            "   Compiling example v1.0.0\n   Compiling example v1.0.0\n"
            "^[[1m   Compiling^[[0m biomcp-cli v0.9.3\n"
            "\x1b[1m    Finished\x1b[0m `release` profile [optimized] target(s) in 1.0s\n"
        )
        args += ["--target-build-log", f"x86_64-unknown-linux-gnu={log}"]
        incomplete = tmp_path / "windows.log"
        incomplete.write_text("   Compiling example v1.0.0\nerror: stopped\n")
        args += ["--target-build-log", f"x86_64-pc-windows-msvc={incomplete}"]
        args += ["--release-record", str(ROOT / "sdlc/records/2215-0-9-3-release.md")]
    proc = subprocess.run(
        [
            bash if retained else "/bin/bash",
            str(ROOT / "tools/measure-0-9-3-baseline"),
            str(clone),
            "--only",
            "working-checkout,release-target-counts,python-test-functions,python-test-count,edit-test-group,release-history",
            *args,
        ],
        env=dict(os.environ, HOME=str(home)),
        text=True,
        capture_output=True,
        timeout=20,
    )
    assert proc.returncode == 0, proc.stdout + proc.stderr
    lines = dict(
        line.removeprefix("RESULT ").split(" = ", 1)
        for line in proc.stdout.splitlines()
        if line.startswith("RESULT ")
    )
    assert int(lines["working_checkout_build_folder_bytes"]) >= 12
    assert lines["python_test_functions"] == "3"
    assert lines["python_tests_collected"] == "4"
    assert lines["python_tooling_test_functions"] == "1"
    assert lines["python_tooling_test_share_pct"] == "33.33"
    inventory = (clone / ".cache/measure/python-test-functions.tsv").read_text()
    assert "tests/test_tool.py\ttooling\t1" in inventory
    assert "tests/test_runtime.py\tother\t2" in inventory
    assert float(lines["edit_test_group_wall_s"]) >= 0
    assert lines["edit_test_group_exit"] == "0"
    assert lines["crates_compiled_windows_release"].startswith("unavailable (")
    if retained:
        assert lines["crates_compiled_linux_release"] == "2"
        assert lines["release_workflow_wall_s"] == "2919"
        assert lines["full_rehearsals_count"] == "8"
        assert "release_archive_bytes_biomcp-linux-x86_64.tar.gz" in lines
    else:
        assert lines["crates_compiled_linux_release"].startswith("unavailable (")
        assert lines["release_workflow_wall_s"].startswith("unavailable (")
        assert lines["release_archive_bytes"].startswith("unavailable (")
        assert lines["full_rehearsals_count"].startswith("unavailable (")
