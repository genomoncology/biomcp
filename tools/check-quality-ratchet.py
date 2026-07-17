#!/usr/bin/env python3
from __future__ import annotations

import argparse
import glob
import json
import re
import subprocess
import sys
from pathlib import Path

MUSTMATCH_JSON_RE = re.compile(r"(?:^|\|\s*)mustmatch\s+json\b")
SHORT_LIKE_RE = re.compile(r'(?:^|\|\s*)mustmatch\s+like\s+("([^"]*)"|\'([^\']*)\')')
MUSTMATCH_ASSERT_RE = re.compile(r"(?:^|[;&|]\s*)mustmatch\b")
CAPTURED_PRINTF_MUSTMATCH_RE = re.compile(
    r"\bprintf\b(?=[^|]*\"\$[A-Za-z_][A-Za-z0-9_]*\")[^|]*\|\s*mustmatch\b"
)
MUSTMATCH_LINT_SKIP = "<!-- mustmatch-lint: skip -->"
CLI_LINE_CAP = 700
SECTION_OUTCOME_POLICY_LINE_CAP = 700
SECTION_OUTCOME_POLICY_MODULES = ["src/entities/section_outcome.rs"]
CLI_LINE_CAP_TICKET_RE = re.compile(r"^\d+(?:[-_][a-z0-9][a-z0-9-]*)?$")
EXPERIMENT_RESULTS_GLOB = "architecture/experiments/**/results/**"
CLI_SURFACE_CONTRACT_CHECKS = [
    "public_flags_and_value_aliases_documented",
    "list_and_reference_docs_cover_public_commands",
    "runnable_helpers_are_discoverable_in_list_pages",
    "json_entity_surfaces_include_next_commands_or_exception",
    "copy_paste_examples_are_shell_safe",
    "entities_do_not_depend_on_markdown_shell_quoting",
]
CLI_SURFACE_EXCEPTION_REGISTRY = "tools/cli-surface-contract-exceptions.json"
CLI_SURFACE_REQUIRED_EXCEPTIONS = {
    "biomcp cache path": "plain_text_operator_path",
    "biomcp --json list": "command_reference_payload",
    "biomcp --json version": "release_identity_payload",
    "biomcp --json search all --counts-only": "current_counts_only_shape",
}
CLI_SURFACE_STATIC_TEXT_PATHS = [
    "src/cli/commands.rs",
    "src/cli/list/clinical.rs",
    "src/cli/list/helpers.rs",
    "src/cli/list/literature.rs",
    "src/cli/list/molecular.rs",
]
CLI_SURFACE_STATIC_TEXT_GLOBS = [
    "architecture/**/*.md",
    "docs/**/*.md",
    "spec/**/*.md",
]
TERMINAL_OUTPUT_BOUNDARY_SEAMS = {
    "src/render/human.rs": [
        "fn sanitize_document(value: &str)",
        "fn sanitize_inline(value: &str)",
    ],
    "src/cli/outcome.rs": [
        "outcome.text = crate::render::human::sanitize_document(&outcome.text)",
        "trusted_terminal_chart = is_charted_mcp_study_command",
    ],
    "src/cli/shared.rs": [
        "sanitize_document(&message)",
        "Err(err) => exit_human_clap_error(err, &args)",
    ],
    "src/main.rs": ["sanitize_human_diagnostic(&error.to_string())"],
    "src/mcp/shell.rs": [
        "sanitize_document(&text)",
        "sanitize_document(&content)",
        "sanitize_inline(&message.into())",
    ],
    "src/render/chart.rs": [
        "fn chart_text(value: &str)",
        "sanitize_inline(value)",
    ],
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run BioMCP's quality-ratchet audits and write JSON artifacts.",
    )
    parser.add_argument("--root-dir", type=Path, default=Path.cwd())
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--spec-glob", required=True)
    parser.add_argument("--cli-file", type=Path, required=True)
    parser.add_argument("--shell-file", type=Path, required=True)
    parser.add_argument("--build-file", type=Path, required=True)
    parser.add_argument("--sources-dir", type=Path, required=True)
    parser.add_argument("--sources-mod", type=Path, required=True)
    parser.add_argument("--health-file", type=Path, required=True)
    parser.add_argument("--cli-line-cap-allowlist", type=Path)
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def run_json_command(
    command: list[str], *, allowed_exit_codes: set[int]
) -> dict[str, object]:
    proc = subprocess.run(command, capture_output=True, text=True, check=False)
    if proc.returncode not in allowed_exit_codes:
        return {
            "status": "error",
            "command": command,
            "exit_code": proc.returncode,
            "stdout": proc.stdout,
            "stderr": proc.stderr,
            "errors": [f"unexpected exit code {proc.returncode}"],
        }
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        return {
            "status": "error",
            "command": command,
            "exit_code": proc.returncode,
            "stdout": proc.stdout,
            "stderr": proc.stderr,
            "errors": [f"invalid JSON output: {exc}"],
        }
    payload["exit_code"] = proc.returncode
    if proc.stderr:
        payload["stderr"] = proc.stderr
    return payload


def tracked_cli_rust_files(root_dir: Path) -> tuple[list[str], list[str]]:
    proc = subprocess.run(
        [
            "git",
            "-C",
            str(root_dir),
            "ls-files",
            "--",
            "src/cli/*.rs",
            "src/cli/**/*.rs",
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        return [], [proc.stderr.strip() or "git ls-files failed"]
    return sorted({line for line in proc.stdout.splitlines() if line}), []


def tracked_rust_files(root_dir: Path) -> tuple[list[str], list[str]]:
    proc = subprocess.run(
        ["git", "-C", str(root_dir), "ls-files", "--", "src/*.rs", "src/**/*.rs"],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        return [], [proc.stderr.strip() or "git ls-files failed"]
    return sorted({line for line in proc.stdout.splitlines() if line}), []


def check_dead_code_allowances(root_dir: Path) -> dict[str, object]:
    tracked_files, errors = tracked_rust_files(root_dir)
    if errors:
        return {"status": "error", "findings": [], "errors": errors}

    findings: list[dict[str, object]] = []
    spans_checked = 0
    reason_re = re.compile(r"^\s*//.*dead-code reason:\s*\S")
    for relative_path in tracked_files:
        path = root_dir / relative_path
        if not path.is_file():
            continue
        lines = path.read_text(encoding="utf-8").splitlines()
        line_index = 0
        while line_index < len(lines):
            stripped = lines[line_index].lstrip()
            if not (stripped.startswith("#[") or stripped.startswith("#![")):
                line_index += 1
                continue

            opening_line = line_index
            span_lines = [lines[line_index]]
            depth = lines[line_index].count("[") - lines[line_index].count("]")
            while depth > 0 and line_index + 1 < len(lines):
                line_index += 1
                span_lines.append(lines[line_index])
                depth += lines[line_index].count("[") - lines[line_index].count("]")

            span = "\n".join(span_lines)
            attribute_tokens = re.sub(r'/\*.*?\*/|//[^\n]*', '', span, flags=re.DOTALL)
            attribute_tokens = re.sub(
                r'r(?P<hashes>#+)".*?"(?P=hashes)|"(?:\\.|[^"\\])*"',
                '',
                attribute_tokens,
                flags=re.DOTALL,
            )
            if re.search(r"\ballow\s*\(.*?\bdead_code\b", attribute_tokens, re.DOTALL):
                spans_checked += 1
                previous = lines[opening_line - 1] if opening_line else ""
                if not reason_re.match(previous):
                    findings.append(
                        {
                            "path": relative_path,
                            "line": opening_line + 1,
                            "message": (
                                "dead-code allowance requires an adjacent concrete "
                                "`dead-code reason:` comment"
                            ),
                        }
                    )
            line_index += 1

    return {
        "status": "fail" if findings else "pass",
        "files_checked": len(tracked_files),
        "allowances_checked": spans_checked,
        "finding_count": len(findings),
        "findings": findings,
        "errors": [],
    }


def tracked_experiment_result_files(root_dir: Path) -> tuple[list[str], list[str]]:
    proc = subprocess.run(
        [
            "git",
            "-C",
            str(root_dir),
            "ls-files",
            "--",
            EXPERIMENT_RESULTS_GLOB,
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        return [], [proc.stderr.strip() or "git ls-files failed"]
    return sorted({line for line in proc.stdout.splitlines() if line}), []


def check_architecture_experiment_results(root_dir: Path) -> dict[str, object]:
    tracked_files, errors = tracked_experiment_result_files(root_dir)
    if errors:
        return {
            "status": "error",
            "glob": EXPERIMENT_RESULTS_GLOB,
            "errors": errors,
        }

    findings = [
        {
            "path": relative_path,
            "message": (
                "generated experiment result payloads must stay out of git; "
                "keep writeups/scripts in architecture/experiments and store "
                "run outputs as untracked local artifacts"
            ),
        }
        for relative_path in tracked_files
    ]
    return {
        "status": "fail" if findings else "pass",
        "glob": EXPERIMENT_RESULTS_GLOB,
        "files_checked": len(tracked_files),
        "finding_count": len(findings),
        "findings": findings,
        "errors": [],
    }


def _rust_production_text(text: str) -> str:
    """Remove test-only Rust modules and comments from static-check input."""
    cfg_test = re.compile(r"#\[cfg\(test\)\]\s*mod\s+\w+\s*\{")
    while match := cfg_test.search(text):
        brace = match.end() - 1
        depth = 1
        cursor = brace + 1
        while cursor < len(text) and depth:
            depth += (text[cursor] == "{") - (text[cursor] == "}")
            cursor += 1
        text = text[: match.start()] + (" " * (cursor - match.start())) + text[cursor:]
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.DOTALL)
    return re.sub(r"//[^\n]*", "", text)


def check_terminal_output_boundaries(root_dir: Path) -> dict[str, object]:
    findings: list[dict[str, object]] = []
    checked_surfaces: list[str] = []

    for relative_path, markers in TERMINAL_OUTPUT_BOUNDARY_SEAMS.items():
        path = root_dir / relative_path
        checked_surfaces.append(relative_path)
        try:
            text = _rust_production_text(path.read_text(encoding="utf-8"))
        except OSError as exc:
            findings.append({"path": relative_path, "message": f"required output boundary is unreadable: {exc}"})
            continue
        for marker in markers:
            if marker not in text:
                findings.append({
                    "path": relative_path,
                    "marker": marker,
                    "message": "required terminal-output sanitization seam is missing",
                })

    src_dir = root_dir / "src"
    for path in sorted(src_dir.rglob("*.rs")):
        relative_path = path.relative_to(root_dir).as_posix()
        checked_surfaces.append(relative_path)
        if relative_path == "src/render/json.rs" or "tests" in path.parts:
            continue
        production_text = _rust_production_text(path.read_text(encoding="utf-8"))
        for match in re.finditer(r"\bto_string_pretty\b", production_text):
            findings.append({
                "path": relative_path,
                "line": production_text.count("\n", 0, match.start()) + 1,
                "message": "production pretty JSON must route through src/render/json.rs",
            })

    return {
        "name": "terminal_output_boundaries",
        "status": "fail" if findings else "pass",
        "checked_surfaces": sorted(set(checked_surfaces)),
        "finding_count": len(findings),
        "findings": findings,
        "errors": [],
    }


def load_cli_line_cap_allowlist(
    allowlist_path: Path,
) -> tuple[dict[str, dict[str, object]], list[str]]:
    try:
        payload = json.loads(allowlist_path.read_text(encoding="utf-8"))
    except OSError as exc:
        return {}, [f"failed to read allowlist {allowlist_path}: {exc}"]
    except json.JSONDecodeError as exc:
        return {}, [f"invalid allowlist JSON {allowlist_path}: {exc}"]

    if not isinstance(payload, dict):
        return {}, ["allowlist root must be a JSON object"]
    if payload.get("cap") != CLI_LINE_CAP:
        return {}, [f"allowlist cap must be {CLI_LINE_CAP}"]

    entries = payload.get("entries")
    if not isinstance(entries, list):
        return {}, ["allowlist entries must be a list"]

    allowlist: dict[str, dict[str, object]] = {}
    errors: list[str] = []
    for index, entry in enumerate(entries):
        prefix = f"allowlist entries[{index}]"
        if not isinstance(entry, dict):
            errors.append(f"{prefix} must be an object")
            continue

        path = entry.get("path")
        lines = entry.get("lines")
        date = entry.get("date")
        follow_up_ticket = entry.get("follow_up_ticket")
        if (
            not isinstance(path, str)
            or not path.startswith("src/cli/")
            or not path.endswith(".rs")
        ):
            errors.append(f"{prefix}.path must be a src/cli/*.rs path")
            continue
        if path in allowlist:
            errors.append(f"duplicate allowlist path: {path}")
            continue
        if not isinstance(lines, int) or lines <= CLI_LINE_CAP:
            errors.append(
                f"{prefix}.lines must be an integer greater than {CLI_LINE_CAP}"
            )
        if not isinstance(date, str) or not re.fullmatch(r"\d{4}-\d{2}-\d{2}", date):
            errors.append(f"{prefix}.date must be YYYY-MM-DD")
        if (
            not isinstance(follow_up_ticket, str)
            or not CLI_LINE_CAP_TICKET_RE.fullmatch(follow_up_ticket)
        ):
            errors.append(
                f"{prefix}.follow_up_ticket must be a ticket number or ticket slug"
            )
        allowlist[path] = entry

    return allowlist, errors


def check_cli_line_cap(root_dir: Path, allowlist_path: Path) -> dict[str, object]:
    allowlist, errors = load_cli_line_cap_allowlist(allowlist_path)
    tracked_files, git_errors = tracked_cli_rust_files(root_dir)
    errors.extend(git_errors)
    if errors:
        return {
            "status": "error",
            "cap": CLI_LINE_CAP,
            "allowlist": str(allowlist_path),
            "errors": errors,
        }

    missing_allowlist_entries: list[dict[str, object]] = []
    grown_allowlist_entries: list[dict[str, object]] = []
    over_cap_files: list[dict[str, object]] = []
    stale_allowlist_entries: list[dict[str, object]] = []

    tracked_set = set(tracked_files)
    for relative_path in tracked_files:
        path = root_dir / relative_path
        line_count = len(path.read_text(encoding="utf-8").splitlines())
        if line_count <= CLI_LINE_CAP:
            continue

        finding = {"path": relative_path, "lines": line_count}
        over_cap_files.append(finding)
        entry = allowlist.get(relative_path)
        if entry is None:
            missing_allowlist_entries.append(
                {
                    **finding,
                    "message": (
                        f"tracked src/cli Rust file exceeds {CLI_LINE_CAP} lines "
                        "without an allowlist entry"
                    ),
                }
            )
            continue

        allowed_lines = entry["lines"]
        if isinstance(allowed_lines, int) and line_count > allowed_lines:
            grown_allowlist_entries.append(
                {
                    **finding,
                    "allowed_lines": allowed_lines,
                    "follow_up_ticket": entry.get("follow_up_ticket"),
                    "message": (
                        "allowlisted file grew beyond its recorded line count; "
                        "decompose it instead of expanding the allowlist"
                    ),
                }
            )

    for relative_path, entry in allowlist.items():
        if relative_path not in tracked_set:
            stale_allowlist_entries.append(
                {
                    "path": relative_path,
                    "lines": entry.get("lines"),
                    "follow_up_ticket": entry.get("follow_up_ticket"),
                    "message": (
                        "allowlist entry no longer points to a tracked "
                        "src/cli Rust file"
                    ),
                }
            )
            continue

        line_count = len(
            (root_dir / relative_path).read_text(encoding="utf-8").splitlines()
        )
        if line_count <= CLI_LINE_CAP:
            stale_allowlist_entries.append(
                {
                    "path": relative_path,
                    "lines": line_count,
                    "follow_up_ticket": entry.get("follow_up_ticket"),
                    "message": "allowlist entry is no longer needed; remove it",
                }
            )

    status = (
        "fail"
        if missing_allowlist_entries
        or grown_allowlist_entries
        or stale_allowlist_entries
        else "pass"
    )
    return {
        "status": status,
        "cap": CLI_LINE_CAP,
        "allowlist": str(allowlist_path),
        "files_checked": len(tracked_files),
        "over_cap_count": len(over_cap_files),
        "allowlist_count": len(allowlist),
        "over_cap_files": over_cap_files,
        "missing_allowlist_entries": missing_allowlist_entries,
        "grown_allowlist_entries": grown_allowlist_entries,
        "stale_allowlist_entries": stale_allowlist_entries,
        "errors": [],
    }




def load_cli_surface_exceptions(root_dir: Path) -> tuple[list[dict[str, object]], list[str]]:
    registry_path = root_dir / CLI_SURFACE_EXCEPTION_REGISTRY
    try:
        payload = json.loads(registry_path.read_text(encoding="utf-8"))
    except OSError as exc:
        return [], [f"failed to read exception registry {CLI_SURFACE_EXCEPTION_REGISTRY}: {exc}"]
    except json.JSONDecodeError as exc:
        return [], [f"invalid exception registry {CLI_SURFACE_EXCEPTION_REGISTRY}: {exc}"]

    errors: list[str] = []
    if not isinstance(payload, dict):
        return [], ["exception registry root must be a JSON object"]
    if payload.get("schema") != "biomcp-cli-surface-contract-exceptions-v1":
        errors.append("exception registry schema must be biomcp-cli-surface-contract-exceptions-v1")
    entries = payload.get("entries")
    if not isinstance(entries, list):
        return [], errors + ["exception registry entries must be a list"]

    by_command: dict[str, dict[str, object]] = {}
    for index, entry in enumerate(entries):
        prefix = f"entries[{index}]"
        if not isinstance(entry, dict):
            errors.append(f"{prefix} must be an object")
            continue
        command = entry.get("command")
        exception = entry.get("exception")
        reason = entry.get("reason")
        owner_test = entry.get("owner_test")
        if not isinstance(command, str) or not command.startswith("biomcp "):
            errors.append(f"{prefix}.command must be a biomcp command string")
            continue
        if command in by_command:
            errors.append(f"duplicate exception command: {command}")
            continue
        if not isinstance(exception, str) or not exception.strip():
            errors.append(f"{prefix}.exception must be a non-empty string")
        if not isinstance(reason, str) or not reason.strip():
            errors.append(f"{prefix}.reason must be a non-empty string")
        if not isinstance(owner_test, str) or not owner_test.startswith("tests/"):
            errors.append(f"{prefix}.owner_test must point at a local test")
        by_command[command] = entry

    for command, exception in CLI_SURFACE_REQUIRED_EXCEPTIONS.items():
        entry = by_command.get(command)
        if entry is None:
            errors.append(f"missing required CLI surface exception: {command}")
        elif entry.get("exception") != exception:
            errors.append(f"{command} exception must be {exception}")

    return list(by_command.values()), errors


def tracked_static_text_paths(root_dir: Path) -> tuple[list[str], list[str]]:
    proc = subprocess.run(
        [
            "git",
            "-C",
            str(root_dir),
            "ls-files",
            "--",
            *CLI_SURFACE_STATIC_TEXT_PATHS,
            *CLI_SURFACE_STATIC_TEXT_GLOBS,
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        return [], [proc.stderr.strip() or "git ls-files failed"]

    return sorted({line for line in proc.stdout.splitlines() if line}), []


def read_existing_text(root_dir: Path, paths: list[str]) -> tuple[dict[str, str], list[str]]:
    texts: dict[str, str] = {}
    errors: list[str] = []
    for relative in paths:
        path = root_dir / relative
        try:
            texts[relative] = path.read_text(encoding="utf-8")
        except OSError as exc:
            errors.append(f"failed to read {relative}: {exc}")
    return texts, errors


def documented_token_corpus(texts: dict[str, str]) -> str:
    return "\n".join(texts.values()).lower()


def check_public_flags_and_value_aliases_documented(root_dir: Path, texts: dict[str, str]) -> dict[str, object]:
    rust_text = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted((root_dir / "src" / "cli").glob("**/*.rs"))
    )
    alias_tokens = sorted(
        {
            match.group(1)
            for match in re.finditer(r'visible_alias\s*=\s*"([^"]+)"', rust_text)
        }
        | {
            match.group(1)
            for match in re.finditer(r"visible_short_alias\s*=\s*'([^']+)'", rust_text)
        }
        | {
            match.group(1)
            for match in re.finditer(r'#\[value\([^\]]*alias\s*=\s*"([^"]+)"', rust_text)
        }
    )
    corpus = documented_token_corpus(texts)
    findings = [
        {
            "token": token,
            "message": "public visible/value alias is accepted by clap but absent from help/list/docs/spec evidence",
        }
        for token in alias_tokens
        if token.lower() not in corpus
    ]
    return {
        "name": "public_flags_and_value_aliases_documented",
        "status": "fail" if findings else "pass",
        "checked_surfaces": ["src/cli/**/*.rs", *texts.keys()],
        "tokens_checked": alias_tokens,
        "findings": findings,
    }


def check_list_and_reference_docs_cover_public_commands(root_dir: Path, texts: dict[str, str]) -> dict[str, object]:
    list_mod = (root_dir / "src" / "cli" / "list" / "mod.rs").read_text(encoding="utf-8")
    commands = sorted(
        {
            match.group(1)
            for match in re.finditer(r'Some\("([a-z0-9-]+)"\)\s*=>', list_mod)
            if match.group(1) not in {"_"}
        }
    )
    docs_text = "\n".join(
        texts.get(path, "")
        for path in ["architecture/ux/cli-reference.md", "docs/user-guide/cli-reference.md"]
    ).lower()
    findings: list[dict[str, object]] = []
    for command in commands:
        command_words = command.replace("-", " ")
        patterns = [
            f"biomcp list {command}",
            f"search {command}",
            f"search {command_words}",
            f"get {command}",
            f"get {command_words}",
            f"biomcp {command}",
            f"biomcp {command_words}",
        ]
        if not any(pattern in docs_text for pattern in patterns):
            findings.append({"command": command, "surface": "CLI reference docs", "message": "command/helper is missing from reference docs"})
    return {
        "name": "list_and_reference_docs_cover_public_commands",
        "status": "fail" if findings else "pass",
        "checked_surfaces": ["src/cli/list/mod.rs", "src/cli/list/*.rs", "architecture/ux/cli-reference.md", "docs/user-guide/cli-reference.md"],
        "commands_checked": commands,
        "findings": findings,
    }


def pascal_to_kebab(value: str) -> str:
    return re.sub(r"(?<!^)([A-Z])", r"-\1", value).lower()


def runnable_helper_commands(root_dir: Path) -> dict[str, list[str]]:
    helper_files = {
        "drug": root_dir / "src" / "cli" / "drug" / "mod.rs",
        "disease": root_dir / "src" / "cli" / "disease" / "mod.rs",
        "variant": root_dir / "src" / "cli" / "variant" / "mod.rs",
    }
    commands: dict[str, list[str]] = {}
    for entity, path in helper_files.items():
        text = path.read_text(encoding="utf-8")
        enum_name = f"{entity.title()}Command"
        match = re.search(rf"pub enum {enum_name} \{{(?P<body>.*?)\n\}}", text, re.DOTALL)
        if match is None:
            commands[entity] = []
            continue
        helpers = []
        for variant in re.findall(r"^    ([A-Z][A-Za-z0-9]*)\b", match.group("body"), re.MULTILINE):
            if variant == "External":
                continue
            helpers.append(f"{entity} {pascal_to_kebab(variant)}")
        commands[entity] = sorted(helpers)
    return commands


def check_runnable_helpers_are_discoverable_in_list_pages(root_dir: Path) -> dict[str, object]:
    list_files = {
        "drug": root_dir / "src" / "cli" / "list" / "clinical.rs",
        "disease": root_dir / "src" / "cli" / "list" / "clinical.rs",
        "variant": root_dir / "src" / "cli" / "list" / "molecular.rs",
    }
    findings: list[dict[str, object]] = []
    commands = runnable_helper_commands(root_dir)
    for entity, helper_commands in commands.items():
        list_text = list_files[entity].read_text(encoding="utf-8").lower()
        for command in helper_commands:
            if command.lower() not in list_text:
                findings.append({
                    "command": f"biomcp {command}",
                    "surface": f"biomcp list {entity}",
                    "message": "runnable helper command is missing from matching list page discovery text",
                })
    return {
        "name": "runnable_helpers_are_discoverable_in_list_pages",
        "status": "fail" if findings else "pass",
        "checked_surfaces": [
            "src/cli/drug/mod.rs",
            "src/cli/disease/mod.rs",
            "src/cli/variant/mod.rs",
            "src/cli/list/clinical.rs",
            "src/cli/list/molecular.rs",
        ],
        "commands_checked": commands,
        "findings": findings,
    }


def check_json_next_commands(root_dir: Path, exceptions: list[dict[str, object]]) -> dict[str, object]:
    shared = (root_dir / "src" / "cli" / "shared.rs").read_text(encoding="utf-8")
    render_json = (root_dir / "src" / "render" / "json.rs").read_text(encoding="utf-8")
    findings: list[dict[str, object]] = []
    required_source_markers = ["SearchJsonMeta", "next_commands", "search_json_with_meta"]
    for marker in required_source_markers:
        if marker not in shared:
            findings.append({"source": "src/cli/shared.rs", "marker": marker, "message": "search JSON metadata seam is missing"})
    for marker in ["_meta", "next_commands", "to_entity_json_value", "to_discover_json"]:
        if marker not in render_json:
            findings.append({"source": "src/render/json.rs", "marker": marker, "message": "entity/discover JSON metadata seam is missing"})
    return {
        "name": "json_entity_surfaces_include_next_commands_or_exception",
        "status": "fail" if findings else "pass",
        "checked_surfaces": ["src/cli/shared.rs", "src/render/json.rs"],
        "applied_exceptions": [entry for entry in exceptions if isinstance(entry.get("command"), str) and "--json" in str(entry.get("command"))],
        "findings": findings,
    }


def shell_has_unquoted_redirect(line: str) -> bool:
    in_single = False
    in_double = False
    escaped = False
    for char in line:
        if escaped:
            escaped = False
            continue
        if char == "\\" and not in_single:
            escaped = True
            continue
        if char == "'" and not in_double:
            in_single = not in_single
            continue
        if char == '"' and not in_single:
            in_double = not in_double
            continue
        if char == ">" and not in_single and not in_double:
            return True
    return False


def check_copy_paste_examples_are_shell_safe(root_dir: Path, texts: dict[str, str]) -> dict[str, object]:
    findings: list[dict[str, object]] = []
    for relative, text in texts.items():
        for lineno, line in enumerate(text.splitlines(), start=1):
            stripped = line.strip()
            if "biomcp " not in stripped or ">" not in stripped:
                continue
            if "→" in stripped:
                continue
            if not re.search(r'[A-Z]{2}_[0-9.]+:[cgmnpr]\.[^\s\'\"]*>', stripped):
                continue
            if shell_has_unquoted_redirect(stripped):
                findings.append({
                    "path": relative,
                    "line": lineno,
                    "text": stripped,
                    "message": "copy-pasteable biomcp example contains an unquoted shell redirection metacharacter",
                })
    return {
        "name": "copy_paste_examples_are_shell_safe",
        "status": "fail" if findings else "pass",
        "checked_surfaces": list(texts.keys()),
        "findings": findings,
    }


def check_entity_markdown_quoting_dependencies(root_dir: Path) -> dict[str, object]:
    findings: list[dict[str, object]] = []
    patterns = {
        "crate::render::markdown::quote_arg": re.compile(
            r"crate::render::markdown::quote_arg"
            r"|use\s+crate::render::markdown::quote_arg\s*;"
            r"|use\s+crate::render::markdown::\{[^}]*\bquote_arg\b[^}]*\}",
            flags=re.DOTALL,
        ),
        "crate::render::markdown::shell_quote_arg": re.compile(
            r"crate::render::markdown::shell_quote_arg"
            r"|use\s+crate::render::markdown::shell_quote_arg\s*;"
            r"|use\s+crate::render::markdown::\{[^}]*\bshell_quote_arg\b[^}]*\}",
            flags=re.DOTALL,
        ),
    }
    entity_root = root_dir / "src" / "entities"
    for path in sorted(entity_root.rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        relative = path.relative_to(root_dir).as_posix()
        for pattern, regex in patterns.items():
            for match in regex.finditer(text):
                findings.append({
                    "path": relative,
                    "line": text.count("\n", 0, match.start()) + 1,
                    "pattern": pattern,
                    "message": "entity code must build typed next commands instead of importing markdown shell quoting helpers",
                })
    return {
        "name": "entities_do_not_depend_on_markdown_shell_quoting",
        "status": "fail" if findings else "pass",
        "checked_surfaces": ["src/entities/**/*.rs"],
        "findings": findings,
    }


def check_cli_surface_contract(root_dir: Path) -> dict[str, object]:
    exceptions, errors = load_cli_surface_exceptions(root_dir)
    text_paths, path_errors = tracked_static_text_paths(root_dir)
    errors.extend(path_errors)
    texts, text_errors = read_existing_text(root_dir, text_paths)
    errors.extend(text_errors)
    if errors:
        return {
            "status": "error",
            "exception_registry": CLI_SURFACE_EXCEPTION_REGISTRY,
            "checks": CLI_SURFACE_CONTRACT_CHECKS,
            "errors": errors,
        }

    check_payloads = [
        check_public_flags_and_value_aliases_documented(root_dir, texts),
        check_list_and_reference_docs_cover_public_commands(root_dir, texts),
        check_runnable_helpers_are_discoverable_in_list_pages(root_dir),
        check_json_next_commands(root_dir, exceptions),
        check_copy_paste_examples_are_shell_safe(root_dir, texts),
        check_entity_markdown_quoting_dependencies(root_dir),
    ]
    statuses = [payload["status"] for payload in check_payloads]
    status = "pass" if all(value == "pass" for value in statuses) else "fail"
    findings = [
        finding
        for payload in check_payloads
        for finding in payload.get("findings", [])
        if isinstance(finding, dict)
    ]
    return {
        "status": status,
        "exception_registry": CLI_SURFACE_EXCEPTION_REGISTRY,
        "checks": CLI_SURFACE_CONTRACT_CHECKS,
        "checked_surfaces": sorted({
            surface
            for payload in check_payloads
            for surface in payload.get("checked_surfaces", [])
            if isinstance(surface, str)
        }),
        "applied_exceptions": exceptions,
        "finding_count": len(findings),
        "findings": findings,
        "results": check_payloads,
        "errors": [],
    }

def make_repo_compatibility_findings(
    spec_path: Path, *, min_like_len: int = 10
) -> list[dict[str, object]]:
    findings: list[dict[str, object]] = []
    text = spec_path.read_text(encoding="utf-8")

    for lineno, line in enumerate(text.splitlines(), start=1):
        if MUSTMATCH_JSON_RE.search(line):
            findings.append(
                {
                    "line": lineno,
                    "rule": "invalid-mustmatch-mode",
                    "message": "uses unsupported `mustmatch json` syntax",
                    "text": line.strip(),
                }
            )

        match = SHORT_LIKE_RE.search(line)
        if match is None:
            continue
        literal = match.group(2) if match.group(2) is not None else match.group(3)
        if literal is not None and len(literal) < min_like_len:
            findings.append(
                {
                    "line": lineno,
                    "rule": "short-like-pattern",
                    "message": (
                        f'uses short `mustmatch like` literal "{literal}" '
                        f"({len(literal)} chars)"
                    ),
                    "text": line.strip(),
                }
            )

    return findings


def make_captured_output_mustmatch_findings(spec_path: Path) -> list[dict[str, object]]:
    findings: list[dict[str, object]] = []
    text = spec_path.read_text(encoding="utf-8")

    for lineno, line in enumerate(text.splitlines(), start=1):
        if CAPTURED_PRINTF_MUSTMATCH_RE.search(line):
            findings.append(
                {
                    "line": lineno,
                    "rule": "captured-output-mustmatch-pipe",
                    "message": (
                        "pipes captured command output into mustmatch via printf; "
                        "pipe the command directly into mustmatch instead"
                    ),
                    "text": line.strip(),
                }
            )

    return findings


def make_missing_bash_mustmatch_findings(spec_path: Path) -> list[dict[str, object]]:
    findings: list[dict[str, object]] = []
    text = spec_path.read_text(encoding="utf-8")

    current_section: dict[str, object] | None = None
    inside_fence = False
    inside_bash = False
    skipped_bash = False

    def flush_section() -> None:
        nonlocal current_section
        if current_section is None:
            return
        if (
            current_section["has_non_skipped_bash"]
            and not current_section["has_mustmatch"]
            and not current_section["opted_out"]
        ):
            findings.append(
                {
                    "line": current_section["line"],
                    "rule": "missing-bash-mustmatch",
                    "section": current_section["section"],
                    "message": (
                        "section has non-skipped bash blocks but no `mustmatch` "
                        "assertion and no `<!-- mustmatch-lint: skip -->` opt-out"
                    ),
                    "text": current_section["text"],
                }
            )
        current_section = None

    for lineno, line in enumerate(text.splitlines(), start=1):
        if inside_fence:
            if line.strip() == "```":
                inside_fence = False
                inside_bash = False
                skipped_bash = False
                continue
            if (
                current_section is not None
                and inside_bash
                and not skipped_bash
                and MUSTMATCH_ASSERT_RE.search(line)
            ):
                current_section["has_mustmatch"] = True
            continue

        if line.startswith("## "):
            flush_section()
            current_section = {
                "line": lineno,
                "rule": "missing-bash-mustmatch",
                "section": line[3:].strip(),
                "text": line.strip(),
                "has_non_skipped_bash": False,
                "has_mustmatch": False,
                "opted_out": False,
            }
            continue

        if line.startswith("```"):
            inside_fence = True
            fence_tokens = line[3:].strip().split()
            inside_bash = bool(fence_tokens) and fence_tokens[0] == "bash"
            skipped_bash = inside_bash and "skip" in fence_tokens[1:]
            if current_section is not None and inside_bash and not skipped_bash:
                current_section["has_non_skipped_bash"] = True
            continue

        if current_section is not None and MUSTMATCH_LINT_SKIP in line:
            current_section["opted_out"] = True

    flush_section()
    return findings


def lint_spec_file(spec_path: Path) -> dict[str, object]:
    payload = run_json_command(
        [
            "mustmatch",
            "lint",
            str(spec_path),
            "--min-like-len",
            "10",
            "--json",
        ],
        allowed_exit_codes={0, 1},
    )
    if payload.get("status") == "error":
        return payload

    findings = payload.get("findings")
    if not isinstance(findings, list):
        return {
            "status": "error",
            "spec": str(spec_path),
            "errors": ["mustmatch lint payload missing findings list"],
        }

    seen = {
        (finding.get("line"), finding.get("rule"), finding.get("text"))
        for finding in findings
        if isinstance(finding, dict)
    }
    for finding in make_repo_compatibility_findings(spec_path):
        key = (finding["line"], finding["rule"], finding["text"])
        if key not in seen:
            findings.append(finding)
            seen.add(key)
    for finding in make_missing_bash_mustmatch_findings(spec_path):
        key = (finding["line"], finding["rule"], finding["text"])
        if key not in seen:
            findings.append(finding)
            seen.add(key)
    for finding in make_captured_output_mustmatch_findings(spec_path):
        key = (finding["line"], finding["rule"], finding["text"])
        if key not in seen:
            findings.append(finding)
            seen.add(key)

    payload["finding_count"] = len(findings)
    payload["status"] = "fail" if findings else "pass"
    return payload


def resolve_spec_paths(spec_glob: str) -> list[Path]:
    return sorted(
        Path(path).resolve()
        for path in glob.glob(spec_glob, recursive=True)
        if Path(path).is_file()
    )


def lint_specs(spec_paths: list[Path], spec_glob: str) -> dict[str, object]:
    lint_results: list[dict[str, object]] = []
    lint_errors: list[str] = []

    for spec_path in spec_paths:
        try:
            payload = lint_spec_file(spec_path)
        except Exception as exc:  # noqa: BLE001
            lint_errors.append(f"{spec_path}: {exc}")
            continue

        if payload.get("status") == "error":
            errors = payload.get("errors", [])
            if isinstance(errors, list) and errors:
                lint_errors.extend(
                    f"{spec_path}: {error}"
                    for error in errors
                    if isinstance(error, str)
                )
            else:
                lint_errors.append(f"{spec_path}: lint command failed")
            continue

        lint_results.append(payload)

    finding_count = sum(
        payload.get("finding_count", 0)
        for payload in lint_results
        if isinstance(payload.get("finding_count"), int)
    )

    if not spec_paths:
        lint_status = "error"
        lint_errors.append(f"no spec files matched {spec_glob!r}")
    elif lint_errors:
        lint_status = "error"
    elif finding_count:
        lint_status = "fail"
    else:
        lint_status = "pass"

    return {
        "status": lint_status,
        "baseline_count": 0,
        "finding_count": finding_count,
        "files_checked": len(spec_paths),
        "results": lint_results,
        "errors": lint_errors,
    }


def check_remote_resource_bounds(root_dir: Path) -> dict[str, object]:
    sources = root_dir / "src" / "sources"
    findings: list[str] = []

    def production_text(name: str) -> str:
        path = sources / name
        if not path.is_file():
            return ""
        return _rust_production_text(path.read_text(encoding="utf-8"))

    shared = production_text("mod.rs")
    migration = shared.find("ensure_body_limited_cache_epoch")
    cache = shared.find("ClientBuilder::new(base_client).with(Cache(")
    limiter = shared.find(".with(ResponseBodyLimitMiddleware", max(cache, 0))
    if migration < 0 or cache < 0 or migration > cache:
        findings.append(
            "src/sources/mod.rs: legacy HTTP cache epoch must be enforced before shared cache construction"
        )
    if cache < 0 or limiter < 0 or limiter < cache:
        findings.append(
            "src/sources/mod.rs: response body limiter must remain inside the cache middleware"
        )

    cbioportal = production_text("cbioportal_download.rs")
    compact_cbioportal = re.sub(r"\s+", "", cbioportal)
    configured_limit = compact_cbioportal.find(
        "max_archive_download_bytes:MAX_ARCHIVE_DOWNLOAD_BYTES"
    )
    declared_limit = compact_cbioportal.find(
        ".content_length().is_some_and(|length|length>self.max_archive_download_bytesasu64)"
    )
    destination_create = compact_cbioportal.find("File::create(dest)")
    if (
        configured_limit < 0
        or declared_limit < 0
        or destination_create < 0
        or declared_limit > destination_create
    ):
        findings.append(
            "src/sources/cbioportal_download.rs: cBioPortal declared archive length must be rejected before destination creation"
        )
    download_accounting = compact_cbioportal.find(
        "account_download_bytes(downloaded,chunk.len(),self.max_archive_download_bytes"
    )
    download_write = compact_cbioportal.find("file.write_all(&chunk)")
    if (
        "MAX_ARCHIVE_DOWNLOAD_BYTES" not in cbioportal
        or download_accounting < 0
        or download_write < 0
        or download_accounting > download_write
    ):
        findings.append(
            "src/sources/cbioportal_download.rs: cBioPortal compressed archive accounting is missing before write"
        )
    cbioportal_archive_limits = (
        "max_entries:MAX_ARCHIVE_ENTRIES,"
        "max_member_bytes:MAX_ARCHIVE_MEMBER_BYTES,"
        "max_total_bytes:MAX_ARCHIVE_EXPANDED_BYTES,"
        "max_metadata_bytes:MAX_ARCHIVE_METADATA_BYTES"
    )
    if (
        "ArchiveBudget::new(limits)" not in cbioportal
        or "entries()?.raw(true)" not in cbioportal
        or cbioportal_archive_limits not in compact_cbioportal
    ):
        findings.append(
            "src/sources/cbioportal_download.rs: cBioPortal archive expansion must use the shared raw archive budget"
        )

    pmc = production_text("pmc_oa.rs")
    compact_pmc = re.sub(r"\s+", "", pmc)
    pmc_archive_limits = (
        "max_entries:MAX_ARCHIVE_ENTRIES,"
        "max_member_bytes:MAX_ARCHIVE_ENTRY_BYTES,"
        "max_total_bytes:MAX_ARCHIVE_EXPANDED_BYTES,"
        "max_metadata_bytes:MAX_ARCHIVE_METADATA_BYTES"
    )
    if (
        "ArchiveBudget::new(limits)" not in pmc
        or "entries()?.raw(true)" not in pmc
        or pmc_archive_limits not in compact_pmc
    ):
        findings.append(
            "src/sources/pmc_oa.rs: PMC OA archive must use the shared raw archive budget"
        )

    for name in ("clinicaltrials.rs", "pubmed.rs"):
        if "bytes.to_vec()" in production_text(name):
            findings.append(
                f"src/sources/{name}: bounded response buffer must be transferred without .to_vec()"
            )

    custom_limit_calls = (
        (
            "src/sources/ema.rs",
            "with_response_body_limit(request,EMA_MAX_BODY_BYTES,EMA_API",
        ),
        (
            "src/sources/europepmc.rs",
            "with_response_body_limit(req,MAX_SUPPLEMENTARY_ZIP_BYTES,EUROPE_PMC_API",
        ),
        (
            "src/sources/gtr.rs",
            "with_response_body_limit(request,max_body_bytes,GTR_API",
        ),
        (
            "src/sources/pmc_oa.rs",
            "with_response_body_limit(request,MAX_TGZ_BYTES,PMC_OA_API",
        ),
        (
            "src/sources/wikipathways.rs",
            "with_response_body_limit(req,WIKIPATHWAYS_MAX_BODY_BYTES,WIKIPATHWAYS_API",
        ),
        (
            "src/sources/who_ivd.rs",
            "with_response_body_limit(request,WHO_IVD_MAX_BODY_BYTES,WHO_IVD_API",
        ),
        (
            "src/sources/who_pq.rs",
            "with_response_body_limit(request,max_body_bytes,WHO_PQ_API",
        ),
        (
            "src/sources/cvx.rs",
            "with_response_body_limit(request,max_body_bytes,CVX_API",
        ),
        (
            "src/entities/article/fulltext.rs",
            "with_response_body_limit(request,PDF_MAX_BODY_BYTES,ARTICLE_FULLTEXT_API",
        ),
    )
    for relative, expected_call in custom_limit_calls:
        path = root_dir / relative
        text = (
            _rust_production_text(path.read_text(encoding="utf-8"))
            if path.is_file()
            else ""
        )
        if expected_call not in re.sub(r"\s+", "", text):
            findings.append(
                f"{relative}: current custom response limit must be attached before send"
            )

    return {
        "name": "remote_resource_bounds",
        "status": "fail" if findings else "pass",
        "checked_surfaces": [
            "shared response/cache middleware",
            "cBioPortal and PMC OA archives",
            "custom response limits",
            "bounded response ownership transfers",
        ],
        "finding_count": len(findings),
        "findings": findings,
        "errors": [],
    }


def _rust_section_values(path: Path, entity: str) -> set[str]:
    text = path.read_text(encoding="utf-8")
    marker = f"pub const {entity.upper()}_SECTION_NAMES"
    start = text.find(marker)
    if start < 0:
        return set()
    body_start = text.find("&[", start)
    body_end = text.find("];", body_start)
    body = text[body_start + 2 : body_end]
    values = set(re.findall(r'"([^"]+)"', body))
    for name in re.findall(r"\b[A-Z][A-Z0-9_]+\b", body):
        match = re.search(
            rf'const\s+{re.escape(name)}:\s*&str\s*=\s*"([^"]+)";', text
        )
        if match:
            values.add(match.group(1))
    return values


def _rust_const_values(path: Path, constant: str) -> set[str]:
    text = path.read_text(encoding="utf-8")
    match = re.search(
        rf"(?:pub(?:\(crate\))?\s+)?const\s+{re.escape(constant)}[^=]*=\s*&\[(.*?)\];",
        text,
        re.DOTALL,
    )
    if not match:
        return set()
    body = match.group(1)
    values = set(re.findall(r'"([^"]+)"', body))
    for name in re.findall(r"\b[A-Z][A-Z0-9_]+\b", body):
        value = re.search(rf'const\s+{re.escape(name)}:\s*&str\s*=\s*"([^"]+)";', text)
        if value:
            values.add(value.group(1))
    return values


def _source_state_registry_rows(
    root_dir: Path,
) -> tuple[
    dict[tuple[str, str], tuple[str, tuple[str, ...], str]],
    dict[tuple[str, str], tuple[str, str | None]],
    list[dict[str, str]],
]:
    text = (root_dir / "src/entities/source_state_registry.rs").read_text(encoding="utf-8")
    errors: list[dict[str, str]] = []
    states: dict[tuple[str, str], tuple[str, tuple[str, ...], str]] = {}
    state_pattern = re.compile(
        r'state\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*&\[(.*?)\]\s*,\s*Aggregation::(\w+)\s*,?\s*\)',
        re.DOTALL,
    )
    for entity, key, label, provider_body, aggregation in state_pattern.findall(text):
        identity = (entity, key)
        if identity in states:
            errors.append({"kind": "duplicate_state", "entity": entity, "section": key})
        providers = tuple(re.findall(r'"([^"]+)"', provider_body))
        states[identity] = (label, providers, aggregation.lower())

    selectors: dict[tuple[str, str], tuple[str, str | None]] = {}
    selector_pattern = re.compile(
        r'selector\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*SelectorClass::(\w+)\s*,\s*(None|Some\("([^"]+)"\))\s*,?\s*\)',
        re.DOTALL,
    )
    for entity, selector, selector_class, canonical_expr, canonical in selector_pattern.findall(text):
        identity = (entity, selector)
        if identity in selectors:
            errors.append({"kind": "duplicate_selector", "entity": entity, "section": selector})
        selectors[identity] = (
            selector_class.lower(),
            canonical if canonical_expr != "None" else None,
        )
    return states, selectors, errors


def _source_state_architecture_rows(root_dir: Path) -> set[tuple[str, ...]]:
    text = (root_dir / "architecture/technical/source-integration.md").read_text(encoding="utf-8")
    table = text.partition("<!-- source-state-registry:start -->")[2].partition(
        "<!-- source-state-registry:end -->"
    )[0]
    rows = set()
    for line in table.splitlines():
        if not line.startswith("|"):
            continue
        columns = tuple(column.strip() for column in line.strip().strip("|").split("|"))
        if len(columns) == 6 and columns[0] not in {"entity", "---"}:
            rows.add(columns)
    return rows


def check_source_state_registry(root_dir: Path) -> dict[str, object]:
    entity_paths = {
        "adverse_event": "src/entities/adverse_event.rs",
        "article": "src/entities/article/mod.rs",
        "diagnostic": "src/entities/diagnostic/mod.rs",
        "disease": "src/entities/disease/mod.rs",
        "drug": "src/entities/drug/mod.rs",
        "gene": "src/entities/gene.rs",
        "pathway": "src/entities/pathway.rs",
        "pgx": "src/entities/pgx.rs",
        "protein": "src/entities/protein.rs",
        "trial": "src/entities/trial/mod.rs",
        "variant": "src/entities/variant/get.rs",
    }
    state_rows, selector_rows, errors = _source_state_registry_rows(root_dir)
    runtime_selectors = {
        (entity, section)
        for entity, relative in entity_paths.items()
        for section in _rust_section_values(root_dir / relative, entity)
    }
    unmapped = sorted(runtime_selectors - set(selector_rows))
    stale = sorted(set(selector_rows) - runtime_selectors)

    for (entity, selector), (selector_class, canonical) in selector_rows.items():
        if selector_class == "canonical" and (entity, canonical or "") not in state_rows:
            errors.append({"kind": "missing_canonical_state", "entity": entity, "section": selector})
        elif selector_class == "alias" and (
            canonical is None or (entity, canonical) not in runtime_selectors
        ):
            errors.append({"kind": "invalid_alias_target", "entity": entity, "section": selector})
        elif selector_class in {"aggregate", "local"} and canonical is not None:
            errors.append({"kind": "unexpected_canonical_target", "entity": entity, "section": selector})

    runtime_key_lists = {
        "adverse_event": ("src/entities/adverse_event.rs", "ADVERSE_EVENT_OUTCOME_KEYS"),
        "article": ("src/entities/article/mod.rs", "ARTICLE_OUTCOME_KEYS"),
        "gene": ("src/entities/gene.rs", "GENE_OUTCOME_KEYS"),
        "pathway": ("src/entities/pathway.rs", "PATHWAY_OUTCOME_KEYS"),
        "protein": ("src/entities/protein.rs", "PROTEIN_OUTCOME_KEYS"),
    }
    runtime_key_factories = {
        "diagnostic": "src/entities/diagnostic/mod.rs",
        "disease": "src/entities/disease/mod.rs",
        "drug": "src/entities/drug/mod.rs",
        "pgx": "src/entities/pgx.rs",
        "variant": "src/entities/variant/mod.rs",
    }
    runtime_key_mismatches = []
    for entity, (relative, constant) in runtime_key_lists.items():
        expected = {key for row_entity, key in state_rows if row_entity == entity}
        actual = _rust_const_values(root_dir / relative, constant)
        runtime_key_mismatches.extend(
            {"entity": entity, "section": key} for key in sorted(expected ^ actual)
        )
    for entity, relative in runtime_key_factories.items():
        expected = {key for row_entity, key in state_rows if row_entity == entity}
        source = re.sub(
            r"\s+", "", (root_dir / relative).read_text(encoding="utf-8")
        )
        factory_call = f'SectionOutcomes::with_keys(&outcome_keys("{entity}"))'
        if factory_call not in source:
            runtime_key_mismatches.extend(
                {"entity": entity, "section": key} for key in sorted(expected)
            )

    canonical_targets = {
        (entity, canonical)
        for (entity, _), (selector_class, canonical) in selector_rows.items()
        if selector_class == "canonical" and canonical is not None
    }
    expected_architecture = {
        (
            entity,
            key,
            "canonical" if (entity, key) in canonical_targets else "outcome-only",
            aggregation,
            " / ".join(providers),
            f"`{key}` outcome and provenance projection",
        )
        for (entity, key), (_, providers, aggregation) in state_rows.items()
    }
    architecture_mismatches = [
        {"entity": row[0], "section": row[1]}
        for row in sorted(expected_architecture ^ _source_state_architecture_rows(root_dir))
    ]

    def records(rows: list[tuple[str, str]]) -> list[dict[str, str]]:
        return [{"entity": entity, "section": section} for entity, section in rows]

    findings = (
        len(unmapped)
        + len(stale)
        + len(runtime_key_mismatches)
        + len(architecture_mismatches)
        + len(errors)
    )
    return {
        "name": "source_state_registry",
        "status": "fail" if findings else "pass",
        "finding_count": findings,
        "unmapped_sections": records(unmapped),
        "stale_registry_entries": records(stale),
        "runtime_key_mismatches": runtime_key_mismatches,
        "architecture_mismatches": architecture_mismatches,
        "errors": errors,
    }


def main() -> int:
    args = parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    cli_line_cap_allowlist = args.cli_line_cap_allowlist or (
        args.root_dir / "tools" / "cli-line-cap-allowlist.json"
    )

    spec_paths = resolve_spec_paths(args.spec_glob)
    lint_payload = lint_specs(spec_paths, args.spec_glob)
    write_json(args.output_dir / "quality-ratchet-lint.json", lint_payload)

    mcp_payload = run_json_command(
        [
            sys.executable,
            str(args.root_dir / "tools" / "check-mcp-allowlist.py"),
            "--cli-file",
            str(args.cli_file),
            "--shell-file",
            str(args.shell_file),
            "--build-file",
            str(args.build_file),
            "--json",
        ],
        allowed_exit_codes={0, 1},
    )
    write_json(args.output_dir / "quality-ratchet-mcp-allowlist.json", mcp_payload)

    source_payload = run_json_command(
        [
            sys.executable,
            str(args.root_dir / "tools" / "check-source-registry.py"),
            "--sources-dir",
            str(args.sources_dir),
            "--sources-mod",
            str(args.sources_mod),
            "--health-file",
            str(args.health_file),
            "--json",
        ],
        allowed_exit_codes={0, 1},
    )
    write_json(args.output_dir / "quality-ratchet-source-registry.json", source_payload)

    dead_code_payload = check_dead_code_allowances(args.root_dir)
    write_json(
        args.output_dir / "quality-ratchet-dead-code-allowances.json",
        dead_code_payload,
    )

    cli_line_cap_payload = check_cli_line_cap(args.root_dir, cli_line_cap_allowlist)
    write_json(args.output_dir / "quality-ratchet-cli-line-cap.json", cli_line_cap_payload)

    policy_module_lines = {
        relative: len((args.root_dir / relative).read_text(encoding="utf-8").splitlines())
        for relative in SECTION_OUTCOME_POLICY_MODULES
    }
    oversized_policy_modules = [
        {"path": path, "lines": lines, "cap": SECTION_OUTCOME_POLICY_LINE_CAP}
        for path, lines in policy_module_lines.items()
        if lines > SECTION_OUTCOME_POLICY_LINE_CAP
    ]
    section_outcome_policy_payload = {
        "status": "fail" if oversized_policy_modules else "pass",
        "cap": SECTION_OUTCOME_POLICY_LINE_CAP,
        "files": policy_module_lines,
        "oversized_files": oversized_policy_modules,
    }
    write_json(
        args.output_dir / "quality-ratchet-section-outcome-policy-line-cap.json",
        section_outcome_policy_payload,
    )

    experiment_results_payload = check_architecture_experiment_results(args.root_dir)
    write_json(
        args.output_dir / "quality-ratchet-experiment-results.json",
        experiment_results_payload,
    )

    terminal_output_payload = check_terminal_output_boundaries(args.root_dir)
    write_json(
        args.output_dir / "quality-ratchet-terminal-output-boundaries.json",
        terminal_output_payload,
    )

    cli_surface_payload = check_cli_surface_contract(args.root_dir)
    write_json(
        args.output_dir / "quality-ratchet-cli-surface-contract.json",
        cli_surface_payload,
    )

    remote_resource_payload = check_remote_resource_bounds(args.root_dir)
    write_json(
        args.output_dir / "quality-ratchet-remote-resource-bounds.json",
        remote_resource_payload,
    )

    source_state_payload = check_source_state_registry(args.root_dir)
    write_json(
        args.output_dir / "quality-ratchet-source-state-registry.json",
        source_state_payload,
    )

    statuses = [
        lint_payload["status"],
        mcp_payload.get("status"),
        source_payload.get("status"),
        dead_code_payload.get("status"),
        cli_line_cap_payload.get("status"),
        section_outcome_policy_payload.get("status"),
        experiment_results_payload.get("status"),
        terminal_output_payload.get("status"),
        cli_surface_payload.get("status"),
        remote_resource_payload.get("status"),
        source_state_payload.get("status"),
    ]
    if "error" in statuses:
        summary_status = "error"
    elif all(status == "pass" for status in statuses):
        summary_status = "pass"
    else:
        summary_status = "fail"

    summary_payload = {
        "status": summary_status,
        "lint": lint_payload,
        "mcp_allowlist": {"status": mcp_payload.get("status")},
        "source_registry": {"status": source_payload.get("status")},
        "dead_code_allowances": {"status": dead_code_payload.get("status")},
        "cli_line_cap": {"status": cli_line_cap_payload.get("status")},
        "section_outcome_policy_line_cap": {
            "status": section_outcome_policy_payload.get("status")
        },
        "experiment_results": {"status": experiment_results_payload.get("status")},
        "terminal_output_boundaries": {"status": terminal_output_payload.get("status")},
        "cli_surface_contract": {"status": cli_surface_payload.get("status")},
        "remote_resource_bounds": {"status": remote_resource_payload.get("status")},
        "source_state_registry": {"status": source_state_payload.get("status")},
    }
    write_json(args.output_dir / "quality-ratchet-summary.json", summary_payload)
    return 0 if summary_status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
