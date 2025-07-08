#!/usr/bin/env python3
"""Generate THIRD_PARTY_ENDPOINTS.md documentation."""

import shutil
import subprocess
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from biomcp.utils.endpoint_registry import get_registry


def main():
    """Generate the endpoints documentation."""
    registry = get_registry()
    output_path = Path(__file__).parent.parent / "THIRD_PARTY_ENDPOINTS.md"

    # Generate new content
    new_content = registry.generate_markdown_report()

    # Write new content
    output_path.write_text(new_content)

    # Run prettier to format the file
    npx_path = shutil.which("npx")
    if npx_path:
        try:
            # Safe: npx_path from shutil.which, output_path is controlled
            subprocess.run(  # noqa: S603
                [npx_path, "prettier", "--write", str(output_path)],
                check=True,
                capture_output=True,
                text=True,
            )
        except subprocess.CalledProcessError as e:
            print(f"Warning: prettier formatting failed: {e.stderr}")
    else:
        print("Warning: npx not found, skipping prettier formatting")

    print(f"Generated {output_path}")


if __name__ == "__main__":
    main()
