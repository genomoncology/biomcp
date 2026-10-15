"""Contract tests for the Rust source-size inventory updater."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UPDATER = ROOT / "tools/update-rust-source-size-inventory"


def path_lines(repo: Path, relative: str) -> int:
    return len((repo / relative).read_text(encoding="utf-8").splitlines())


def _fixture_repo(tmp_path: Path, sources: dict[str, int]) -> Path:
    """A git repo carrying the updater and over-threshold sources."""
    repo = tmp_path / "repo"
    (repo / "tools").mkdir(parents=True)
    shutil.copy2(UPDATER, repo / "tools/update-rust-source-size-inventory")
    for relative, lines in sources.items():
        target = repo / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("fn main() {}\n" * lines, encoding="utf-8")
    subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
    subprocess.run(
        ["git", "add", "."],
        cwd=repo,
        check=True,
    )
    subprocess.run(
        ["git", "-c", "user.email=t@example.com", "-c", "user.name=t", "commit", "-qm", "x"],
        cwd=repo,
        check=True,
    )
    return repo


def _run(repo: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "tools/update-rust-source-size-inventory", *args],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )


def _inventory(repo: Path) -> dict[str, dict[str, object]]:
    payload = json.loads(
        (repo / "tools/rust-source-size-inventory.json").read_text(encoding="utf-8")
    )
    return {entry["path"]: entry for entry in payload["entries"]}


def _bootstrap(repo: Path, sources: dict[str, int]) -> None:
    result = _run(repo, "--bootstrap")
    assert result.returncode == 0, result.stdout + result.stderr
    for path, lines in sources.items():
        entry = _inventory(repo)[path]
        assert entry["baseline_lines"] == lines
        assert entry["floor_lines"] == lines


def test_authorize_accepts_a_comma_joined_path_list(tmp_path: Path) -> None:
    """One run authorizes several grown files with the same ticket metadata."""
    sources = {"src/grown_a.rs": 1010, "src/grown_b.rs": 1030, "src/stable.rs": 1005}
    repo = _fixture_repo(tmp_path, sources)
    _bootstrap(repo, sources)
    for path in ("src/grown_a.rs", "src/grown_b.rs"):
        target = repo / path
        target.write_text(
            target.read_text(encoding="utf-8") + "fn extra() {}\n" * 5, encoding="utf-8"
        )

    result = _run(
        repo,
        "--authorize",
        "src/grown_a.rs,src/grown_b.rs",
        "--ticket",
        "2047",
        "--reason",
        "one lane grew both files",
        "--removal-condition",
        "shrink when the shared helper lands",
    )
    assert result.returncode == 0, result.stdout + result.stderr
    entries = _inventory(repo)
    for path, floor in (("src/grown_a.rs", 1010), ("src/grown_b.rs", 1030)):
        entry = entries[path]
        assert entry["baseline_lines"] == path_lines(repo, path)
        assert entry["floor_lines"] == floor
        assert entry["authorized_increase"] == {
            "ticket": "2047",
            "delta": path_lines(repo, path) - floor,
            "reason": "one lane grew both files",
            "removal_condition": "shrink when the shared helper lands",
        }
    assert entries["src/stable.rs"]["authorized_increase"] is None


def test_the_comma_joined_list_tolerates_separator_spaces(tmp_path: Path) -> None:
    sources = {"src/grown_a.rs": 1010, "src/grown_b.rs": 1030}
    repo = _fixture_repo(tmp_path, sources)
    _bootstrap(repo, sources)
    for path in sources:
        target = repo / path
        target.write_text(
            target.read_text(encoding="utf-8") + "fn extra() {}\n", encoding="utf-8"
        )

    result = _run(
        repo,
        "--authorize",
        "src/grown_a.rs, src/grown_b.rs",
        "--ticket",
        "2047",
        "--reason",
        "one lane grew both files",
        "--removal-condition",
        "shrink when the shared helper lands",
    )
    assert result.returncode == 0, result.stdout + result.stderr
    for path in sources:
        assert _inventory(repo)[path]["authorized_increase"] is not None


def test_a_comma_joined_list_still_names_every_grown_file(tmp_path: Path) -> None:
    """A growth outside the named list fails the run even when a list is given."""
    sources = {"src/grown_a.rs": 1010, "src/grown_b.rs": 1030}
    repo = _fixture_repo(tmp_path, sources)
    _bootstrap(repo, sources)
    for path in sources:
        target = repo / path
        target.write_text(
            target.read_text(encoding="utf-8") + "fn extra() {}\n", encoding="utf-8"
        )

    result = _run(
        repo,
        "--authorize",
        "src/grown_a.rs",
        "--ticket",
        "2047",
        "--reason",
        "one lane grew both files",
        "--removal-condition",
        "shrink when the shared helper lands",
    )
    assert result.returncode != 0
    assert "growth requires --authorize src/grown_b.rs" in result.stderr


def test_a_comma_joined_list_re_stamps_several_metadata_repairs(tmp_path: Path) -> None:
    """The reason-only arm also serves a comma-joined list at one ticket."""
    sources = {"src/pinned_a.rs": 1010, "src/pinned_b.rs": 1030}
    repo = _fixture_repo(tmp_path, sources)
    _bootstrap(repo, sources)
    # Authorize a growth first so each entry carries a floor and an
    # allowance to repair.
    for path, extra in (("src/pinned_a.rs", 4), ("src/pinned_b.rs", 6)):
        target = repo / path
        target.write_text(
            target.read_text(encoding="utf-8") + "fn extra() {}\n" * extra, encoding="utf-8"
        )
    result = _run(
        repo,
        "--authorize",
        "src/pinned_a.rs,src/pinned_b.rs",
        "--ticket",
        "2046",
        "--reason",
        "original growth",
        "--removal-condition",
        "shrink later",
    )
    assert result.returncode == 0, result.stdout + result.stderr

    # A reason-only re-stamp over the same list rewrites only the metadata.
    result = _run(
        repo,
        "--authorize",
        "src/pinned_a.rs,src/pinned_b.rs",
        "--ticket",
        "2047",
        "--reason",
        "repaired history",
        "--removal-condition",
        "shrink when the split lands",
    )
    assert result.returncode == 0, result.stdout + result.stderr
    for path, floor in (("src/pinned_a.rs", 1010), ("src/pinned_b.rs", 1030)):
        entry = _inventory(repo)[path]
        assert entry["baseline_lines"] == path_lines(repo, path)
        assert entry["floor_lines"] == floor
        assert entry["authorized_increase"]["ticket"] == "2047"
        assert entry["authorized_increase"]["reason"] == "repaired history"


def test_a_dangling_comma_is_rejected(tmp_path: Path) -> None:
    repo = _fixture_repo(tmp_path, {"src/stable.rs": 1005})
    _bootstrap(repo, {"src/stable.rs": 1005})
    result = _run(
        repo,
        "--authorize",
        "src/stable.rs,",
        "--ticket",
        "2047",
        "--reason",
        "x",
        "--removal-condition",
        "y",
    )
    assert result.returncode != 0
    assert "comma-joined list of source paths" in result.stderr
