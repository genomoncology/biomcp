from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
CHECK_SCRIPT = REPO_ROOT / "tools" / "check-ticket-numbers.sh"
MAIN_REF = "origin/main"
LINE_REF = "origin/biodata/biomcp-1.0"


def _git(repo: Path, *arguments: str) -> str:
    return subprocess.run(
        ["git", "-C", str(repo), *arguments],
        check=True,
        capture_output=True,
        text=True,
    ).stdout


def _write_ticket(repo: Path, name: str) -> None:
    path = repo / "sdlc" / "tickets" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"# {name[:-3]}\n\nStatus: OPEN.\n", encoding="utf-8")


def _commit_all(repo: Path, message: str) -> None:
    _git(repo, "add", ".")
    _git(repo, "commit", "-m", message)


def _build_two_line_repository(root: Path) -> Path:
    """A small stand-in for the real two-line history.

    Main files a shared ticket past and its own later numbers; the 1.0
    line branches off before main's later numbers and files its own
    tickets from 2001 up, exactly as the real repository does.
    """
    repo = root / "repo"
    repo.mkdir()
    _git(repo, "init", "-b", "main")
    _git(repo, "config", "user.email", "fixture@example.test")
    _git(repo, "config", "user.name", "Fixture")
    _write_ticket(repo, "1142-shared-inherited-ticket.md")
    _write_ticket(repo, "2006-keep-ticket-numbers-unique.md")
    _commit_all(repo, "File the shared ticket past")

    _write_ticket(repo, "2096-main-only-ticket.md")
    _commit_all(repo, "File main's own ticket")
    main_sha = _git(repo, "rev-parse", "HEAD").strip()

    _git(repo, "checkout", "-b", "line", "HEAD~1")
    _write_ticket(repo, "2028-line-only-ticket.md")
    _commit_all(repo, "File a 1.0-line-only ticket")
    line_sha = _git(repo, "rev-parse", "HEAD").strip()

    _git(repo, "update-ref", f"refs/remotes/{MAIN_REF}", main_sha)
    _git(repo, "update-ref", f"refs/remotes/{LINE_REF}", line_sha)
    _git(repo, "checkout", "-b", "ticket/2100-fixture")

    (repo / "tools").mkdir()
    shutil.copy2(CHECK_SCRIPT, repo / "tools" / "check-ticket-numbers.sh")
    return repo


def _run_check(repo: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "tools/check-ticket-numbers.sh"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )


def test_clean_1_0_line_tree_passes_silently(tmp_path: Path) -> None:
    repo = _build_two_line_repository(tmp_path)

    result = _run_check(repo)

    assert result.returncode == 0
    assert result.stdout == ""
    assert result.stderr == ""


def test_line_ticket_reusing_a_main_number_in_a_different_file_fails(
    tmp_path: Path,
) -> None:
    repo = _build_two_line_repository(tmp_path)
    _write_ticket(repo, "2096-a-different-file-with-mains-number.md")

    result = _run_check(repo)

    assert result.returncode == 1
    assert (
        "sdlc/tickets/2096-a-different-file-with-mains-number.md: "
        "ticket 2096 already exists on origin/main as "
        "sdlc/tickets/2096-main-only-ticket.md" in result.stdout
    )


def test_1_0_line_only_ticket_numbered_below_the_floor_fails(
    tmp_path: Path,
) -> None:
    repo = _build_two_line_repository(tmp_path)
    _write_ticket(repo, "1999-a-line-only-ticket-below-the-floor.md")

    result = _run_check(repo)

    assert result.returncode == 1
    assert result.stdout.splitlines()[0] == (
        "sdlc/tickets/1999-a-line-only-ticket-below-the-floor.md: "
        "ticket 1999 is below 2001 on the 1.0 line"
    )


def test_1_0_line_only_ticket_at_the_floor_passes(tmp_path: Path) -> None:
    repo = _build_two_line_repository(tmp_path)
    _write_ticket(repo, "2001-a-line-only-ticket-at-the-floor.md")

    result = _run_check(repo)

    assert result.returncode == 0
    assert result.stdout == ""


def test_main_tree_passes_silently_even_with_a_low_numbered_ticket(
    tmp_path: Path,
) -> None:
    repo = _build_two_line_repository(tmp_path)
    _git(repo, "checkout", "main")
    _write_ticket(repo, "1999-main-may-file-low-numbers.md")

    result = _run_check(repo)

    assert result.returncode == 0
    assert result.stdout == ""
    assert result.stderr == ""


def test_missing_line_ref_fails_the_gate(tmp_path: Path) -> None:
    repo = _build_two_line_repository(tmp_path)
    _git(repo, "update-ref", "-d", f"refs/remotes/{LINE_REF}")

    result = _run_check(repo)

    assert result.returncode == 1
    assert LINE_REF in result.stderr
    assert "fetch origin" in result.stderr


def test_missing_main_ref_fails_on_the_1_0_line(tmp_path: Path) -> None:
    repo = _build_two_line_repository(tmp_path)
    _git(repo, "update-ref", "-d", f"refs/remotes/{MAIN_REF}")

    result = _run_check(repo)

    assert result.returncode == 1
    assert MAIN_REF in result.stderr


def test_make_lint_runs_the_ticket_number_check() -> None:
    makefile = (REPO_ROOT / "Makefile").read_text(encoding="utf-8")
    lint_recipe = makefile.split("lint:", 1)[1].split("\n\n", 1)[0]

    assert "tools/check-ticket-numbers.sh" in lint_recipe


def test_check_passes_on_this_repository_tree() -> None:
    result = subprocess.run(
        ["bash", str(CHECK_SCRIPT)],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stdout + result.stderr
