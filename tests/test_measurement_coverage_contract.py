from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]


def executable(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    path.chmod(0o755)


@pytest.mark.parametrize("failed_python", [False, True])
def test_coverage_scopes_compiler_environment_and_marks_failed_collections(
    tmp_path, failed_python
):
    repo = tmp_path / "repo"
    repo.mkdir()
    (repo / ".cache").mkdir()
    tool_bin = tmp_path / "bin"
    executable(
        tool_bin / "cargo",
        """#!/usr/bin/env bash
set -eu
case "$*" in
  "llvm-cov clean --workspace") ;;
  "llvm-cov show-env")
    printf '%s\\n' 'RUSTFLAGS="--cfg instrumented"' 'RUSTC_WRAPPER="coverage-wrapper"' 'CARGO_LLVM_COV="1"' 'LLVM_PROFILE_FILE="build-%p.profraw"'
    ;;
  "build "*|"nextest "*)
    test "$RUSTC_WRAPPER" = coverage-wrapper
    if [[ "$1" = nextest ]]; then printf 'Summary [0.1s] 1 tests run: 1 passed, 0 skipped\\n'; fi
    ;;
  "llvm-cov report "*) printf 'TOTAL 1 0 100%% 1 0 100%% 1 0 100%%\\n' ;;
  *) printf 'unexpected cargo: %s\\n' "$*" >&2; exit 90 ;;
esac
""",
    )
    check_env = """test "${RUSTFLAGS-unset}" = '--cfg normal_linker'
test "${RUSTC_WRAPPER-unset}" = normal-cache-wrapper
test -z "${CARGO_LLVM_COV+x}"
test "${CARGO_TARGET_DIR-unset}" = normal-target
test "${CARGO_INCREMENTAL-unset}" = 1
[[ "$LLVM_PROFILE_FILE" = /*/target/llvm-cov-target/out-%8m.profraw ]]
"""
    executable(
        tool_bin / "uv",
        "#!/usr/bin/env bash\nset -eu\n"
        + check_env
        + """if [[ "$COVERAGE_FAIL_PYTHON" = 1 ]]; then
printf '===== 1 failed, 1 passed in 0.1s =====\\n'
exit 1
fi
printf '===== 2 passed in 0.1s =====\\n'
""",
    )
    executable(repo / "tools/run-offline", '#!/usr/bin/env bash\nshift\nexec "$@"\n')
    executable(
        repo / "scripts/run-specs.sh",
        "#!/usr/bin/env bash\nset -eu\n"
        + check_env
        + "printf '=== spec page: example\\n1 passed\\n'\n",
    )
    env = os.environ.copy()
    env.update(
        PATH=f"{tool_bin}{os.pathsep}{env['PATH']}",
        RUSTFLAGS="--cfg normal_linker",
        RUSTC_WRAPPER="normal-cache-wrapper",
        CARGO_TARGET_DIR="normal-target",
        CARGO_INCREMENTAL="1",
        COVERAGE_FAIL_PYTHON=str(int(failed_python)),
    )
    bash = shutil.which("bash5") or shutil.which("bash")
    result = subprocess.run(
        [
            bash,
            str(ROOT / "tools/measure-0-9-3-baseline"),
            str(repo),
            "--only",
            "coverage",
        ],
        env=env,
        text=True,
        capture_output=True,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    assert (
        "RESULT coverage_integration_tests = exit 0; 1 run: 1 passed" in result.stdout
    )
    assert "RESULT coverage_unit_tests = exit 0; 1 run: 1 passed" in result.stdout
    assert "RESULT coverage_spec_lane = exit 0; 1 pages, 0 failed" in result.stdout
    assert "RESULT coverage_unit_lines_pct = 100" in result.stdout
    if failed_python:
        assert (
            "RESULT coverage_python_lane = exit 1; 1 passed, 1 failed" in result.stdout
        )
        for group in ("outside_in", "combined"):
            assert (
                f"RESULT coverage_{group}_lines_pct = incomplete (python_lane: 1 failed)"
                in result.stdout
            )
    else:
        assert (
            "RESULT coverage_python_lane = exit 0; 2 passed, 0 failed" in result.stdout
        )
        for group in ("outside_in", "combined"):
            assert f"RESULT coverage_{group}_lines_pct = 100" in result.stdout
