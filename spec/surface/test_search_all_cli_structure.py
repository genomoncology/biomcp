from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SEARCH_ALL_DIR = REPO_ROOT / "src/cli/search_all"
FLAT_SEARCH_ALL = REPO_ROOT / "src/cli/search_all.rs"


EXPECTED_SEARCH_ALL_RS_FILES = [
    Path("src/cli/search_all/mod.rs"),
    Path("src/cli/search_all/plan.rs"),
    Path("src/cli/search_all/dispatch.rs"),
    Path("src/cli/search_all/links.rs"),
    Path("src/cli/search_all/format.rs"),
    Path("src/cli/search_all/tests/plan.rs"),
    Path("src/cli/search_all/tests/dispatch.rs"),
    Path("src/cli/search_all/tests/links.rs"),
    Path("src/cli/search_all/tests/format.rs"),
]

FORBIDDEN_PLACEHOLDER_MODULES = [
    Path("src/cli/search_all/plan/mod.rs"),
    Path("src/cli/search_all/dispatch/mod.rs"),
    Path("src/cli/search_all/links/mod.rs"),
    Path("src/cli/search_all/format/mod.rs"),
]


def _actual_search_all_rs_files() -> list[Path]:
    if not SEARCH_ALL_DIR.is_dir():
        return []
    return sorted(
        path.relative_to(REPO_ROOT)
        for path in SEARCH_ALL_DIR.rglob("*.rs")
        if path.is_file()
    )


def _source(path: Path) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _assert_decomposed_layout_exists() -> list[Path]:
    assert SEARCH_ALL_DIR.is_dir(), "missing decomposed search_all module directory"
    actual = _actual_search_all_rs_files()
    assert actual == sorted(EXPECTED_SEARCH_ALL_RS_FILES), (
        "unexpected Rust file layout under src/cli/search_all"
    )
    return actual


def test_search_all_flat_module_is_replaced_by_directory_facade() -> None:
    assert not FLAT_SEARCH_ALL.exists(), (
        "flat search_all.rs must be replaced by src/cli/search_all/mod.rs"
    )
    assert SEARCH_ALL_DIR.is_dir(), "missing decomposed search_all module directory"


def test_search_all_decomposed_layout_has_expected_ownership_zones() -> None:
    _assert_decomposed_layout_exists()


def test_search_all_decomposed_layout_has_no_placeholder_modules() -> None:
    _assert_decomposed_layout_exists()
    for forbidden in FORBIDDEN_PLACEHOLDER_MODULES:
        assert not (REPO_ROOT / forbidden).exists(), (
            f"unexpected placeholder module present: {forbidden}"
        )


def test_search_all_decomposed_files_have_module_headers() -> None:
    for path in _assert_decomposed_layout_exists():
        assert _source(path).startswith("//!"), f"missing //! module header: {path}"


def test_search_all_decomposed_files_stay_under_700_lines() -> None:
    for path in _assert_decomposed_layout_exists():
        line_count = len(_source(path).splitlines())
        assert line_count <= 700, f"{path} exceeds 700 lines: {line_count}"
