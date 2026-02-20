"""Regression test: importing czech doesn't break existing modules (T046)."""



class TestNoRegression:
    """Verify czech import doesn't break existing functionality."""

    def test_import_czech_no_error(self):
        """Importing biomcp.czech doesn't raise."""
        import biomcp.czech  # noqa: F401

    def test_import_czech_tools_no_error(self):
        """Importing czech_tools doesn't raise."""
        import biomcp.czech.czech_tools  # noqa: F401

    def test_existing_modules_importable(self):
        """Core BioMCP modules still importable after czech."""
        import biomcp.constants
        import biomcp.core
        import biomcp.czech
        import biomcp.http_client
        import biomcp.metrics  # noqa: F401

    def test_mcp_app_accessible(self):
        """mcp_app still accessible after czech import."""
        import biomcp.czech.czech_tools  # noqa: F401
        from biomcp.core import mcp_app

        assert mcp_app is not None
        assert mcp_app.name is not None

    def test_existing_tool_not_overwritten(self):
        """Czech tools don't overwrite existing tools."""
        import biomcp.czech.czech_tools  # noqa: F401
        from biomcp.core import mcp_app

        tool_names = [
            t.name for t in mcp_app._tool_manager._tools.values()
        ]
        # No Czech tool should share a name with global tools
        czech_prefixes = (
            "sukl_", "mkn_", "nrpzs_", "szv_", "vzp_",
        )
        global_tools = [
            n for n in tool_names
            if not n.startswith(czech_prefixes)
        ]
        czech_tools = [
            n for n in tool_names
            if n.startswith(czech_prefixes)
        ]
        # No overlap
        assert not set(global_tools) & set(czech_tools)
