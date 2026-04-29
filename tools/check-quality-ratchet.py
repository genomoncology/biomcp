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
MUSTMATCH_PIPE_RE = re.compile(r"\|\s*mustmatch\b")
MUSTMATCH_LINT_SKIP = "<!-- mustmatch-lint: skip -->"
CLI_LINE_CAP = 700
CLI_LINE_CAP_TICKET_RE = re.compile(r"^\d+(?:[-_][a-z0-9][a-z0-9-]*)?$")


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
                        "section has non-skipped bash blocks but no `| mustmatch` "
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

    cli_line_cap_payload = check_cli_line_cap(args.root_dir, cli_line_cap_allowlist)
    write_json(args.output_dir / "quality-ratchet-cli-line-cap.json", cli_line_cap_payload)

    statuses = [
        lint_payload["status"],
        mcp_payload.get("status"),
        source_payload.get("status"),
        cli_line_cap_payload.get("status"),
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
        "cli_line_cap": {"status": cli_line_cap_payload.get("status")},
    }
    write_json(args.output_dir / "quality-ratchet-summary.json", summary_payload)
    return 0 if summary_status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
