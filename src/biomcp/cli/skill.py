"""CLI commands for BioMCP agent skills (view, list, install)."""

import re
import shutil
from pathlib import Path

import click
import typer
from typer.core import TyperGroup

SKILLS_SOURCE = Path(__file__).parent.parent / "skills" / "biomcp"
SKILL_MD = SKILLS_SOURCE / "SKILL.md"
USE_CASES_DIR = SKILLS_SOURCE / "use-cases"

CLI_USAGE_SUFFIX = """
---
## CLI Usage

View skills:
  biomcp skill              Show this help
  biomcp skill list         List use-case patterns
  biomcp skill <name>       Show specific use-case (by name or number)

Install skills:
  biomcp skill install      Auto-detect agent and prompt
  biomcp skill install <dir>  Install to specific directory

Examples:
  biomcp skill 01                    # Show 01-variant-to-treatment
  biomcp skill variant-to-treatment  # Same (without number prefix)
  biomcp skill install ~/.claude     # Install for Claude Code
"""

KNOWN_AGENTS: list[tuple[str, Path]] = [
    ("Claude Code", Path.home() / ".claude"),
    ("OpenAI Codex", Path.home() / ".codex"),
    ("OpenCode", Path.home() / ".config" / "opencode"),
    ("Gemini CLI", Path.home() / ".gemini"),
    ("Pi Agent", Path.home() / ".pi"),
]


def _path_with_tilde(path: Path) -> str:
    home = Path.home()
    try:
        relative = path.resolve().relative_to(home.resolve())
    except ValueError:
        return str(path)
    return str(Path("~") / relative)


def _load_use_case_patterns() -> dict[str, str]:
    if not SKILL_MD.exists():
        return {}

    patterns: dict[str, str] = {}
    content = SKILL_MD.read_text(encoding="utf-8")
    for line in content.splitlines():
        match = re.match(
            r"^\|\s*`(?P<stem>[^`]+)`\s*\|\s*(?P<pattern>[^|]+?)\s*\|\s*$",
            line,
        )
        if match:
            stem = match.group("stem").strip()
            patterns[stem] = match.group("pattern").strip()
    return patterns


def _build_use_case_index() -> dict[str, Path]:
    index: dict[str, Path] = {}
    if not USE_CASES_DIR.exists():
        return index

    for path in sorted(USE_CASES_DIR.glob("*.md")):
        stem = path.stem
        if "-" not in stem:
            continue
        number, name = stem.split("-", 1)
        if not number.isdigit():
            continue
        index[stem] = path
        index[number] = path
        index[name] = path
    return index


def _resolve_use_case_path(name_or_number: str) -> Path | None:
    query = name_or_number.strip().removesuffix(".md")
    if not query:
        return None

    if query.isdigit():
        query = f"{int(query):02d}"

    return _build_use_case_index().get(query)


def _render_use_case_list() -> str:
    patterns = _load_use_case_patterns()

    use_cases: list[tuple[str, str, str]] = []
    if USE_CASES_DIR.exists():
        for path in sorted(USE_CASES_DIR.glob("*.md")):
            stem = path.stem
            if "-" not in stem:
                continue
            number, name = stem.split("-", 1)
            if not number.isdigit():
                continue
            use_cases.append((number, name, patterns.get(stem, "")))

    if not use_cases:
        return "No use-cases found."

    name_width = max(len(name) for _, name, _ in use_cases)
    lines = ["Available use-cases:", ""]
    for number, name, pattern in use_cases:
        padded_name = name.ljust(name_width)
        lines.append(f"  {number}  {padded_name}   {pattern}".rstrip())
    lines.append("")
    lines.append("View details: biomcp skill <number-or-name>")
    return "\n".join(lines)


def _show_skill_overview() -> None:
    if not SKILL_MD.exists():
        typer.echo(f"Error: SKILL.md not found at {SKILL_MD}", err=True)
        raise typer.Exit(1)

    typer.echo(SKILL_MD.read_text(encoding="utf-8").rstrip())
    typer.echo(CLI_USAGE_SUFFIX.rstrip())


def _show_use_case(name_or_number: str) -> None:
    use_case_path = _resolve_use_case_path(name_or_number)
    if use_case_path is None or not use_case_path.exists():
        typer.echo(
            (
                f"Use-case '{name_or_number}' not found. "
                "Run 'biomcp skill list' to see available options."
            ),
            err=True,
        )
        raise typer.Exit(1)

    typer.echo(use_case_path.read_text(encoding="utf-8").rstrip())


class SkillGroup(TyperGroup):
    def get_command(
        self, ctx: click.Context, cmd_name: str
    ) -> click.Command | None:
        command = super().get_command(ctx, cmd_name)
        if command is not None:
            return command

        def show_dynamic_use_case() -> None:
            _show_use_case(cmd_name)

        return click.Command(cmd_name, callback=show_dynamic_use_case)


def _resolve_install_target(directory: str) -> Path:
    path = Path(directory).expanduser()
    parts = path.parts

    if len(parts) >= 2 and parts[-2] == "skills":
        return path
    if parts and parts[-1] == "skills":
        return path / "biomcp"
    return path / "skills" / "biomcp"


def _install_to_target(
    target_dir: Path,
    *,
    force: bool,
    replace_hint: str,
) -> None:
    if not SKILLS_SOURCE.exists():
        typer.echo(
            f"Error: Skills source not found at {SKILLS_SOURCE}", err=True
        )
        typer.echo(
            "This may indicate an incomplete BioMCP installation.", err=True
        )
        raise typer.Exit(1)

    target_dir = target_dir.expanduser()
    target_dir.parent.mkdir(parents=True, exist_ok=True)

    if target_dir.exists() and not force:
        typer.echo(
            f"BioMCP skills already installed at {_path_with_tilde(target_dir)}/"
        )
        typer.echo("")
        typer.echo(f"To replace: biomcp skill install {replace_hint} --force")
        typer.echo("To view:    biomcp skill")
        return

    if target_dir.exists() and force:
        typer.echo(
            "Replacing existing BioMCP skills at "
            f"{_path_with_tilde(target_dir)}/"
        )
        shutil.rmtree(target_dir)

    shutil.copytree(SKILLS_SOURCE, target_dir)
    typer.echo(f"Installed BioMCP skills to {_path_with_tilde(target_dir)}/")


def _install_interactive(
    target_dir: Path,
    *,
    force: bool,
    replace_hint: str,
) -> None:
    if not typer.confirm(
        f"Install BioMCP skills to {_path_with_tilde(target_dir)}?",
        default=True,
    ):
        return

    _install_to_target(target_dir, force=force, replace_hint=replace_hint)


def _install_autodetect(*, force: bool) -> None:
    found: list[tuple[str, Path, Path]] = []
    for agent_name, agent_dir in KNOWN_AGENTS:
        if agent_dir.exists():
            target_dir = agent_dir / "skills" / "biomcp"
            found.append((agent_name, agent_dir, target_dir))

    if not found:
        typer.echo("No known agent directories found.")
        typer.echo("Specify a directory: biomcp skill install <directory>")
        typer.echo("")
        typer.echo("Common locations:")
        typer.echo("  ~/.claude/skills/     Claude Code")
        typer.echo("  ~/.codex/skills/      OpenAI Codex")
        typer.echo("  ~/.pi/skills/         Pi Agent")
        raise typer.Exit(1)

    for agent_name, agent_dir, target_dir in found:
        typer.echo(f"Found {agent_name} at {_path_with_tilde(agent_dir)}/")
        _install_interactive(
            target_dir, force=force, replace_hint=_path_with_tilde(agent_dir)
        )


def build_skill_app() -> typer.Typer:
    app = typer.Typer(
        cls=SkillGroup,
        help="Agent skills for the BioMCP CLI (view, list, install).",
        invoke_without_command=True,
        no_args_is_help=False,
    )

    @app.callback(invoke_without_command=True)
    def skill_callback(ctx: typer.Context) -> None:
        if ctx.invoked_subcommand is None:
            _show_skill_overview()

    @app.command("list")
    def list_use_cases() -> None:
        """List available use-cases."""
        typer.echo(_render_use_case_list())

    @app.command("install")
    def install(
        directory: str | None = typer.Argument(
            None,
            help="Target directory (e.g., ~/.claude or ~/.claude/skills/)",
        ),
        force: bool = typer.Option(
            False,
            "--force",
            help="Replace existing installation if present.",
        ),
    ) -> None:
        """Install BioMCP skills to an agent skills directory."""
        if directory is None:
            _install_autodetect(force=force)
            return

        target_dir = _resolve_install_target(directory)
        _install_interactive(target_dir, force=force, replace_hint=directory)

    return app
