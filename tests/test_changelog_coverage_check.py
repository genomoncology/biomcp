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


def test_release_candidate_coverage_needs_no_created_tag(tmp_path: Path) -> None:
    def git(*args: str) -> None:
        subprocess.run([
            "git", "-C", str(tmp_path), "-c", "user.name=Test", "-c",
            "user.email=test@example.invalid", *args,
        ], check=True, capture_output=True)

    git("init", "-q")
    git("commit", "--allow-empty", "-qm", "Previous release")
    git("tag", "v0.9.0")
    changelog = tmp_path / "CHANGELOG.md"
    changelog.write_text(
        "## 0.9.1 — 2026-10-17\n\n- Qualify the release candidate without creating a tag. (2161)\n",
        encoding="utf-8",
    )
    git("add", ".")
    git("commit", "-qm", "Land 2161: qualify the candidate")
    result = subprocess.run([
        sys.executable, str(CHECK), "--tag", "v0.9.1", "--revision", "HEAD",
    ], cwd=tmp_path, text=True, capture_output=True)
    assert result.returncode == 0, result.stderr
    assert "covers all 1 tickets" in result.stdout
    assert subprocess.check_output([
        "git", "-C", str(tmp_path), "tag", "--list", "v0.9.1",
    ], text=True) == ""


@pytest.mark.parametrize("describe_release_record", [False, True])
def test_release_candidate_ignores_records_from_other_release_lines(
    tmp_path: Path, describe_release_record: bool,
) -> None:
    def git(*args: str) -> None:
        subprocess.run([
            "git", "-C", str(tmp_path), "-c", "user.name=Test", "-c",
            "user.email=test@example.invalid", *args,
        ], check=True, capture_output=True)

    git("init", "-q")
    git("commit", "--allow-empty", "-qm", "Previous release")
    git("tag", "v0.9.0")
    tickets = tmp_path / "sdlc/tickets"
    records = tmp_path / "sdlc/records"
    tickets.mkdir(parents=True)
    records.mkdir()
    (tickets / "2041-sync.md").write_text("Status: OPEN.\nMilestone: 1.0\n")
    (records / "2041-sync.md").write_text("Record a synchronization on another line.\n")
    (tickets / "2096-stress.md").write_text("Status: COMPLETE.\nMilestone: 0.9.1\n")
    (records / "2096-stress.md").write_text("Prepare the stress test directory.\n")
    description = "- Prepare the stress test directory. (2096)\n" if describe_release_record else ""
    (tmp_path / "CHANGELOG.md").write_text("## 0.9.1 — 2026-10-17\n\n" + description)
    git("add", ".")
    git("commit", "-qm", "Record the two independent release lines")
    result = subprocess.run([
        sys.executable, str(CHECK), "--tag", "v0.9.1", "--revision", "HEAD",
    ], cwd=tmp_path, text=True, capture_output=True)
    assert "2041" not in result.stderr
    assert result.returncode == (0 if describe_release_record else 1), result.stderr
    if not describe_release_record:
        assert "2096" in result.stderr

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

    The coverage check runs `git tag`, `git log`, `git diff`, `git
    ls-tree` and `git show <ref>:<path>` in whatever order it needs
    them, so dispatching on the first argument keeps the harness
    honest when the script changes. The `show` value maps `ref:path`
    to a body; the `diff-files` value maps a commit sha to the files
    the landing at that sha changed (the sha is the call's last
    argument), falling back to the plain `diff` lines for every
    other `git diff` call.
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
        elif subcommand == "diff-files":
            # The landing diff `git diff --name-only <sha>^1 <sha>`:
            # keyed on the sha, the call's last argument, with the
            # plain diff lines answering every other `git diff` call.
            body += "  diff)\n"
            body += "    for last; do :; done\n"
            for sha, files in lines.items():
                body += f"    if [ \"$last\" = \"{sha}\" ]; then printf '%s\\n' {shlex.join(files)}; exit 0; fi\n"
            plain = responses.get("diff", [])
            body += f"    printf '%s\\n' {shlex.join(plain)}; exit 0 ;;\n"
        else:
            if subcommand == "diff" and "diff-files" in responses:
                continue
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
    tagged_tickets: list[str] | None = None,
) -> subprocess.CompletedProcess[str]:
    """Ticket files map path to the file's content at the tag; they
    reach the gate through the `diff` name list and the `git show
    tag:path` body, the two calls the status source makes.
    `tagged_tickets` lists the ticket files present in the tagged
    tree, the source the milestone check reads."""
    _fake_git(
        tmp_path,
        {
            "tag": ["v0.9.0", "v0.9.1"],
            "log": subjects,
            "diff": list(records or []) + list(tickets or {}),
            "ls-tree": tagged_tickets or [],
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


def test_land_branch_subject_ticket_requires_a_bullet(tmp_path: Path) -> None:
    # 2042 landed as "Land tickets/2042-fix": the Land shape can
    # name its branch, and the gate must count that subject too.
    result = _run(
        tmp_path,
        subjects=["Land tickets/2042-fix"],
        changelog="# C\n\n## Unreleased\n\n- Something else entirely. (1234)\n",
    )
    assert result.returncode == 1
    assert "2042" in result.stderr


def test_land_branch_subject_passes_with_a_described_bullet(tmp_path: Path) -> None:
    result = _run(
        tmp_path,
        subjects=["Land tickets/2042-fix"],
        changelog="# C\n\n## Unreleased\n\n- Checked protein numbering without canonical facts. (2042)\n",
    )
    assert result.returncode == 0, result.stderr


def test_land_subject_without_a_colon_still_counts_its_leading_ticket(
    tmp_path: Path,
) -> None:
    # "Land 2044 and 2045: ..." carried two tickets with no colon
    # after the first number; the leading number is the landing.
    result = _run(
        tmp_path,
        subjects=["Land 2044 and 2045: MCP errors carry no local path"],
        changelog="# C\n\n## Unreleased\n\n- Something else entirely. (1234)\n",
    )
    assert result.returncode == 1
    assert "2044" in result.stderr


def test_land_subject_without_a_colon_passes_with_a_described_bullet(
    tmp_path: Path,
) -> None:
    result = _run(
        tmp_path,
        subjects=["Land 2044 and 2045: MCP errors carry no local path"],
        changelog="# C\n\n## Unreleased\n\n- Kept every flag in the refused hint. (2044)\n",
    )
    assert result.returncode == 0, result.stderr


def test_both_land_shapes_count_once_each(tmp_path: Path) -> None:
    # The two Land spellings of one landing are one ticket; the
    # branch-named subject does not double-count it.
    result = _run(
        tmp_path,
        subjects=["Land tickets/2042-fix", "Land 2042: same ticket again"],
        changelog="# C\n\n## Unreleased\n\n- Checked protein numbering. (2042)\n",
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


def test_a_ticket_number_that_appears_only_in_prose_is_not_coverage(
    tmp_path: Path,
) -> None:
    """Ticket 2049's promised check: a number named inside a bullet's
    prose covers nothing. Reverting the closing-group logic in
    scripts/check-changelog-coverage.py to counting any number makes
    this test fail, so the prose rule cannot silently regress."""
    assert (
        described_tickets("- The drug card names the brand; ticket 2049 tracks the rest.")
        == set()
    )
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/2049-gate'"],
        changelog=(
            "# C\n\n## Unreleased\n\n"
            "- The drug card names the brand; ticket 2049 tracks the rest.\n"
        ),
    )
    assert result.returncode == 1
    assert "2049" in result.stderr


def test_a_period_after_the_closing_group_counts_only_when_the_group_is_real(
    tmp_path: Path,
) -> None:
    """Punctuation trailing the closing group must not hide it
    (ticket 2049), and a parenthesized number that is not a closing
    group still covers nothing, period or not."""
    assert described_tickets("- Reworked the release gates. (1234).") == {"1234"}
    assert (
        described_tickets("- Reworked the release gates (1234, see the review).") == set()
    )
    covered = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog="# C\n\n## Unreleased\n\n- Reworked the release gates. (1234).\n",
    )
    assert covered.returncode == 0, covered.stderr
    wrong_group = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        changelog=(
            "# C\n\n## Unreleased\n\n"
            "- Reworked the release gates (1234, see the review).\n"
        ),
    )
    assert wrong_group.returncode == 1
    assert "1234" in wrong_group.stderr

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

def test_milestone_problems_names_each_kind_of_wrong_ticket() -> None:
    # A ticket filed for the milestone raises no problem; a missing
    # file or a milestone-less file proves nothing and only names
    # itself for the strict dev-cycle shape test.
    tickets = {
        "1234": "Status: complete.\nMilestone: 0.9.2\n",
        "1235": "Status: complete.\n",
        "1236": None,
        "1237": "Milestone: 0.9.3\n",
    }
    section = "- Fixed the gates. (1234, 1235 and 1236)\n- Trimmed the lints. (1237)\n"
    assert _MODULE.milestone_problems(section, "0.9.3", tickets.get) == {
        "1234": "0.9.2",
        "1235": _MODULE.NO_MILESTONE,
        "1236": _MODULE.NO_TICKET_FILE,
    }


def test_cited_ticket_of_another_milestone_fails_the_tag_run(
    tmp_path: Path,
) -> None:
    # A section that cites another milestone's ticket misfiles its
    # bullets, and the release gate must refuse it.
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1234-gate'"],
        tickets={"sdlc/tickets/1234-lane.md": "Status: complete.\nMilestone: 1.0\n"},
        tagged_tickets=["sdlc/tickets/1234-lane.md"],
        changelog="# C\n\n## 0.9.1 — 2026-09-23\n\n- Fixed release publication. (1234)\n",
    )
    assert result.returncode == 1
    assert "1234 (1.0)" in result.stderr


def test_cited_ticket_without_a_milestone_line_passes_the_tag_run(
    tmp_path: Path,
) -> None:
    # Early 0.9.2 tickets carry no Milestone line; a milestone-less
    # file cannot prove another release, so the tag run accepts it.
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1287-gate'"],
        tickets={"sdlc/tickets/1287-sweep.md": "Status: complete.\n"},
        tagged_tickets=["sdlc/tickets/1287-sweep.md"],
        changelog="# C\n\n## 0.9.1 — 2026-09-23\n\n- Swept after the release. (1287)\n",
    )
    assert result.returncode == 0, result.stderr


def test_cited_ticket_without_a_ticket_file_passes_the_tag_run(
    tmp_path: Path,
) -> None:
    # Some early tickets carry no ticket file at all; a missing
    # record proves nothing about the milestone.
    result = _run(
        tmp_path,
        subjects=["Merge branch 'tickets/1288-gate'"],
        changelog="# C\n\n## 0.9.1 — 2026-09-23\n\n- Recorded release planning. (1288)\n",
    )
    assert result.returncode == 0, result.stderr



def _run_branch(
    tmp_path: Path,
    *,
    changelog: str,
    ref_name: str,
    landings: dict[str, tuple[str, list[str]]],
    branch_files: list[str] | None = None,
) -> subprocess.CompletedProcess[str]:
    """Drive the --branch form. `landings` maps a commit sha to its
    subject and the files it changed; `branch_files` is the branch's
    own diff against the trunk merge-base."""
    _fake_git(
        tmp_path,
        {
            "tag": ["v0.9.0", "v0.9.1"],
            "log": [f"{sha}\t{subject}" for sha, (subject, _) in landings.items()],
            "diff-files": {sha: files for sha, (_, files) in landings.items()},
            "merge-base": ["base0"],
            "diff": list(branch_files or []),
        },
    )
    path = tmp_path / "CHANGELOG.md"
    path.write_text(changelog, encoding="utf-8")
    return subprocess.run(
        [sys.executable, str(CHECK), "--branch", "--changelog", str(path)],
        capture_output=True,
        text=True,
        env=os.environ
        | {
            "PATH": f"{tmp_path}:{os.environ.get('PATH', '')}",
            "GIT_FAKE_CALLS": str(tmp_path / "calls"),
            "GITHUB_REF_NAME": ref_name,
        },
    )


def test_branch_mode_fails_a_src_change_without_a_bullet(tmp_path: Path) -> None:
    # A ticket branch whose own diff touches shipped behavior must
    # carry its described bullet before review, not at the tag.
    result = _run_branch(
        tmp_path,
        ref_name="ticket/1234-add-the-gate",
        landings={},
        branch_files=["src/cli/gate.rs"],
        changelog="# C\n\n## Unreleased\n\n- Something else entirely. (1300)\n",
    )
    assert result.returncode == 1
    assert "1234" in result.stderr


def test_branch_mode_passes_the_same_branch_with_its_bullet(tmp_path: Path) -> None:
    result = _run_branch(
        tmp_path,
        ref_name="ticket/1234-add-the-gate",
        landings={},
        branch_files=["src/cli/gate.rs"],
        changelog="# C\n\n## Unreleased\n\n- Added the release gate. (1234)\n",
    )
    assert result.returncode == 0, result.stderr


def test_branch_mode_demands_no_bullet_for_record_and_test_changes(
    tmp_path: Path,
) -> None:
    # A branch that changes no shipped behavior needs no bullet.
    result = _run_branch(
        tmp_path,
        ref_name="ticket/1998-write-the-record",
        landings={},
        branch_files=["sdlc/records/1998-write.md", "tests/test_gate.py"],
        changelog="# C\n\n## Unreleased\n\n- Unrelated work. (1300)\n",
    )
    assert result.returncode == 0, result.stderr


def test_branch_mode_demands_a_bullet_for_a_landed_src_landing(
    tmp_path: Path,
) -> None:
    # A landing since the last tag that touched shipped behavior
    # needs its bullet in the live section even on a bare push.
    result = _run_branch(
        tmp_path,
        ref_name="main",
        landings={"sha1": ("Land 1299: make the deadline honest", ["src/search.rs"])},
        changelog="# C\n\n## Unreleased\n\n- Something else entirely. (1300)\n",
    )
    assert result.returncode == 1
    assert "1299" in result.stderr
    # A landing is charged with the diff against its first parent,
    # exactly this call.
    calls = (tmp_path / "calls").read_text(encoding="utf-8")
    assert "diff --name-only sha1^1 sha1" in calls


def test_a_stale_branch_merge_is_charged_only_with_its_own_files(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Ticket 2125: the 2116 landing, on a real repository.

    A merge landing is charged with the diff against its first
    parent alone. The old `diff-tree -m --first-parent` also diffed
    the merge against its second parent, so a stale branch's landing
    was charged with every file main gained since it forked and the
    branch lane went red on a branch that changed one file."""

    def git(*args: str, cwd: Path) -> None:
        subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True)

    repo = tmp_path / "repo"
    (repo / "src").mkdir(parents=True)
    (repo / "CHANGELOG.md").write_text("# Changelog\n", encoding="utf-8")
    git("init", "-q", "-b", "main", cwd=repo)
    git("config", "user.email", "t@example.com", cwd=repo)
    git("config", "user.name", "t", cwd=repo)
    (repo / "src" / "base.rs").write_text("base\n", encoding="utf-8")
    git("add", "-A", cwd=repo)
    git("commit", "-q", "-m", "base", cwd=repo)
    git("tag", "v0.9.2", cwd=repo)
    git("checkout", "-q", "-b", "ticket/2116-audit", cwd=repo)
    (repo / "src" / "own.rs").write_text("own\n", encoding="utf-8")
    git("add", "-A", cwd=repo)
    git("commit", "-q", "-m", "branch work", cwd=repo)
    git("checkout", "-q", "main", cwd=repo)
    for index in range(1, 4):
        (repo / f"main-gained-{index}.rs").write_text("main\n", encoding="utf-8")
        git("add", "-A", cwd=repo)
        git("commit", "-q", "-m", f"main advance {index}", cwd=repo)
    git(
        "merge",
        "-q",
        "--no-ff",
        "-m",
        "Land 2116: the stale branch lands",
        "ticket/2116-audit",
        cwd=repo,
    )

    monkeypatch.chdir(repo)
    assert _MODULE.landing_files("v0.9.2", "HEAD") == {"2116": {"src/own.rs"}}


def test_branch_mode_passes_that_landing_with_its_bullet(tmp_path: Path) -> None:
    result = _run_branch(
        tmp_path,
        ref_name="main",
        landings={"sha1": ("Land 1299: make the deadline honest", ["src/search.rs"])},
        changelog="# C\n\n## Unreleased\n\n- Made the deadline honest. (1299)\n",
    )
    assert result.returncode == 0, result.stderr


def test_branch_mode_demands_no_bullet_for_a_record_only_landing(
    tmp_path: Path,
) -> None:
    result = _run_branch(
        tmp_path,
        ref_name="main",
        landings={
            "sha1": ("Land 1998: write the record", ["sdlc/records/1998-record.md"])
        },
        changelog="# C\n\n## Unreleased\n\n- Unrelated work. (1300)\n",
    )
    assert result.returncode == 0, result.stderr


def test_branch_mode_counts_the_manifest_and_install_files_as_shipped(
    tmp_path: Path,
) -> None:
    result = _run_branch(
        tmp_path,
        ref_name="main",
        landings={
            "sha1": ("Land 1299: bump the floor", ["Cargo.lock"]),
            "sha2": ("Land 1300: pin the installer", ["install.sh"]),
        },
        changelog="# C\n\n## Unreleased\n\n- Unrelated work. (1301)\n",
    )
    assert result.returncode == 1
    assert "1299" in result.stderr and "1300" in result.stderr


def test_branch_mode_reads_the_legacy_branch_prefix_too(tmp_path: Path) -> None:
    result = _run_branch(
        tmp_path,
        ref_name="tickets/1300-legacy-shape",
        landings={},
        branch_files=["docs/getting-started/index.md"],
        changelog="# C\n\n## Unreleased\n\n- Unrelated work. (1301)\n",
    )
    assert result.returncode == 1
    assert "1300" in result.stderr


def test_branch_mode_ignores_a_pull_request_ref(tmp_path: Path) -> None:
    # A pull request ref names no ticket branch; only landings count.
    result = _run_branch(
        tmp_path,
        ref_name="42/merge",
        landings={},
        branch_files=[],
        changelog="# C\n\n## Unreleased\n\n- Unrelated work. (1301)\n",
    )
    assert result.returncode == 0, result.stderr


def test_branch_mode_reads_the_release_under_way_not_dated_sections(
    tmp_path: Path,
) -> None:
    # A dated section is closed; only an Unreleased or bare under-way
    # heading carries the dev cycle's bullets.
    result = _run_branch(
        tmp_path,
        ref_name="main",
        landings={"sha1": ("Land 1299: make the deadline honest", ["src/search.rs"])},
        changelog=(
            "# C\n\n## 0.9.2 — 2026-09-23\n\n- Made the deadline honest. (1299)\n"
        ),
    )
    assert result.returncode == 1
    assert "no Unreleased or under-way version section" in result.stderr


def test_branch_mode_passes_without_a_live_section_when_nothing_needs_one(
    tmp_path: Path,
) -> None:
    result = _run_branch(
        tmp_path,
        ref_name="main",
        landings={
            "sha1": ("Land 1998: write the record", ["sdlc/records/1998-record.md"])
        },
        changelog="# C\n\n## 0.9.2 — 2026-09-23\n\n- Old release only. (1300)\n",
    )
    assert result.returncode == 0, result.stderr


def test_carried_tickets_from_another_release_line_do_not_demand_bullets() -> None:
    """A v0.9.x tag covers milestone 0.9.x tickets only (2046).

    The 1.0 planning records landed mid-cycle complete on their own
    line; the coverage gate must not demand 0.9.2 bullets for them.
    """
    class Git:
        def __init__(self, replies: dict[str, str]) -> None:
            self.replies = replies

        def __call__(self, *args: str) -> str:
            key = " ".join(args)
            if key not in self.replies:
                raise AssertionError(f"unexpected git call: {key}")
            return self.replies[key]

    zero_nine = "Status: complete.\nMilestone: 0.9.2\n"
    one_zero = "Status: complete.\nMilestone: 1.0\n"
    no_milestone = "Status: complete.\n"
    original = _MODULE.run_git
    _MODULE.run_git = Git(
        {
            "diff --name-only --diff-filter=A v0.9.1 v0.9.2 -- sdlc/tickets/": (
                "sdlc/tickets/1200-zero.md\n"
                "sdlc/tickets/2100-one.md\n"
                "sdlc/tickets/2200-none.md\n"
            ),
            "show v0.9.2:sdlc/tickets/1200-zero.md": zero_nine,
            "show v0.9.2:sdlc/tickets/2100-one.md": one_zero,
            "show v0.9.2:sdlc/tickets/2200-none.md": no_milestone,
        }
    )
    try:
        found = _MODULE.completed_status_tickets("v0.9.1", "v0.9.2")
    finally:
        _MODULE.run_git = original
    assert found == {"1200", "2200"}, found
    assert "2100" not in found
