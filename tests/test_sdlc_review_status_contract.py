"""Landed tickets must record their review verdicts in one Review
grammar, and no tracked file may carry merge-conflict markers.

The review rules catch the class Ian's 2026-09-26 review found (six
landed tickets still said a review was pending) and the shapes the
2026-09-27 review showed still leaked (wrapped text, bold bullets,
any scope containing "batch", a REJECT with no later ACCEPT, and
tickets without records skipped entirely). Structure, not phrase
bans:

* Logical lines: a Review bullet and its indented continuations join
  into one line before matching, so no wrapped spelling passes.
* One grammar: ``- <Kind>[(<scope>)]: <verdict>`` where Kind is one
  of the six house kinds. Any line that looks review-ish but does not
  parse is a format failure — bold, starred, or misspelled kinds all
  land here.
* Resolution: a verdict's state is read from its state tokens
  (ACCEPT, REJECT, BLOCK, pending). A pending token in a landed
  ticket fails unless its scope is a genuinely open slice (no other
  line gives that same kind+scope a verdict). A REJECT or BLOCK
  requires a later ACCEPT for the same kind, or its own line must
  name what resolved it (fixed/folded/verified/superseded/accepted).
* Coverage: every ticket file is scanned. Legacy tickets without
  records keep honest pending lines only while their work is unlanded;
  the 2026-09-27 backfill gave the landed legacy tickets their honest
  "record not kept at the time" verdicts.

The second rule catches what merge 7206240b shipped: conflict markers
committed to main. Scanning happens against `git ls-files`, so the
check fails the suite before a bisect ever lands on the broken commit.
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]

# --- The one Review grammar -------------------------------------------------
#
# A Review line is a markdown bullet whose text begins with a house
# kind, an optional parenthetical scope, a colon, and a verdict:
#
#     - Code review (batch 1): ACCEPT with P2s 2026-09-25
#
# Re-review forms are their own kinds, so a re-review verdict never
# masquerades as the original review.
KIND = (
    r"(?:design\s+re-review|code\s+re-review|design\s+review|"
    r"code\s+review|re-review|verification)"
)
# The grammar matches the bullet TEXT (the marker is checked
# separately so a starred or bolded variant is a format failure).
REVIEW_LINE = re.compile(
    rf"^(?P<kind>{KIND})(?:\s*\((?P<scope>[^)]*)\))?\s*:\s*(?P<verdict>\S.*)$",
    re.IGNORECASE,
)
# Whole-line allowlist: a logical line is either an exact grammar
# record (a `-` bullet matching REVIEW_LINE) or plain text. A line is
# NOT plain text when it contains a state word anywhere, or when it
# declares a review/verification status — a review-ish phrase
# followed by a scope and a colon, in any decoration (bullet, bold,
# heading, plain paragraph). The next spelling over cannot pass
# because the test is on what the line IS, not on a phrase list.
DECORATION = re.compile(r"^[\s#*>|`_-]*")
STATE_ANYWHERE = re.compile(r"\b(ACCEPT|REJECT|BLOCK|pending)\b", re.IGNORECASE)
STATUS_DECLARATION = re.compile(
    r"(?:design\s+|code\s+)?re(?:view|-review)s?"
    r"[\s*]*(?:\([^)]*\))?[\s*]*:",
    re.IGNORECASE,
)
# The kind vocabulary is an allowlist: design, code, bare, or
# verification. These are ANCHORED shapes — a status line begins
# (after decoration, including table pipes) with a kind word and a
# colon or a table pipe. Mid-sentence prose ("change from the
# review: ...") is not a status line, and a paragraph describing
# what a review returned ("review returned REJECT ...") does not
# carry the colon shape.
STATUS_HEAD = re.compile(
    r"^(?:design\s+|code\s+)?re(?:view|-review)s?\s*(?:\([^)]*\))?\s*(?::|\|)",
    re.IGNORECASE,
)
UNKNOWN_KIND_HEAD = re.compile(
    r"^(\w[\w-]*)\s+re(?:view|-review)s?\s*(?::|\|)",
    re.IGNORECASE,
)
ALLOWED_KIND_WORDS = {"design", "code", "review", "re-review", "verification"}
VERIFICATION_DECLARATION = re.compile(r"verifica(?:tion|tions)?\s*:", re.IGNORECASE)
# Honest legacy null states: the backfilled verdicts for tickets
# that predate the review-record convention say exactly this.
NULL_STATE = re.compile(
    r"\b(?:record not kept at the time|no gate record was kept|shipped in the 0\.8\.x series)\b",
    re.IGNORECASE,
)
# Completion words a verdict may declare instead of the four state
# tokens: a verification line that says it merged, passed, or was
# not required has declared an outcome.
COMPLETION = re.compile(
    r"\b(?:merged|done|shipped|passed|skipped|n/a|not required)\b",
    re.IGNORECASE,
)


# A line that is nothing but state/resolution words (an unindented
# wrapped verdict, e.g. "pending") is a status line, not prose.
STATE_AND_RESOLUTION = re.compile(
    r"(?:\b(?:ACCEPT|REJECT|BLOCK|pending|fixed|folded|verified|superseded|accepted)"
    r"[\s.,;:|!-]*)+",
    re.IGNORECASE,
)

# pm's own verdict line (2048): `pm ticket review` writes exactly
# "Reviews: revision <sha>, accept|findings" at the top of the ticket,
# and pm reads verdicts only from that line, so the contract accepts it
# as a valid record rather than forcing a hand-rewrite.
PM_REVIEW = re.compile(
    r"^Reviews: revision (?P<rev>[0-9a-f]{7,40}), (?P<verdict>accept|findings)$",
    re.IGNORECASE,
)

# Verdict state tokens, in the order they appear in the text.
STATE_TOKEN = re.compile(r"\b(ACCEPT|REJECT|BLOCK|pending)\b", re.IGNORECASE)
# Words that say what happened to a rejection on the same line. A
# closed set with real semantics: each names an outcome, so a bare
# "REJECT once (bad)" cannot smuggle itself through.
RESOLVED_BY = re.compile(
    r"\b(fixed|folded|verified|superseded|accepted)\b", re.IGNORECASE
)
# A resolution word negated is not a resolution: "REJECT, not yet
# fixed" declares the fix has NOT happened.
NOT_RESOLVED = re.compile(
    r"\b(?:not(?:\s+yet)?|never|to\s+be)\s+"
    r"(?:fixed|folded|verified|superseded|accepted)\b",
    re.IGNORECASE,
)
# A scope names a slice of the ticket; it exempts a pending verdict
# only when that slice is genuinely open (no sibling verdict line for
# the same kind and scope anywhere in the file).
SCOPE_SLICE = re.compile(r"\b(batch|item)s?\b", re.IGNORECASE)
SCOPE_TOKEN = re.compile(r"\b\d+\b")


def _scope_names_a_real_slice(scope: str, body: str) -> bool:
    """A pending scope may only name slices the ticket itself defines.

    Every numeric token in the scope must appear word-bounded in the
    ticket's non-review body; a scope with no numeric token (a bare
    "batches" or "items") stays allowed.
    """
    tokens = SCOPE_TOKEN.findall(scope)
    return all(re.search(rf"\b{re.escape(token)}\b", body) for token in tokens)


def _scope_is_only_slice_words(scope: str) -> bool:
    """A scope may contain only batch/item words and numbers/ranges.

    `(batch 3)` and `items 1-4` are slices; `(batch final)` and
    `(eventually)` are not — "final" is a word, and words outside
    the slice vocabulary have no defined meaning here.
    """
    words = scope.replace(",", " ").split()
    return all(
        SCOPE_SLICE.fullmatch(word) or re.fullmatch(r"[0-9][0-9\-/]*", word)
        for word in words
    )


def _ticket_number(path: Path) -> str:
    return path.name.split("-", 1)[0]


def _ticket_paths() -> list[Path]:
    return sorted((REPO_ROOT / "sdlc" / "tickets").glob("[0-9]" * 4 + "-*.md"))


def _landed_ticket_numbers() -> set[str]:
    records_dir = REPO_ROOT / "sdlc" / "records"
    return {
        _ticket_number(record) for record in records_dir.glob("[0-9]" * 4 + "-*.md")
    }


def _logical_lines(text: str) -> list[tuple[int, str]]:
    """Physical lines with indented continuations joined into one.

    A Review bullet's verdict wraps onto following indented lines
    (the house style). Joining before matching is what makes wrapped
    spellings ("re-review\npending") impossible to miss. Non-bullet
    lines (headings, paragraphs, stray state words) pass through as
    plain entries so the format scan sees them too — a status
    heading next to an honest record must still fail.
    """
    lines: list[tuple[int, str, str]] = []
    current: tuple[int, str, list[str]] | None = None
    for number, line in enumerate(text.splitlines(), start=1):
        bullet = re.match(r"^\s*(-|\*|\+|\d+\.)\s+(.*)$", line)
        if bullet is not None:
            if current is not None:
                lines.append((current[0], current[1], " ".join(current[2])))
            current = (number, bullet.group(1), [bullet.group(2)])
        elif current is not None and line.startswith((" ", "\t")) and line.strip():
            current[2].append(line.strip())
        else:
            if current is not None:
                lines.append((current[0], current[1], " ".join(current[2])))
                current = None
            if line.strip():
                lines.append((number, "", line.strip()))
    if current is not None:
        lines.append((current[0], current[1], " ".join(current[2])))
    return lines


def _non_review_body(text: str) -> str:
    """The ticket text outside grammar-parsed review records.

    Mirrors `_logical_lines`' joining so a wrapped review bullet is
    removed whole, while plain prose lines pass through untouched.
    """
    keep: list[str] = []
    current: list[str] | None = None
    for line in text.splitlines():
        bullet = re.match(r"^\s*(-|\*|\+|\d+\.)\s+(.*)$", line)
        if bullet is not None:
            if current is not None:
                keep.append(" ".join(current))
            current = [bullet.group(2)]
        elif current is not None and (line.startswith("  ") or line.strip()):
            current.append(line.strip())
        else:
            if current is not None:
                keep.append(" ".join(current))
                current = None
            keep.append(line)
    if current is not None:
        keep.append(" ".join(current))
    return "\n".join(
        joined for joined in keep if REVIEW_LINE.match(joined) is None
    )


_last_no_state_failures: list[str] = []


def _review_records(path: Path) -> list[dict[str, object]]:
    """Parsed Review lines: kind, scope, verdict, states, line number."""
    records: list[dict[str, object]] = []
    global _last_no_state_failures
    _last_no_state_failures = []
    for number, marker, text in _logical_lines(path.read_text(encoding="utf-8")):
        pm_match = PM_REVIEW.match(text)
        if pm_match is not None:
            verdict = pm_match.group("verdict")
            records.append(
                {
                    "line": number,
                    "kind": "review",
                    "scope": "pm revision " + pm_match.group("rev"),
                    "verdict": verdict,
                    "states": ["accept" if verdict.lower() == "accept" else "findings"],
                }
            )
            continue
        if marker != "-":
            continue
        match = REVIEW_LINE.match(text)
        if match is None:
            continue
        verdict = match.group("verdict")
        kind = re.sub(r"\s+", " ", match.group("kind")).lower()
        # A review verdict must declare a state (the "awaiting
        # reviewer" hole); a verification verdict states evidence,
        # and its completion words vary with what was run.
        # Ticket 1269: the review state vocabulary is an allowlist —
        # the four state tokens, the recorded null states, or the
        # exact recorded null forms. Completion words (done, passed,
        # skipped, merged) state evidence for VERIFICATION lines and
        # cannot smuggle a review verdict through "not done".
        REVIEW_NULL_FORM = re.compile(r"^(?:n/a\b|not required\b)", re.IGNORECASE)
        declares_review_state = (
            STATE_TOKEN.search(verdict)
            or RESOLVED_BY.search(verdict)
            or NULL_STATE.search(verdict)
            or REVIEW_NULL_FORM.match(verdict.strip())
        )
        if "review" in kind and not declares_review_state:
            _last_no_state_failures.append(
                f"{path.relative_to(REPO_ROOT)}:{number}: verdict declares no state: "
                f"{verdict[:60]}"
            )
        records.append(
            {
                "line": number,
                "kind": re.sub(r"\s+", " ", match.group("kind")).lower(),
                "scope": (match.group("scope") or "").strip() or None,
                "verdict": verdict,
                "states": [s.lower() for s in STATE_TOKEN.findall(verdict)],
            }
        )
    return records


def _base_kind(kind: str) -> str:
    """design re-review and code re-review resolve to their review kind."""
    return kind.replace(" re-review", " review").replace("re-review", "review")


def _review_failures(path: Path, landed: bool) -> list[str]:
    failures: list[str] = []
    relative = path.relative_to(REPO_ROOT)
    text_lines = path.read_text(encoding="utf-8")
    records = _review_records(path)
    failures.extend(_last_no_state_failures)

    # Format: the whole-line allowlist. A logical line is either an
    # exact grammar record or plain text; a state word anywhere, or a
    # review/verification declaration in any decoration, makes it a
    # status line that must be the grammar.
    for number, marker, logical_text in _logical_lines(text_lines):
        if marker == "-" and REVIEW_LINE.match(logical_text) is not None:
            continue
        stripped = DECORATION.sub("", logical_text)
        # Quoted status text is prose about the grammar ("Update the
        # stale 'Code review: pending' line"), not a status line.
        unquoted = re.sub(r'"[^"]*"', '""', stripped)
        declared = STATUS_DECLARATION.search(unquoted) or VERIFICATION_DECLARATION.search(
            unquoted
        )
        # A wrapped continuation can leave a bare state word alone on
        # an unindented line ("pending"); a whole line that is only
        # state/resolution words and punctuation is a status line.
        only_state = STATE_AND_RESOLUTION.fullmatch(
            unquoted.replace("|", " ").replace(",", " ").replace(";", " ").strip(" .:!-")
        )
        # A bullet whose HEAD is review/verification-ish and that
        # carries a state word is a glued status bullet ("Code
        # review verdict: pending"); state words in prose bodies
        # (an ACCEPT quoted mid-explanation) stay prose.
        head = " ".join(unquoted.split()[:3])
        # A glued status bullet ("- Code review verdict: pending")
        # carries a review-ish head AND a state word; the term is
        # restored after round three's review caught its loss.
        bullet_status = bool(marker == "-" and re.search(r"review|verifica", head, re.IGNORECASE) and STATE_ANYWHERE.search(unquoted))
        # A status line begins with a kind head (colon or table
        # pipe after decoration); anything else is prose even when
        # it mentions review states mid-sentence. An unknown kind
        # word in the head position fails whatever follows it.
        head_word = UNKNOWN_KIND_HEAD.match(unquoted)
        unknown_kind = bool(
            head_word and head_word.group(1).lower() not in ALLOWED_KIND_WORDS
        )
        status_head = bool(STATUS_HEAD.match(unquoted))
        pm_verdict = bool(PM_REVIEW.match(unquoted))
        grammar_bullet = marker == "-" and REVIEW_LINE.match(logical_text) is not None
        if not pm_verdict and (
            unknown_kind
            or only_state
            or bullet_status
            or (declared and STATE_ANYWHERE.search(unquoted))
            or (status_head and STATE_ANYWHERE.search(unquoted) and not grammar_bullet)
        ):
            failures.append(
                f"{relative}:{number}: review line is not the grammar: {logical_text[:70]}"
            )

    # Scope conflicts: a pending scoped line whose kind+scope already
    # carries a verdict somewhere in the file is stale, wherever the
    # two lines sit.
    verdict_scopes = {
        (r["kind"], r["scope"]) for r in records if r["states"] and "pending" not in r["states"]
    }
    for record in records:
        # Terminal state only: a verdict that ends in ACCEPT with the
        # word "pending" quoted inside its history (1235's "third
        # review pending" rejection note) is resolved, not pending.
        if record["states"][-1:] != ["pending"]:
            continue
        if not landed:
            continue
        scope = record["scope"]
        if scope is None:
            failures.append(
                f'{relative}:{record["line"]}: landed ticket says a review is pending'
            )
            continue
        if (record["kind"], scope) in verdict_scopes:
            failures.append(
                f'{relative}:{record["line"]}: scope {scope!r} says pending after a '
                f"verdict for the same slice"
            )
        elif not SCOPE_SLICE.search(scope):
            failures.append(
                f'{relative}:{record["line"]}: scope {scope!r} names no batch or item slice'
            )
        elif not _scope_is_only_slice_words(scope):
            failures.append(
                f'{relative}:{record["line"]}: scope {scope!r} carries words other than '
                f"slice names and numbers"
            )
        elif not _scope_names_a_real_slice(scope, _non_review_body(text_lines)):
            failures.append(
                f'{relative}:{record["line"]}: scope {scope!r} names a batch or item '
                f"the ticket never mentions"
            )

    # A landed ticket carries at least one grammar record: a ticket
    # with no Review lines at all has no recorded verdict.
    if landed and not records:
        failures.append(f"{relative}: no Review lines on a landed ticket")

    # Rejections resolve: a REJECT/BLOCK state needs a later ACCEPT
    # for the same kind, or its own line must say what resolved it.
    for index, record in enumerate(records):
        terminal = record["states"][-1] if record["states"] else None
        if terminal not in ("reject", "block"):
            continue
        later_accept = any(
            "accept" in r["states"] and _base_kind(r["kind"]) == _base_kind(record["kind"])
            for r in records[index + 1 :]
        )
        resolved_inline = (
            RESOLVED_BY.search(record["verdict"]) is not None
            and NOT_RESOLVED.search(record["verdict"]) is None
        )
        if not later_accept and not resolved_inline:
            failures.append(
                f'{relative}:{record["line"]}: {terminal} without a later ACCEPT or an '
                f"inline resolution"
            )
    return failures


def test_landed_tickets_record_their_review_verdicts() -> None:
    landed_numbers = _landed_ticket_numbers()
    tickets = _ticket_paths()
    assert tickets, "no ticket files found; the scan itself is broken"
    assert landed_numbers, "no records found; the scan itself is broken"

    failures: list[str] = []
    for ticket in tickets:
        landed = _ticket_number(ticket) in landed_numbers
        failures.extend(_review_failures(ticket, landed))

    assert not failures, (
        "review-line failures (grammar, unresolved rejections, stale pendings):\n"
        + "\n".join(failures)
    )


def test_the_grammar_catches_shapes_it_was_never_told_about(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Mutations the old phrase bans did not name all go red."""
    import sys

    monkeypatch.setattr(sys.modules[__name__], "REPO_ROOT", tmp_path)
    (tmp_path / "records").mkdir()
    (tmp_path / "tickets").mkdir()
    # A landed ticket: its record exists.
    (tmp_path / "records" / "9001-done.md").write_text("record\n", encoding="utf-8")

    def probe(body: str) -> str | None:
        ticket = tmp_path / "tickets" / "9001-done.md"
        ticket.write_text(body, encoding="utf-8")
        failures = _review_failures(ticket, landed=True)
        return failures[0] if failures else None

    # Never-told-about mutation 1: a wrapped pending verdict.
    assert probe("## Review\n\n- Code review: REJECT once (bad), re-review\n  pending\n")
    # Never-told-about mutation 2: a bold kind name.
    assert probe("## Review\n\n- **Code review**: pending\n")
    # Never-told-about mutation 3: a starred bullet.
    assert probe("## Review\n\n* Code review: pending\n")
    # Never-told-about mutation 4: a scope that names no slice.
    assert probe("## Review\n\n- Code review (eventually): pending\n")
    # Never-told-about mutation 5: pending for a slice that already
    # has a verdict elsewhere in the file.
    assert probe(
        "## Review\n\n- Code review (batch 3): ACCEPT\n\n- Code review (batch 3): pending\n"
    )
    # Never-told-about mutation 6: a rejection with no resolution.
    assert probe("## Review\n\n- Code review: REJECT once (bad)\n")
    # Never-told-about mutation 7: a kind glued to other words.
    assert probe("## Review\n\n- Code review verdict: pending\n")
    # Never-told-about mutation 8: a scope naming a batch the ticket
    # never mentions.
    assert probe(
        "## Order\n\nItems 1-4 land in one batch.\n\n## Review\n\n"
        "- Code review (batch 7): pending\n"
    )
    # Never-told-about mutation 8b: a status heading next to an
    # honest record (the compound shape the whole-line allowlist
    # exists to catch).
    assert probe(
        "## Review\n\n- Code review: ACCEPT 2026-09-25\n\n### Code review: pending\n"
    )
    # Never-told-about mutation 8c: a stray state word on its own
    # unindented line under an accepted verdict.
    assert probe("## Review\n\n- Code review: ACCEPT 2026-09-25\n\npending\n")
    # Never-told-about mutation 8d: a glued status bullet beside an
    # honest record (round three's regression shape, restored).
    assert probe(
        "## Review\n\n- Code review: ACCEPT 2026-09-25\n\n- Code review verdict: pending\n"
    )

    # Round-three mutations (ticket 1269): smuggled completion words,
    # unknown kinds, table rows, and negated resolutions.
    assert probe("## Review\n\n- Code review: not done\n")
    assert probe("## Review\n\n- Code review: not yet passed\n")
    assert probe("## Review\n\n- Code review: skipped\n")
    assert probe("## Review\n\n- Security review: awaiting\n")
    assert probe("## Review\n\n- Ticket review: pending\n")
    assert probe("| Code review | pending |\n")
    assert probe("## Review\n\n- Code review: REJECT, never fixed\n")
    assert probe("## Review\n\n- Code review: REJECT, to be folded\n")

    # Mutation 9 (2026-09-28): a verdict with no state word at all.
    assert probe("## Review\n\n- Code review: awaiting reviewer\n")
    # Mutation 10: a kind outside the house set.
    assert probe("## Review\n\n- Security review: pending\n")
    assert probe("## Review\n\n- Ticket review: pending\n")
    # Mutation 11: a heading-shaped status.
    assert probe("## Review\n\n### Code review: pending\n")
    # Mutation 12: a plain-paragraph status.
    assert probe("## Review\n\nCode review: pending, not started\n")
    # Mutation 13: a scope word outside the slice vocabulary.
    assert probe("## Review\n\n- Code review (batch final): pending\n")
    # Mutation 14: a negated resolution word is not a resolution.
    assert probe("## Review\n\n- Code review: REJECT, not yet fixed\n")
    # Mutation 15: an unindented wrapped verdict leaves a bare state
    # word on its own line.
    assert probe("## Review\n\n- Code review: re-review\npending\n")

    # A landed ticket with no Review lines at all fails.
    assert probe("## Order\n\nNothing here but work.\n")

    # The honest shapes stay green.
    assert probe("## Review\n\n- Code review: ACCEPT 2026-09-25\n") is None
    assert probe(
        "## Review\n\n- Code review: REJECT once (bad),\n  fixed and pinned 2026-09-25\n"
    ) is None
    assert probe(
        "## Review\n\n- Design review: REJECT once (bad)\n- Design re-review: ACCEPT 2026-09-26\n"
    ) is None
    # An unlanded ticket may keep honest pendings and bare rejections.
    (tmp_path / "tickets" / "9002-open.md").write_text(
        "## Review\n\n- Code review: pending\n", encoding="utf-8"
    )
    assert (
        probe(
            "## Order\n\nItems 5-9 are batch 2.\n\n## Review\n\n"
            "- Code review (batch 2): pending\n"
        )
        is None
    )
    assert _review_failures(tmp_path / "tickets" / "9002-open.md", landed=False) == []


MARKER_START = re.compile(r"^<{7} ")
MARKER_END = re.compile(r"^>{7} ")
MARKER_SEP = re.compile(r"^={7}$")


def _tracked_files() -> list[str]:
    output = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    ).stdout
    return [name.decode("utf-8") for name in output.split(b"\0") if name]


def _conflict_marker_lines(path: Path) -> list[tuple[int, str]]:
    text = path.read_text(encoding="utf-8", errors="replace")
    offenses: list[tuple[int, str]] = []
    for number, line in enumerate(text.splitlines(), start=1):
        if MARKER_START.match(line) or MARKER_END.match(line) or MARKER_SEP.match(line):
            offenses.append((number, line))
    return offenses


def test_no_tracked_file_carries_merge_conflict_markers() -> None:
    failures: list[str] = []
    for name in _tracked_files():
        path = REPO_ROOT / name
        if not path.is_file():
            continue
        for number, line in _conflict_marker_lines(path):
            failures.append(f"{name}:{number}: {line[:60]}")

    assert not failures, (
        "tracked files carry merge-conflict markers "
        "(merge 7206240b shipped these once; bisect lands on a broken commit):\n"
        + "\n".join(failures)
    )


def test_conflict_marker_scan_fails_on_a_synthetic_marker_file(
    tmp_path: Path,
) -> None:
    marked = tmp_path / "marked.md"
    marked.write_text(
        "before\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\nafter\n",
        encoding="utf-8",
    )
    offenses = _conflict_marker_lines(marked)
    assert [number for number, _ in offenses] == [2, 4, 6]


def test_conflict_marker_scan_passes_clean_text(tmp_path: Path) -> None:
    clean = tmp_path / "clean.md"
    clean.write_text(
        "# Title\n\nA list:\n\n- item\n\nA table row: a || b\n\n"
        "Python code fences with <<< repeated: <<<<<<\n",
        encoding="utf-8",
    )
    assert _conflict_marker_lines(clean) == []


def test_review_scan_requires_a_records_directory(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The landed set comes from sdlc/records; without it nothing is landed."""
    (tmp_path / "tickets").mkdir()
    (tmp_path / "tickets" / "0001-open.md").write_text(
        "## Review\n\n- Code review: pending\n", encoding="utf-8"
    )
    import sys

    monkeypatch.setattr(sys.modules[__name__], "REPO_ROOT", tmp_path)
    assert _landed_ticket_numbers() == set()
