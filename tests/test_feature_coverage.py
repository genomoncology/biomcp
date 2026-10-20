"""Exercise the feature gate as callers invoke it, with small source trees."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
GATE = ROOT / "tools/check-feature-coverage.py"


def tree(tmp_path: Path) -> Path:
    (tmp_path / "sdlc").mkdir()
    (tmp_path / "tests").mkdir()
    (tmp_path / "src").mkdir()
    (tmp_path / "sdlc/features.md").write_text("- gene: Find a gene.\n")
    (tmp_path / "tests/test_gene.py").write_text("def test_lookup(): pass\n")
    (tmp_path / "tests/feature-links.json").write_text(json.dumps({
        "python": {"tests/test_gene.py": {"feature": "gene"}},
        "rust": {}, "spec": {}, "internal": {},
    }))
    return tmp_path


def run(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run([sys.executable, str(GATE), "--root", str(root), *args],
                          capture_output=True, text=True)


def test_every_outside_case_names_a_known_feature(tmp_path: Path) -> None:
    root = tree(tmp_path)
    assert run(root, "links").returncode == 0
    (root / "sdlc/features.md").write_text("- drug: Find a drug.\n")
    rejected = run(root, "links")
    assert rejected.returncode != 0
    assert "unknown feature gene" in rejected.stderr
    assert "drug has no outside-in test" in rejected.stderr


def test_unclassified_tests_and_missing_selectors_fail(tmp_path: Path) -> None:
    root = tree(tmp_path)
    (root / "tests/test_unclassified.py").write_text("def test_new(): pass\n")
    assert "unclassified test module" in run(root, "links").stderr
    (root / "tests/test_unclassified.py").unlink()
    (root / "tests/test_gene.py").write_text("# former tests\n")
    assert "no test" in run(root, "links").stderr


@pytest.mark.parametrize("hits,exit_code", [(1, 0), (0, 1)])
def test_only_product_lines_count_toward_the_floor(
    tmp_path: Path, hits: int, exit_code: int,
) -> None:
    root = tree(tmp_path)
    (root / "src/main.rs").write_text(
        'fn main() {}\n#[cfg(test)]\nmod tests {\n #[test] fn unit() {}\n}\n'
    )
    report = root / "outside.lcov"
    report.write_text(
        f"SF:{root}/src/main.rs\nDA:1,{hits}\nDA:4,1\nend_of_record\n"
        f"SF:{root}/tests/test.rs\nDA:1,1\nend_of_record\n"
    )
    status = root / "suites.json"
    status.write_text(json.dumps({"python": 0, "rust": 0, "spec": 0}))
    result = run(root, "report", "--lcov", str(report), "--statuses", str(status),
                 "--floor", "50")
    assert result.returncode == exit_code, result.stderr
    data = json.loads(result.stdout)
    assert data["product_lines"] == 1
    assert data["reached_lines"] == hits
    assert data["outside_in_percent"] == 100 * hits


def test_failed_or_missing_collections_never_report_a_percentage(tmp_path: Path) -> None:
    root = tree(tmp_path)
    report = root / "outside.lcov"
    report.write_text(f"SF:{root}/src/main.rs\nDA:1,1\nend_of_record\n")
    (root / "src/main.rs").write_text("fn main() {}\n")
    status = root / "suites.json"
    status.write_text(json.dumps({"python": 1, "rust": 0}))
    result = run(root, "report", "--lcov", str(report), "--statuses", str(status),
                 "--floor", "50")
    assert result.returncode != 0
    data = json.loads(result.stdout)
    assert data["status"] == "incomplete"
    assert "outside_in_percent" not in data
