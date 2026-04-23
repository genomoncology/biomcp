from __future__ import annotations

import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from types import ModuleType

REPO_ROOT = Path(__file__).resolve().parents[1]
MCP_SCRIPT = REPO_ROOT / "tools" / "check-mcp-allowlist.py"
SOURCE_SCRIPT = REPO_ROOT / "tools" / "check-source-registry.py"
WRAPPER_SCRIPT = REPO_ROOT / "tools" / "check-quality-ratchet.sh"
RATCHET_TOOL = REPO_ROOT / "tools" / "check-quality-ratchet.py"
SPEC_SMOKE_ARGS_SCRIPT = REPO_ROOT / "tools" / "spec_smoke_args.py"


def _run_python_script(
    script: Path,
    *args: str,
    cwd: Path = REPO_ROOT,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(script), *args],
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )


def _run_wrapper(env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    wrapper_env = os.environ.copy()
    wrapper_env.update(env)
    return subprocess.run(
        ["bash", str(WRAPPER_SCRIPT)],
        cwd=REPO_ROOT,
        env=wrapper_env,
        capture_output=True,
        text=True,
        check=False,
    )


def _load_json(stdout: str) -> dict[str, object]:
    return json.loads(stdout)


def _load_ratchet_module() -> ModuleType:
    spec = importlib.util.spec_from_file_location("quality_ratchet", RATCHET_TOOL)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _copy_mcp_fixture(tmp_path: Path) -> Path:
    fixture_root = tmp_path / "mcp-fixture"
    for relative_path in (
        "src/cli/mod.rs",
        "src/cli/commands.rs",
        "src/mcp/shell.rs",
        "build.rs",
    ):
        source = REPO_ROOT / relative_path
        target = fixture_root / relative_path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
    return fixture_root


def _copy_source_fixture(tmp_path: Path) -> Path:
    fixture_root = tmp_path / "source-fixture"
    shutil.copytree(REPO_ROOT / "src" / "sources", fixture_root / "src" / "sources")
    target = fixture_root / "src" / "cli" / "health.rs"
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(REPO_ROOT / "src" / "cli" / "health.rs", target)
    return fixture_root


def _write_clean_spec(spec_dir: Path) -> Path:
    spec_dir.mkdir(parents=True, exist_ok=True)
    spec_path = spec_dir / "clean-spec.md"
    spec_path.write_text(
        "# Quality Ratchet Fixture\n\n"
        "```bash\n"
        'echo "# BioMCP Command Reference"\n'
        "```\n"
        "```mustmatch\n"
        'mustmatch like "# BioMCP Command Reference"\n'
        "```\n",
        encoding="utf-8",
    )
    return spec_path


def _write_failing_spec(spec_dir: Path) -> Path:
    spec_dir.mkdir(parents=True, exist_ok=True)
    spec_path = spec_dir / "failing-spec.md"
    spec_path.write_text(
        "# Quality Ratchet Failure Fixture\n\n"
        "```bash\n"
        'out="ok"\n'
        'echo "$out" | mustmatch like "ok"\n'
        "```\n",
        encoding="utf-8",
    )
    return spec_path


def _write_invalid_mode_spec(spec_dir: Path) -> Path:
    spec_dir.mkdir(parents=True, exist_ok=True)
    spec_path = spec_dir / "invalid-mode-spec.md"
    spec_path.write_text(
        "# Quality Ratchet Invalid Mode Fixture\n\n"
        "```bash\n"
        'echo \'{"status":"ok"}\'\n'
        "```\n"
        "```mustmatch\n"
        'mustmatch json \'{"status":"ok"}\'\n'
        "```\n",
        encoding="utf-8",
    )
    return spec_path


def _write_invalid_shell_spec(spec_dir: Path) -> Path:
    spec_dir.mkdir(parents=True, exist_ok=True)
    spec_path = spec_dir / "invalid-shell-spec.md"
    spec_path.write_text(
        "# Quality Ratchet Invalid Shell Fixture\n\n"
        "```bash\n"
        "if then\n"
        "  echo broken\n"
        "fi\n"
        "```\n",
        encoding="utf-8",
    )
    return spec_path


def _write_h2_bash_spec(spec_dir: Path, name: str, body: str) -> Path:
    spec_dir.mkdir(parents=True, exist_ok=True)
    spec_path = spec_dir / f"{name}.md"
    spec_path.write_text(body, encoding="utf-8")
    return spec_path


def _write_smoke_lane_fixture(
    root: Path,
    *,
    makefile: str,
    readme: str,
    spec_body: str,
) -> Path:
    (root / "spec").mkdir(parents=True, exist_ok=True)
    (root / "Makefile").write_text(makefile, encoding="utf-8")
    (root / "spec" / "README-timings.md").write_text(readme, encoding="utf-8")
    spec_path = root / "spec" / "example.md"
    spec_path.write_text(spec_body, encoding="utf-8")
    return spec_path


def _remove_allowlisted_discover(shell_file: Path) -> None:
    content = shell_file.read_text(encoding="utf-8")
    updated = content.replace(' | "discover"', "")
    assert updated != content
    shell_file.write_text(updated, encoding="utf-8")


def _break_study_download_guard(shell_file: Path) -> None:
    content = shell_file.read_text(encoding="utf-8")
    updated = content.replace('args.len() == 4 && args[3] == "--list"', "true", count=1)
    assert updated != content
    shell_file.write_text(updated, encoding="utf-8")


def _remove_description_filter_term(build_file: Path) -> None:
    content = build_file.read_text(encoding="utf-8")
    updated = content.replace('    "`skill install`",\n', "", count=1)
    assert updated != content
    build_file.write_text(updated, encoding="utf-8")


def _remove_mygene_health_entry(health_file: Path) -> None:
    content = health_file.read_text(encoding="utf-8")
    updated, count = re.subn(
        r"    SourceDescriptor \{\n"
        r'        api: "MyGene",\n'
        r".*?"
        r"    \},\n",
        "",
        content,
        count=1,
        flags=re.DOTALL,
    )
    assert count == 1
    health_file.write_text(updated, encoding="utf-8")


def _append_orphan_health_entry(health_file: Path) -> None:
    content = health_file.read_text(encoding="utf-8")
    entry = (
        "    SourceDescriptor {\n"
        '        api: "Imaginary Source",\n'
        '        affects: Some("fixture"),\n'
        "        probe: ProbeKind::Get {\n"
        '            url: "https://example.com/fixture",\n'
        "        },\n"
        "    },\n"
    )
    updated = content.replace("];\n", f"{entry}];\n", count=1)
    assert updated != content
    health_file.write_text(updated, encoding="utf-8")


def test_mcp_allowlist_audit_passes_for_repo() -> None:
    result = _run_python_script(MCP_SCRIPT, "--json")

    assert result.returncode == 0, result.stderr
    payload = _load_json(result.stdout)
    assert payload["status"] == "pass"
    assert payload["unclassified_families"] == []
    assert payload["stale_allowlist_families"] == []
    assert payload["study_policy_ok"] is True
    assert payload["skill_policy_ok"] is True
    assert payload["description_policy_ok"] is True


def test_mcp_allowlist_audit_reports_allowlist_drift(tmp_path: Path) -> None:
    fixture_root = _copy_mcp_fixture(tmp_path)
    _remove_allowlisted_discover(fixture_root / "src/mcp/shell.rs")

    result = _run_python_script(
        MCP_SCRIPT,
        "--cli-file",
        str(fixture_root / "src/cli/mod.rs"),
        "--shell-file",
        str(fixture_root / "src/mcp/shell.rs"),
        "--build-file",
        str(fixture_root / "build.rs"),
        "--json",
    )

    assert result.returncode == 1
    payload = _load_json(result.stdout)
    assert payload["status"] == "fail"
    assert "discover" in payload["unclassified_families"]


def test_mcp_allowlist_audit_reports_study_policy_drift(tmp_path: Path) -> None:
    fixture_root = _copy_mcp_fixture(tmp_path)
    _break_study_download_guard(fixture_root / "src/mcp/shell.rs")

    result = _run_python_script(
        MCP_SCRIPT,
        "--cli-file",
        str(fixture_root / "src/cli/mod.rs"),
        "--shell-file",
        str(fixture_root / "src/mcp/shell.rs"),
        "--build-file",
        str(fixture_root / "build.rs"),
        "--json",
    )

    assert result.returncode == 1
    payload = _load_json(result.stdout)
    assert payload["status"] == "fail"
    assert payload["study_policy_ok"] is False


def test_mcp_allowlist_audit_reports_description_policy_drift(tmp_path: Path) -> None:
    fixture_root = _copy_mcp_fixture(tmp_path)
    _remove_description_filter_term(fixture_root / "build.rs")

    result = _run_python_script(
        MCP_SCRIPT,
        "--cli-file",
        str(fixture_root / "src/cli/mod.rs"),
        "--shell-file",
        str(fixture_root / "src/mcp/shell.rs"),
        "--build-file",
        str(fixture_root / "build.rs"),
        "--json",
    )

    assert result.returncode == 1
    payload = _load_json(result.stdout)
    assert payload["status"] == "fail"
    assert payload["description_policy_ok"] is False


def test_source_registry_audit_passes_for_repo() -> None:
    result = _run_python_script(SOURCE_SCRIPT, "--json")

    assert result.returncode == 0, result.stderr
    payload = _load_json(result.stdout)
    assert payload["status"] == "pass"
    assert payload["undeclared_modules"] == []
    assert payload["missing_health_modules"] == []
    assert payload["orphan_health_entries"] == []


def test_source_registry_audit_reports_missing_health_entry(tmp_path: Path) -> None:
    fixture_root = _copy_source_fixture(tmp_path)
    _remove_mygene_health_entry(fixture_root / "src/cli/health.rs")

    result = _run_python_script(
        SOURCE_SCRIPT,
        "--sources-dir",
        str(fixture_root / "src/sources"),
        "--sources-mod",
        str(fixture_root / "src/sources/mod.rs"),
        "--health-file",
        str(fixture_root / "src/cli/health.rs"),
        "--json",
    )

    assert result.returncode == 1
    payload = _load_json(result.stdout)
    assert payload["status"] == "fail"
    assert "mygene" in payload["missing_health_modules"]


def test_source_registry_audit_reports_orphan_health_entry(tmp_path: Path) -> None:
    fixture_root = _copy_source_fixture(tmp_path)
    _append_orphan_health_entry(fixture_root / "src/cli/health.rs")

    result = _run_python_script(
        SOURCE_SCRIPT,
        "--sources-dir",
        str(fixture_root / "src/sources"),
        "--sources-mod",
        str(fixture_root / "src/sources/mod.rs"),
        "--health-file",
        str(fixture_root / "src/cli/health.rs"),
        "--json",
    )

    assert result.returncode == 1
    payload = _load_json(result.stdout)
    assert payload["status"] == "fail"
    assert "Imaginary Source" in payload["orphan_health_entries"]


def test_wrapper_writes_summary_artifacts_for_pass_fixture(tmp_path: Path) -> None:
    spec_path = _write_clean_spec(tmp_path / "spec")
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 0, result.stderr
    for name in (
        "quality-ratchet-lint.json",
        "quality-ratchet-smoke-lane.json",
        "quality-ratchet-mcp-allowlist.json",
        "quality-ratchet-source-registry.json",
        "quality-ratchet-summary.json",
    ):
        assert (output_dir / name).exists(), name

    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "pass"
    assert summary["lint"]["status"] == "pass"
    assert summary["lint"]["files_checked"] == 1
    assert summary["lint"]["finding_count"] == 0
    assert summary["smoke_lane"]["status"] == "pass"

    smoke_lane = json.loads(
        (output_dir / "quality-ratchet-smoke-lane.json").read_text()
    )
    assert smoke_lane["status"] == "pass"
    assert smoke_lane["finding_count"] == 0


def test_wrapper_is_thin_shell_around_committed_python_tool() -> None:
    wrapper = WRAPPER_SCRIPT.read_text(encoding="utf-8")

    assert RATCHET_TOOL.exists()
    assert "python3 - <<'PY'" not in wrapper
    assert "lint_spec_file" not in wrapper
    assert "collect_shell_blocks" not in wrapper
    assert "MUSTMATCH_JSON_RE" not in wrapper
    assert "SHORT_LIKE_RE" not in wrapper
    assert "FENCE_RE" not in wrapper
    assert "uv run --extra dev python" in wrapper
    assert "tools/check-quality-ratchet.py" in wrapper


def test_wrapper_propagates_lint_failures(tmp_path: Path) -> None:
    spec_path = _write_failing_spec(tmp_path / "spec")
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 1
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "fail"
    assert summary["lint"]["status"] == "fail"
    findings = summary["lint"]["results"][0]["findings"]
    assert findings[0]["rule"] == "short-like-pattern"


def test_wrapper_reports_marked_smoke_section_missing_deselect(tmp_path: Path) -> None:
    spec_path = _write_h2_bash_spec(
        tmp_path / "spec",
        "marked-smoke-section",
        "# Quality Ratchet Smoke Fixture\n\n"
        "## Marked Smoke Section\n"
        "<!-- smoke-lane -->\n\n"
        "```bash\n"
        "echo 'marked smoke section' | mustmatch like 'marked smoke section'\n"
        "```\n",
    )
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 1
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "fail"
    assert summary["lint"]["status"] == "pass"
    assert summary["smoke_lane"]["status"] == "fail"

    smoke_lane = json.loads(
        (output_dir / "quality-ratchet-smoke-lane.json").read_text()
    )
    rules = {finding["rule"] for finding in smoke_lane["findings"]}
    assert "smoke-lane-not-deselected" in rules


def test_spec_smoke_args_cli_resolves_makefile_stable_targets(tmp_path: Path) -> None:
    node_ids = [
        "spec/example.md::First Smoke Section",
        "spec/example.md::Second Smoke Section",
    ]
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile=(f'SPEC_SMOKE_ARGS = \\\n\t"{node_ids[0]}" \\\n\t"{node_ids[1]}"\n'),
        readme="# Spec Lane Audit\n",
        spec_body=(
            "# Smoke Fixture\n\n"
            "## First Smoke Section\n"
            "<!-- smoke-lane -->\n\n"
            "```bash\n"
            "echo 'first smoke section' | mustmatch like 'first smoke section'\n"
            "```\n\n"
            "## Second Smoke Section\n"
            "<!-- smoke-lane -->\n\n"
            "```bash\n"
            "echo 'second smoke section' | mustmatch like 'second smoke section'\n"
            "```\n"
        ),
    )

    result = _run_python_script(
        SPEC_SMOKE_ARGS_SCRIPT,
        "--root-dir",
        str(tmp_path),
        "--makefile",
        str(tmp_path / "Makefile"),
        "--makefile-variable",
        "SPEC_SMOKE_ARGS",
    )

    assert result.returncode == 0, result.stderr
    assert result.stderr == ""
    resolved = result.stdout.splitlines()
    assert len(resolved) == 2
    assert resolved[0].startswith(f"{node_ids[0]} (line ")
    assert resolved[0].endswith(") [bash]")
    assert resolved[1].startswith(f"{node_ids[1]} (line ")
    assert resolved[1].endswith(") [bash]")
    assert [line.split(" (line ", maxsplit=1)[0] for line in resolved] == node_ids
    assert spec_path.exists()


def test_smoke_lane_sync_passes_when_marker_makefile_and_readme_align(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    node_id = "spec/example.md::Smoke Section"
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile=(
            "SPEC_PR_DESELECT_ARGS = \\\n"
            f'\t--deselect "{node_id}"\n\n'
            "SPEC_SMOKE_ARGS = \\\n"
            f'\t"{node_id}"\n'
        ),
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
            f"| `{node_id}` | Ticket fixture smoke lane. |\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Smoke Section\n"
            "<!-- smoke-lane -->\n\n"
            "```bash\n"
            "biomcp search article -g BRAF --limit 1 | mustmatch like 'Articles'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "pass"
    assert payload["finding_count"] == 0


def test_smoke_lane_sync_ignores_deselects_outside_pr_deselect_variable(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    node_id = "spec/example.md::Smoke Section"
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile=(
            "SPEC_PR_DESELECT_ARGS_EXTRA = \\\n"
            f'\t--deselect "{node_id}"\n\n'
            "SPEC_PR_DESELECT_ARGS =\n\n"
            "SPEC_SMOKE_ARGS = \\\n"
            f'\t"{node_id}"\n'
        ),
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
            f"| `{node_id}` | Ticket fixture smoke lane. |\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Smoke Section\n"
            "<!-- smoke-lane -->\n\n"
            "```bash\n"
            "echo 'local section' | mustmatch like 'local section'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "fail"
    assert {finding["rule"] for finding in payload["findings"]} == {
        "smoke-lane-not-deselected",
    }


def test_smoke_lane_sync_reports_smoke_target_without_marker(tmp_path: Path) -> None:
    ratchet = _load_ratchet_module()
    node_id = "spec/example.md::Smoke Section"
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile=(
            "SPEC_PR_DESELECT_ARGS = \\\n"
            f'\t--deselect "{node_id}"\n\n'
            "SPEC_SMOKE_ARGS = \\\n"
            f'\t"{node_id}"\n'
        ),
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
            f"| `{node_id}` | Ticket fixture smoke lane. |\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Smoke Section\n\n"
            "```bash\n"
            "echo 'local section' | mustmatch like 'local section'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "fail"
    assert {finding["rule"] for finding in payload["findings"]} == {
        "smoke-target-not-marked",
    }


def test_smoke_lane_sync_reports_smoke_target_missing_section(tmp_path: Path) -> None:
    ratchet = _load_ratchet_module()
    node_id = "spec/example.md::Missing Section"
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile=(
            "SPEC_PR_DESELECT_ARGS = \\\n"
            f'\t--deselect "{node_id}"\n\n'
            "SPEC_SMOKE_ARGS = \\\n"
            f'\t"{node_id}"\n'
        ),
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
            f"| `{node_id}` | Ticket fixture smoke lane. |\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Other Section\n\n"
            "```bash\n"
            "echo 'local section' | mustmatch like 'local section'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "fail"
    findings = payload["findings"]
    rules = {finding["rule"] for finding in findings}
    assert rules == {"smoke-target-not-marked", "smoke-target-not-collectable"}
    not_marked = next(
        finding for finding in findings if finding["rule"] == "smoke-target-not-marked"
    )
    not_collectable = next(
        finding
        for finding in findings
        if finding["rule"] == "smoke-target-not-collectable"
    )
    assert not_marked["node_id"] == node_id
    assert "no matching spec section was scanned" in not_marked["message"]
    assert not_collectable["node_id"] == node_id
    assert not_collectable["smoke_target"] == node_id
    assert "not collectable" in not_collectable["message"]


def test_smoke_lane_sync_reports_stale_line_qualified_smoke_target(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    node_id = "spec/example.md::Smoke Section"
    stale_smoke_item_id = f"{node_id} (line 999) [bash]"
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile=(
            "SPEC_PR_DESELECT_ARGS = \\\n"
            f'\t--deselect "{node_id}"\n\n'
            "SPEC_SMOKE_ARGS = \\\n"
            f'\t"{stale_smoke_item_id}"\n'
        ),
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
            f"| `{node_id}` | Ticket fixture smoke lane. |\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Smoke Section\n"
            "<!-- smoke-lane -->\n\n"
            "```bash\n"
            "echo 'local section' | mustmatch like 'local section'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "fail"
    findings = payload["findings"]
    assert len(findings) == 1
    finding = findings[0]
    assert finding["rule"] == "smoke-target-not-collectable"
    assert finding["smoke_target"] == stale_smoke_item_id
    assert finding["node_id"] == node_id
    assert finding["section"] == "Smoke Section"
    assert stale_smoke_item_id in finding["message"]


def test_smoke_lane_sync_reports_ambiguous_stable_smoke_target(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    node_id = "spec/example.md::Smoke Section"
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile=(
            "SPEC_PR_DESELECT_ARGS = \\\n"
            f'\t--deselect "{node_id}"\n\n'
            "SPEC_SMOKE_ARGS = \\\n"
            f'\t"{node_id}"\n'
        ),
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
            f"| `{node_id}` | Ticket fixture smoke lane. |\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Smoke Section\n"
            "<!-- smoke-lane -->\n\n"
            "```bash\n"
            "echo 'first local section' | mustmatch like 'first local section'\n"
            "```\n\n"
            "```bash\n"
            "echo 'second local section' | mustmatch like 'second local section'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "fail"
    findings = payload["findings"]
    assert len(findings) == 1
    finding = findings[0]
    assert finding["rule"] == "smoke-target-ambiguous"
    assert finding["smoke_target"] == node_id
    assert finding["node_id"] == node_id
    assert finding["section"] == "Smoke Section"
    candidates = finding["candidates"]
    assert isinstance(candidates, list)
    assert len(candidates) == 2
    assert all(candidate.startswith(f"{node_id} (line ") for candidate in candidates)


def test_smoke_lane_sync_reports_collect_only_failure(tmp_path: Path) -> None:
    ratchet = _load_ratchet_module()
    node_id = "spec/example.md::Smoke Section"
    (tmp_path / "conftest.py").write_text(
        "raise RuntimeError('fixture collect-only failure')\n",
        encoding="utf-8",
    )
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile=(
            "SPEC_PR_DESELECT_ARGS = \\\n"
            f'\t--deselect "{node_id}"\n\n'
            "SPEC_SMOKE_ARGS = \\\n"
            f'\t"{node_id}"\n'
        ),
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
            f"| `{node_id}` | Ticket fixture smoke lane. |\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Smoke Section\n"
            "<!-- smoke-lane -->\n\n"
            "```bash\n"
            "echo 'local section' | mustmatch like 'local section'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "error"
    assert payload["errors"]
    assert "collection failed" in payload["errors"][0]
    assert payload["exit_code"] != 0
    assert "fixture collect-only failure" in payload["stderr"]
    assert payload["target_files"] == ["spec/example.md"]
    assert "pytest" in payload["collection_command"]


def test_smoke_lane_sync_reports_marker_missing_readme_inventory(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    node_id = "spec/example.md::Smoke Section"
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile=(
            "SPEC_PR_DESELECT_ARGS = \\\n"
            f'\t--deselect "{node_id}"\n\n'
            "SPEC_SMOKE_ARGS = \\\n"
            f'\t"{node_id}"\n'
        ),
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Smoke Section\n"
            "<!-- smoke-lane -->\n\n"
            "```bash\n"
            "echo 'local section' | mustmatch like 'local section'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "fail"
    assert {finding["rule"] for finding in payload["findings"]} == {
        "smoke-lane-not-documented",
    }


def test_smoke_lane_sync_reports_piped_unclassified_live_network_section(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile="SPEC_PR_DESELECT_ARGS =\n\nSPEC_SMOKE_ARGS =\n",
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Live Section\n\n"
            "```bash\n"
            "biomcp search article -g BRAF --limit 1 | mustmatch like 'Articles'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "fail"
    findings = payload["findings"]
    assert len(findings) == 1
    assert findings[0]["rule"] == "live-network-unclassified"
    assert findings[0]["node_id"] == "spec/example.md::Live Section"


def test_smoke_lane_sync_reports_unclassified_live_network_section(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    spec_path = _write_smoke_lane_fixture(
        tmp_path,
        makefile="SPEC_PR_DESELECT_ARGS =\n\nSPEC_SMOKE_ARGS =\n",
        readme=(
            "# Spec Lane Audit\n\n"
            "## spec-pr Timing Audit\n\n"
            "| File | Heading | First-pass Time | First-pass Result | Warm-pass Time | Warm-pass Result | Category | Disposition | Rationale |\n"
            "|---|---|---|---|---|---|---|---|---|\n\n"
            "## Smoke-Only Headings (SPEC_PR_DESELECT_ARGS)\n\n"
            "| Node ID | Reason |\n"
            "|---|---|\n"
        ),
        spec_body=(
            "# Smoke Fixture\n\n"
            "## Live Section\n\n"
            "```bash\n"
            'out="$(biomcp search article -g BRAF --limit 1)"\n'
            "echo \"$out\" | mustmatch like 'Articles'\n"
            "```\n"
        ),
    )

    payload = ratchet.check_smoke_lane_sync([spec_path], tmp_path)

    assert payload["status"] == "fail"
    findings = payload["findings"]
    assert len(findings) == 1
    assert findings[0]["rule"] == "live-network-unclassified"
    assert findings[0]["node_id"] == "spec/example.md::Live Section"


def test_wrapper_reports_error_when_no_specs_match(tmp_path: Path) -> None:
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(tmp_path / "spec" / "*.md"),
        }
    )

    assert result.returncode == 1
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "error"
    assert summary["lint"]["status"] == "error"
    assert "no spec files matched" in summary["lint"]["errors"][0]


def test_wrapper_propagates_mcp_failures_from_override_paths(tmp_path: Path) -> None:
    fixture_root = _copy_mcp_fixture(tmp_path)
    _remove_allowlisted_discover(fixture_root / "src/mcp/shell.rs")
    spec_path = _write_clean_spec(tmp_path / "spec")
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
            "QUALITY_RATCHET_CLI_FILE": str(fixture_root / "src/cli/mod.rs"),
            "QUALITY_RATCHET_SHELL_FILE": str(fixture_root / "src/mcp/shell.rs"),
            "QUALITY_RATCHET_BUILD_FILE": str(fixture_root / "build.rs"),
        }
    )

    assert result.returncode == 1
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "fail"
    assert summary["lint"]["status"] == "pass"
    assert summary["mcp_allowlist"]["status"] == "fail"


def test_wrapper_reports_invalid_mustmatch_mode(tmp_path: Path) -> None:
    spec_path = _write_invalid_mode_spec(tmp_path / "spec")
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 1
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    findings = summary["lint"]["results"][0]["findings"]
    assert findings[0]["rule"] == "invalid-mustmatch-mode"


def test_wrapper_reports_invalid_shell_syntax(tmp_path: Path) -> None:
    spec_path = _write_invalid_shell_spec(tmp_path / "spec")
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 1
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    findings = summary["lint"]["results"][0]["findings"]
    assert findings[0]["rule"] == "invalid-shell-syntax"


def test_wrapper_reports_missing_bash_mustmatch(tmp_path: Path) -> None:
    spec_path = _write_h2_bash_spec(
        tmp_path / "spec",
        "missing-bash-mustmatch",
        "# Quality Ratchet Missing Mustmatch Fixture\n\n"
        "## Missing Collection Anchor\n\n"
        "```bash\n"
        'out=\'{"status":"ok"}\'\n'
        'echo "$out" | jq -e \'.status == "ok"\' >/dev/null\n'
        "```\n",
    )
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 1
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "fail"
    assert summary["lint"]["status"] == "fail"
    findings = summary["lint"]["results"][0]["findings"]
    assert findings[0]["rule"] == "missing-bash-mustmatch"
    assert findings[0]["line"] == 3
    assert findings[0]["section"] == "Missing Collection Anchor"
    assert findings[0]["message"] == (
        "section has non-skipped bash blocks but no `| mustmatch` assertion and no "
        "`<!-- mustmatch-lint: skip -->` opt-out"
    )
    assert findings[0]["text"] == "## Missing Collection Anchor"


def test_wrapper_allows_h2_section_with_bash_mustmatch(tmp_path: Path) -> None:
    spec_path = _write_h2_bash_spec(
        tmp_path / "spec",
        "section-with-bash-mustmatch",
        "# Quality Ratchet Mustmatch Fixture\n\n"
        "## Collected Section\n\n"
        "```bash\n"
        'out=\'{"status":"ok"}\'\n'
        'echo "$out" | mustmatch like \'"status":"ok"\'\n'
        'echo "$out" | jq -e \'.status == "ok"\' >/dev/null\n'
        "```\n",
    )
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 0, result.stderr
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "pass"
    assert summary["lint"]["status"] == "pass"
    assert summary["lint"]["finding_count"] == 0


def test_wrapper_allows_h2_section_with_mustmatch_opt_out(tmp_path: Path) -> None:
    spec_path = _write_h2_bash_spec(
        tmp_path / "spec",
        "section-with-opt-out",
        "# Quality Ratchet Opt-out Fixture\n\n"
        "## Exit Code Only Section\n"
        "<!-- mustmatch-lint: skip -->\n\n"
        "```bash\n"
        "test -n 'still-runs'\n"
        "```\n",
    )
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 0, result.stderr
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "pass"
    assert summary["lint"]["status"] == "pass"
    assert summary["lint"]["finding_count"] == 0


def test_wrapper_ignores_skipped_bash_only_section(tmp_path: Path) -> None:
    spec_path = _write_h2_bash_spec(
        tmp_path / "spec",
        "section-with-skipped-bash",
        "# Quality Ratchet Skipped Bash Fixture\n\n"
        "## Skipped Section\n\n"
        "```bash skip\n"
        "echo 'not collected by design'\n"
        "```\n",
    )
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 0, result.stderr
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "pass"
    assert summary["lint"]["status"] == "pass"
    assert summary["lint"]["finding_count"] == 0


def test_wrapper_accepts_section_when_one_of_multiple_bash_blocks_has_mustmatch(
    tmp_path: Path,
) -> None:
    spec_path = _write_h2_bash_spec(
        tmp_path / "spec",
        "section-with-multiple-bash-blocks",
        "# Quality Ratchet Multi-block Fixture\n\n"
        "## Multi Block Section\n\n"
        "```bash\n"
        'echo \'{"phase":"setup"}\' | jq -e \'.phase == "setup"\' >/dev/null\n'
        "```\n\n"
        "```bash\n"
        'echo \'{"phase":"proof"}\' | mustmatch like \'"phase":"proof"\'\n'
        "```\n",
    )
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 0, result.stderr
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "pass"
    assert summary["lint"]["status"] == "pass"
    assert summary["lint"]["finding_count"] == 0


def test_wrapper_allows_mustmatch_opt_out_later_in_section(tmp_path: Path) -> None:
    spec_path = _write_h2_bash_spec(
        tmp_path / "spec",
        "section-with-late-opt-out",
        "# Quality Ratchet Late Opt-out Fixture\n\n"
        "## Exit Code Only Section\n\n"
        "```bash\n"
        "test -n 'still-runs'\n"
        "```\n\n"
        "<!-- mustmatch-lint: skip -->\n",
    )
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )

    assert result.returncode == 0, result.stderr
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "pass"
    assert summary["lint"]["status"] == "pass"
    assert summary["lint"]["finding_count"] == 0
