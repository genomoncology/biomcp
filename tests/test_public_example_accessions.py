from __future__ import annotations

import gzip
import os
import re
import shlex
import shutil
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = REPO_ROOT / "target" / "release" / "biomcp"
GTR_FIXTURE_DIR = REPO_ROOT / "spec" / "fixtures" / "gtr"
GTR_TEST_VERSION_FILE = "test_version.gz"
GTR_CONDITION_GENE_FILE = "test_condition_gene.txt"
LIVE_GTR_ACCESSION = "GTR000006692.3"
FICTIONAL_GTR_ACCESSION = "GTR000000001.1"
REGULATORY_EMPTY_STATE = "No FDA device 510(k) or PMA records matched this diagnostic."
PUBLIC_DIAGNOSTIC_EXAMPLE_SURFACES = (
    "README.md",
    "docs/index.md",
    "docs/user-guide/diagnostic.md",
    "docs/user-guide/cli-reference.md",
    "architecture/ux/cli-reference.md",
    "skills/SKILL.md",
    "src/cli/commands.rs",
    "src/cli/diagnostic/mod.rs",
    "src/entities/diagnostic/get.rs",
)
EXPECTED_PUBLIC_GTR_COMMANDS = {
    "biomcp get diagnostic GTR000006692.3",
    "biomcp get diagnostic GTR000006692.3 genes",
    "biomcp get diagnostic GTR000006692.3 conditions",
    "biomcp get diagnostic GTR000006692.3 methods",
    "biomcp get diagnostic GTR000006692.3 regulatory",
    "biomcp get diagnostic GTR000006692.3 all",
    "biomcp get diagnostic GTR000006692.3 genes conditions",
}
PUBLIC_GTR_COMMAND_RE = re.compile(
    r"\bbiomcp\s+get\s+diagnostic\s+GTR\d{9}\.\d+"
    r"(?:\s+(?:genes|conditions|methods|regulatory|all))*"
)


def _read_repo_text(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _extract_public_gtr_commands(text: str) -> set[str]:
    return {
        " ".join(match.group(0).split())
        for match in PUBLIC_GTR_COMMAND_RE.finditer(text)
    }


def _surface_commands() -> set[str]:
    commands: set[str] = set()
    for surface in PUBLIC_DIAGNOSTIC_EXAMPLE_SURFACES:
        commands.update(_extract_public_gtr_commands(_read_repo_text(surface)))
    return commands


def _run_biomcp_command(
    command: str,
    env: dict[str, str],
    timeout: int = 60,
) -> subprocess.CompletedProcess[str]:
    argv = shlex.split(command)
    assert argv[0] == "biomcp", command
    return subprocess.run(
        [str(RELEASE_BIN), *argv[1:]],
        cwd=REPO_ROOT,
        env=env,
        shell=False,
        capture_output=True,
        text=True,
        timeout=timeout,
    )


def _runtime_help() -> str:
    assert RELEASE_BIN.exists(), f"release binary is required: {RELEASE_BIN}"
    result = subprocess.run(
        [str(RELEASE_BIN), "get", "diagnostic", "--help"],
        cwd=REPO_ROOT,
        shell=False,
        capture_output=True,
        text=True,
        timeout=30,
    )
    assert result.returncode == 0, (
        "biomcp get diagnostic --help failed\n"
        f"stdout:\n{result.stdout}\n"
        f"stderr:\n{result.stderr}"
    )
    return result.stdout


def _prepare_public_gtr_bundle(target_dir: Path) -> Path:
    shutil.copytree(GTR_FIXTURE_DIR, target_dir)
    condition_gene_path = target_dir / GTR_CONDITION_GENE_FILE
    condition_gene_path.write_text(
        condition_gene_path.read_text(encoding="utf-8").replace(
            FICTIONAL_GTR_ACCESSION,
            LIVE_GTR_ACCESSION,
        ),
        encoding="utf-8",
    )
    version_path = target_dir / GTR_TEST_VERSION_FILE
    with gzip.open(version_path, "rt", encoding="utf-8") as fh:
        version_payload = fh.read()
    with gzip.open(version_path, "wt", encoding="utf-8") as fh:
        fh.write(version_payload.replace(FICTIONAL_GTR_ACCESSION, LIVE_GTR_ACCESSION))
    return target_dir


def test_public_diagnostic_surfaces_use_live_gtr_accession() -> None:
    missing_live = []
    fictional_leaks = []

    for surface in PUBLIC_DIAGNOSTIC_EXAMPLE_SURFACES:
        text = _read_repo_text(surface)
        if LIVE_GTR_ACCESSION not in text:
            missing_live.append(surface)
        if FICTIONAL_GTR_ACCESSION in text:
            fictional_leaks.append(surface)

    assert missing_live == []
    assert fictional_leaks == []

    commands = _surface_commands()
    assert EXPECTED_PUBLIC_GTR_COMMANDS <= commands
    assert all(LIVE_GTR_ACCESSION in command for command in commands)
    assert all(FICTIONAL_GTR_ACCESSION not in command for command in commands)


def test_runtime_help_uses_live_gtr_examples() -> None:
    help_text = _runtime_help()

    assert "biomcp get diagnostic GTR000006692.3" in help_text
    assert "biomcp get diagnostic GTR000006692.3 genes" in help_text
    assert "biomcp get diagnostic GTR000006692.3 regulatory" in help_text
    assert 'GTR000006692.3 or "ITPW02232- TC40"' in help_text
    assert FICTIONAL_GTR_ACCESSION not in help_text


def test_public_gtr_examples_resolve_against_live_gtr_bundle(tmp_path: Path) -> None:
    env = os.environ.copy()
    env["BIOMCP_GTR_DIR"] = str(_prepare_public_gtr_bundle(tmp_path / "gtr"))
    env["BIOMCP_OPENFDA_BASE"] = "http://127.0.0.1:9"

    commands = _surface_commands() | _extract_public_gtr_commands(_runtime_help())
    assert EXPECTED_PUBLIC_GTR_COMMANDS <= commands

    for command in sorted(commands):
        result = _run_biomcp_command(command, env=env, timeout=60)
        combined_output = f"{result.stdout}\n{result.stderr}"
        assert result.returncode == 0, (
            f"{command} failed\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
        assert LIVE_GTR_ACCESSION in result.stdout, command
        assert "not found" not in combined_output.lower(), command

        sections = shlex.split(command)[4:]
        if sections == ["all"]:
            assert "## Genes" in result.stdout, command
            assert "## Conditions" in result.stdout, command
            assert "## Methods" in result.stdout, command
            continue

        for section in sections:
            if section == "genes":
                assert "## Genes" in result.stdout, command
                assert "BRCA1" in result.stdout, command
            elif section == "conditions":
                assert "## Conditions" in result.stdout, command
            elif section == "methods":
                assert "## Methods" in result.stdout, command
            elif section == "regulatory":
                assert "## Regulatory (FDA Device)" in result.stdout, command
                assert REGULATORY_EMPTY_STATE in result.stdout, command
