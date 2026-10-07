from __future__ import annotations

import importlib.util
import os

import pytest
from pathlib import Path
import re
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts" / "check-changelog-coverage.py"

_SPEC = importlib.util.spec_from_file_location("check_changelog_coverage", CHECK)
_MODULE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MODULE)
described_tickets = _MODULE.described_tickets
record_tickets = _MODULE.record_tickets


def _fake_git(
    directory: Path,
    responses: dict[str, list[str]] | dict[str, dict[str, str]],
) -> None:
    """Answer git subprocess calls by leading subcommand, not by order.

    The coverage check runs `git tag`, `git log`, `git diff` and
    `git show <ref>:<path>` in whatever order it needs them, so
    dispatching on the first argument keeps the harness honest when
    the script changes. The `show` value maps `ref:path` to a body.
    """
    script = directory / "git"
    body = "#!/usr/bin/env bash\nset -euo pipefail\n"
    body += 'printf \'%s\\n\' "$*" >> "$GIT_FAKE_CALLS"\n'
    body += 'case "$1" in\n'
    for subcommand, lines in responses.items():
        if subcommand == "show":
            body += "  show)\n"
            for ref_path, content in lines.items():
                safe = shlex.quote(content)
                body += f"    if [ \"$*\" = \"show {shlex.quote(ref_path)}\" ]; then printf %s {safe}; exit 0; fi\n"
            body += '    printf ""; exit 0 ;;\n'
        else:
            body += f"  {subcommand}) printf '%s\\n' {shlex.join(lines)}; exit 0 ;;\n"
    body += 'esac\necho "unexpected git call: $*" >&2\nexit 1\n'
    script.write_text(body, encoding="utf-8")
    script.chmod(0o755)


def _run(
    tmp_path: Path,
    *,
    changelog: str,
    subjects: list[str],
    records: list[str] | None = None,
    tickets: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    """Ticket files map path to the file's content at the tag; they
    reach the gate through the `diff` name list and the `git show
    tag:path` body, the two calls the status source makes."""
    _fake_git(
        tmp_path,
        {
            "tag": ["v0.9.0", "v0.9.1"],
            "log": subjects,
            "diff": list(records or []) + list(tickets or {}),
            "show": {
                f"v0.9.1:{path}": content for path, content in (tickets or {}).items()
            },
        },
    )
    path = tmp_path / "CHANGELOG.md"
    path.write_text(changelog, encoding="utf-8")
    return subprocess.run(
        [sys.executable, str(CHECK), "--tag", "v0.9.1", "--changelog", str(path)],
        capture_output=True,
        text=True,
        env=os.environ
        | {
            "PATH": f"{tmp_path}:{os.environ.get('PATH', '')}",
            "GIT_FAKE_CALLS": str(tmp_path / "calls"),
        },
    )


def test_version_section_at_end_of_file_passes(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge remote-tracking branch 'origin/tickets/1995-fix'"],
        changelog="# C\n\n## 0.9.1 — 2026-09-23\n\n- Fixed release publication. (1995)\n",
    )
    assert result.returncode == 0, result.stderr


def test_unreleased_fallback_passes(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- Reworked release gates. (1234)\n\n## 0.9.0\n",
    )
    assert result.returncode == 0, result.stderr


def test_only_merge_ticket_subjects_count(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Mention 1086 in a test", "Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- Reworked release gates. (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_bare_ticket_number_does_not_satisfy_coverage(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- 1234\n",
    )
    assert result.returncode == 1
    assert "1234" in result.stderr


def test_bare_ticket_number_list_does_not_satisfy_coverage(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=[
            "Merge branch 'tickets/1226-gate'",
            "Merge branch 'tickets/1227-other'",
        ],
        changelog="# C\n\n## Unreleased\n\n- 1226, 1227, 1228\n",
    )
    assert result.returncode == 1
    assert "1226" in result.stderr and "1227" in result.stderr


def test_label_word_bullets_do_not_satisfy_coverage(tmp_path: Path) -> None:
    # "Tickets" and "see" are boilerplate labels, not descriptions:
    # the gate must not count them as covered.
    result = _run(
        tmp_path,
        subjects=[
            "Merge branch 'tickets/1226-gate'",
            "Merge branch 'tickets/1227-other'",
        ],
        changelog="# C\n\n## Unreleased\n\n- Tickets 1226, 1227\n- see 1226\n",
    )
    assert result.returncode == 1
    assert "1226" in result.stderr and "1227" in result.stderr


def test_described_number_only_bullet_after_numbers_removed(tmp_path: Path) -> None:
    # Even with three stray digits gone, at least three word
    # characters of description must remain.
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- a 1 b 2 c 3 (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_record_only_ticket_requires_a_bullet(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Record ticket 1998 directly"],
        records=["sdlc/records/1998-fix-the-gate.md"],
        changelog="# C\n\n## Unreleased\n\n- Something else entirely. (1234)\n",
    )
    assert result.returncode == 1
    assert "1998" in result.stderr


def test_record_only_ticket_passes_with_a_described_bullet(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Record ticket 1998 directly"],
        records=["sdlc/records/1998-fix-the-gate.md"],
        changelog="# C\n\n## Unreleased\n\n- Fixed the coverage gate. (1998)\n",
    )
    assert result.returncode == 0, result.stderr


def test_union_of_merge_subjects_and_records_requires_both_bullets(
    tmp_path: Path,
) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        records=["sdlc/records/1996-record-only.md"],
        changelog="# C\n\n## Unreleased\n\n- Reworked the gates. (1234)\n",
    )
    assert result.returncode == 1
    assert "1996" in result.stderr


def test_union_passes_when_both_have_bullets(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        records=["sdlc/records/1996-record-only.md"],
        changelog="# C\n\n## Unreleased\n\n- Reworked the gates. (1234)\n- Widened the ticket scan. (1996)\n",
    )
    assert result.returncode == 0, result.stderr


def test_land_subject_ticket_requires_a_bullet(tmp_path: Path) -> None:
    # Tickets land on main as "Land NNNN: ..." subjects; the gate
    # must count them, not only old-pattern merge subjects.
    result = _run(
        tmp_path,
        subjects=["Land 1299: make the article search deadline honest"],
        changelog="# C\n\n## Unreleased\n\n- Something else entirely. (1234)\n",
    )
    assert result.returncode == 1
    assert "1299" in result.stderr


def test_land_subject_passes_with_a_described_bullet(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Land 1299: make the article search deadline honest"],
        changelog="# C\n\n## Unreleased\n\n- Made the article search deadline honest. (1299)\n",
    )
    assert result.returncode == 0, result.stderr


def test_both_landing_shapes_demand_their_bullets(tmp_path: Path) -> None:
    # A range with both landing shapes must demand a bullet for each:
    # covering the old-pattern merge does not cover the Land subject.
    result = _run(
        tmp_path,
        subjects=[
            "Land 1306: fold the 1302 review carryovers",
            "Merge branch 'tickets/1234-gate'",
        ],
        changelog="# C\n\n## Unreleased\n\n- Folded the review carryovers. (1306)\n",
    )
    assert result.returncode == 1
    assert "1234" in result.stderr


def test_both_landing_shapes_pass_with_both_bullets(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=[
            "Land 1306: fold the 1302 review carryovers",
            "Merge branch 'tickets/1234-gate'",
        ],
        changelog="# C\n\n## Unreleased\n\n- Folded the review carryovers. (1306)\n- Reworked the gates. (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_land_subject_text_does_not_count_other_tickets(tmp_path: Path) -> None:
    # "Land 1306: fold the 1302 review carryovers" names 1302 in its
    # text; only the ticket in the Land prefix lands a ticket.
    result = _run(
        tmp_path,
        subjects=["Land 1306: fold the 1302 review carryovers"],
        changelog="# C\n\n## Unreleased\n\n- Folded the carryovers. (1306)\n",
    )
    assert result.returncode == 0, result.stderr


def test_same_ticket_across_shapes_counts_once(tmp_path: Path) -> None:
    # One ticket arriving as a merge subject and again as a record
    # file is one ticket: a single described bullet covers it.
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        records=["sdlc/records/1234-gate-build.md"],
        changelog="# C\n\n## Unreleased\n\n- Reworked the gates. (1234)\n",
    )
    assert result.returncode == 0, result.stderr
    assert "covers all 1 tickets" in result.stdout


def test_same_ticket_across_all_three_shapes_counts_once(tmp_path: Path) -> None:
    # Land subject, record file and a complete ticket status in one
    # range: still one ticket, one bullet.
    result = _run(
        tmp_path,
        subjects=["Land 1234: rework the gates"],
        records=["sdlc/records/1234-gate-build.md"],
        tickets={"sdlc/tickets/1234-rework-the-gates.md": "Status: complete.\n"},
        changelog="# C\n\n## Unreleased\n\n- Reworked the gates. (1234)\n",
    )
    assert result.returncode == 0, result.stderr
    assert "covers all 1 tickets" in result.stdout


def test_completed_status_ticket_demands_a_bullet(tmp_path: Path) -> None:
    # A ticket file added in the range whose status reads complete
    # demands a bullet with no Land subject and no record: 2010
    # closed this way after its work landed as plain commits.
    result = _run(
        tmp_path,
        subjects=["Some plain commit"],
        tickets={"sdlc/tickets/1998-status-only.md": "Status: complete.\n"},
        changelog="# C\n\n## Unreleased\n\n- Something else entirely. (1234)\n",
    )
    assert result.returncode == 1
    assert "1998" in result.stderr


def test_completed_status_ticket_passes_with_a_described_bullet(
    tmp_path: Path,
) -> None:
    result = _run(
        tmp_path,
        subjects=["Some plain commit"],
        tickets={"sdlc/tickets/1998-status-only.md": "Status: complete.\n"},
        changelog="# C\n\n## Unreleased\n\n- Closed the status-only ticket. (1998)\n",
    )
    assert result.returncode == 0, result.stderr


def test_open_status_ticket_demands_nothing(tmp_path: Path) -> None:
    # A filed ticket still OPEN at the tag has not landed; it must
    # not demand a bullet.
    result = _run(
        tmp_path,
        subjects=["Some plain commit"],
        tickets={"sdlc/tickets/1998-still-open.md": "Status: OPEN.\n"},
        changelog="# C\n\n## Unreleased\n\n- Something else entirely. (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_status_token_matches_pm_ticket_grammar() -> None:
    # pm reads the first Status line in the first 25 lines, strips
    # emphasis, cuts at the first sentence terminator and lowercases;
    # real tickets spell completion with trailing text and capitals.
    assert _MODULE.status_token("Status: complete.\n") == "complete"
    assert _MODULE.status_token("Status: COMPLETE. Built 2026-10-06.\n") == "complete"
    assert (
        _MODULE.status_token("Status: complete. Absorbed by ticket 1304, split.\n")
        == "complete"
    )
    assert _MODULE.status_token("Status: **COMPLETE**.\n") == "complete"
    assert _MODULE.status_token("Status: OPEN.\n") == "open"
    assert _MODULE.status_token("Status: deferred\n") == "deferred"
    assert _MODULE.status_token("Status: complete. Cap 726.0 lines.\n") == "complete"
    assert _MODULE.status_token("""# 1998 — a ticket

Filed once.

Status: complete.\n""") == "complete"
    assert _MODULE.status_token("no status line at all\n") == ""


def test_completed_status_discovery_uses_real_git_history(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The status scan runs `git show tag:path` against a real
    repository, not a canned response."""
    import subprocess

    def git(*args: str, cwd: Path) -> None:
        subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True)

    repo = tmp_path / "repo"
    (repo / "sdlc" / "tickets").mkdir(parents=True)
    (repo / "CHANGELOG.md").write_text("# Changelog\n", encoding="utf-8")
    git("init", "-q", cwd=repo)
    git("config", "user.email", "t@example.com", cwd=repo)
    git("config", "user.name", "t", cwd=repo)
    git("add", "-A", cwd=repo)
    git("commit", "-q", "-m", "base", cwd=repo)
    git("tag", "v0.9.0", cwd=repo)
    (repo / "sdlc" / "tickets" / "1997-real-history.md").write_text(
        "# 1997\n\nStatus: COMPLETE. Built from history.\n", encoding="utf-8"
    )
    (repo / "sdlc" / "tickets" / "1996-real-history.md").write_text(
        "# 1996\n\nStatus: OPEN.\n", encoding="utf-8"
    )
    git("add", "-A", cwd=repo)
    git("commit", "-q", "-m", "file tickets", cwd=repo)
    git("tag", "v0.9.1", cwd=repo)

    monkeypatch.chdir(repo)
    assert _MODULE.completed_status_tickets("v0.9.0", "v0.9.1") == {"1997"}


def test_record_discovery_uses_real_git_history(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The records scan runs against a real repository, not a canned response."""
    import subprocess

    def git(*args: str, cwd: Path) -> None:
        subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True)

    repo = tmp_path / "repo"
    (repo / "sdlc" / "records").mkdir(parents=True)
    (repo / "CHANGELOG.md").write_text("# Changelog\n", encoding="utf-8")
    git("init", "-q", cwd=repo)
    git("config", "user.email", "t@example.com", cwd=repo)
    git("config", "user.name", "t", cwd=repo)
    git("add", "-A", cwd=repo)
    git("commit", "-q", "-m", "base", cwd=repo)
    git("tag", "v0.9.0", cwd=repo)
    (repo / "sdlc" / "records" / "1997-real-history.md").write_text(
        "x\n", encoding="utf-8"
    )
    git("add", "-A", cwd=repo)
    git("commit", "-q", "-m", "ticket 1997", cwd=repo)
    git("tag", "v0.9.1", cwd=repo)

    monkeypatch.chdir(repo)
    tickets = record_tickets("v0.9.0", "v0.9.1")
    assert "1997" in tickets


def test_non_ticket_record_files_do_not_count(tmp_path: Path) -> None:
    # Only sdlc/records/NNNN-*.md names map to tickets; a note file
    # in the same directory must not.
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        records=["sdlc/records/release-notes-only.md"],
        changelog="# C\n\n## Unreleased\n\n- Reworked the gates. (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_internal_only_described_bullet_passes(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- Internal only: release contract tests. (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_missing_both_sections_fails_clearly(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## 0.9.0\n",
    )
    assert result.returncode == 1
    assert "neither a 0.9.1 nor an Unreleased section" in result.stderr


def test_no_previous_release_passes_without_git_log(tmp_path: Path) -> None:
    _fake_git(tmp_path, {"tag": ["v0.9.1"]})
    path = tmp_path / "CHANGELOG.md"
    path.write_text("## Unreleased\n", encoding="utf-8")
    result = subprocess.run(
        [sys.executable, str(CHECK), "--tag", "v0.9.1", "--changelog", str(path)],
        capture_output=True,
        text=True,
        env=os.environ
        | {
            "PATH": f"{tmp_path}:{os.environ.get('PATH', '')}",
            "GIT_FAKE_CALLS": str(tmp_path / "calls"),
        },
    )
    assert result.returncode == 0, result.stderr


def test_every_real_unreleased_bullet_describes_its_ticket() -> None:
    content = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    # After the 0.9.1 release the live section is the 0.9.1 heading
    # (an Unreleased section returns with the next dev cycle).
    match = re.search(
        r"^## (?:Unreleased|0\.9\.1 — 2026-10-01)\s*([\s\S]*?)(?=^## |\Z)",
        content,
        re.MULTILINE,
    )
    assert match is not None
    section = match.group(1)
    described = described_tickets(section)
    mentioned = set(re.findall(r"\((\d{4})\)", section))
    assert mentioned, "the Unreleased section has no ticket markers"
    missing = sorted(mentioned - described)
    assert not missing, f"bullets for {missing} carry no described text"


def test_described_tickets_rejects_a_number_only_bullet_directly() -> None:
    assert described_tickets("- 1226, 1227, 1228") == set()
    assert described_tickets("- Fixed the wheel floor check (1246)") == {"1246"}


def test_described_tickets_rejects_label_only_bullets_directly() -> None:
    assert described_tickets("- Tickets 1226, 1227") == set()
    assert described_tickets("- see 1226") == set()
    assert described_tickets("- and fixes 1226") == set()
    assert described_tickets("- The changes 1226") == set()
    assert described_tickets("- Fixed 1226") == set()
    assert described_tickets("- Updated 1226") == set()
    assert described_tickets("- Added 1226") == set()
    # Real description words survive the stoplist.
    assert described_tickets("- Restored container publication (1219)") == {"1219"}

def test_dated_and_yearly_records_do_not_count_as_tickets(tmp_path: Path, monkeypatch) -> None:
    """Only ticket-range numbers count, whatever the slug looks like.

    Found by the 2026-09-28 review: the leading year parsed as a
    ticket number, and the first fix dropped digit- and capital-start
    slugs. This drives `record_tickets()` itself against a real
    records-directory shape.
    """
    records = tmp_path / "records" / "sdlc" / "records"
    records.mkdir(parents=True)
    names = [
        "2026-09-27-source-licensing-review-pass.md",
        "1999-12-31-pre-y2k-dated-note.md",
        "2000-fix-the-four-digit-boundary.md",
        "2027-three-years-out.md",
        "1265-3-sources-behind-one-api.md",
        "1265-Alpha-sort-the-catalog.md",
        "0843-something-old.md",
        "12345-five-digit.md",
        "1255-route-the-last-four-xml-sources-through-the-depth-cap.md",
    ]
    git_env = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@t",
               "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@t"}
    repo = tmp_path / "records"
    (repo / "seed.txt").write_text("base\n", encoding="utf-8")
    subprocess.run(["git", "init", "-q", str(repo)], check=True, env=git_env)
    subprocess.run(["git", "-C", str(repo), "add", "-A"], check=True, env=git_env)
    subprocess.run(["git", "-C", str(repo), "commit", "-qm", "base"], check=True, env=git_env)
    for name in names:
        (records / name).write_text("record\n", encoding="utf-8")
    subprocess.run(["git", "-C", str(repo), "add", "-A"], check=True, env=git_env)
    subprocess.run(["git", "-C", str(repo), "commit", "-qm", "records"], check=True, env=git_env)
    base = subprocess.run(["git", "-C", str(repo), "rev-parse", "HEAD~1"],
                          capture_output=True, text=True, check=True).stdout.strip()
    monkeypatch.chdir(repo)
    found = _MODULE.record_tickets(base, "HEAD")
    # Date-shaped names never count (any year); ticket numbers of
    # 2000 and above count — the cap is gone (2026-09-29 review).
    assert found == {"1265", "0843", "1255", "2000", "2027"}, found

