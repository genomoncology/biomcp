#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

BLOCKED_FAMILIES = {
    "cache",
    "chart",
    "cvx",
    "ddinter",
    "ema",
    "gtr",
    "who-ivd",
    "mcp",
    "serve",
    "serve-http",
    "serve-sse",
    "uninstall",
    "update",
    "who",
}
SPECIAL_FAMILIES = {"skill", "study"}
EXPECTED_STUDY_SUBCOMMANDS = {
    "co-occurrence",
    "cohort",
    "compare",
    "filter",
    "list",
    "query",
    "survival",
    "top-mutated",
}
EXPECTED_SKILL_BLOCKED_SUBCOMMANDS = {"install"}
EXPECTED_DESCRIPTION_BLOCKED_TERMS = {
    "`ema sync`",
    "`gtr sync`",
    "`who-ivd sync`",
    "`skill install`",
    "`uninstall`",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Audit BioMCP MCP allowlist coverage against CLI routing.",
    )
    parser.add_argument("--cli-file", type=Path, default=Path("src/cli/mod.rs"))
    parser.add_argument("--shell-file", type=Path, default=Path("src/mcp/shell.rs"))
    parser.add_argument("--build-file", type=Path, default=Path("build.rs"))
    parser.add_argument("--json", action="store_true")
    return parser.parse_args()


def camel_to_kebab(name: str) -> str:
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1-\2", name)
    value = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1-\2", value)
    return value.lower()


def extract_braced_block(text: str, marker: str) -> str:
    start = text.find(marker)
    if start == -1:
        raise ValueError(f"missing marker: {marker}")
    brace_start = text.find("{", start)
    if brace_start == -1:
        raise ValueError(f"missing opening brace after marker: {marker}")

    depth = 0
    for index in range(brace_start, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[brace_start + 1 : index]
    raise ValueError(f"unterminated braced block after marker: {marker}")


def parse_cli_families(cli_text: str) -> list[str]:
    enum_body = extract_braced_block(cli_text, "pub enum Commands")
    variants = re.findall(r"^\s*([A-Z][A-Za-z0-9]*)\s*(?:\{|,|\()", enum_body, flags=re.MULTILINE)
    if not variants:
        raise ValueError("failed to parse CLI command families from Commands enum")
    return sorted(camel_to_kebab(variant) for variant in variants)


def parse_cli_families_with_fallback(cli_file: Path) -> list[str]:
    cli_text = cli_file.read_text(encoding="utf-8")
    try:
        return parse_cli_families(cli_text)
    except ValueError as exc:
        sibling_commands = cli_file.with_name("commands.rs")
        if cli_file.name != "mod.rs" or not sibling_commands.exists():
            raise
        try:
            return parse_cli_families(sibling_commands.read_text(encoding="utf-8"))
        except ValueError:
            raise exc


def parse_allowed_families(shell_text: str) -> list[str]:
    match = re.search(
        r'match cmd\.as_str\(\)\s*\{\s*(?P<body>.*?)\s*"study"\s*=>\s*\{',
        shell_text,
        flags=re.DOTALL,
    )
    if match is None:
        raise ValueError("failed to parse MCP allowlisted top-level families")
    return sorted(set(re.findall(r'"([^"]+)"', match.group("body"))))


def parse_study_policy(shell_text: str) -> tuple[set[str], bool]:
    study_body = extract_braced_block(shell_text, '"study" =>')
    sub_match_body = extract_braced_block(study_body, "match sub.as_str()")
    allowed_match = re.search(r"(?P<body>.*?)\s*\"download\"\s*=>", sub_match_body, flags=re.DOTALL)
    if allowed_match is None:
        raise ValueError("failed to parse study allowlist body")
    allowed = set(re.findall(r'"([^"]+)"', allowed_match.group("body")))
    download_ok = 'args.len() == 4 && args[3] == "--list"' in sub_match_body
    return allowed, download_ok


def parse_skill_policy(shell_text: str) -> set[str]:
    skill_body = extract_braced_block(shell_text, '"skill" =>')
    match = re.search(r"matches!\(sub\.as_str\(\),\s*(?P<body>.*?)\)", skill_body, flags=re.DOTALL)
    if match is None:
        raise ValueError("failed to parse skill policy")
    return set(re.findall(r'"([^"]+)"', match.group("body")))


def check_description_policy(build_text: str) -> bool:
    required_markers = {
        "const MCP_SAFE_STUDY_PATTERN_LINE",
        "const MCP_SAFE_STUDY_DOWNLOAD_LINE",
        "const STUDY_PATTERN_LINE",
        "const STUDY_DOWNLOAD_LINE",
        "fn mcp_safe_description_line",
        "fn mcp_safe_list_reference",
    }
    return (
        re.search(r'\.trim_start\(\)\.starts_with\(\s*"- `update "\s*\)', build_text) is not None
        and all(term in build_text for term in EXPECTED_DESCRIPTION_BLOCKED_TERMS)
        and all(marker in build_text for marker in required_markers)
    )


def make_payload(cli_file: Path, shell_file: Path, build_file: Path) -> dict[str, object]:
    errors: list[str] = []
    try:
        shell_text = shell_file.read_text(encoding="utf-8")
        build_text = build_file.read_text(encoding="utf-8")

        cli_families = parse_cli_families_with_fallback(cli_file)
        allowed_families = parse_allowed_families(shell_text)
        study_allowed, study_download_ok = parse_study_policy(shell_text)
        skill_blocked = parse_skill_policy(shell_text)
        description_policy_ok = check_description_policy(build_text)
    except Exception as exc:  # noqa: BLE001
        errors.append(str(exc))
        cli_families = []
        allowed_families = []
        study_allowed = set()
        study_download_ok = False
        skill_blocked = set()
        description_policy_ok = False

    cli_family_set = set(cli_families)
    allowed_family_set = set(allowed_families)
    unclassified = sorted(cli_family_set - allowed_family_set - BLOCKED_FAMILIES - SPECIAL_FAMILIES)
    stale_allowlist = sorted(allowed_family_set - cli_family_set)
    study_policy_ok = study_allowed == EXPECTED_STUDY_SUBCOMMANDS and study_download_ok
    skill_policy_ok = skill_blocked == EXPECTED_SKILL_BLOCKED_SUBCOMMANDS

    if errors:
        status = "error"
    elif unclassified or stale_allowlist or not study_policy_ok or not skill_policy_ok or not description_policy_ok:
        status = "fail"
    else:
        status = "pass"

    return {
        "status": status,
        "cli_families": cli_families,
        "allowed_families": sorted(allowed_families),
        "blocked_families": sorted(BLOCKED_FAMILIES),
        "special_families": sorted(SPECIAL_FAMILIES),
        "unclassified_families": unclassified,
        "stale_allowlist_families": stale_allowlist,
        "study_policy_ok": study_policy_ok,
        "skill_policy_ok": skill_policy_ok,
        "description_policy_ok": description_policy_ok,
        "errors": errors,
    }


def main() -> int:
    args = parse_args()
    payload = make_payload(args.cli_file, args.shell_file, args.build_file)
    if args.json:
        json.dump(payload, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        print(f"status={payload['status']}")
        print(f"unclassified={','.join(payload['unclassified_families'])}")
        print(f"stale={','.join(payload['stale_allowlist_families'])}")
    return 0 if payload["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
