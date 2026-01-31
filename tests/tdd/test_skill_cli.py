"""Unit tests for the BioMCP skill CLI commands."""

from pathlib import Path

from typer.testing import CliRunner

from biomcp.cli.main import app

runner = CliRunner()


def test_main_help_shows_skill_and_hides_install_skill():
    result = runner.invoke(app, ["--help"])
    assert result.exit_code == 0
    output = result.stdout.lower()
    assert "skill" in output
    assert "install-skill" not in output


def test_skill_default_shows_skill_md_and_cli_usage():
    result = runner.invoke(app, ["skill"])
    assert result.exit_code == 0
    assert "biomcp cli" in result.stdout.lower()
    assert "cli usage" in result.stdout.lower()


def test_skill_list_shows_use_cases():
    result = runner.invoke(app, ["skill", "list"])
    assert result.exit_code == 0
    output = result.stdout.lower()
    assert "available use-cases" in output
    assert "variant-to-treatment" in output


def test_skill_show_by_number():
    result = runner.invoke(app, ["skill", "01"])
    assert result.exit_code == 0
    assert "variant to treatment" in result.stdout.lower()


def test_skill_show_by_name():
    result = runner.invoke(app, ["skill", "variant-to-treatment"])
    assert result.exit_code == 0
    assert "variant to treatment" in result.stdout.lower()


def test_skill_unknown_use_case_shows_hint():
    result = runner.invoke(app, ["skill", "nope"])
    assert result.exit_code == 1
    assert "not found" in result.stdout.lower()
    assert "skill list" in result.stdout.lower()


def test_skills_alias_works():
    result = runner.invoke(app, ["skills", "list"])
    assert result.exit_code == 0
    assert "available use-cases" in result.stdout.lower()


def test_skill_install_resolves_root_to_skills_dir():
    with runner.isolated_filesystem():
        result = runner.invoke(
            app,
            ["skill", "install", "agent_root"],
            input="y\n",
        )

        assert result.exit_code == 0
        assert Path("agent_root/skills/biomcp/SKILL.md").exists()


def test_skill_install_existing_without_force_shows_hint():
    with runner.isolated_filesystem():
        first = runner.invoke(
            app,
            ["skill", "install", "agent_root"],
            input="y\n",
        )
        assert first.exit_code == 0

        second = runner.invoke(
            app,
            ["skill", "install", "agent_root"],
            input="y\n",
        )
        assert second.exit_code == 0
        assert "already installed" in second.stdout.lower()


def test_skill_install_force_replaces():
    with runner.isolated_filesystem():
        first = runner.invoke(
            app,
            ["skill", "install", "agent_root"],
            input="y\n",
        )
        assert first.exit_code == 0

        second = runner.invoke(
            app,
            ["skill", "install", "agent_root", "--force"],
            input="y\n",
        )
        assert second.exit_code == 0
        assert "replacing existing" in second.stdout.lower()
