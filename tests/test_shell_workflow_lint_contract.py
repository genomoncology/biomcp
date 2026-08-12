from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def test_shell_workflow_inventory_has_explanations() -> None:
    policy = json.loads((ROOT / "tools/lint-inventory.json").read_text())
    assert policy["schema"] == "biomcp-shell-workflow-inventory-v1"
    assert policy["shellcheck_exclusions"]
    for exclusion in policy["shellcheck_exclusions"]:
        assert exclusion["prefix"]
        assert exclusion["reason"]


def test_canonical_lint_owns_all_three_shell_workflow_checks() -> None:
    lint = (ROOT / "bin/lint").read_text()
    checker = (ROOT / "tools/check-shell-workflows").read_text()
    assert "tools/check-shell-workflows" in lint
    for command in ("bash", "shellcheck", "actionlint"):
        assert command in checker
    assert "prove_negative_controls()" in checker
