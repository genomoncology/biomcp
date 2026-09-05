from __future__ import annotations

import json
import os
import re
import shlex
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
BIOMCP_BIN = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/release/biomcp"))
LONG_FLAG = re.compile(r"(?<![\w-])--[a-z][a-z0-9-]*")
TRIAL_SECTIONS = {"eligibility", "contacts", "locations", "outcomes", "arms", "references", "all"}
CAPABILITY_CONTRACT = REPO_ROOT / "sdlc/planning/clinical-trial-capabilities.md"
DECLARATION = re.compile(
    r"<!-- contract:(?P<name>[a-z-]+) -->\n```text\n(?P<values>.*?)\n```",
    re.DOTALL,
)


def _run_help(*args: str) -> str:
    assert BIOMCP_BIN.exists(), f"missing biomcp binary: {BIOMCP_BIN}"
    result = subprocess.run(
        [str(BIOMCP_BIN), *args],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def _section(text: str, start_heading: str, end_heading_prefix: str) -> str:
    capture = False
    lines: list[str] = []
    for line in text.splitlines():
        if line == start_heading:
            capture = True
            continue
        if capture and line.startswith(end_heading_prefix):
            break
        if capture:
            lines.append(line)
    return "\n".join(lines)


def _get_trial_examples(help_text: str) -> list[str]:
    examples = _section(help_text, "EXAMPLES:", "See also:")
    commands = [
        line.strip()
        for line in examples.splitlines()
        if line.strip().startswith("biomcp get trial ")
    ]
    assert commands, "get trial help should include copy-pasteable examples"
    return commands


def _flags_after_first_section(command: str) -> list[str]:
    tokens = shlex.split(command)
    seen_section = False
    late_flags: list[str] = []
    for token in tokens[4:]:
        if token in TRIAL_SECTIONS:
            seen_section = True
            continue
        if seen_section and token.startswith("--"):
            late_flags.append(token)
    return late_flags


def _declarations() -> dict[str, set[str]]:
    text = CAPABILITY_CONTRACT.read_text(encoding="utf-8")
    return {
        match.group("name"): {
            value.strip()
            for value in match.group("values").splitlines()
            if value.strip()
        }
        for match in DECLARATION.finditer(text)
    }


def _trial_tool_schema(tool_name: str) -> dict[str, object]:
    result = subprocess.run(
        [str(BIOMCP_BIN), "mcp", "tools"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    tools = json.loads(result.stdout)
    tool = next(tool for tool in tools if tool["name"] == tool_name)
    return next(
        branch
        for branch in tool["inputSchema"]["oneOf"]
        if branch["properties"]["entity"].get("const") == "trial"
    )


def _long_options(help_text: str) -> set[str]:
    options = _section(help_text, "Options:", "EXAMPLES:")
    return set(LONG_FLAG.findall(options))


def _cli_trial_sections(help_text: str) -> set[str]:
    match = re.search(r"Sections or document form \(([^)]+)\)", help_text)
    assert match, "get trial help should declare its section forms"
    return {
        value.strip().split(maxsplit=1)[0]
        for value in match.group(1).split(",")
    }


def _cli_trial_sources(help_text: str) -> set[str]:
    match = re.search(r"Trial data source \(([^)]+)\)", help_text)
    assert match, "trial help should declare its source names"
    return {value.strip() for value in match.group(1).split(" or ")}


def test_get_trial_help_examples_reference_only_declared_options() -> None:
    help_text = _run_help("get", "trial", "--help")
    examples = _section(help_text, "EXAMPLES:", "See also:")
    options = _section(help_text, "Options:", "EXAMPLES:")

    example_flags = set(LONG_FLAG.findall(examples))
    option_flags = set(LONG_FLAG.findall(options))

    missing = sorted(example_flags - option_flags)
    assert not missing, (
        "get trial help examples reference flags missing from the declared "
        f"Options block: {', '.join(missing)}"
    )


def test_get_trial_help_examples_place_options_before_sections() -> None:
    help_text = _run_help("get", "trial", "--help")
    late_flags_by_command = {
        command: _flags_after_first_section(command)
        for command in _get_trial_examples(help_text)
    }
    late_flags_by_command = {
        command: late_flags
        for command, late_flags in late_flags_by_command.items()
        if late_flags
    }

    assert not late_flags_by_command, (
        "get trial help examples place named options after section tokens, "
        f"which trailing section parsing treats as sections: {late_flags_by_command}"
    )


def test_trial_capability_declarations_match_cli_and_typed_mcp() -> None:
    declarations = _declarations()
    assert declarations["cli-search-flags"] == _long_options(
        _run_help("search", "trial", "--help")
    )

    search = _trial_tool_schema("search")
    get = _trial_tool_schema("get")
    assert declarations["typed-mcp-search-fields"] == set(search["properties"])
    assert declarations["typed-mcp-detail-sections"] == set(
        get["properties"]["sections"]["items"]["enum"]
    )

    get_help = _run_help("get", "trial", "--help")
    declared_cli_sections = declarations["cli-detail-sections"]
    assert declared_cli_sections == _cli_trial_sections(get_help)

    sources = declarations["trial-sources"]
    assert sources == set(search["properties"]["source"]["enum"])
    assert sources == _cli_trial_sources(_run_help("search", "trial", "--help"))

    exclusions = declarations["typed-mcp-cli-only-exclusions"]
    assert exclusions == {"document", "documents"}
    assert exclusions <= declared_cli_sections
    assert exclusions.isdisjoint(declarations["typed-mcp-detail-sections"])


def test_trial_capability_inventory_references_existing_contracts() -> None:
    text = CAPABILITY_CONTRACT.read_text(encoding="utf-8")
    rows = [line for line in text.splitlines() if re.match(r"^\| CT-[A-Z-]+ \|", line)]
    assert 1 <= len(rows) <= 15
    inventory: dict[str, list[str]] = {}

    for row in rows:
        cells = [cell.strip() for cell in row.strip("|").split("|")]
        assert len(cells) == 7, row
        capability, operation, sources, behavior, owner, contract, public_doc = cells
        inventory[capability] = cells
        assert operation and sources and behavior
        for reference in [*owner.split("<br>"), *public_doc.split("<br>")]:
            path = reference.split("#", 1)[0].strip("`")
            assert (REPO_ROOT / path).is_file(), reference

        spec_path, heading = contract.split("#", 1)
        spec_text = (REPO_ROOT / spec_path.strip("`")).read_text(encoding="utf-8")
        assert f"## {heading}" in spec_text, contract

    assert set(inventory["CT-PIVOTS"][4].split("<br>")) == {
        "src/cli/disease/dispatch.rs",
        "src/cli/drug/dispatch.rs",
        "src/cli/gene/related.rs",
        "src/cli/variant/dispatch.rs",
    }
    assert inventory["CT-PIVOTS"][5] == (
        "spec/entity/trial.md#Cross-entity Trial Pivot Commands"
    )
    assert inventory["CT-BATCH"][5] == "spec/entity/trial.md#Trial Batch Detail"
