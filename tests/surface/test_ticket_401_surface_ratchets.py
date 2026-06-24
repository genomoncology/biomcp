from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


def _read_repo(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _run_quality_ratchet_on_spec(tmp_path: Path, markdown: str) -> dict[str, object]:
    spec_dir = tmp_path / "spec"
    spec_dir.mkdir()
    spec_path = spec_dir / "captured-output.md"
    spec_path.write_text(markdown, encoding="utf-8")
    output_dir = tmp_path / "out"

    subprocess.run(
        [
            "uv",
            "run",
            "--no-project",
            "python",
            str(REPO_ROOT / "tools" / "check-quality-ratchet.py"),
            "--root-dir",
            str(REPO_ROOT),
            "--output-dir",
            str(output_dir),
            "--spec-glob",
            str(spec_dir / "*.md"),
            "--cli-file",
            str(REPO_ROOT / "src" / "cli" / "mod.rs"),
            "--shell-file",
            str(REPO_ROOT / "src" / "mcp" / "shell.rs"),
            "--build-file",
            str(REPO_ROOT / "build.rs"),
            "--sources-dir",
            str(REPO_ROOT / "src" / "sources"),
            "--sources-mod",
            str(REPO_ROOT / "src" / "sources" / "mod.rs"),
            "--health-file",
            str(REPO_ROOT / "src" / "cli" / "health" / "catalog.rs"),
            "--cli-line-cap-allowlist",
            str(REPO_ROOT / "tools" / "cli-line-cap-allowlist.json"),
        ],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    return json.loads((output_dir / "quality-ratchet-lint.json").read_text(encoding="utf-8"))


def _lint_rules(payload: dict[str, object]) -> set[str]:
    rules: set[str] = set()
    for result in payload.get("results", []):
        if not isinstance(result, dict):
            continue
        for finding in result.get("findings", []):
            if isinstance(finding, dict) and isinstance(finding.get("rule"), str):
                rules.add(finding["rule"])
    return rules


def test_ticket_401_quality_ratchet_rejects_printf_captured_output_mustmatch(tmp_path: Path) -> None:
    payload = _run_quality_ratchet_on_spec(
        tmp_path,
        "# Captured Output Spec\n\n"
        "This synthetic document uses the weak captured-output pattern ticket 401 wants banned.\n\n"
        "## Captured printf pipe\n\n"
        "```bash\n"
        'out="$(biomcp list)"\n'
        'printf \'%s\\n\' "$out" | mustmatch like "BioMCP Command Reference"\n'
        "```\n\n"
        "## Direct printf variable pipe\n\n"
        "```bash\n"
        'out="$(biomcp --help)"\n'
        'printf "$out" | mustmatch like "leading public biomedical data sources"\n'
        "```\n",
    )

    assert "captured-output-mustmatch-pipe" in _lint_rules(payload), (
        "quality ratchet must reject printf-based captured-output plumbing, not only "
        "the older echo spelling"
    )


def test_ticket_401_article_figshare_fixture_uses_realistic_aacr_sibling_shapes() -> None:
    fixture = _read_repo("spec/fixtures/setup-article-fulltext-source-fixture.sh")

    assert "10.1158/1078-0432.22474817.v1" in fixture, (
        "the Figshare sibling fixture should use an AACR-style record-specific DOI so "
        "same-paper matching is proven against realistic provider metadata"
    )
    assert re.search(r"Supplementary\s+(?:Table|Data)\s+S[12]\s+from", fixture, re.IGNORECASE), (
        "the Figshare sibling fixture should use provider-like 'Supplementary ... from' "
        "titles instead of repeated exact-match toy titles"
    )
    assert "unrelated-table.xlsx" in fixture, "the negative sibling fixture must remain present"


def test_ticket_401_request_plan_ratchets_execute_named_contracts_not_list_only() -> None:
    failures: list[str] = []
    for spec_path in sorted((REPO_ROOT / "spec").glob("**/*.md")):
        path = spec_path.relative_to(REPO_ROOT).as_posix()
        text = _read_repo(path)
        for match in re.finditer(r"cargo\s+test[^\n]*--\s+--list", text):
            line = text[: match.start()].count("\n") + 1
            failures.append(f"{path}:{line}: {match.group(0)}")

    assert not failures, (
        "Spec-lane Rust contract wrappers must execute named contracts, not only list "
        "that a test exists; otherwise ignored/skipped Cargo contracts can leave the "
        "wrapper green:\n" + "\n".join(failures)
    )


def test_ticket_401_routine_modes_are_markdown_plus_parallel_canary_only() -> None:
    runner = _read_repo("scripts/run-specs.sh")
    match = re.search(r"(?ms)^\s*spec\|spec-pr\)\n(?P<body>.*?)\n\s*;;", runner)
    assert match is not None, "missing run-specs routine modes"
    body = match.group("body")
    assert "SPEC_ROUTINE_PATHS" in body, "routine modes must run the path inventory"
    assert "run_markdown_specs" in runner, "routine modes must run Markdown specs"

    py_paths = re.findall(r"tests/surface/\S+\.py", runner)
    assert py_paths == ["tests/surface/test_parallel_isolation_contract.py"]
    assert "uv sync --extra dev --no-install-project" not in runner
    assert "uv run --no-sync pytest" not in runner
