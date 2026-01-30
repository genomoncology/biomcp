"""CLI command for installing BioMCP agent skills."""

import shutil
from pathlib import Path

import typer

# Get the skills directory bundled with the package
SKILLS_SOURCE = Path(__file__).parent.parent / "skills" / "biomcp"


def install_skill(
    directory: str = typer.Argument(
        None,
        help="Target directory for skills installation (e.g., ~/.claude/skills/)",
    ),
) -> None:
    """
    Install BioMCP agent skills to the specified directory.

    The skills teach AI coding agents how to use the BioMCP CLI
    for biomedical research queries.
    """
    if directory is None:
        typer.echo("Usage: biomcp install-skill <directory>\n")
        typer.echo("Install the BioMCP skill to your AI agent's skills directory.\n")
        typer.echo("Examples:")
        typer.echo("  biomcp install-skill ~/.claude/skills/    # Claude Code")
        typer.echo("  biomcp install-skill ~/.codex/skills/     # OpenAI Codex")
        typer.echo("  biomcp install-skill ~/.opencode/skills/  # OpenCode")
        typer.echo("  biomcp install-skill ~/.gemini/skills/    # Gemini CLI")
        typer.echo("  biomcp install-skill ~/.pi/skills/        # Pi Agent")
        typer.echo("  biomcp install-skill ./.claude/skills/    # Project-local")
        raise typer.Exit(1)

    # Expand user path and resolve
    target_dir = Path(directory).expanduser().resolve()
    target_skill_dir = target_dir / "biomcp"

    # Check source exists
    if not SKILLS_SOURCE.exists():
        typer.echo(f"Error: Skills source not found at {SKILLS_SOURCE}", err=True)
        typer.echo("This may indicate an incomplete BioMCP installation.", err=True)
        raise typer.Exit(1)

    # Create target directory
    target_dir.mkdir(parents=True, exist_ok=True)

    # Copy skills (remove existing if present)
    if target_skill_dir.exists():
        shutil.rmtree(target_skill_dir)

    shutil.copytree(SKILLS_SOURCE, target_skill_dir)

    typer.echo(f"Installed BioMCP skill to {target_skill_dir}/")
