from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[1]


def test_shell_workflow_inventory_has_explanations() -> None:
    policy = json.loads((ROOT / "tools/lint-inventory.json").read_text())
    assert policy["schema"] == "biomcp-shell-workflow-inventory-v1"
    assert policy["shellcheck_exclusions"]
    for exclusion in policy["shellcheck_exclusions"]:
        assert exclusion["prefix"]
        assert exclusion["reason"]


def test_canonical_lint_owns_all_three_shell_workflow_checks() -> None:
    makefile = (ROOT / "Makefile").read_text()
    lint = (ROOT / "bin/lint").read_text()
    checker = (ROOT / "tools/check-shell-workflows").read_text()
    assert "tools/check-shell-workflows" in lint
    for command in ("bash", "shellcheck", "actionlint"):
        assert command in checker
    assert "prove_negative_controls()" in checker
    assert "tools/bootstrap-lint-tools" in makefile


def test_shell_workflow_checker_reports_missing_tools_without_traceback(tmp_path: Path) -> None:
    python = tmp_path / "python3"
    python.symlink_to(Path(os.environ.get("PYTHON", "/usr/bin/python3")))
    result = subprocess.run(
        [str(python), str(ROOT / "tools/check-shell-workflows")],
        cwd=ROOT,
        env=os.environ | {"PATH": str(tmp_path)},
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    assert "Missing required lint tool" in result.stderr
    assert "Run: tools/bootstrap-lint-tools" in result.stderr
    assert "Traceback" not in result.stderr
