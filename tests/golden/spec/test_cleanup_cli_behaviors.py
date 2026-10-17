"""Caller-visible replacements for redundant help and label-construction tests."""

from __future__ import annotations

import json
import subprocess
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory

import pytest

from replay_env import REPO_ROOT, ReplayEnvironment, _read_env_file
from test_golden_output import BINARY

pytestmark = pytest.mark.needs_binary


@pytest.mark.parametrize(
    ("command", "fragments"),
    [
        (
            ("search", "article", "--help"),
            (
                "When to use:", "QUERY FORMULATION:", "Tokens are not secrets",
                "Published after date (YYYY, YYYY-MM, or YYYY-MM-DD)",
                "--max-per-source <N>", "Prefer --type review",
            ),
        ),
        (
            ("get", "article", "--help"),
            (
                "Allow Semantic Scholar PDF as a final fulltext fallback",
                "--output <FILE>",
                "Binary or unknown-type assets are refused when standard output is a terminal",
            ),
        ),
    ],
)
def test_article_help_through_the_command_line(
    replay: ReplayEnvironment, command: tuple[str, ...], fragments: tuple[str, ...]
) -> None:
    result = replay.run(BINARY, command).process
    assert result.returncode == 0, result.stderr
    for fragment in fragments:
        assert fragment in result.stdout


@pytest.fixture
def drug_cli(replay: ReplayEnvironment):
    """Use the existing recorded provider fixture and shared command isolation."""
    cache = REPO_ROOT / ".cache"
    cache.mkdir(exist_ok=True)
    scripts = REPO_ROOT / "spec/fixtures"
    with TemporaryDirectory(prefix="label-cli-", dir=cache) as directory:
        workspace = Path(directory)
        try:
            subprocess.run(
                ["bash", str(scripts / "setup-provider-contract-spec-fixture.sh"), directory],
                check=True, capture_output=True, text=True, timeout=60,
            )
            environment = _read_env_file(workspace / ".cache/spec-provider-contract-env")
            yield replace(replay, provider_env=environment)
        finally:
            subprocess.run(
                ["bash", str(scripts / "cleanup-provider-contract-spec-fixture.sh"), directory],
                check=True, capture_output=True, text=True, timeout=60,
            )


def test_raw_label_keeps_whole_sections_and_approved_indications(drug_cli) -> None:
    run = drug_cli.run(BINARY, ("--json", "get", "drug", "osimertinib", "label", "--raw"))
    assert run.process.returncode == 0, run.process.stderr
    label = json.loads(run.process.stdout)["label"]
    assert "TAGRISSO is a kinase inhibitor" in label["indications"]
    assert "80 mg orally once daily" in label["dosage"]
    assert label["warnings"]
    assert label["indication_summary"]


def test_interactions_show_the_labels_own_guidance(drug_cli) -> None:
    run = drug_cli.run(BINARY, ("get", "drug", "gefitinib", "interactions"))
    assert run.process.returncode == 0, run.process.stderr
    assert "## Additive Label Text (OpenFDA)" in run.process.stdout
    assert "CYP3A4 Inducer: Increase gefitinib tablets to 500 mg daily" in run.process.stdout
