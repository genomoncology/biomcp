"""Test that endpoint documentation is kept up to date."""

import subprocess
import sys
from pathlib import Path


class TestEndpointDocumentation:
    """Test the endpoint documentation generation."""

    def test_third_party_endpoints_file_exists(self):
        """Test that THIRD_PARTY_ENDPOINTS.md exists."""
        endpoints_file = (
            Path(__file__).parent.parent.parent / "THIRD_PARTY_ENDPOINTS.md"
        )
        assert endpoints_file.exists(), "THIRD_PARTY_ENDPOINTS.md must exist"

    def test_endpoints_documentation_is_current(self):
        """Test that the endpoints documentation can be generated without errors."""
        # Run the generation script
        script_path = (
            Path(__file__).parent.parent.parent
            / "scripts"
            / "generate_endpoints_doc.py"
        )
        result = subprocess.run(  # noqa: S603
            [sys.executable, str(script_path)],
            capture_output=True,
            text=True,
            check=False,
        )

        assert result.returncode == 0, f"Script failed: {result.stderr}"

        # The script should report that it generated the file
        assert (
            "Generated" in result.stdout or result.stdout == ""
        ), f"Unexpected output: {result.stdout}"

    def test_all_endpoints_documented(self):
        """Test that all endpoints in the registry are documented."""
        from biomcp.utils.endpoint_registry import get_registry

        registry = get_registry()
        endpoints = registry.get_all_endpoints()

        # Read the documentation
        endpoints_file = (
            Path(__file__).parent.parent.parent / "THIRD_PARTY_ENDPOINTS.md"
        )
        content = endpoints_file.read_text()

        # Check each endpoint is mentioned
        for key, info in endpoints.items():
            assert key in content, f"Endpoint {key} not found in documentation"
            assert (
                info.url in content
            ), f"URL {info.url} not found in documentation"

    def test_documentation_contains_required_sections(self):
        """Test that documentation contains all required sections."""
        endpoints_file = (
            Path(__file__).parent.parent.parent / "THIRD_PARTY_ENDPOINTS.md"
        )
        content = endpoints_file.read_text()

        required_sections = [
            "# Third-Party Endpoints Used by BioMCP",
            "## Overview",
            "## Endpoints by Category",
            "### Biomedical Literature",
            "### Clinical Trials",
            "### Variant Databases",
            "### Cancer Genomics",
            "## Domain Summary",
            "## Compliance and Privacy",
            "## Network Control",
            "BIOMCP_OFFLINE",
        ]

        for section in required_sections:
            assert (
                section in content
            ), f"Required section '{section}' not found in documentation"

    def test_endpoint_counts_accurate(self):
        """Test that endpoint counts in the overview are accurate."""
        from biomcp.utils.endpoint_registry import get_registry

        registry = get_registry()
        endpoints = registry.get_all_endpoints()
        domains = registry.get_unique_domains()

        endpoints_file = (
            Path(__file__).parent.parent.parent / "THIRD_PARTY_ENDPOINTS.md"
        )
        content = endpoints_file.read_text()

        # Extract counts from overview
        import re

        match = re.search(
            r"BioMCP connects to (\d+) external domains across (\d+) endpoints",
            content,
        )

        assert match, "Could not find endpoint counts in overview"

        doc_domains = int(match.group(1))
        doc_endpoints = int(match.group(2))

        assert (
            doc_domains == len(domains)
        ), f"Document says {doc_domains} domains but registry has {len(domains)}"
        assert (
            doc_endpoints == len(endpoints)
        ), f"Document says {doc_endpoints} endpoints but registry has {len(endpoints)}"
