from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
THIS_FILE = Path(__file__).resolve()
ALLOWED = {
    "struct TempDirGuard": {"src/test_support.rs"},
}
DEFINITION_PATTERNS = {
    "struct TempDirGuard": re.compile(
        r"^\s*(?:pub(?:\(.+?\))?\s+)?struct\s+TempDirGuard\b",
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
    # A test build's default cache root is a runtime scratch
    # the process shares, not a fixture a test owns, so no RAII guard can
    # outlive it; the OS temp cleaner handles the folder.
    return (
        relative_path == "src/cache/config.rs"
        and "biomcp-test-cache-{}" in match.group(0)
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
        "pub(crate) struct TempDirGuard {}\n",
        encoding="utf-8",
    )
    (src_root / "duplicate.rs").write_text(
        "struct TempDirGuard {}\n",
        encoding="utf-8",
    )

    with pytest.raises(AssertionError, match="src/duplicate.rs"):
        assert_marker_is_centralized(tmp_path, "struct TempDirGuard")


def test_temp_scratch_scanner_reports_forbidden_patterns_in_fixture(
    tmp_path: Path,
) -> None:
    src_root = tmp_path / "src"
    src_root.mkdir()
    tests_root = tmp_path / "tests"
    tests_root.mkdir()
    (src_root / "test_support.rs").write_text(
        "pub(crate) struct TempDirGuard {}\n",
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
        "pub(crate) struct TempDirGuard {}\n",
        encoding="utf-8",
    )
    (src_root / "reexports.rs").write_text(
        "use crate::test_support::TempDirGuard;\n"
        "// struct TempDirGuard should not count here.\n"
        'const NOTE: &str = "TempDirGuard is centralized";\n',
        encoding="utf-8",
    )

    for marker in ALLOWED:
        assert_marker_is_centralized(tmp_path, marker)


def cache_retention_bypasses(root: Path) -> list[str]:
    """Cache-enabling fixtures use the helper that owns their disk floor."""
    pattern = re.compile(
        r"[\"']BIOMCP_CACHE_MODE[\"']\s*(?:\]\s*=(?!=)|:)\s*[\"'](?:default|infinite)[\"']"
        r"|\.pop\(\s*[\"']BIOMCP_CACHE_MODE[\"']"
        r"|BIOMCP_CACHE_MODE\s*=\s*[\"'](?:default|infinite)[\"']"
        r"|\.setenv\(\s*[\"']BIOMCP_CACHE_MODE[\"']\s*,\s*[\"'](?:default|infinite)[\"']"
        r"|\.delenv\(\s*[\"']BIOMCP_CACHE_MODE[\"']"
        r"|\bdel\s+\w+\[\s*[\"']BIOMCP_CACHE_MODE[\"']\s*\]"
    )
    allowed = {"tests/golden/replay_env.py", "tests/test_cache_invalid_settings.py"}
    found = []
    for path in sorted((root / "tests").rglob("*.py")):
        relative = path.relative_to(root).as_posix()
        if relative in allowed or path.resolve() == THIS_FILE:
            continue
        text = path.read_text(encoding="utf-8")
        found.extend(f"{relative}:{line_number(text, match.start())}" for match in pattern.finditer(text))
    return found


def test_cache_retention_fixtures_use_the_shared_environment() -> None:
    assert cache_retention_bypasses(REPO_ROOT) == []


@pytest.mark.parametrize("statement", [
    'env.pop("BIOMCP_CACHE_MODE", None)',
    'env["BIOMCP_CACHE_MODE"] = "default"',
    'env.update(BIOMCP_CACHE_MODE="infinite")',
    'env = {"BIOMCP_CACHE_MODE": "default"}',
    'monkeypatch.setenv("BIOMCP_CACHE_MODE", "default")',
    'monkeypatch.delenv("BIOMCP_CACHE_MODE", raising=False)',
    'del env["BIOMCP_CACHE_MODE"]',
])
def test_cache_retention_guard_rejects_an_unisolated_fixture(tmp_path: Path, statement: str) -> None:
    tests = tmp_path / "tests"
    tests.mkdir()
    (tests / "retention.py").write_text(
        statement + "\n", encoding="utf-8",
    )
    assert cache_retention_bypasses(tmp_path) == [
        "tests/retention.py:1",
    ]

CACHE_ROOT_ASSIGNMENT = re.compile(
    r'(?:\.env|\.set|\.setenv|::set_var)\(\s*["\']BIOMCP_CACHE_DIR["\']\s*,'
    r'|["\']BIOMCP_CACHE_DIR["\']\s*(?:\]\s*=(?!=)|:)'
    r'|(?<![\w:])\(\s*["\']BIOMCP_CACHE_DIR["\']\s*,'
    r'(?!\s*["\'](?:BIOMCP_|XDG_)[A-Z_]*["\'])'
    r'|\bBIOMCP_CACHE_DIR\s*='
)

def shared_root_assignment(relative: str, text: str, offset: int) -> bool:
    """Exceptions cover the actual setter bodies, never an entire file."""
    definitions = {
        "tests/cache_fixture.py": ("def set_cache_root(", "\n\ndef "),
        "crates/biomcp-mcp-contract-client/src/lib.rs": (
            "pub fn child_cache_env(", "\n}"),
        "spec/fixtures/cache-policy.sh": ("set_fixture_cache_root() {", "\n}"),
    }
    definition = definitions.get(relative)
    if definition is None:
        return False
    signature, terminator = definition
    if relative == "spec/fixtures/cache-policy.sh":
        printer = text.find("print_fixture_cache_exports() {")
        end_printer = text.find("\n}", printer)
        if printer >= 0 and printer <= offset < end_printer:
            return True
    start = text.find(signature)
    if start < 0:
        return False
    end = text.find(terminator, start + len(signature))
    if relative == "tests/cache_fixture.py":
        end = text.find("\n", start)
        for line in text[end + 1:].splitlines(keepends=True):
            if line.strip() and not line[0].isspace():
                break
            end += len(line)
    return start <= offset < (len(text) if end < 0 else end)

def cache_root_bypasses(root: Path) -> list[str]:
    paths = support_sources_under(root)
    for directory, suffixes in (("crates", {".rs"}), ("spec", {".sh", ".md"}),
                                ("scripts", {".sh"})):
        paths.extend(path for path in (root / directory).rglob("*")
                     if path.is_file() and path.suffix in suffixes)
    wrapper = root / "tools/biomcp-ci"
    if wrapper.is_file():
        paths.append(wrapper)
    found = []
    for path in sorted(paths):
        relative = path.relative_to(root).as_posix()
        text = path.read_text(encoding="utf-8")
        for match in CACHE_ROOT_ASSIGNMENT.finditer(text):
            if shared_root_assignment(relative, text, match.start()):
                continue
            found.append(f"{relative}:{line_number(text, match.start())}")
    return found

def test_cache_root_fixtures_use_the_shared_guard() -> None:
    assert cache_root_bypasses(REPO_ROOT) == []

@pytest.mark.parametrize(("path", "statement"), [
    ("src/fixture.rs", 'env.set("BIOMCP_CACHE_DIR", root);'),
    ("tests/child.rs", 'command.env("BIOMCP_CACHE_DIR", root);'),
    ("tests/child.rs", '("BIOMCP_CACHE_DIR", root),'),
    ("tests/fixture.py", 'env["BIOMCP_CACHE_DIR"] = str(root)'),
    ("tests/fixture.py", 'env = {"BIOMCP_CACHE_DIR": str(root)}'),
    ("tests/fixture.py", 'monkeypatch.setenv("BIOMCP_CACHE_DIR", str(root))'),
    ("spec/fixtures/fixture.sh", 'export BIOMCP_CACHE_DIR="$root"'),
    ("spec/entity/fixture.md", 'BIOMCP_CACHE_DIR="$root" biomcp get gene BRAF'),
    ("scripts/fixture.sh", 'BIOMCP_CACHE_DIR="$root"'),
    ("tests/cache_fixture.py", 'env["BIOMCP_CACHE_DIR"] = str(root)'),
    ("tests/cache_fixture.py", 'def set_cache_root(env, root):\n'
     '    env["BIOMCP_CACHE_DIR"] = str(root)\n'
     'env["BIOMCP_CACHE_DIR"] = str(root)'),
])
def test_cache_root_guard_rejects_an_uncontrolled_assignment(
    tmp_path: Path, path: str, statement: str,
) -> None:
    fixture = tmp_path / path
    fixture.parent.mkdir(parents=True)
    fixture.write_text(statement + "\n", encoding="utf-8")
    assert cache_root_bypasses(tmp_path) == [f"{path}:{len(statement.splitlines())}"]
