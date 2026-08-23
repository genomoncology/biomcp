from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
CLEAN_SCRIPT = REPO_ROOT / "sdlc" / "scripts" / "clean"


def _clean_fixture(tmp_path: Path) -> Path:
    repo = tmp_path / "repo"
    (repo / "sdlc" / "scripts").mkdir(parents=True)
    (repo / "src").mkdir()
    shutil.copy2(CLEAN_SCRIPT, repo / "sdlc" / "scripts" / "clean")
    (repo / ".gitignore").write_text("target/\n.cache/\n.venv/\n", encoding="utf-8")
    (repo / "Cargo.toml").write_text(
        "[package]\nname = \"clean-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        encoding="utf-8",
    )
    (repo / "src" / "lib.rs").write_text(
        "pub fn answer() -> u8 { 42 }\n",
        encoding="utf-8",
    )
    subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(
        ["git", "-c", "user.email=test@example.com", "-c", "user.name=Test", "commit", "-qm", "initial"],
        cwd=repo,
        check=True,
    )
    return repo


def _run_clean(repo: Path, home: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(repo / "sdlc" / "scripts" / "clean")],
        cwd=repo,
        env=os.environ | {"HOME": str(home)},
        capture_output=True,
        text=True,
        check=False,
    )


def test_clean_discards_only_target_and_leaves_a_rebuildable_worktree(
    tmp_path: Path,
) -> None:
    repo = _clean_fixture(tmp_path)
    target_marker = repo / "target" / "debug" / "biomcp"
    target_marker.parent.mkdir(parents=True)
    target_marker.write_text("rebuild me\n", encoding="utf-8")
    evidence = repo / ".cache" / "run.json"
    evidence.parent.mkdir()
    evidence.write_text('{"keep": true}\n', encoding="utf-8")
    environment = repo / ".venv" / "keep"
    environment.parent.mkdir()
    environment.write_text("keep\n", encoding="utf-8")
    shared_cache = tmp_path / "home" / ".cache" / "sccache" / "keep"
    shared_cache.parent.mkdir(parents=True)
    shared_cache.write_text("keep\n", encoding="utf-8")
    (repo / "src" / "lib.rs").write_text(
        "pub fn answer() -> u8 { 43 }\n",
        encoding="utf-8",
    )
    (repo / "agent-note.txt").write_text("keep\n", encoding="utf-8")
    status_before = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=True,
    ).stdout

    removed = _run_clean(repo, tmp_path / "home")

    assert removed.returncode == 0
    assert "removed" in removed.stdout.lower()
    assert "target" in removed.stdout.lower()
    assert not (repo / "target").exists()
    assert evidence.read_text(encoding="utf-8") == '{"keep": true}\n'
    assert environment.read_text(encoding="utf-8") == "keep\n"
    assert shared_cache.read_text(encoding="utf-8") == "keep\n"
    assert (
        subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=repo,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
        == status_before
    )

    empty = _run_clean(repo, tmp_path / "home")

    assert empty.returncode == 0
    assert "nothing" in empty.stdout.lower()
    rebuilt = subprocess.run(
        ["cargo", "test", "--offline"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )
    assert rebuilt.returncode == 0, rebuilt.stderr
