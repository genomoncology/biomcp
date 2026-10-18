#!/usr/bin/env python3
"""Refresh or check registry-owned facts in the licensing reference."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys

BLOCK = re.compile(
    r"<!-- licensing:([a-z0-9_-]+) -->\n(.*?)<!-- /licensing -->", re.DOTALL
)


def url(value: object) -> str:
    return f"<{value}>" if value else "none published"


def detail(entry: dict) -> str:
    lines = [
        f"### {entry['name']}",
        "",
        "- BioMCP surfaces: "
        + "; ".join(f"`{surface}`" for surface in entry["bioMcp_surfaces"]),
        f"- Integration mode: `{entry['integration_mode']}`",
        f"- BioMCP auth: `{entry['bioMcp_auth']}`",
        f"- Commercial terms: `{entry['commercial_terms'] or 'null'}`",
    ]
    if entry["via"]:
        lines.append(f"- Via: {entry['via']}")
    if entry["env_var"]:
        lines.append(f"- Credential environment variable: `{entry['env_var']}`")
    lines.extend(
        [
            f"- Provider access / registration: {entry['provider_access']}",
            f"- License / terms summary: {entry['license_summary']}",
            f"- Redistribution / reuse summary: {entry['redistribution_summary']}",
            f"- Official terms URL: {url(entry['terms_url'])}",
        ]
    )
    if entry["key_url"]:
        lines.append(f"- API key / account URL: {url(entry['key_url'])}")
    lines.extend(
        [f"- Reviewed on: `{entry['reviewed_on']}`", f"- Notes: {entry['notes']}"]
    )
    return "\n".join(lines) + "\n\n"


def table(entries: list[dict]) -> str:
    lines = [
        "| Source | Tier | Mode | BioMCP auth | License / terms summary | Redistribution summary | Terms URL |",
        "|---|---|---|---|---|---|---|",
    ]
    for entry in entries:
        cells = [
            entry[key]
            for key in (
                "name",
                "tier",
                "integration_mode",
                "bioMcp_auth",
                "license_summary",
                "redistribution_summary",
            )
        ]
        cells.append(url(entry["terms_url"]))
        lines.append(
            "| "
            + " | ".join(
                str(cell).replace("|", r"\|").replace("\n", "<br>") for cell in cells
            )
            + " |"
        )
    return "\n".join(lines) + "\n\n"


def expected_blocks(entries: list[dict]) -> dict[str, str]:
    blocks = {entry["id"]: detail(entry) for entry in entries}
    if len(blocks) != len(entries):
        raise ValueError("duplicate source ids in registry")
    blocks["summary"] = table(entries)
    groups = []
    for group in ("license-required", "non-commercial", "commercial-restricted"):
        names = [
            entry["name"]
            for entry in entries
            if entry["commercial_terms"] is not None
            and (group == "commercial-restricted" or entry["commercial_terms"] == group)
        ]
        groups.append(f"- `{group}`: {', '.join(names) or 'none'}.")
    blocks["groups"] = "\n".join(groups) + "\n\n"
    return blocks


def render(page: str, expected: dict[str, str]) -> str:
    found = [match[1] for match in BLOCK.finditer(page)]
    if len(found) != len(set(found)) or set(found) != set(expected):
        raise ValueError(
            f"licensing blocks must occur once per registry source plus summary/groups; missing={sorted(set(expected) - set(found))}, extra={sorted(set(found) - set(expected))}"
        )
    return BLOCK.sub(
        lambda match: f"<!-- licensing:{match[1]} -->\n{expected[match[1]]}<!-- /licensing -->",
        page,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--registry", type=Path, default=Path("docs/reference/sources.json")
    )
    parser.add_argument(
        "--page", type=Path, default=Path("docs/reference/source-licensing.md")
    )
    parser.add_argument(
        "--write",
        action="store_true",
        help="refresh generated blocks; default checks them",
    )
    args = parser.parse_args()
    try:
        entries = json.loads(args.registry.read_text(encoding="utf-8"))
        page = args.page.read_text(encoding="utf-8")
        expected = expected_blocks(entries)
        updated = render(page, expected)
        stale = [
            match[1] for match in BLOCK.finditer(page) if match[2] != expected[match[1]]
        ]
        if args.write:
            args.page.write_text(updated, encoding="utf-8")
        elif stale:
            print(
                f"stale licensing blocks: {', '.join(stale)}; run tools/render-source-licensing.py --write",
                file=sys.stderr,
            )
            return 1
    except (OSError, ValueError, KeyError) as error:
        print(f"licensing reference: {error}", file=sys.stderr)
        return 1
    print(f"licensing reference agrees with {len(entries)} registry sources")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
