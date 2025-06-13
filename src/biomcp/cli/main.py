import importlib.metadata
from typing import Annotated, Optional

import typer

from .articles import article_app
from .health import health_app
from .server import run_server
from .trials import trial_app
from .variants import variant_app

# --- Get version from installed package metadata ---
try:
    __version__ = importlib.metadata.version("biomcp-python")
except importlib.metadata.PackageNotFoundError:
    __version__ = "unknown"  # Fallback if package not installed properly


# --- Callback for --version option ---
def version_callback(value: bool):
    if value:
        typer.echo(f"biomcp version: {__version__}")
        raise typer.Exit()


# --- Main Typer App ---
app = typer.Typer(
    help="BioMCP: Biomedical Model Context Protocol",
    no_args_is_help=True,
    # Add a callback to handle top-level options like --version
    # This callback itself doesn't do much, but allows defining eager options
    callback=lambda: None,
)

app.add_typer(
    trial_app,
    name="trial",
    no_args_is_help=True,
)

app.add_typer(
    article_app,
    name="article",
    no_args_is_help=True,
)

app.add_typer(
    variant_app,
    name="variant",
    no_args_is_help=True,
)

app.add_typer(
    health_app,
    name="health",
    no_args_is_help=True,
)


# --- Add --version Option using Annotation ---
# We add this directly to the app's callback invocation signature via annotation
# Note: This relies on Typer magic linking Annotated options in the callback signature
# This approach is cleaner than adding it to every subcommand.
@app.callback()
def main_callback(
    version: Annotated[
        Optional[bool],  # Allows the option to not be present
        typer.Option(
            "--version",  # The flag name
            callback=version_callback,  # Function to call when flag is used
            is_eager=True,  # Process this option before any commands
            help="Show the application's version and exit.",
        ),
    ] = None,  # Default value
):
    """
    BioMCP main application callback. Handles global options like --version.
    """
    # The actual logic is in version_callback due to is_eager=True
    pass


# --- Add Explicit 'version' Command ---
@app.command()
def version():
    """
    Display the installed biomcp version.
    """
    typer.echo(f"biomcp version: {__version__}")


# Directly expose run_server as the 'run' command with all its options
app.command("run")(run_server)


if __name__ == "__main__":
    app()
