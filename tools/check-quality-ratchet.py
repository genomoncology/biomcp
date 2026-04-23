#!/usr/bin/env python3
from __future__ import annotations

import argparse
import glob
import json
import re
import subprocess
import sys
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parent
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from spec_smoke_args import (  # noqa: E402
    canonical_section_id,
    parse_make_variable_values,
    resolve_smoke_targets,
    target_spec_path,
)

MUSTMATCH_JSON_RE = re.compile(r"(?:^|\|\s*)mustmatch\s+json\b")
SHORT_LIKE_RE = re.compile(r'(?:^|\|\s*)mustmatch\s+like\s+("([^"]*)"|\'([^\']*)\')')
MUSTMATCH_PIPE_RE = re.compile(r"\|\s*mustmatch\b")
MUSTMATCH_LINT_SKIP = "<!-- mustmatch-lint: skip -->"
SMOKE_LANE_MARKER = "<!-- smoke-lane -->"
LIVE_NETWORK_COMMAND_RES = [
    re.compile(
        r'(?:\bbiomcp|"\$bin"|\$bin|"\$\{bin\}"|\$\{bin\})\s+'
        r"(?:--json\s+)?get\s+article\s+\d+\b"
    ),
    re.compile(
        r'(?:\bbiomcp|"\$bin"|\$bin|"\$\{bin\}"|\$\{bin\})\s+'
        r"(?:--json\s+)?article\s+batch\s+\d+\b"
    ),
    re.compile(
        r'(?:\bbiomcp|"\$bin"|\$bin|"\$\{bin\}"|\$\{bin\})\s+'
        r"(?:--json\s+)?search\s+article\b"
    ),
    re.compile(
        r'(?:\bbiomcp|"\$bin"|\$bin|"\$\{bin\}"|\$\{bin\})\s+'
        r"(?:--json\s+)?search\s+all\b"
    ),
    re.compile(
        r'(?:\bbiomcp|"\$bin"|\$bin|"\$\{bin\}"|\$\{bin\})\s+'
        r"(?:--json\s+)?gene\s+articles\b"
    ),
    re.compile(
        r'(?:\bbiomcp|"\$bin"|\$bin|"\$\{bin\}"|\$\{bin\})\s+'
        r"(?:--json\s+)?variant\s+articles\b"
    ),
    re.compile(
        r'(?:\bbiomcp|"\$bin"|\$bin|"\$\{bin\}"|\$\{bin\})\s+'
        r"(?:--json\s+)?disease\s+articles\b"
    ),
]


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
                    "message": f'uses short `mustmatch like` literal "{literal}" ({len(literal)} chars)',
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
                        "section has non-skipped bash blocks but no `| mustmatch` assertion "
                        "and no `<!-- mustmatch-lint: skip -->` opt-out"
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
                and MUSTMATCH_PIPE_RE.search(line)
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
            sys.executable,
            "-m",
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

    payload["finding_count"] = len(findings)
    payload["status"] = "fail" if findings else "pass"
    return payload


def resolve_spec_paths(spec_glob: str) -> list[Path]:
    return sorted(Path(path).resolve() for path in glob.glob(spec_glob))


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


def read_text_or_empty(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return ""


def parse_deselected_ids(makefile_path: Path) -> set[str]:
    return parse_quoted_make_variable_ids(makefile_path, "SPEC_PR_DESELECT_ARGS")


def parse_quoted_make_variable_ids(makefile_path: Path, variable_name: str) -> set[str]:
    return set(parse_make_variable_values(makefile_path, variable_name))


def markdown_section(text: str, heading: str) -> str:
    match = re.search(
        rf"^## {re.escape(heading)}\n(.*?)(?=^## |\Z)",
        text,
        flags=re.MULTILINE | re.DOTALL,
    )
    return match.group(1) if match else ""


def markdown_table_rows(text: str) -> list[list[str]]:
    rows: list[list[str]] = []
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if all(re.fullmatch(r":?-+:?", cell) for cell in cells):
            continue
        rows.append(cells)
    return rows


def strip_markdown_code(cell: str) -> str:
    stripped = cell.strip()
    if stripped.startswith("`") and stripped.endswith("`"):
        return stripped[1:-1]
    return stripped


def parse_readme_inventory(readme_path: Path) -> tuple[set[str], set[str]]:
    text = read_text_or_empty(readme_path)
    timing_ids: set[str] = set()
    smoke_ids: set[str] = set()

    timing_rows = markdown_table_rows(markdown_section(text, "spec-pr Timing Audit"))
    for row in timing_rows[1:]:
        if len(row) < 2:
            continue
        file_path = strip_markdown_code(row[0])
        heading = strip_markdown_code(row[1])
        if file_path and heading:
            timing_ids.add(f"{file_path}::{heading}")

    smoke_rows = markdown_table_rows(
        markdown_section(text, "Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)")
    )
    for row in smoke_rows[1:]:
        if row:
            smoke_ids.add(canonical_section_id(strip_markdown_code(row[0])))

    return timing_ids, smoke_ids


def node_id_for_section(spec_path: Path, root_dir: Path, heading: str) -> str:
    resolved_root = root_dir.resolve()
    resolved_spec = spec_path.resolve()
    try:
        display_path = resolved_spec.relative_to(resolved_root).as_posix()
    except ValueError:
        display_path = resolved_spec.as_posix()
    return f"{display_path}::{heading}"


def iter_spec_sections(
    spec_paths: list[Path],
    root_dir: Path,
) -> list[dict[str, object]]:
    sections: list[dict[str, object]] = []
    for spec_path in spec_paths:
        text = spec_path.read_text(encoding="utf-8")
        current_heading: str | None = None
        current_line = 0
        current_body: list[str] = []

        def flush_section() -> None:
            if current_heading is None:
                return
            body = "\n".join(current_body)
            sections.append(
                {
                    "path": str(spec_path),
                    "line": current_line,
                    "section": current_heading,
                    "node_id": node_id_for_section(
                        spec_path, root_dir, current_heading
                    ),
                    "text": f"## {current_heading}",
                    "body": body,
                    "marked": SMOKE_LANE_MARKER in body,
                }
            )

        for lineno, line in enumerate(text.splitlines(), start=1):
            if line.startswith("## "):
                flush_section()
                current_heading = line[3:].strip()
                current_line = lineno
                current_body = []
                continue
            if current_heading is not None:
                current_body.append(line)

        flush_section()
    return sections


def line_executes_live_command(line: str) -> bool:
    stripped = line.strip()
    if not stripped or stripped.startswith("#"):
        return False
    if stripped.startswith(("echo ", "printf ", "grep ", "jq ", "mustmatch ")):
        return False
    if "--help" in stripped:
        return False
    if "|| status" in stripped or "2>&1" in stripped:
        return False
    command_segment = stripped.split("|", 1)[0]
    return any(pattern.search(command_segment) for pattern in LIVE_NETWORK_COMMAND_RES)


def section_has_live_network_command(section: dict[str, object]) -> bool:
    body = section.get("body")
    if not isinstance(body, str):
        return False
    return any(line_executes_live_command(line) for line in body.splitlines())


def is_classified_live_node(node_id: str, classified_live_ids: set[str]) -> bool:
    spec_path, separator, _heading = node_id.partition("::")
    return node_id in classified_live_ids or (
        bool(separator) and spec_path in classified_live_ids
    )


def check_smoke_lane_sync(spec_paths: list[Path], root_dir: Path) -> dict[str, object]:
    makefile_path = root_dir / "Makefile"
    readme_path = root_dir / "spec" / "README-timings.md"
    resolved_root = root_dir.resolve()
    scanned_relative_paths: set[str] = set()
    for spec_path in spec_paths:
        try:
            scanned_relative_paths.add(
                spec_path.resolve().relative_to(resolved_root).as_posix()
            )
        except ValueError:
            continue
    deselected_ids = {
        canonical_section_id(node_id) for node_id in parse_deselected_ids(makefile_path)
    }
    smoke_target_ids = parse_make_variable_values(makefile_path, "SPEC_SMOKE_ARGS")
    smoke_target_section_ids = {
        canonical_section_id(node_id) for node_id in smoke_target_ids
    }
    timing_audit_ids, smoke_readme_ids = parse_readme_inventory(readme_path)
    classified_live_ids = timing_audit_ids | smoke_readme_ids
    sections = iter_spec_sections(spec_paths, root_dir)
    scanned_sections = {
        section["node_id"]: section
        for section in sections
        if isinstance(section.get("node_id"), str)
    }
    findings: list[dict[str, object]] = []

    for section in sections:
        node_id = section["node_id"]
        if not isinstance(node_id, str):
            continue
        if section["marked"]:
            if node_id not in deselected_ids:
                findings.append(
                    {
                        "line": section["line"],
                        "rule": "smoke-lane-not-deselected",
                        "section": section["section"],
                        "node_id": node_id,
                        "message": (
                            f"section is marked {SMOKE_LANE_MARKER} but '{node_id}' "
                            "is not in SPEC_PR_DESELECT_ARGS in the Makefile"
                        ),
                        "text": section["text"],
                    }
                )
            if node_id not in smoke_target_section_ids:
                findings.append(
                    {
                        "line": section["line"],
                        "rule": "smoke-lane-not-in-smoke-target",
                        "section": section["section"],
                        "node_id": node_id,
                        "message": (
                            f"section is marked {SMOKE_LANE_MARKER} but '{node_id}' "
                            "is not in SPEC_SMOKE_ARGS in the Makefile"
                        ),
                        "text": section["text"],
                    }
                )
            if node_id not in smoke_readme_ids:
                findings.append(
                    {
                        "line": section["line"],
                        "rule": "smoke-lane-not-documented",
                        "section": section["section"],
                        "node_id": node_id,
                        "message": (
                            f"section is marked {SMOKE_LANE_MARKER} but '{node_id}' "
                            "is not documented in the README smoke-only table"
                        ),
                        "text": section["text"],
                    }
                )

        if section_has_live_network_command(section) and not is_classified_live_node(
            node_id, classified_live_ids
        ):
            findings.append(
                {
                    "line": section["line"],
                    "rule": "live-network-unclassified",
                    "section": section["section"],
                    "node_id": node_id,
                    "message": (
                        "section contains a known live-network BioMCP command pattern "
                        "but is absent from both the timing audit and smoke-only inventory"
                    ),
                    "text": section["text"],
                }
            )

    for smoke_target_id in sorted(smoke_target_ids):
        node_id = canonical_section_id(smoke_target_id)
        smoke_target_path = node_id.partition("::")[0]
        if smoke_target_path not in scanned_relative_paths:
            continue
        section = scanned_sections.get(node_id)
        if section is None:
            _path, _separator, section_name = node_id.partition("::")
            findings.append(
                {
                    "line": 0,
                    "rule": "smoke-target-not-marked",
                    "section": section_name or node_id,
                    "node_id": node_id,
                    "smoke_target": smoke_target_id,
                    "message": (
                        f"'{smoke_target_id}' is in SPEC_SMOKE_ARGS but no matching "
                        "spec section was scanned"
                    ),
                    "text": "",
                }
            )
        elif not section["marked"]:
            findings.append(
                {
                    "line": section["line"],
                    "rule": "smoke-target-not-marked",
                    "section": section["section"],
                    "node_id": node_id,
                    "smoke_target": smoke_target_id,
                    "message": (
                        f"'{smoke_target_id}' is in SPEC_SMOKE_ARGS but the section lacks "
                        f"{SMOKE_LANE_MARKER}"
                    ),
                    "text": section["text"],
                }
            )

    scanned_smoke_targets = [
        smoke_target_id
        for smoke_target_id in smoke_target_ids
        if target_spec_path(smoke_target_id) in scanned_relative_paths
    ]
    collectable_smoke_target_count = 0
    if scanned_smoke_targets:
        resolution = resolve_smoke_targets(root_dir, scanned_smoke_targets)
        if resolution.collection.failure is not None:
            failure = resolution.collection.failure
            return {
                "status": "error",
                "finding_count": len(findings),
                "files_checked": len(spec_paths),
                "marked_count": sum(1 for section in sections if section["marked"]),
                "smoke_target_count": len(smoke_target_ids),
                "collectable_smoke_target_count": 0,
                "findings": findings,
                "errors": failure.errors,
                "collection_command": failure.command,
                "exit_code": failure.exit_code,
                "stdout": failure.stdout,
                "stderr": failure.stderr,
                "target_files": failure.target_files,
            }

        collectable_smoke_target_count = len(resolution.resolved)
        for error in resolution.errors:
            section = scanned_sections.get(error.node_id)
            finding: dict[str, object] = {
                "line": section["line"] if section is not None else 0,
                "rule": error.rule,
                "section": error.section,
                "node_id": error.node_id,
                "smoke_target": error.smoke_target,
                "message": error.message,
                "text": section["text"] if section is not None else "",
            }
            if error.candidates:
                finding["candidates"] = error.candidates
            findings.append(finding)

    return {
        "status": "fail" if findings else "pass",
        "finding_count": len(findings),
        "files_checked": len(spec_paths),
        "marked_count": sum(1 for section in sections if section["marked"]),
        "smoke_target_count": len(smoke_target_ids),
        "collectable_smoke_target_count": collectable_smoke_target_count,
        "findings": findings,
    }


def main() -> int:
    args = parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)

    spec_paths = resolve_spec_paths(args.spec_glob)
    lint_payload = lint_specs(spec_paths, args.spec_glob)
    write_json(args.output_dir / "quality-ratchet-lint.json", lint_payload)

    smoke_payload = check_smoke_lane_sync(spec_paths, args.root_dir)
    write_json(args.output_dir / "quality-ratchet-smoke-lane.json", smoke_payload)

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

    statuses = [
        lint_payload["status"],
        smoke_payload["status"],
        mcp_payload.get("status"),
        source_payload.get("status"),
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
        "smoke_lane": {"status": smoke_payload["status"]},
        "mcp_allowlist": {"status": mcp_payload.get("status")},
        "source_registry": {"status": source_payload.get("status")},
    }
    write_json(args.output_dir / "quality-ratchet-summary.json", summary_payload)
    return 0 if summary_status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
