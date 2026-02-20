"""Test that all Czech MCP tools register correctly (T045)."""

from typing import ClassVar


class TestToolRegistration:
    """Verify all 14 Czech tools are registered."""

    EXPECTED_CZECH_TOOLS: ClassVar[list[str]] = [
        "sukl_drug_searcher",
        "sukl_drug_getter",
        "sukl_spc_getter",
        "sukl_pil_getter",
        "sukl_availability_checker",
        "mkn_diagnosis_searcher",
        "mkn_diagnosis_getter",
        "mkn_category_browser",
        "nrpzs_provider_searcher",
        "nrpzs_provider_getter",
        "szv_procedure_searcher",
        "szv_procedure_getter",
        "vzp_codebook_searcher",
        "vzp_codebook_getter",
    ]

    def test_all_czech_tools_registered(self):
        """All 14 Czech tools must be registered."""
        # Force import of czech_tools to register
        import biomcp.czech.czech_tools  # noqa: F401
        from biomcp.core import mcp_app

        tool_names = [
            t.name for t in mcp_app._tool_manager._tools.values()
        ]

        for name in self.EXPECTED_CZECH_TOOLS:
            assert name in tool_names, (
                f"Tool '{name}' not registered"
            )

    def test_czech_tool_count(self):
        """Exactly 14 Czech tools registered."""
        import biomcp.czech.czech_tools  # noqa: F401
        from biomcp.core import mcp_app

        tool_names = [
            t.name for t in mcp_app._tool_manager._tools.values()
        ]
        czech_tools = [
            n
            for n in tool_names
            if n.startswith(
                ("sukl_", "mkn_", "nrpzs_", "szv_", "vzp_")
            )
        ]
        assert len(czech_tools) == 14

    def test_global_tools_still_present(self):
        """Global BioMCP tools coexist with Czech tools."""
        import biomcp.czech.czech_tools  # noqa: F401
        from biomcp.core import mcp_app

        tool_names = [
            t.name for t in mcp_app._tool_manager._tools.values()
        ]
        # At least some global tools should be present
        assert len(tool_names) > 14
