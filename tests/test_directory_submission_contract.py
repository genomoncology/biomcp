from __future__ import annotations

from collections.abc import Iterable, Sequence
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
REQUIRED_EXAMPLE_MARKERS = (
    "**User prompt:**",
    "**Expected tool call:**",
    "**Expected behavior:**",
    "**Expected output:**",
)
SPEC_BARE_PYTHON_PATTERN = re.compile(r"(?<![A-Za-z0-9_])python(?=(?: |$|-))")
TRACKED_MARCH_ALLOWLIST = frozenset(
    {
        ".march/code-review-log.md",
        ".march/validation-profiles.toml",
    }
)


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _read_json(path: str) -> dict[str, object]:
    return json.loads(_read(path))


def _markdown_section_block(text: str, heading: str, next_heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    end = remainder.find(next_heading)
    if end == -1:
        return remainder
    return remainder[:end]


def _unexpected_tracked_march_paths(tracked_files: Iterable[str]) -> list[str]:
    return sorted(
        path
        for path in tracked_files
        if path.startswith(".march/") and path not in TRACKED_MARCH_ALLOWLIST
    )


def _format_unexpected_tracked_march_paths(paths: Sequence[str]) -> str:
    lines = [
        "Unexpected tracked .march artifacts detected.",
        "Offending paths:",
        *(f"- {path}" for path in paths),
        "Allowed tracked .march paths:",
        *(f"- {path}" for path in sorted(TRACKED_MARCH_ALLOWLIST)),
        "Remove non-allowlisted artifacts from the index with:",
        "git rm --cached -- <path>",
    ]
    return "\n".join(lines)


def test_manifest_matches_directory_bundle_contract() -> None:
    manifest = _read_json("manifest.json")
    cargo = tomllib.loads(_read("Cargo.toml"))
    pyproject = tomllib.loads(_read("pyproject.toml"))

    assert manifest["manifest_version"] == "0.3"
    assert manifest["name"] == "biomcp"
    assert manifest["display_name"] == "BioMCP"
    assert manifest["version"] == cargo["package"]["version"]
    assert manifest["version"] == pyproject["project"]["version"]
    assert manifest["privacy_policies"] == ["https://biomcp.org/policies/"]
    assert "tools_generated" not in manifest

    server = manifest["server"]
    assert isinstance(server, dict)
    assert server["type"] == "binary"
    assert server["entry_point"] == "server/biomcp"

    mcp_config = server["mcp_config"]
    assert isinstance(mcp_config, dict)
    assert mcp_config["command"] == "server/biomcp"
    assert mcp_config["args"] == ["serve"]
    assert mcp_config["env"] == {
        "ONCOKB_TOKEN": "${user_config.oncokb_token}",
        "DISGENET_API_KEY": "${user_config.disgenet_api_key}",
        "S2_API_KEY": "${user_config.s2_api_key}",
    }

    tools = manifest["tools"]
    assert isinstance(tools, list)
    assert len(tools) == 1
    assert tools[0]["name"] == "biomcp"
    assert "read-only" in str(tools[0]["description"]).lower()
    assert "ONCOKB_API_KEY" not in json.dumps(manifest)

    compatibility = manifest["compatibility"]
    assert isinstance(compatibility, dict)
    assert sorted(compatibility["platforms"]) == ["darwin", "win32"]


def test_packaging_workspace_is_ignored_and_bundle_payload_is_filtered() -> None:
    mcpbignore = _read(".mcpbignore")

    for entry in (
        ".git/",
        ".march/",
        "src/",
        "tests/",
        "docs/",
        "spec/",
        "target/",
        "dist/",
        "*.md",
    ):
        assert entry in mcpbignore

    gitignore = _read(".gitignore")
    assert "/server/" in gitignore

    for entry in (".march/", ".claude/", ".agents/"):
        assert entry in gitignore

    assert "architecture/" in mcpbignore
    for removed_entry in ("design/", "demo/", "presentations/"):
        assert removed_entry not in mcpbignore


def test_tracked_march_allowlist_is_exhaustive_and_actionable() -> None:
    assert TRACKED_MARCH_ALLOWLIST == frozenset(
        {
            ".march/code-review-log.md",
            ".march/validation-profiles.toml",
        }
    )

    tracked_files = [
        ".march/code-review-log.md",
        ".march/validation-profiles.toml",
        ".march/verify-log.md",
        ".march/blueprint.md",
        ".claude/settings.local.json",
    ]

    unexpected_paths = _unexpected_tracked_march_paths(tracked_files)

    assert unexpected_paths == [".march/blueprint.md", ".march/verify-log.md"]

    message = _format_unexpected_tracked_march_paths(unexpected_paths)
    for path in unexpected_paths:
        assert path in message
    for path in TRACKED_MARCH_ALLOWLIST:
        assert path in message
    assert "git rm --cached -- <path>" in message
    assert ".claude/settings.local.json" not in unexpected_paths


def test_repo_cleanup_removes_local_artifacts_and_deleted_dirs_from_git() -> None:
    tracked_files = subprocess.run(
        ["git", "ls-files"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()

    unexpected_march_paths = _unexpected_tracked_march_paths(tracked_files)
    assert not unexpected_march_paths, _format_unexpected_tracked_march_paths(
        unexpected_march_paths
    )

    assert not [
        path for path in tracked_files if path.startswith((".claude/", ".agents/"))
    ]

    assert not (REPO_ROOT / "presentations").exists()
    assert not (REPO_ROOT / "demo").exists()
    assert not (REPO_ROOT / "design").exists()

    for path in (
        "architecture/functional/overview.md",
        "architecture/technical/overview.md",
        "architecture/technical/source-integration.md",
        "architecture/technical/staging-demo.md",
        "architecture/ux/cli-reference.md",
    ):
        assert (REPO_ROOT / path).is_file()

    for path in (
        "examples/test-predictions-output.md",
        "examples/test-predictions-prompt.md",
        "examples/test-predictions-stderr.log",
    ):
        assert not (REPO_ROOT / path).exists()


def test_specs_do_not_depend_on_bare_python_alias() -> None:
    bare_python_refs: list[str] = []

    for path in sorted((REPO_ROOT / "spec").rglob("*.md")):
        for line_no, line in enumerate(
            path.read_text(encoding="utf-8").splitlines(), start=1
        ):
            if SPEC_BARE_PYTHON_PATTERN.search(line):
                bare_python_refs.append(
                    f"{path.relative_to(REPO_ROOT)}:{line_no}: {line.strip()}"
                )

    assert not bare_python_refs, "\n".join(bare_python_refs)


def test_gene_canary_runs_as_a_targeted_heading() -> None:
    env = dict(os.environ)
    env["PATH"] = f"{REPO_ROOT / 'target' / 'release'}:{env['PATH']}"
    spec_root = REPO_ROOT / "spec"

    try:
        subprocess.run(
            [
                sys.executable,
                "-m",
                "pytest",
                "spec/entity/gene.md",
                "--mustmatch-lang",
                "bash",
                "--mustmatch-timeout",
                "60",
                "-k",
                "Symbol and Based",
                "-v",
            ],
            cwd=REPO_ROOT,
            env=env,
            check=True,
            capture_output=True,
            text=True,
        )
    finally:
        subprocess.run(["rm", "-rf", str(spec_root / ".cache")], check=False)


def test_examples_tree_has_linked_index_and_readmes() -> None:
    examples_index = _read("examples/README.md")
    example_dirs = sorted(
        path for path in (REPO_ROOT / "examples").iterdir() if path.is_dir()
    )

    assert "| [geneagent/](geneagent/README.md) |" in examples_index
    assert "| [genegpt/](genegpt/README.md) |" in examples_index
    assert "| [pubmed-beyond/](pubmed-beyond/README.md) |" in examples_index
    assert "| [trialgpt/](trialgpt/README.md) |" in examples_index
    assert "| [streamable-http/](streamable-http/README.md) |" in examples_index

    for path in example_dirs:
        assert (path / "README.md").is_file()


def test_example_scripts_pass_minimum_syntax_validation() -> None:
    subprocess.run(
        [
            sys.executable,
            "-m",
            "py_compile",
            "examples/streamable-http/streamable_http_client.py",
        ],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )

    subprocess.run(
        [
            "bash",
            "-n",
            "examples/geneagent/run.sh",
            "examples/geneagent/score.sh",
            "examples/genegpt/run.sh",
            "examples/genegpt/score.sh",
            "examples/pubmed-beyond/run.sh",
            "examples/pubmed-beyond/score.sh",
            "examples/trialgpt/run.sh",
            "examples/trialgpt/score.sh",
        ],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )


def test_readme_is_directory_review_complete() -> None:
    readme = _read("README.md")

    required_sections = [
        "## Description",
        "## Features",
        "## Installation",
        "## Configuration",
        "## Usage Examples",
        "## Privacy Policy",
        "## Support",
        "## Documentation",
        "## Citation",
        "## Data Sources and Licensing",
        "## License",
    ]
    positions = [readme.index(section) for section in required_sections]
    assert positions == sorted(positions)

    installation = _markdown_section_block(
        readme, "## Installation", "\n## Quick start"
    )
    assert "### Claude Desktop extension (.mcpb)" in installation
    assert "Anthropic Directory" in installation

    configuration = _markdown_section_block(
        readme,
        "## Configuration",
        "\n## Usage Examples",
    )
    assert "### Claude Desktop extension settings" in configuration
    for label, env_var in (
        ("OncoKB Token", "ONCOKB_TOKEN"),
        ("DisGeNET API Key", "DISGENET_API_KEY"),
        ("Semantic Scholar API Key", "S2_API_KEY"),
    ):
        assert label in configuration
        assert env_var in configuration
    assert "first directory build exposes only those three optional settings" in (
        configuration.lower()
    )

    examples = _markdown_section_block(
        readme, "## Usage Examples", "\n## Privacy Policy"
    )
    assert examples.count("**User prompt:**") >= 4
    assert examples.count("**Expected tool call:**") >= 4
    assert examples.count("**Expected behavior:**") >= 4
    assert examples.count("**Expected output:**") >= 4
    for prompt, call in (
        (
            "Give me a low-noise overview of BRAF in melanoma.",
            "biomcp search all --gene BRAF --disease melanoma --counts-only",
        ),
        (
            "Summarize ClinVar significance and population frequency for BRAF V600E.",
            'biomcp get variant "BRAF V600E" clinvar population',
        ),
        (
            "Show OncoKB therapy evidence for BRAF V600E.",
            'biomcp variant oncokb "BRAF V600E"',
        ),
        (
            "Show scored DisGeNET associations for TP53.",
            "biomcp get gene TP53 disgenet",
        ),
    ):
        assert prompt in examples
        assert call in examples

    example_blocks = [
        block.strip() for block in re.split(r"(?m)^### ", examples) if block.strip()
    ]
    complete_examples = [
        block
        for block in example_blocks
        if all(marker in block for marker in REQUIRED_EXAMPLE_MARKERS)
    ]
    assert len(complete_examples) >= 4

    privacy = _markdown_section_block(readme, "## Privacy Policy", "\n## Support")
    assert "https://biomcp.org/policies/" in privacy

    support = _markdown_section_block(readme, "## Support", "\n## Documentation")
    assert "github.com/genomoncology/biomcp/issues" in support
    assert "docs/troubleshooting.md" in support


def test_policies_page_covers_directory_privacy_requirements() -> None:
    policies = _read("docs/policies.md")
    lowered = policies.lower()

    assert "# privacy policy" in lowered
    assert "telemetry" in lowered
    assert "anthropic" in lowered
    assert "claude" in lowered
    assert "upstream providers" in lowered or "third-party providers" in lowered
    assert "retention" in lowered
    assert "api keys" in lowered
    assert "read-only" in lowered
    assert "privacy@" in lowered or "contact" in lowered
    assert "source licensing reference" in lowered
    assert "clinical" in lowered


def test_release_workflow_updates_manifest_version_for_release_builds() -> None:
    release = _read(".github/workflows/release.yml")

    assert "Sync Cargo.toml and manifest.json version from release tag" in release
    assert (
        "Sync Cargo.toml, pyproject.toml, and manifest.json version from release tag"
        in release
    )
    assert release.count("manifest.json") >= 4
