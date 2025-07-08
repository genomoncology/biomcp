#!/usr/bin/env python3
"""Check that all markdown files in docs/ are referenced in mkdocs.yml."""

import sys
from pathlib import Path

import yaml  # DEP004


def main():
    """Check documentation files are in mkdocs.yml."""
    docs_dir = Path(__file__).parent.parent / "docs"
    mkdocs_path = Path(__file__).parent.parent / "mkdocs.yml"

    # Load mkdocs.yml
    with open(mkdocs_path) as f:
        mkdocs_config = yaml.safe_load(f)

    # Extract all referenced files from nav
    referenced_files = set()

    def extract_files(nav_item, prefix=""):
        """Recursively extract file paths from nav structure."""
        if isinstance(nav_item, dict):
            for _key, value in nav_item.items():
                extract_files(value, prefix)
        elif isinstance(nav_item, list):
            for item in nav_item:
                extract_files(item, prefix)
        elif isinstance(nav_item, str) and nav_item.endswith(".md"):
            referenced_files.add(nav_item)

    extract_files(mkdocs_config.get("nav", []))

    # Find all markdown files in docs/
    all_md_files = set()
    for md_file in docs_dir.rglob("*.md"):
        # Get relative path from docs/
        rel_path = md_file.relative_to(docs_dir)
        all_md_files.add(str(rel_path))

    # Find unreferenced files
    unreferenced = all_md_files - referenced_files

    # Exclude some files that shouldn't be in nav
    exclude_patterns = {
        "CNAME",  # GitHub pages config
        "README.md",  # If exists
    }

    unreferenced = {
        f
        for f in unreferenced
        if not any(pattern in f for pattern in exclude_patterns)
    }

    if unreferenced:
        print(
            "The following documentation files are not referenced in mkdocs.yml:"
        )
        for file in sorted(unreferenced):
            print(f"  - {file}")
        print("\nPlease add them to the appropriate section in mkdocs.yml")
        return 1
    else:
        print("All documentation files are referenced in mkdocs.yml ✓")
        return 0


if __name__ == "__main__":
    sys.exit(main())
