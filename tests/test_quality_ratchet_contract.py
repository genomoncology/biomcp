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
    target = fixture_root / "src" / "cli" / "health" / "catalog.rs"
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(REPO_ROOT / "src" / "cli" / "health" / "catalog.rs", target)
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


def _init_git_fixture(root: Path) -> None:
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)


def _write_cli_line_cap_allowlist(
    allowlist_path: Path,
    entries: list[dict[str, object]],
) -> None:
    allowlist_path.parent.mkdir(parents=True, exist_ok=True)
    allowlist_path.write_text(
        json.dumps(
            {
                "cap": 700,
                "created": "2026-04-28",
                "scope": "tracked Rust files under src/cli",
                "entries": entries,
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _write_tracked_file(root: Path, relative_path: str, line_count: int) -> Path:
    path = root / relative_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "\n".join(["//! fixture", *("// filler" for _ in range(line_count - 1))])
        + "\n",
        encoding="utf-8",
    )
    subprocess.run(["git", "add", relative_path], cwd=root, check=True)
    return path


def _write_dead_code_fixture(
    root: Path, source: str, relative_path: str = "src/lib.rs"
) -> None:
    root.mkdir(parents=True)
    _init_git_fixture(root)
    path = root / relative_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source, encoding="utf-8")
    subprocess.run(["git", "add", relative_path], cwd=root, check=True)


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


def _remove_allowlisted_discover(shell_file: Path) -> None:
    content = shell_file.read_text(encoding="utf-8")
    updated = content.replace(' | "discover"', "")
    assert updated != content
    shell_file.write_text(updated, encoding="utf-8")


def _break_study_download_guard(shell_file: Path) -> None:
    content = shell_file.read_text(encoding="utf-8")
    updated = content.replace('args.len() == 4 && args[3] == "--list"', "true", 1)
    assert updated != content
    shell_file.write_text(updated, encoding="utf-8")


def _break_skill_positive_policy(shell_file: Path) -> None:
    content = shell_file.read_text(encoding="utf-8")
    updated = content.replace(
        '            matches!(sub.as_str(), "list" | "render")\n'
        "                || crate::cli::skill::show_use_case(&sub).is_ok()\n",
        '            !matches!(sub.as_str(), "install")\n',
        1,
    )
    assert updated != content
    shell_file.write_text(updated, encoding="utf-8")


def _remove_description_filter_term(build_file: Path) -> None:
    content = build_file.read_text(encoding="utf-8")
    updated = content.replace('    "`skill install`",\n', "", 1)
    assert updated != content
    build_file.write_text(updated, encoding="utf-8")


def _remove_structural_update_description_filter(build_file: Path) -> None:
    content = build_file.read_text(encoding="utf-8")
    updated = content.replace(
        '        || line.trim_start().starts_with("- `update ")\n',
        "",
        1,
    )
    assert updated != content
    assert '"`update [--check]`"' in updated
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
    updated = content.replace("];\n", f"{entry}];\n", 1)
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


def test_mcp_allowlist_audit_reports_skill_policy_drift(tmp_path: Path) -> None:
    fixture_root = _copy_mcp_fixture(tmp_path)
    _break_skill_positive_policy(fixture_root / "src/mcp/shell.rs")

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
    assert payload["skill_policy_ok"] is False


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


def test_mcp_description_policy_rejects_legacy_update_marker_only(
    tmp_path: Path,
) -> None:
    fixture_root = _copy_mcp_fixture(tmp_path)
    _remove_structural_update_description_filter(fixture_root / "build.rs")

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

    payload = _load_json(result.stdout)
    assert result.returncode == 1, result.stdout
    assert payload["status"] == "fail"
    assert payload["description_policy_ok"] is False


def test_dead_code_allowance_audit_passes_for_repo() -> None:
    ratchet = _load_ratchet_module()

    payload = ratchet.check_dead_code_allowances(REPO_ROOT)

    assert payload["status"] == "pass", payload
    assert payload["findings"] == []


def test_dead_code_allowance_audit_rejects_unreasoned_suppression(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    fixture_root = tmp_path / "unreasoned-dead-code"
    _write_dead_code_fixture(
        fixture_root,
        "#[allow(dead_code)]\n"
        "fn ordinary() {}\n\n"
        "#[cfg_attr(not(test), allow(dead_code))]\n"
        "fn conditional() {}\n\n"
        "#![allow(\n"
        "    dead_code,\n"
        ")]\n",
    )

    payload = ratchet.check_dead_code_allowances(fixture_root)

    assert payload["status"] == "fail"
    assert payload["finding_count"] == 3
    assert all(row["path"] == "src/lib.rs" for row in payload["findings"])
    assert all("dead-code reason:" in row["message"] for row in payload["findings"])


def test_dead_code_allowance_audit_scans_tracked_rust_outside_src(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    fixture_root = tmp_path / "test-helper-dead-code"
    _write_dead_code_fixture(
        fixture_root,
        "#![allow(dead_code)]\nfn helper() {}\n",
        "tests/helper.rs",
    )

    payload = ratchet.check_dead_code_allowances(fixture_root)

    assert payload["status"] == "fail"
    assert payload["findings"][0]["path"] == "tests/helper.rs"


def test_dead_code_allowance_audit_accepts_adjacent_reason(tmp_path: Path) -> None:
    ratchet = _load_ratchet_module()
    fixture_root = tmp_path / "reasoned-dead-code"
    _write_dead_code_fixture(
        fixture_root,
        "// dead-code reason: retained for the binary-only dispatch seam\n"
        "#[allow(dead_code)]\n"
        "fn ordinary() {}\n\n"
        "// dead-code reason: retained for non-test target compatibility\n"
        "#[cfg_attr(not(test), allow(dead_code))]\n"
        "fn conditional() {}\n\n"
        "// dead-code reason: generated provider client includes unused RPC methods\n"
        "#![allow(\n"
        "    dead_code,\n"
        ")]\n",
    )

    payload = ratchet.check_dead_code_allowances(fixture_root)

    assert payload["status"] == "pass", payload
    assert payload["findings"] == []


def test_dead_code_allowance_audit_ignores_comment_and_string_tokens(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    fixture_root = tmp_path / "lexical-dead-code"
    _write_dead_code_fixture(
        fixture_root,
        "#[cfg_attr(\n"
        "    not(test), // ] must not close the attribute\n"
        '    doc = "https://example.test/[contract]",\n'
        "    allow(dead_code),\n"
        ")]\n"
        "fn conditional() {}\n",
    )

    payload = ratchet.check_dead_code_allowances(fixture_root)

    assert payload["status"] == "fail"
    assert payload["allowances_checked"] == 1
    assert payload["findings"][0]["line"] == 1


def test_dead_code_allowance_audit_does_not_match_deny_group(tmp_path: Path) -> None:
    ratchet = _load_ratchet_module()
    fixture_root = tmp_path / "unrelated-dead-code"
    _write_dead_code_fixture(
        fixture_root,
        "#[cfg_attr(test, allow(unused_variables), deny(dead_code))]\n"
        "fn conditional() {}\n",
    )

    payload = ratchet.check_dead_code_allowances(fixture_root)

    assert payload["status"] == "pass", payload
    assert payload["allowances_checked"] == 0


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
    _remove_mygene_health_entry(fixture_root / "src/cli/health/catalog.rs")

    result = _run_python_script(
        SOURCE_SCRIPT,
        "--sources-dir",
        str(fixture_root / "src/sources"),
        "--sources-mod",
        str(fixture_root / "src/sources/mod.rs"),
        "--health-file",
        str(fixture_root / "src/cli/health/catalog.rs"),
        "--json",
    )

    assert result.returncode == 1
    payload = _load_json(result.stdout)
    assert payload["status"] == "fail"
    assert "mygene" in payload["missing_health_modules"]


def test_source_registry_audit_reports_orphan_health_entry(tmp_path: Path) -> None:
    fixture_root = _copy_source_fixture(tmp_path)
    _append_orphan_health_entry(fixture_root / "src/cli/health/catalog.rs")

    result = _run_python_script(
        SOURCE_SCRIPT,
        "--sources-dir",
        str(fixture_root / "src/sources"),
        "--sources-mod",
        str(fixture_root / "src/sources/mod.rs"),
        "--health-file",
        str(fixture_root / "src/cli/health/catalog.rs"),
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
        "quality-ratchet-mcp-allowlist.json",
        "quality-ratchet-source-registry.json",
        "quality-ratchet-dead-code-allowances.json",
        "quality-ratchet-cli-line-cap.json",
        "quality-ratchet-terminal-output-boundaries.json",
        "quality-ratchet-summary.json",
    ):
        assert (output_dir / name).exists(), name

    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "pass"
    assert summary["lint"]["status"] == "pass"
    assert summary["lint"]["files_checked"] == 1
    assert summary["lint"]["finding_count"] == 0
    assert summary["cli_line_cap"]["status"] == "pass"
    assert summary["dead_code_allowances"]["status"] == "pass"
    assert summary["terminal_output_boundaries"]["status"] == "pass"
    assert "smoke_lane" not in summary


def test_terminal_output_boundary_ratchet_detects_removed_seams_and_pretty_bypass(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    source_files = [
        "src/render/human.rs",
        "src/cli/outcome.rs",
        "src/cli/shared.rs",
        "src/main.rs",
        "src/mcp/shell.rs",
        "src/render/chart.rs",
        "src/render/json.rs",
    ]
    expected_markers = {
        "src/render/human.rs": {
            "fn sanitize_document(value: &str)",
            "fn sanitize_inline(value: &str)",
        },
        "src/cli/outcome.rs": {
            "outcome.text = crate::render::human::sanitize_document(&outcome.text)",
            "trusted_terminal_chart = is_charted_mcp_study_command",
        },
        "src/cli/shared.rs": {
            "sanitize_document(&message)",
            "Err(err) => exit_human_clap_error(err, &args)",
        },
        "src/main.rs": {"sanitize_human_diagnostic(&error.to_string())"},
        "src/mcp/shell.rs": {
            "sanitize_document(&text)",
            "sanitize_document(&content)",
            "sanitize_inline(&message.into())",
        },
        "src/render/chart.rs": {
            "fn chart_text(value: &str)",
            "sanitize_inline(value)",
        },
    }
    assert {
        path: set(markers)
        for path, markers in ratchet.TERMINAL_OUTPUT_BOUNDARY_SEAMS.items()
    } == expected_markers
    mutations = [
        (relative_path, marker)
        for relative_path, markers in expected_markers.items()
        for marker in markers
    ]

    for index, (relative_path, marker) in enumerate(mutations):
        fixture = tmp_path / f"seam-{index}"
        for source_file in source_files:
            target = fixture / source_file
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(REPO_ROOT / source_file, target)
        path = fixture / relative_path
        text = path.read_text(encoding="utf-8")
        assert marker in text
        path.write_text(
            text.replace(marker, f"// removed seam: {marker}", 1), encoding="utf-8"
        )

        payload = ratchet.check_terminal_output_boundaries(fixture)
        assert payload["status"] == "fail"
        assert any(finding["path"] == relative_path for finding in payload["findings"])

    bypass_fixture = tmp_path / "pretty-bypass"
    bypass = bypass_fixture / "src/cli/bypass.rs"
    bypass.parent.mkdir(parents=True)
    bypass.write_text(
        "fn render(value: &serde_json::Value) { let _ = serde_json::to_string_pretty(value); }\n",
        encoding="utf-8",
    )
    for source_file in source_files:
        target = bypass_fixture / source_file
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(REPO_ROOT / source_file, target)

    payload = ratchet.check_terminal_output_boundaries(bypass_fixture)
    assert payload["status"] == "fail"
    assert any(
        finding["path"] == "src/cli/bypass.rs" and "pretty JSON" in finding["message"]
        for finding in payload["findings"]
    )

    for source in (
        "fn render(value: &serde_json::Value) { let _ = serde_json :: to_string_pretty(value); }\n",
        "use serde_json::to_string_pretty as pretty;\nfn render() {}\n",
        "#[cfg(test)]\nmod tests {}\nfn render(value: &serde_json::Value) { let _ = serde_json::to_string_pretty(value); }\n",
    ):
        bypass.write_text(source, encoding="utf-8")
        payload = ratchet.check_terminal_output_boundaries(bypass_fixture)
        assert payload["status"] == "fail", source

    bypass.write_text(
        "#[cfg(test)]\nmod tests { fn render(value: &serde_json::Value) { let _ = serde_json::to_string_pretty(value); } }\n",
        encoding="utf-8",
    )
    payload = ratchet.check_terminal_output_boundaries(bypass_fixture)
    assert payload["status"] == "pass"


def test_cli_line_cap_audit_reports_unallowlisted_tracked_overcap_file(
    tmp_path: Path,
) -> None:
    fixture_root = tmp_path / "line-cap-fixture"
    fixture_root.mkdir()
    _init_git_fixture(fixture_root)
    _write_tracked_file(fixture_root, "src/cli/new_over_cap.rs", 701)
    allowlist_path = fixture_root / "tools" / "cli-line-cap-allowlist.json"
    _write_cli_line_cap_allowlist(allowlist_path, [])

    ratchet = _load_ratchet_module()
    payload = ratchet.check_cli_line_cap(fixture_root, allowlist_path)

    assert payload["status"] == "fail"
    assert payload["missing_allowlist_entries"] == [
        {
            "path": "src/cli/new_over_cap.rs",
            "lines": 701,
            "message": (
                "tracked src/cli Rust file exceeds 700 lines without an allowlist entry"
            ),
        }
    ]


def test_cli_line_cap_audit_reports_stale_allowlist_entry(tmp_path: Path) -> None:
    fixture_root = tmp_path / "line-cap-fixture"
    fixture_root.mkdir()
    _init_git_fixture(fixture_root)
    _write_tracked_file(fixture_root, "src/cli/cache.rs", 12)
    allowlist_path = fixture_root / "tools" / "cli-line-cap-allowlist.json"
    _write_cli_line_cap_allowlist(
        allowlist_path,
        [
            {
                "path": "src/cli/cache.rs",
                "lines": 759,
                "date": "2026-04-28",
                "follow_up_ticket": "347-decompose-residual-over-cap-src-cli-files-under-global-ratchet",
            }
        ],
    )

    ratchet = _load_ratchet_module()
    payload = ratchet.check_cli_line_cap(fixture_root, allowlist_path)

    assert payload["status"] == "fail"
    assert payload["stale_allowlist_entries"] == [
        {
            "path": "src/cli/cache.rs",
            "lines": 12,
            "follow_up_ticket": "347-decompose-residual-over-cap-src-cli-files-under-global-ratchet",
            "message": "allowlist entry is no longer needed; remove it",
        }
    ]


def test_cli_line_cap_audit_reports_allowlisted_file_growth(tmp_path: Path) -> None:
    fixture_root = tmp_path / "line-cap-fixture"
    fixture_root.mkdir()
    _init_git_fixture(fixture_root)
    _write_tracked_file(fixture_root, "src/cli/drug/tests.rs", 705)
    allowlist_path = fixture_root / "tools" / "cli-line-cap-allowlist.json"
    _write_cli_line_cap_allowlist(
        allowlist_path,
        [
            {
                "path": "src/cli/drug/tests.rs",
                "lines": 704,
                "date": "2026-04-28",
                "follow_up_ticket": "347-decompose-residual-over-cap-src-cli-files-under-global-ratchet",
            }
        ],
    )

    ratchet = _load_ratchet_module()
    payload = ratchet.check_cli_line_cap(fixture_root, allowlist_path)

    assert payload["status"] == "fail"
    assert payload["grown_allowlist_entries"] == [
        {
            "path": "src/cli/drug/tests.rs",
            "lines": 705,
            "allowed_lines": 704,
            "follow_up_ticket": "347-decompose-residual-over-cap-src-cli-files-under-global-ratchet",
            "message": (
                "allowlisted file grew beyond its recorded line count; decompose it "
                "instead of expanding the allowlist"
            ),
        }
    ]


def test_wrapper_is_thin_shell_around_committed_python_tool() -> None:
    wrapper = WRAPPER_SCRIPT.read_text(encoding="utf-8")

    assert RATCHET_TOOL.exists()
    assert "python3 - <<'PY'" not in wrapper
    assert "lint_spec_file" not in wrapper
    assert "collect_shell_blocks" not in wrapper
    assert "MUSTMATCH_JSON_RE" not in wrapper
    assert "SHORT_LIKE_RE" not in wrapper
    assert "FENCE_RE" not in wrapper
    assert "uv run --no-project python" in wrapper
    assert "tools/check-quality-ratchet.py" in wrapper
    assert "spec/**/*.md" in wrapper
    assert "QUALITY_RATCHET_CLI_LINE_CAP_ALLOWLIST" in wrapper
    assert "tools/spec_smoke_args.py" not in wrapper


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


def test_resolve_spec_paths_matches_nested_v2_specs(tmp_path: Path) -> None:
    ratchet = _load_ratchet_module()
    gene = _write_h2_bash_spec(
        tmp_path / "spec" / "entity",
        "gene-canary",
        "# Nested Entity Fixture\n\n"
        "## Entity Section\n\n"
        "```bash\n"
        'echo "gene" | mustmatch like "gene"\n'
        "```\n",
    )
    surface = _write_h2_bash_spec(
        tmp_path / "spec" / "surface",
        "surface-canary",
        "# Nested Surface Fixture\n\n"
        "## Surface Section\n\n"
        "```bash\n"
        'echo "surface" | mustmatch like "surface"\n'
        "```\n",
    )

    spec_paths = ratchet.resolve_spec_paths(str(tmp_path / "spec" / "**" / "*.md"))

    assert set(spec_paths) == {gene.resolve(), surface.resolve()}


def test_wrapper_accepts_nested_specs_with_recursive_glob(tmp_path: Path) -> None:
    spec_path = _write_clean_spec(tmp_path / "spec" / "entity")
    output_dir = tmp_path / "out"

    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(tmp_path / "spec" / "**" / "*.md"),
        }
    )

    assert result.returncode == 0, result.stderr
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["status"] == "pass"
    assert summary["lint"]["status"] == "pass"
    assert summary["lint"]["files_checked"] == 1
    assert spec_path.exists()


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
        "section has non-skipped bash blocks but no `mustmatch` assertion and no "
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


def test_remote_resource_bound_ratchet_detects_buffer_and_archive_regressions(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    sources = tmp_path / "src/sources"
    sources.mkdir(parents=True)
    (sources / "mod.rs").write_text(
        "ensure_body_limited_cache_epoch(cache_root)\n"
        "ClientBuilder::new(base_client).with(Cache(())\n"
        ".with(ResponseBodyLimitMiddleware {})\n",
        encoding="utf-8",
    )
    (sources / "cbioportal_download.rs").write_text(
        "max_archive_download_bytes: MAX_ARCHIVE_DOWNLOAD_BYTES\n"
        "response.content_length().is_some_and(\n"
        "    |length| length > self.max_archive_download_bytes as u64\n"
        ")\n"
        "File::create(dest);\n"
        "account_download_bytes(\n"
        "    downloaded, chunk.len(), self.max_archive_download_bytes\n"
        ")?;\n"
        "file.write_all(&chunk);\n"
        "max_entries: MAX_ARCHIVE_ENTRIES,\n"
        "max_member_bytes: MAX_ARCHIVE_MEMBER_BYTES,\n"
        "max_total_bytes: MAX_ARCHIVE_EXPANDED_BYTES,\n"
        "max_metadata_bytes: MAX_ARCHIVE_METADATA_BYTES\n"
        "ArchiveBudget::new(limits); archive.entries()?.raw(true);",
        encoding="utf-8",
    )
    (sources / "pmc_oa.rs").write_text(
        "with_response_body_limit(request, MAX_ARCHIVE_ENTRY_BYTES as usize, PMC_OA_API);\n"
        "max_entries: MAX_ARCHIVE_ENTRIES,\n"
        "max_member_bytes: MAX_ARCHIVE_ENTRY_BYTES,\n"
        "max_total_bytes: MAX_ARCHIVE_EXPANDED_BYTES,\n"
        "max_metadata_bytes: MAX_ARCHIVE_METADATA_BYTES\n"
        "ArchiveBudget::new(limits); archive.entries()?.raw(true);",
        encoding="utf-8",
    )
    (sources / "clinicaltrials.rs").write_text("bounded bytes", encoding="utf-8")
    (sources / "pubmed.rs").write_text("bounded bytes", encoding="utf-8")
    custom_limit_calls = {
        "ema.rs": "with_response_body_limit(request, EMA_MAX_BODY_BYTES, EMA_API)",
        "europepmc.rs": (
            "with_response_body_limit(req, MAX_SUPPLEMENTARY_ZIP_BYTES, EUROPE_PMC_API)"
        ),
        "gtr.rs": "with_response_body_limit(request, max_body_bytes, GTR_API)",
        "wikipathways.rs": (
            "with_response_body_limit(req, WIKIPATHWAYS_MAX_BODY_BYTES, WIKIPATHWAYS_API)"
        ),
        "who_ivd.rs": (
            "with_response_body_limit(request, WHO_IVD_MAX_BODY_BYTES, WHO_IVD_API)"
        ),
        "who_pq.rs": "with_response_body_limit(request, max_body_bytes, WHO_PQ_API)",
        "cvx.rs": "with_response_body_limit(request, max_body_bytes, CVX_API)",
    }
    for name, call in custom_limit_calls.items():
        (sources / name).write_text(call, encoding="utf-8")
    fulltext = tmp_path / "src/entities/article/fulltext.rs"
    fulltext.parent.mkdir(parents=True)
    fulltext.write_text(
        "with_response_body_limit(request, PDF_MAX_BODY_BYTES, ARTICLE_FULLTEXT_API)",
        encoding="utf-8",
    )

    assert ratchet.check_remote_resource_bounds(tmp_path)["status"] == "pass"

    (sources / "mod.rs").write_text(
        ".with(ResponseBodyLimitMiddleware {})\n"
        "ClientBuilder::new(base_client).with(Cache(())\n",
        encoding="utf-8",
    )
    (sources / "cbioportal_download.rs").write_text(
        "MAX_ARCHIVE_DOWNLOAD_BYTES account_download_bytes(", encoding="utf-8"
    )
    (sources / "pmc_oa.rs").write_text("unbounded archive", encoding="utf-8")
    (sources / "clinicaltrials.rs").write_text(
        "read_limited_body(response, source).await?; bytes.to_vec()", encoding="utf-8"
    )
    (sources / "pubmed.rs").write_text(
        "read_limited_body(response, source).await?; bytes.to_vec()", encoding="utf-8"
    )
    (sources / "gtr.rs").write_text(
        "with_response_body_limit(request, DEFAULT_MAX_BODY_BYTES, GTR_API)\n"
        "read_limited_body_with_limit(response, GTR_API, GTR_TEST_VERSION_MAX_BODY_BYTES)",
        encoding="utf-8",
    )
    payload = ratchet.check_remote_resource_bounds(tmp_path)

    assert payload["status"] == "fail"
    assert any("inside the cache" in finding for finding in payload["findings"])
    assert any("legacy HTTP cache" in finding for finding in payload["findings"])
    assert any("declared archive length" in finding for finding in payload["findings"])
    assert any(
        "cBioPortal archive expansion" in finding for finding in payload["findings"]
    )
    assert any("PMC OA archive" in finding for finding in payload["findings"])
    assert any("clinicaltrials.rs" in finding for finding in payload["findings"])
    assert any("pubmed.rs" in finding for finding in payload["findings"])
    assert any("custom response limit" in finding for finding in payload["findings"])

    spec_path = _write_clean_spec(tmp_path / "spec")
    output_dir = tmp_path / "out"
    result = _run_wrapper(
        {
            "QUALITY_RATCHET_OUTPUT_DIR": str(output_dir),
            "QUALITY_RATCHET_SPEC_GLOB": str(spec_path),
        }
    )
    assert result.returncode == 0, result.stderr
    summary = json.loads((output_dir / "quality-ratchet-summary.json").read_text())
    assert summary["remote_resource_bounds"]["status"] == "pass"


def test_source_attributed_status_typing_ratchet_rejects_owned_strings_and_allows_relays(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    fixture_root = tmp_path / "source-status-typing"
    _write_dead_code_fixture(
        fixture_root,
        "#[derive(Serialize)]\n"
        "struct ClaimedProviderStatus {\n"
        "    source: String,\n"
        "    status: String,\n"
        "}\n\n"
        "#[derive(Deserialize)]\n"
        "struct UpstreamProviderRelay {\n"
        "    source: String,\n"
        "    status: String,\n"
        "}\n",
    )
    allowlist = fixture_root / "tools" / "source-status-typing-allowlist.json"
    allowlist.parent.mkdir(parents=True)
    allowlist.write_text('{"entries": []}\n', encoding="utf-8")

    audit = getattr(
        ratchet,
        "check_source_attributed_status_is_typed",
        lambda _root: {"status": "unimplemented", "findings": []},
    )
    rejected = audit(fixture_root)

    assert rejected["status"] == "fail"
    assert any(
        finding["path"] == "src/lib.rs"
        and "ClaimedProviderStatus" in finding["message"]
        for finding in rejected["findings"]
    )

    source = fixture_root / "src" / "lib.rs"
    source.write_text(
        source.read_text(encoding="utf-8").replace(
            "status: String,\n}\n\n#[derive(Deserialize)]",
            "status: ClaimedProviderStatusKind,\n}\n\n#[derive(Deserialize)]",
            1,
        ),
        encoding="utf-8",
    )
    accepted = audit(fixture_root)
    assert accepted["status"] == "pass", accepted


def test_source_state_registry_rejects_unmapped_and_stale_sections(
    tmp_path: Path,
) -> None:
    ratchet = _load_ratchet_module()
    fixture_root = tmp_path / "source-state-registry"
    shutil.copytree(REPO_ROOT / "src" / "entities", fixture_root / "src" / "entities")
    architecture = fixture_root / "architecture" / "technical" / "source-integration.md"
    architecture.parent.mkdir(parents=True)
    shutil.copy2(
        REPO_ROOT / "architecture" / "technical" / "source-integration.md", architecture
    )

    clean = ratchet.check_source_state_registry(fixture_root)
    assert clean["status"] == "pass", clean
    assert clean["unmapped_sections"] == []
    assert clean["stale_registry_entries"] == []
    assert clean["architecture_mismatches"] == []

    disease = fixture_root / "src" / "entities" / "disease" / "mod.rs"
    original = disease.read_text(encoding="utf-8")
    with_unmapped = original.replace(
        "    DISEASE_SECTION_ALL,\n];",
        '    "fixture_unmapped",\n    DISEASE_SECTION_ALL,\n];',
        1,
    )
    assert with_unmapped != original
    disease.write_text(with_unmapped, encoding="utf-8")
    unmapped = ratchet.check_source_state_registry(fixture_root)
    assert unmapped["status"] == "fail"
    assert any(
        row["entity"] == "disease" and row["section"] == "fixture_unmapped"
        for row in unmapped["unmapped_sections"]
    )

    with_stale = original.replace("    DISEASE_SECTION_SURVIVAL,\n", "", 1)
    assert with_stale != original
    disease.write_text(with_stale, encoding="utf-8")
    stale = ratchet.check_source_state_registry(fixture_root)
    assert stale["status"] == "fail"
    assert any(
        row["entity"] == "disease" and row["section"] == "survival"
        for row in stale["stale_registry_entries"]
    )

    without_keyed_default = original.replace(
        'SectionOutcomes::with_keys(&outcome_keys("disease"))',
        "SectionOutcomes::default()",
        1,
    )
    assert without_keyed_default != original
    disease.write_text(without_keyed_default, encoding="utf-8")
    runtime_default_drift = ratchet.check_source_state_registry(fixture_root)
    assert runtime_default_drift["status"] == "fail"
    assert {"entity": "disease", "section": "survival"} in runtime_default_drift[
        "runtime_key_mismatches"
    ]

    disease.write_text(original, encoding="utf-8")
    architecture_text = architecture.read_text(encoding="utf-8")
    architecture_without_survival = architecture_text.replace(
        "| disease | survival |", "| disease | omitted-survival |", 1
    )
    assert architecture_without_survival != architecture_text
    architecture.write_text(architecture_without_survival, encoding="utf-8")
    architecture_drift = ratchet.check_source_state_registry(fixture_root)
    assert architecture_drift["status"] == "fail"
    assert any(
        row["entity"] == "disease" and row["section"] == "survival"
        for row in architecture_drift["architecture_mismatches"]
    )

    architecture.write_text(architecture_text, encoding="utf-8")
    article = fixture_root / "src" / "entities" / "article" / "mod.rs"
    article_text = article.read_text(encoding="utf-8")
    article.write_text(
        article_text.replace(', "tldr"]', "]", 1),
        encoding="utf-8",
    )
    runtime_drift = ratchet.check_source_state_registry(fixture_root)
    assert runtime_drift["status"] == "fail"
    assert {"entity": "article", "section": "tldr"} in runtime_drift[
        "runtime_key_mismatches"
    ]

    article.write_text(article_text, encoding="utf-8")
    architecture.write_text(
        architecture_text.replace(
            "MyDisease.info / Monarch Initiative / HPO",
            "Monarch Initiative / HPO",
            1,
        ),
        encoding="utf-8",
    )
    provider_drift = ratchet.check_source_state_registry(fixture_root)
    assert provider_drift["status"] == "fail"
    assert any(
        row["entity"] == "disease" and row["section"] == "phenotypes"
        for row in provider_drift["architecture_mismatches"]
    )


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


def test_profile_independence_audit_flags_a_build_specific_spec_assertion(
    tmp_path: Path,
) -> None:
    root = tmp_path
    quality_ratchet = _load_ratchet_module()
    (root / "src" / "cli").mkdir(parents=True)
    (root / "src" / "cli" / "list.rs").write_text(
        'pub fn page() -> String {\n'
        '    let line = if cfg!(feature = "alphagenome") {\n'
        '        "AlphaGenome prediction (requires `ALPHAGENOME_API_KEY`)"\n'
        '    } else {\n'
        '        "AlphaGenome support was not built into this binary"\n'
        '    };\n'
        '    line.to_string()\n'
        '}\n',
        encoding="utf-8",
    )
    (root / "scripts").mkdir()
    (root / "scripts" / "run-specs.sh").write_text(
        "SPEC_ROUTINE_PATHS=(\n  spec/surface/page.md\n)\n", encoding="utf-8"
    )
    (root / "spec" / "surface").mkdir(parents=True)
    page = root / "spec" / "surface" / "page.md"
    page.write_text(
        "# Page\n\n```bash\nbiomcp list variant | mustmatch like 'not built into this binary'\n```\n",
        encoding="utf-8",
    )

    failing = quality_ratchet.check_profile_independent_specs(root)
    assert failing["status"] == "fail"
    assert failing["findings"][0]["path"] == "spec/surface/page.md"
    assert "not built into this binary" in failing["findings"][0]["fragment"]

    page.write_text(
        "# Page\n\n```bash\nbiomcp list variant | mustmatch like 'get variant <id> predict'\n```\n",
        encoding="utf-8",
    )
    assert quality_ratchet.check_profile_independent_specs(root)["status"] == "pass"


def test_profile_independence_audit_reads_the_dual_lane_list_from_the_runner(
    tmp_path: Path,
) -> None:
    root = tmp_path
    quality_ratchet = _load_ratchet_module()
    (root / "scripts").mkdir()
    (root / "scripts" / "run-specs.sh").write_text(
        "SPEC_ROUTINE_PATHS=(\n"
        "  spec/entity/one.md\n"
        "  tests/surface/contract.py\n"
        "  spec/surface/two.md\n"
        ")\n\n"
        "SPEC_LIVE_PATHS=(\n  spec/entity/three-live.md\n)\n",
        encoding="utf-8",
    )
    paths = quality_ratchet.dual_lane_spec_paths(root)
    # Live pages run only against the release binary, so they are single-lane and
    # legitimately may assert release-only behavior.
    assert paths == ["spec/entity/one.md", "spec/surface/two.md"]
