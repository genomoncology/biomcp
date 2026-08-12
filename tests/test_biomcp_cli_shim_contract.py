from __future__ import annotations

from pathlib import Path
import tomllib


ROOT = Path(__file__).resolve().parents[1]


def test_compatibility_command_is_a_small_std_only_shim() -> None:
    source = (ROOT / "src/main_biomcp_cli.rs").read_text()
    assert len(source.splitlines()) <= 61
    assert "include!" not in source
    assert "biomcp_cli::" not in source
    assert "Command::new(sibling)" in source


def test_cargo_keeps_the_supported_biomcp_cli_command() -> None:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    bins = {item["name"]: item["path"] for item in cargo["bin"]}
    assert bins == {
        "biomcp": "src/main.rs",
        "biomcp-cli": "src/main_biomcp_cli.rs",
    }


def test_installation_docs_explain_package_name_and_command_alias() -> None:
    installation = (ROOT / "docs/getting-started/installation.md").read_text()
    assert "small `biomcp-cli` compatibility command" in installation
    assert "forwards to the sibling `biomcp` executable" in installation
