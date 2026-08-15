#!/usr/bin/env python3
"""Extract one exact curated release section from the changelog."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

HEADING = re.compile(r"^## (?P<version>[0-9]+\.[0-9]+\.[0-9]+) — .+$", re.MULTILINE)


class ReleaseNotesError(ValueError):
    pass


def extract_release_notes(changelog: Path, version: str) -> str:
    text = changelog.read_text(encoding="utf-8")
    headings = list(HEADING.finditer(text))
    matches = [heading for heading in headings if heading.group("version") == version]
    if not matches:
        raise ReleaseNotesError(f"missing curated changelog section for {version}")
    if len(matches) != 1:
        raise ReleaseNotesError(f"duplicate curated changelog section for {version}")
    if not headings or matches[0] != headings[0]:
        raise ReleaseNotesError(f"{version} is not the latest curated release section")
    start = matches[0].start()
    next_heading = re.search(r"^## ", text[matches[0].end() :], re.MULTILINE)
    end = matches[0].end() + next_heading.start() if next_heading else len(text)
    section = text[start:end].rstrip() + "\n"
    body = section.split("\n", 1)[1].strip()
    if not body:
        raise ReleaseNotesError(f"empty curated changelog section for {version}")
    return section


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--changelog", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        notes = extract_release_notes(args.changelog, args.version)
        args.output.write_text(notes, encoding="utf-8")
        return 0
    except (OSError, ReleaseNotesError) as error:
        print(f"release notes: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
