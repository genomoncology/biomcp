#!/usr/bin/env python3
"""Generate THIRD_PARTY_ENDPOINTS.md documentation."""

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
    
    # Check if file exists and has same content
    if output_path.exists():
        existing_content = output_path.read_text()
        if existing_content == new_content:
            print(f"{output_path} is up to date")
            return
    
    # Write new content
    output_path.write_text(new_content)
    print(f"Generated {output_path}")


if __name__ == "__main__":
    main()