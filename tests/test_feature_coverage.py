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
        "accounting_cases": {"gene": "tests/test_gene.py::test_lookup"},
        "source_features": {r"^src/": "gene"},
    }))
    return tmp_path


def run(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run([sys.executable, str(GATE), "--root", str(root), *args],
                          capture_output=True, text=True)



def collector_command(tools: Path, platform: str, *args: str) -> list[str]:
    # Vary only the collector's platform dependency. Changing sys.platform
    # globally also changes lazy stdlib imports into unsupported host modules.
    program = ("import runpy,sys,types; sys.path.insert(0," + repr(str(tools)) + "); "
               "module=runpy.run_path(" + repr(str(tools / "collect_feature_coverage.py")) + "); "
               "module['collect'].__globals__['sys']=types.SimpleNamespace(platform=" + repr(platform) + "); "
               "sys.argv=" + repr(["collect_feature_coverage.py", *args]) + "; "
               "raise SystemExit(module['main']())")
    return [sys.executable, "-c", program]


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
def test_only_product_regions_count_toward_the_floor(
    tmp_path: Path, hits: int, exit_code: int,
) -> None:
    root = tree(tmp_path)
    (root / "src/main.rs").write_text(
        'fn main() {}\n#[cfg(test)]\nmod tests {\n #[test] fn unit() {}\n}\n'
    )
    for name in ("observed_tests", "unobserved_tests"):
        directory = root / "src" / name
        directory.mkdir()
        (directory / "mod.rs").write_text("#[test] fn internal() {}\n")
    explicit_test = root / "src/consumer_only.rs"
    explicit_test.write_text("fn only_a_test_helper() {}\n")
    metadata = root / "tests/feature-links.json"
    config = json.loads(metadata.read_text())
    config["internal"]["src/consumer_only.rs"] = "Imported only by a cfg(test) module."
    metadata.write_text(json.dumps(config))
    report = root / "outside.json"
    report.write_text(json.dumps({"type": "llvm.coverage.json.export", "version": "3.1.0", "data": [{
        "functions": [{"filenames": [str(root / path)], "regions": regions}
                      for path, regions in [
                          ("src/main.rs", [[1, 1, 1, 13, hits, 0, 0, 0], [4, 1, 4, 24, 1, 0, 0, 0]]),
                          ("tests/test.rs", [[1, 1, 1, 13, 1, 0, 0, 0]]),
                          ("src/observed_tests/mod.rs", [[1, 1, 1, 13, 1, 0, 0, 0]]),
                          ("src/consumer_only.rs", [[1, 1, 1, 13, 1, 0, 0, 0]]),
                      ]]}]}))
    (root / "python.xml").write_text('<testsuites><testsuite><testcase classname="tests.test_gene" name="test_lookup" /></testsuite></testsuites>')
    status = root / "suites.json"
    status.write_text(json.dumps({"python": 0, "rust": 0, "spec": 0}))
    result = run(root, "report", "--json", str(report), "--statuses", str(status),
                 "--floor", "50")
    assert result.returncode == exit_code, result.stderr
    data = json.loads(result.stdout)
    assert data["product_regions"] == 1
    assert data["reached_regions"] == hits
    assert data["outside_in_percent"] == 100 * hits
    assert not data["unobserved_files"]
    if not hits:
        assert data["item_accounting"]["src/main.rs"]["case"] == "tests/test_gene.py::test_lookup"
        assert data["item_accounting"]["src/main.rs"]["case_execution"] == "Python suite: passed"


def test_failed_or_missing_collections_never_report_a_percentage(tmp_path: Path) -> None:
    root = tree(tmp_path)
    report = root / "outside.json"
    report.write_text("{}")
    (root / "src/main.rs").write_text("fn main() {}\n")
    status = root / "suites.json"
    status.write_text(json.dumps({"python": 1, "rust": 0}))
    result = run(root, "report", "--json", str(report), "--statuses", str(status),
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


@pytest.mark.parametrize("encoded", [False, True])
def test_failed_build_replaces_an_old_success_with_incomplete_collection(
    tmp_path: Path, encoded: bool,
) -> None:
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
    (tooling / "cargo-llvm-cov").write_text('#!/bin/sh\n[ "$*" = "llvm-cov --version" ] || exit 88\necho cargo-llvm-cov 0.9.1\n')
    (tooling / "cargo-llvm-cov").chmod(0o755)
    for name in ("rustup", "uv"):
        executable = commands / name
        executable.write_text("#!/bin/sh\nexit 0\n")
        executable.chmod(0o755)
    make = commands / "make"
    make.write_text("#!/bin/sh\nexit 0\n")
    make.chmod(0o755)
    cargo = commands / "cargo"
    flag_key = "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS" if encoded else "RUSTFLAGS"
    separator = "\x1f" if encoded else " "
    instrumentation = separator.join(("-C", "instrument-coverage", "--cfg=coverage"))
    owned = output / "target"
    owned.mkdir()
    (owned / "stale.profraw").touch()
    ordinary_artifact = root / "target/debug/ordinary-sentinel"
    ordinary_artifact.parent.mkdir(parents=True)
    ordinary_artifact.touch()
    cargo.write_text(
        '#!/bin/sh\ncase "$1 $2" in\n'
        '"llvm-cov clean") [ "$CARGO_TARGET_DIR" = "$CARGO_LLVM_COV_TARGET_DIR" ] || exit 89; rm -f "$CARGO_LLVM_COV_TARGET_DIR"/*.profraw; exit 0;;\n'
        f'"llvm-cov show-env") printf \'%s\\n\' "{flag_key}=\'{instrumentation}\'"; exit 0;;\n'
        f'build*) printf "%s\\n%s\\n" "${{{flag_key}}}" "$LLVM_PROFILE_FILE" > profile-env.txt; exit 17;;\n'
        '*) exit 90;;\nesac\n'
    )
    cargo.chmod(0o755)
    result = subprocess.run(
        collector_command(tools, "linux", "--baseline"),
        env=os.environ | {"PATH": str(commands) + os.pathsep + os.environ["PATH"]},
        text=True, capture_output=True,
    )
    assert not (owned / "stale.profraw").exists()
    assert ordinary_artifact.exists()
    assert result.returncode != 0
    assert "product-build: exit 17" in result.stdout
    data = json.loads((output / "coverage.json").read_text())
    assert data["status"] == "incomplete"
    assert "outside_in_percent" not in data
    flags, pattern = (root / "profile-env.txt").read_text().splitlines()
    assert instrumentation in flags
    assert "-Cllvm-args=-runtime-counter-relocation" in flags
    assert Path(pattern).is_absolute()
    assert pattern == str(root / ".cache/feature-coverage/target/outside-%c%p.profraw")
    # Even errors before compilation must not leave yesterday's passing report.
    (output / "coverage.json").write_text('{"status":"pass","outside_in_percent":100}')
    (root / "tests/test_unknown.py").write_text("def test_new(): pass\n")
    rejected = subprocess.run(
        collector_command(tools, "linux", "--baseline"),
        text=True, capture_output=True,
    )
    assert rejected.returncode != 0
    assert "unclassified test module" in rejected.stdout
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


def test_nested_rust_consumers_keep_qualified_feature_ownership(tmp_path: Path) -> None:
    root = tree(tmp_path)
    target = root / "tests/consumer.rs"
    target.write_text('#[path = "cases/child.rs"]\nmod child;\n#[test]\nfn direct() {}\n')
    child = root / "tests/cases/child.rs"
    child.parent.mkdir()
    child.write_text('#[test]\nfn nested() {}\n// #[test] fn not_a_test() {}\n')
    config_path = root / "tests/feature-links.json"
    config = json.loads(config_path.read_text())
    config["rust"]["tests/consumer.rs"] = {"feature": "gene", "tests": {"child::nested": "drug"}}
    config_path.write_text(json.dumps(config))
    rejected = run(root, "links")
    assert "unknown feature drug" in rejected.stderr
    assert "missing test" not in rejected.stderr
    (root / "sdlc/features.md").write_text("- gene: Find a gene.\n- drug: Find a drug.\n")
    assert run(root, "links").returncode == 0


@pytest.mark.parametrize("goal", ["release-gate-markers", "release-gate"])
def test_release_entry_points_collect_coverage_once(tmp_path: Path, goal: str) -> None:
    """Run both real recipes with only their costly external work substituted."""
    for directory in ("tools", "scripts"):
        (tmp_path / directory).mkdir()
    offline = tmp_path / "tools/run-offline"
    offline.write_text("#!/bin/sh\nprintf 'marker-check\\n'\n")
    offline.chmod(0o755)
    (tmp_path / "scripts/collect-feature-coverage.sh").write_text(
        '#!/bin/sh\nprintf "coverage:%s:%s\\n" "$CARGO_BUILD_JOBS" "$NEXTEST_TEST_THREADS"\n'
    )
    overlay = tmp_path / "cheap.mk"
    overlay.write_text("gate-preflight lint test test-long prepare-test prepare-routine-test-tmp repo-audit sync-python-dev full-feature-check spec:\n\t@:\n")
    command = ["make", "--no-print-directory", "-f", str(ROOT / "Makefile"), "-f", str(overlay)]
    result = subprocess.run([*command, goal, "TEST_LANE_WORKERS=2", "MAKE=" + " ".join(command)],
                            cwd=tmp_path, text=True, capture_output=True)
    assert result.returncode == 0, result.stdout + result.stderr
    assert result.stdout.splitlines().count("coverage:2:2") == 1
    assert result.stdout.splitlines().count("marker-check") == 1


def test_lint_rejects_unclassified_tests_before_preparing_python(tmp_path: Path) -> None:
    import os
    import shutil

    root = tree(tmp_path)
    (root / "tests/test_unclassified.py").write_text("def test_new(): pass\n")
    tools = root / "tools"
    tools.mkdir()
    for name in ("check-feature-coverage.py", "feature_coverage.py"):
        shutil.copy2(ROOT / "tools" / name, tools / name)
    shutil.copy2(ROOT / "Makefile", root / "Makefile")
    overlay = root / "fixture.mk"
    overlay.write_text("gate-preflight:\n\t@:\n")
    uv = tools / "uv"
    uv.write_text(
        "#!/bin/sh\n"
        "touch prepared\nexit 99\n"
    )
    uv.chmod(0o755)
    env = {key: value for key, value in os.environ.items()
           if key not in ("MAKEFLAGS", "MAKEOVERRIDES", "MAKELEVEL", "MAKEFILES")}
    env["PATH"] = str(tools) + os.pathsep + env["PATH"]
    result = subprocess.run(
        ["make", "-f", "Makefile", "-f", str(overlay), "lint"],
        cwd=root, env=env, capture_output=True, text=True,
    )
    assert result.returncode != 0
    assert "unclassified test module tests/test_unclassified.py" in result.stderr
    assert not (root / "prepared").exists()


def test_unrun_closure_on_a_covered_line_remains_unreached(tmp_path: Path) -> None:
    root = tree(tmp_path)
    (root / "src/main.rs").write_text(
        'fn main() { let unused = || std::hint::black_box(42); let called = || std::hint::black_box(7); println!("{}", std::hint::black_box(&called as &dyn Fn() -> i32)()); if std::hint::black_box(false) { println!("{}", std::hint::black_box(&unused as &dyn Fn() -> i32)()); } }\n'
    )
    report = root / "outside.json"
    # LLVM function regions: start/end line/column, count, file id,
    # expansion file id, kind. Both functions occupy the same source line.
    report.write_text(json.dumps({"type": "llvm.coverage.json.export", "version": "3.1.0", "data": [{
        "functions": [
            {"filenames": [str(root / "src/main.rs")], "regions": [[1, 71, 1, 91, 1, 0, 0, 0]]},
            {"filenames": [str(root / "src/main.rs")], "regions": [[1, 29, 1, 49, 0, 0, 0, 0]]},
        ]}]}))
    status = root / "suites.json"
    status.write_text(json.dumps({"python": 0, "rust": 0, "spec": 0}))
    result = run(root, "report", "--json", str(report), "--statuses", str(status), "--floor", "75")
    assert result.returncode == 1, result.stderr
    data = json.loads(result.stdout)
    assert data["product_regions"] == 2
    assert data["reached_regions"] == 1
    assert data["outside_in_percent"] == 50
    assert data["unreached"]["src/main.rs"] == [{"start": [1, 29], "end": [1, 49]}]


@pytest.mark.parametrize("second_hits,percent", [(0, 0), (3, 100)])
def test_regions_union_binaries_and_map_expansion_file_ids(
    tmp_path: Path, second_hits: int, percent: int,
) -> None:
    root = tree(tmp_path)
    source = root / "src/main.rs"
    source.write_text("fn main() {}\n")
    other = root / "tests/helper.rs"
    report = root / "outside.json"
    report.write_text(json.dumps({"type": "llvm.coverage.json.export", "version": "3.1.0", "data": [
        {"functions": [{"filenames": [str(other), str(source)], "regions": [
            [1, 1, 1, 13, 0, 1, 0, 0],
            *[[1, 2, 1, 4, 0, 1, 0, kind] for kind in (1, 2, 3)],
        ]}]},
        {"functions": [{"filenames": [str(source), str(other)], "regions": [
            [1, 1, 1, 13, second_hits, 0, 1, 0],
            [1, 1, 1, 13, 0, 0, 1, 0],
        ]}]},
    ]}))
    status = root / "suites.json"
    status.write_text(json.dumps({"python": 0, "rust": 0, "spec": 0}))
    result = run(root, "report", "--json", str(report), "--statuses", str(status), "--floor", "0")
    assert result.returncode == 0, result.stderr
    data = json.loads(result.stdout)
    assert data["product_regions"] == 1
    assert data["outside_in_percent"] == percent
    assert bool(data["unreached"]) == (second_hits == 0)


@pytest.mark.parametrize("report", [{}, {"type": "llvm.coverage.json.export", "version": "3.1.0", "data": [{"functions": []}]}])
def test_empty_or_wrong_coverage_export_cannot_pass_a_zero_floor(tmp_path: Path, report: dict) -> None:
    root = tree(tmp_path)
    path = root / "outside.json"
    path.write_text(json.dumps(report))
    status = root / "suites.json"
    status.write_text(json.dumps({"python": 0, "rust": 0, "spec": 0}))
    rejected = run(root, "report", "--json", str(path), "--statuses", str(status), "--floor", "0")
    assert rejected.returncode == 1
    assert not rejected.stdout
    missing = run(root, "report", "--floor", "0")
    assert missing.returncode == 1
    assert "report requires --json and --statuses" in missing.stderr


@pytest.mark.parametrize("platform", ["darwin", "win32"])
def test_collection_requires_linux_before_preparing_or_building(tmp_path: Path, platform: str) -> None:
    import shutil

    root = tree(tmp_path)
    tools = root / "tools"
    tools.mkdir()
    for name in ("collect_feature_coverage.py", "feature_coverage.py"):
        shutil.copyfile(ROOT / "tools" / name, tools / name)
    output = root / ".cache/feature-coverage"
    output.mkdir(parents=True)
    (output / "coverage.json").write_text('{"status":"pass","outside_in_percent":100}')
    # No pyproject or installed tools: reject the platform before reading
    # collection settings, preparing dependencies or compiling anything.
    result = subprocess.run(collector_command(tools, platform), text=True, capture_output=True)
    assert result.returncode == 1
    assert "outside-in feature coverage requires a Linux host" in result.stdout
    assert not result.stderr
    data = json.loads((output / "coverage.json").read_text())
    assert data["status"] == "incomplete"
    assert "outside_in_percent" not in data
    assert not list(output.glob("*.command.json"))
