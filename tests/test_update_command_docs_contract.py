"""Docs contract for the fail-closed `biomcp update` checksum behavior.

These assertions pin the docs surface for the self-update verification
requirement and the explicit `--allow-missing-checksum` UNSAFE override.
The runtime change lives in `src/cli/update.rs`; these tests catch
docs/runtime drift where the runtime is fail-closed but a doc surface
still describes the old fail-open behavior.
"""

from __future__ import annotations

import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def test_update_command_docs_describe_verification_requirement() -> None:
    cli_reference = _read("docs/user-guide/cli-reference.md")

    assert "biomcp update [--check] [--allow-missing-checksum]" in cli_reference, (
        "docs/user-guide/cli-reference.md must list the new flag in the "
        "command grammar so the docs match runtime"
    )
    assert re.search(r"SHA-?256", cli_reference, flags=re.IGNORECASE), (
        "cli-reference.md must name SHA256 verification for biomcp update"
    )
    assert "checksum" in cli_reference.lower(), (
        "cli-reference.md must describe checksum verification for biomcp update"
    )
    assert "--allow-missing-checksum" in cli_reference, (
        "cli-reference.md must name the unsafe override flag in prose so "
        "operators can find it from the docs surface"
    )


def test_update_troubleshooting_describes_failclosed_and_unsafe_override() -> None:
    troubleshooting = _read("docs/troubleshooting.md")

    assert "biomcp update" in troubleshooting
    assert "--allow-missing-checksum" in troubleshooting, (
        "troubleshooting.md must point operators at the unsafe override when "
        "a legitimate release ships without a sidecar"
    )
    assert "UNSAFE" in troubleshooting, (
        "troubleshooting.md must mark the override UNSAFE so operators "
        "do not turn it on casually"
    )
    assert re.search(r"fail(?:s|ed|ing)?[- ]closed", troubleshooting, re.IGNORECASE), (
        "troubleshooting.md must describe the new fail-closed behavior so "
        "operators know an update can stop on missing sidecar"
    )


def test_architecture_cli_reference_lists_update_verification_flag() -> None:
    architecture = _read("architecture/ux/cli-reference.md")

    assert "biomcp update [--check] [--allow-missing-checksum]" in architecture, (
        "architecture/ux/cli-reference.md ops grammar must track the new "
        "flag so the durable architecture doc matches runtime"
    )
