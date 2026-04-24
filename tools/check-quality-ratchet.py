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

    statuses = [
        lint_payload["status"],
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
        "mcp_allowlist": {"status": mcp_payload.get("status")},
        "source_registry": {"status": source_payload.get("status")},
    }
    write_json(args.output_dir / "quality-ratchet-summary.json", summary_payload)
    return 0 if summary_status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
