from pathlib import Path

from .. import mcp_app

RESOURCES_ROOT = Path(__file__).parent


@mcp_app.resource("biomcp://instructions.md")
def get_instructions() -> str:
    return (RESOURCES_ROOT / "instructions.md").read_text()
