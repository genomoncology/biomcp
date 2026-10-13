"""Every landed ticket carries an accept verdict.

pm's merge-branch landing flow does not gate review verdicts (pm ticket
0325), so this one minimal check stays in the repo until pm 0325 lands
and a re-probe shows `pm ticket land --merge-branch` refusing every
non-accept state; that day, delete this file whole. The check is not a
grammar: glued, bold and wrapped spellings are pm's to police, and pm's
record-only flow already refuses them. A landed ticket — its number is
the leading number of a file in `sdlc/records/` — passes when it
carries one of:

* pm's own verdict line, `Reviews: revision <hex>, accept`;
* a review line whose state token is ACCEPT
  (`- Code review: ACCEPT 2026-10-07`);
* a review line recording one of the honest no-accept dispositions the
  1259 records round wrote for tickets that predate recorded verdicts.
  That list is closed — the exact strings measured in the repo today —
  so a new spelling fails the check instead of matching a pattern.

The two line shapes above are recognition, not policing: a line pm
would refuse (glued kinds, bold kinds, a `Reviews:` line wrapped around
a verdict) simply does not count as a verdict here either.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]

# pm's own verdict line: `pm ticket review` writes exactly this shape.
PM_ACCEPT = re.compile(r"^Reviews: revision [0-9a-f]{7,40}, accept$", re.IGNORECASE)

# A review line is a bullet whose text begins with a house review kind,
# an optional parenthetical scope, a colon, and a verdict:
#
#     - Code re-review (follow-up delta): ACCEPT 2026-10-07
#
# Only lines this shape count as verdicts; anything else is prose.
REVIEW_KIND = (
    r"(?:design\s+re-review|code\s+re-review|design\s+review|"
    r"code\s+review|re-review|verification)"
)
REVIEW_LINE = re.compile(
    rf"^(?P<kind>{REVIEW_KIND})(?:\s*\((?P<scope>[^)]*)\))?\s*:\s*(?P<verdict>\S.*)$",
    re.IGNORECASE,
)
ACCEPT_TOKEN = re.compile(r"\bACCEPT\b")

# The closed exemption: the exact no-accept dispositions in the repo
# today (the 1259 records round's honest states), as fixed literals. A
# landed ticket predating recorded verdicts passes only through one of
# these strings or a verdict opening "n/a"; a new spelling fails.
NO_ACCEPT_DISPOSITIONS = (
    "record not kept at the time",
    "no gate record was kept",
    "shipped in the 0.8.x series",
    "self-verified by the orchestrator",
    "no independent acceptance is recorded",
    "no dedicated reviewer dispatch existed at landing",
    "folded into ticket",
)


def _ticket_number(path: Path) -> str:
    return path.name.split("-", 1)[0]


def _ticket_paths() -> list[Path]:
    return sorted((REPO_ROOT / "sdlc" / "tickets").glob("[0-9][0-9][0-9][0-9]-*.md"))


def _landed_ticket_numbers() -> set[str]:
    records_dir = REPO_ROOT / "sdlc" / "records"
    return {
        _ticket_number(record)
        for record in records_dir.glob("[0-9][0-9][0-9][0-9]-*.md")
    }


def _logical_bullets(text: str) -> list[str]:
    """Bullets with their indented continuations joined into one line.

    A verdict wraps onto following indented lines (the house style), so
    `- Code review: ACCEPT` and its wrapped date are one record.
    """
    joined: list[str] = []
    current: list[str] | None = None
    for line in text.splitlines():
        bullet = re.match(r"^\s*-\s+(.*)$", line)
        if bullet is not None:
            if current is not None:
                joined.append(" ".join(current))
            current = [bullet.group(1)]
        elif current is not None and line.startswith((" ", "\t")) and line.strip():
            current.append(line.strip())
        else:
            if current is not None:
                joined.append(" ".join(current))
                current = None
    if current is not None:
        joined.append(" ".join(current))
    return joined


def _carries_accept(path: Path) -> str | None:
    """How the ticket carries its accept, or None when it does not."""
    text = path.read_text(encoding="utf-8")
    if any(PM_ACCEPT.match(line) for line in text.splitlines()):
        return "pm Reviews line"
    for bullet in _logical_bullets(text):
        match = REVIEW_LINE.match(bullet)
        if match is None:
            continue
        verdict = match.group("verdict")
        if ACCEPT_TOKEN.search(verdict):
            return "ACCEPT on a review line"
        if verdict.startswith("n/a") or any(
            disposition in verdict for disposition in NO_ACCEPT_DISPOSITIONS
        ):
            return "honest no-accept disposition"
    return None


def test_every_landed_ticket_carries_an_accept() -> None:
    landed = _landed_ticket_numbers()
    tickets = _ticket_paths()
    assert tickets, "no ticket files found; the scan itself is broken"
    assert landed, "no records found; the scan itself is broken"

    failures: list[str] = []
    for ticket in tickets:
        if _ticket_number(ticket) not in landed:
            continue
        if _carries_accept(ticket) is None:
            failures.append(f"sdlc/tickets/{ticket.name}: no accept verdict")

    assert not failures, (
        "landed tickets carry no accept verdict (pm's merge-branch flow "
        "does not gate this; pm 0325 tracks that gap):\n" + "\n".join(failures)
    )


def test_the_check_fails_the_states_pm_0325_will_refuse(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Red probes: pending, findings and missing verdicts all fail."""
    import sys

    monkeypatch.setattr(sys.modules[__name__], "REPO_ROOT", tmp_path)
    (tmp_path / "records").mkdir()
    (tmp_path / "tickets").mkdir()
    # A landed ticket: its record exists.
    (tmp_path / "records" / "9001-done.md").write_text("record\n", encoding="utf-8")
    ticket = tmp_path / "tickets" / "9001-done.md"

    def carries(body: str) -> str | None:
        ticket.write_text(body, encoding="utf-8")
        return _carries_accept(ticket)

    # The incident class: a landed ticket still saying a review is
    # pending, in either verdict form.
    assert carries("## Review\n\n- Code review: pending\n") is None
    assert carries("Reviews: revision abc1234, pending\n") is None
    # pm 0325's second probe state: a findings verdict, not an accept.
    assert carries("Reviews: revision deadbeef, findings\n") is None
    # A landed ticket with no review lines at all.
    assert carries("## Order\n\nNothing here but work.\n") is None
    # A glued or bold spelling is not a verdict line, so it cannot
    # carry the accept either.
    assert carries("## Review\n\n- Code review verdict: ACCEPT 2026-10-07\n") is None
    assert carries("## Review\n\n- **Code review**: ACCEPT 2026-10-07\n") is None
    assert carries("Reviews: revision abc1234,\naccept\n") is None
    # A no-accept spelling outside the closed exemption list fails.
    assert carries("## Review\n\n- Code review: no verdict was kept\n") is None

    # The two ruled accept shapes pass.
    assert carries("## Review\n\n- Code review: ACCEPT 2026-10-07\n") is not None
    assert (
        carries("## Review\n\n- Design review: ACCEPT 2026-09-23 (reviewer) — P2s\n")
        is not None
    )
    assert (
        carries(
            "## Review\n\n- Code re-review (delta): ACCEPT\n  2026-10-07, fresh reviewer\n"
        )
        is not None
    )
    assert carries("Reviews: revision 31db0bb, accept\n") is not None
    # A wrapped legacy verdict counts as one record.
    assert (
        carries("## Review\n\n- Code review: ACCEPT\n  2026-10-07\n") is not None
    )

    # The closed no-accept dispositions pass only through the list.
    assert (
        carries("## Review\n\n- Code review: record not kept at the time (...)\n")
        is not None
    )
    assert carries("## Review\n\n- Design review: n/a (documentation)\n") is not None


def test_the_landed_set_comes_from_the_records_directory(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Without records nothing is landed, so an open pending ticket is."""
    import sys

    monkeypatch.setattr(sys.modules[__name__], "REPO_ROOT", tmp_path)
    (tmp_path / "tickets").mkdir()
    (tmp_path / "tickets" / "0001-open.md").write_text(
        "## Review\n\n- Code review: pending\n", encoding="utf-8"
    )
    assert _landed_ticket_numbers() == set()
