from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
THIS_FILE = Path(__file__).resolve()
ALLOWED = {
    "struct EnvVarGuard": {"src/test_support.rs"},
    "struct TempDirGuard": {"src/test_support.rs"},
    "fn set_env_var": {"src/test_support.rs"},
}
DEFINITION_PATTERNS = {
    "struct EnvVarGuard": re.compile(
        r"^\s*(?:pub(?:\(.+?\))?\s+)?struct\s+EnvVarGuard\b",
        re.MULTILINE,
    ),
    "struct TempDirGuard": re.compile(
        r"^\s*(?:pub(?:\(.+?\))?\s+)?struct\s+TempDirGuard\b",
        re.MULTILINE,
    ),
    "fn set_env_var": re.compile(
        r"^\s*(?:pub(?:\(.+?\))?\s+)?fn\s+set_env_var\b",
        re.MULTILINE,
    ),
}
FORBIDDEN_TEMP_SCRATCH_PATTERNS = {
    "fn unique_temp_dir": re.compile(
        r"^\s*(?:pub(?:\(.+?\))?\s+)?fn\s+unique_temp_dir\b",
        re.MULTILINE,
    ),
    "tempfile.mkdtemp(prefix='biomcp-')": re.compile(
        r"tempfile\.mkdtemp\(\s*prefix\s*=\s*['\"]biomcp-",
        re.MULTILINE,
    ),
    "std env temp_dir join format biomcp-": re.compile(
        r"std::env::temp_dir\(\)\s*\.join\(\s*format!\(\s*\"biomcp-[^\"\n]*",
        re.MULTILINE,
    ),
}


def rust_sources_under(root: Path) -> list[Path]:
    sources = list((root / "src").rglob("*.rs"))
    sources.extend((root / "tests").rglob("*.rs"))
    return sorted(path for path in sources if path.is_file())


def support_sources_under(root: Path) -> list[Path]:
    sources = rust_sources_under(root)
    sources.extend((root / "tests").rglob("*.py"))
    return sorted(
        path for path in sources if path.is_file() and path.resolve() != THIS_FILE
    )


def files_with_marker(root: Path, marker: str) -> list[str]:
    matches = []
    for path in rust_sources_under(root):
        contents = path.read_text(encoding="utf-8")
        if DEFINITION_PATTERNS[marker].search(contents):
            matches.append(path.relative_to(root).as_posix())
    return matches


def assert_marker_is_centralized(root: Path, marker: str) -> None:
    matches = files_with_marker(root, marker)
    expected = sorted(ALLOWED[marker])
    assert matches == expected, (
        f"{marker!r} must be defined only in {expected}, found {matches}"
    )


def line_number(contents: str, offset: int) -> int:
    return contents.count("\n", 0, offset) + 1


def is_allowed_runtime_temp_scratch(relative_path: str, match: re.Match[str]) -> bool:
    return (
        relative_path == "src/cli/benchmark/run/execute.rs"
        and "biomcp-benchmark-{}-{}" in match.group(0)
    )


def forbidden_temp_scratch_matches(root: Path) -> list[str]:
    matches = []
    for path in support_sources_under(root):
        contents = path.read_text(encoding="utf-8")
        relative_path = path.relative_to(root).as_posix()
        for description, pattern in FORBIDDEN_TEMP_SCRATCH_PATTERNS.items():
            for match in pattern.finditer(contents):
                if is_allowed_runtime_temp_scratch(relative_path, match):
                    continue
                line = line_number(contents, match.start())
                matches.append(f"{relative_path}:{line}: {description}")
    return matches


def test_helper_definitions_are_centralized() -> None:
    for marker in ALLOWED:
        assert_marker_is_centralized(REPO_ROOT, marker)


def test_named_tmp_biomcp_test_scratch_uses_raii_helpers() -> None:
    assert forbidden_temp_scratch_matches(REPO_ROOT) == []


def test_scanner_reports_duplicate_definition_in_fixture(tmp_path: Path) -> None:
    src_root = tmp_path / "src"
    src_root.mkdir()
    (src_root / "test_support.rs").write_text(
        "pub(crate) struct EnvVarGuard {}\n"
        "pub(crate) struct TempDirGuard {}\n"
        "pub(crate) fn set_env_var() {}\n",
        encoding="utf-8",
    )
    (src_root / "duplicate.rs").write_text(
        "struct EnvVarGuard {}\n",
        encoding="utf-8",
    )

    with pytest.raises(AssertionError, match="src/duplicate.rs"):
        assert_marker_is_centralized(tmp_path, "struct EnvVarGuard")


def test_temp_scratch_scanner_reports_forbidden_patterns_in_fixture(
    tmp_path: Path,
) -> None:
    src_root = tmp_path / "src"
    src_root.mkdir()
    tests_root = tmp_path / "tests"
    tests_root.mkdir()
    (src_root / "test_support.rs").write_text(
        "pub(crate) struct EnvVarGuard {}\n"
        "pub(crate) struct TempDirGuard {}\n"
        "pub(crate) fn set_env_var() {}\n",
        encoding="utf-8",
    )
    bad_raw_temp = (
        "    let _ = std::env::temp_dir().join(" + 'format!("biomcp-raw-{}", 1));\n'
    )
    (tests_root / "raw_temp.rs").write_text(
        f"fn unique_temp_dir() {{}}\nfn scratch() {{\n{bad_raw_temp}}}\n",
        encoding="utf-8",
    )
    nested_tests_root = tests_root / "support"
    nested_tests_root.mkdir()
    (nested_tests_root / "raw_temp.rs").write_text(
        "fn scratch() {\n"
        '    let _ = std::env::temp_dir().join(format!("biomcp-nested-{}", 1));\n'
        "}\n",
        encoding="utf-8",
    )
    bad_mkdtemp = "tempfile.mkd" + 'temp(prefix="biomcp-study-tests-")\n'
    (tests_root / "conftest.py").write_text(bad_mkdtemp, encoding="utf-8")

    assert forbidden_temp_scratch_matches(tmp_path) == [
        "tests/conftest.py:1: tempfile.mkdtemp(prefix='biomcp-')",
        "tests/raw_temp.rs:1: fn unique_temp_dir",
        "tests/raw_temp.rs:3: std env temp_dir join format biomcp-",
        "tests/support/raw_temp.rs:2: std env temp_dir join format biomcp-",
    ]


def test_scanner_ignores_imports_and_comments_in_fixture(tmp_path: Path) -> None:
    src_root = tmp_path / "src"
    src_root.mkdir()
    (src_root / "test_support.rs").write_text(
        "pub(crate) struct EnvVarGuard {}\n"
        "pub(crate) struct TempDirGuard {}\n"
        "pub(crate) fn set_env_var() {}\n",
        encoding="utf-8",
    )
    (src_root / "reexports.rs").write_text(
        "use crate::test_support::{EnvVarGuard, TempDirGuard, set_env_var};\n"
        "// struct EnvVarGuard should not count here.\n"
        'const NOTE: &str = "fn set_env_var is centralized";\n',
        encoding="utf-8",
    )

    for marker in ALLOWED:
        assert_marker_is_centralized(tmp_path, marker)
