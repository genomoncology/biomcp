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


def test_golden_links_follow_existing_command_metadata(tmp_path: Path) -> None:
    root = tree(tmp_path)
    golden = root / "tests/golden/test_golden_output.py"
    golden.parent.mkdir()
    golden.write_text(
        'CASES = (Case("lookup", ("get", "gene", "BRAF")),)\n'
        'def test_golden_output(): pass\n'
    )
    path = root / "tests/feature-links.json"
    config = json.loads(path.read_text())
    config["python"]["tests/golden/test_golden_output.py"] = {"feature": "@golden"}
    path.write_text(json.dumps(config))
    assert run(root, "links").returncode == 0
    caller = root / "tests/test_spec_case.py"
    caller.write_text('def test_case(): return _run(replay, "lookup")\n')
    config["python"]["tests/test_spec_case.py"] = {"feature": "@golden-calls"}
    path.write_text(json.dumps(config))
    assert run(root, "links").returncode == 0
    golden.write_text(golden.read_text().replace('"gene"', '"drug"'))
    rejected = run(root, "links")
    assert rejected.returncode != 0
    assert "unknown feature drug" in rejected.stderr


def test_failed_build_replaces_an_old_success_with_incomplete_collection(tmp_path: Path) -> None:
    import os
    import shutil

    root = tree(tmp_path)
    tools = root / "tools"
    tools.mkdir()
    for name in ("collect_feature_coverage.py", "feature_coverage.py"):
        shutil.copyfile(ROOT / "tools" / name, tools / name)
    (root / "pyproject.toml").write_text('[tool.biomcp.feature-coverage]\nllvm-cov-version = "0.9.1"\n')
    workflow = root / ".github/workflows/ci.yml"
    workflow.parent.mkdir(parents=True)
    workflow.write_text("  MUSTMATCH_VERSION: 0.1.0\n")
    output = root / ".cache/feature-coverage"
    output.mkdir(parents=True)
    (output / "coverage.json").write_text('{"status":"pass","outside_in_percent":100}')
    commands = root / "commands"
    commands.mkdir()
    tooling = output / "tools/bin"
    tooling.mkdir(parents=True)
    (tooling / "cargo-llvm-cov").write_text("#!/bin/sh\necho cargo-llvm-cov 0.9.1\n")
    (tooling / "cargo-llvm-cov").chmod(0o755)
    for name in ("rustup", "uv"):
        executable = commands / name
        executable.write_text("#!/bin/sh\nexit 0\n")
        executable.chmod(0o755)
    make = commands / "make"
    make.write_text("#!/bin/sh\nexit 0\n")
    make.chmod(0o755)
    cargo = commands / "cargo"
    cargo.write_text(
        '#!/bin/sh\ncase "$1 $2" in\n'
        '"llvm-cov clean") exit 0;;\n'
        '"llvm-cov show-env") echo \'RUSTFLAGS="--cfg instrumented"\'; exit 0;;\n'
        'build*) exit 17;;\n'
        '*) exit 90;;\nesac\n'
    )
    cargo.chmod(0o755)
    result = subprocess.run(
        [sys.executable, str(tools / "collect_feature_coverage.py"), "--baseline"],
        env=os.environ | {"PATH": str(commands) + os.pathsep + os.environ["PATH"]},
        text=True, capture_output=True,
    )
    assert result.returncode != 0
    assert "product-build: exit 17" in result.stdout
    data = json.loads((output / "coverage.json").read_text())
    assert data["status"] == "incomplete"
    assert "outside_in_percent" not in data


def test_a_new_executable_spec_requires_a_feature_owner(tmp_path: Path) -> None:
    root = tree(tmp_path)
    page = root / "spec/entity/new.md"
    page.parent.mkdir(parents=True)
    page.write_text("# A new contract\n\n```bash\nbiomcp list\n```\n")
    rejected = run(root, "links")
    assert rejected.returncode != 0
    assert "unclassified test module spec/entity/new.md" in rejected.stderr
