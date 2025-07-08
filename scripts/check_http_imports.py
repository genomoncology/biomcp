#!/usr/bin/env python3
"""Check for direct HTTP library imports outside of allowed files."""

import ast
import sys
from pathlib import Path

# HTTP libraries to check for
HTTP_LIBRARIES = {
    "httpx",
    "aiohttp",
    "requests",
    "urllib3",
}  # Note: urllib is allowed for URL parsing

# Files allowed to import HTTP libraries
ALLOWED_FILES = {
    "http_client.py",
    "http_client_simple.py",
    "http_client_test.py",
    "test_http_client.py",
    "connection_pool.py",  # Connection pooling infrastructure
}

# Additional allowed patterns (for version checks, etc.)
ALLOWED_PATTERNS = {
    # Allow httpx import just for version check
    ("health.py", "httpx"): "version check only",
}


def _check_import_node(
    node: ast.Import, file_name: str
) -> set[tuple[str, int]]:
    """Check ast.Import node for violations."""
    violations = set()
    for alias in node.names:
        module_name = alias.name.split(".")[0]
        if module_name in HTTP_LIBRARIES:
            pattern_key = (file_name, module_name)
            if pattern_key not in ALLOWED_PATTERNS:
                violations.add((module_name, node.lineno))
    return violations


def _check_import_from_node(
    node: ast.ImportFrom, file_name: str
) -> set[tuple[str, int]]:
    """Check ast.ImportFrom node for violations."""
    violations = set()
    if node.module:
        module_name = node.module.split(".")[0]
        if module_name in HTTP_LIBRARIES:
            pattern_key = (file_name, module_name)
            if pattern_key not in ALLOWED_PATTERNS:
                violations.add((module_name, node.lineno))
    return violations


def check_imports(file_path: Path) -> set[tuple[str, int]]:
    """Check a Python file for HTTP library imports.

    Returns set of (library, line_number) tuples for violations.
    """
    violations = set()

    # Check if this file is allowed
    if file_path.name in ALLOWED_FILES:
        return violations

    try:
        with open(file_path, encoding="utf-8") as f:
            content = f.read()

        tree = ast.parse(content)

        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                violations.update(_check_import_node(node, file_path.name))
            elif isinstance(node, ast.ImportFrom):
                violations.update(
                    _check_import_from_node(node, file_path.name)
                )

    except Exception as e:
        print(f"Error parsing {file_path}: {e}", file=sys.stderr)

    return violations


def find_python_files(root_dir: Path) -> list[Path]:
    """Find all Python files in the project."""
    python_files = []

    for path in root_dir.rglob("*.py"):
        # Skip virtual environments, cache, etc.
        if any(
            part.startswith(".")
            or part in ["__pycache__", "venv", "env", ".tox"]
            for part in path.parts
        ):
            continue
        python_files.append(path)

    return python_files


def main():
    """Main function to check all Python files."""
    # Get project root (parent of scripts directory)
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    src_dir = project_root / "src"

    # Find all Python files
    python_files = find_python_files(src_dir)

    all_violations = []

    for file_path in python_files:
        violations = check_imports(file_path)
        if violations:
            for lib, line in violations:
                all_violations.append((file_path, lib, line))

    if all_violations:
        print("❌ Found direct HTTP library imports:\n")
        for file_path, lib, line in sorted(all_violations):
            rel_path = file_path.relative_to(project_root)
            print(f"  {rel_path}:{line} - imports '{lib}'")

        print(f"\n❌ Total violations: {len(all_violations)}")
        print(
            "\nPlease use the centralized HTTP client (biomcp.http_client) instead."
        )
        print(
            "If you need to add an exception, update ALLOWED_FILES or ALLOWED_PATTERNS in this script."
        )
        return 1
    else:
        print("✅ No direct HTTP library imports found outside allowed files.")
        return 0


if __name__ == "__main__":
    sys.exit(main())
