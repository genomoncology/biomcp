"""Integration tests for MCP server functionality."""

import json
from unittest.mock import patch

import pytest

from biomcp.core import mcp_app


@pytest.mark.asyncio
class TestMCPIntegration:
    """Integration tests for the MCP server."""

    async def test_mcp_server_tools_registered(self):
        """Test that MCP tools are properly registered."""
        # Get the registered tools
        tools = await mcp_app.list_tools()

        # Should have exactly 2 tools
        assert len(tools) == 2

        # Check tool names
        tool_names = [tool.name for tool in tools]
        assert "search" in tool_names
        assert "fetch" in tool_names

    async def test_mcp_search_tool_schema(self):
        """Test the search tool schema."""
        tools = await mcp_app.list_tools()
        search_tool = next(t for t in tools if t.name == "search")

        # Check required parameters
        assert "call_benefit" in search_tool.inputSchema["properties"]
        assert "domain" in search_tool.inputSchema["properties"]
        assert "query" in search_tool.inputSchema["properties"]

        # Check domain enum values
        domain_schema = search_tool.inputSchema["properties"]["domain"]
        # The enum is nested in anyOf
        enum_values = domain_schema["anyOf"][0]["enum"]
        assert "article" in enum_values
        assert "trial" in enum_values
        assert "variant" in enum_values
        assert "thinking" in enum_values

    async def test_mcp_fetch_tool_schema(self):
        """Test the fetch tool schema."""
        tools = await mcp_app.list_tools()
        fetch_tool = next(t for t in tools if t.name == "fetch")

        # Check required parameters
        required = fetch_tool.inputSchema["required"]
        assert "call_benefit" in required
        assert "domain" in required
        assert "id_" in required

        # Check domain enum values (no thinking for fetch)
        domain_schema = fetch_tool.inputSchema["properties"]["domain"]
        # For required enums, the structure is different
        if "enum" in domain_schema:
            enum_values = domain_schema["enum"]
        else:
            # Check if it's in anyOf structure
            enum_values = domain_schema.get("anyOf", [{}])[0].get("enum", [])
        assert "article" in enum_values
        assert "trial" in enum_values
        assert "variant" in enum_values
        assert "thinking" not in enum_values

    async def test_mcp_search_article_integration(self):
        """Test end-to-end article search through MCP."""
        mock_result = json.dumps([
            {
                "pmid": "12345",
                "title": "Test Article",
                "abstract": "Test abstract",
            }
        ])

        with patch("biomcp.articles.search.search_articles") as mock_search:
            mock_search.return_value = mock_result

            # Import search function directly since we can't test through MCP without Context
            from biomcp.router import search

            # Call the search function
            result = await search(
                call_benefit="Testing MCP integration",
                domain="article",
                genes="BRAF",
                page_size=10,
            )

            # Verify the result structure
            assert "results" in result
            assert len(result["results"]) == 1
            assert result["results"][0]["id"] == "12345"

    async def test_mcp_fetch_variant_integration(self):
        """Test end-to-end variant fetch through MCP."""
        mock_result = json.dumps([
            {
                "_id": "rs121913529",
                "gene": {"symbol": "BRAF"},
                "clinvar": {"clinical_significance": "Pathogenic"},
            }
        ])

        with patch("biomcp.variants.getter.get_variant") as mock_get:
            mock_get.return_value = mock_result

            from biomcp.router import fetch

            # Call the fetch function
            result = await fetch(
                call_benefit="Testing variant fetch",
                domain="variant",
                id_="rs121913529",
            )

            # Verify the result structure
            assert result["id"] == "rs121913529"
            assert "title" in result
            assert "text" in result
            assert "url" in result
            assert "metadata" in result

    async def test_mcp_unified_query_integration(self):
        """Test unified query through MCP."""
        with patch("biomcp.query_router.execute_routing_plan") as mock_execute:
            mock_execute.return_value = {
                "articles": json.dumps([
                    {"pmid": "111", "title": "Article 1"}
                ]),
                "variants": json.dumps([
                    {"_id": "rs222", "gene": {"symbol": "TP53"}}
                ]),
            }

            from biomcp.router import search

            # Call search with unified query
            result = await search(
                call_benefit="Testing unified search",
                query="gene:BRAF AND disease:cancer",
                max_results_per_domain=10,
            )

            # Should get results from multiple domains
            assert "results" in result
            assert len(result["results"]) >= 2

    async def test_mcp_thinking_integration(self):
        """Test sequential thinking through MCP."""
        with patch(
            "biomcp.thinking.sequential._sequential_thinking"
        ) as mock_think:
            mock_think.return_value = {
                "thought": "Processed thought",
                "analysis": "Test analysis",
            }

            from biomcp.router import search

            # Call search with thinking domain
            result = await search(
                call_benefit="Testing thinking",
                domain="thinking",
                thought="Test thought",
                thoughtNumber=1,
                totalThoughts=3,
                nextThoughtNeeded=True,
            )

            # Verify thinking result
            assert result["domain"] == "thinking"
            assert result["thoughtNumber"] == 1
            assert result["nextThoughtNeeded"] is True

    async def test_mcp_error_handling(self):
        """Test MCP error handling."""
        from biomcp.exceptions import InvalidDomainError
        from biomcp.router import search

        # Test with invalid domain
        with pytest.raises(InvalidDomainError) as exc_info:
            await search(
                call_benefit="Testing error",
                domain="invalid_domain",
            )

        assert "Unknown domain" in str(exc_info.value)

    async def test_mcp_fetch_all_trial_sections(self):
        """Test fetching trial with all sections through MCP."""
        mock_protocol = {"title": "Test Trial", "nct_id": "NCT123"}
        mock_locations = {"locations": [{"city": "Boston"}]}

        with (
            patch("biomcp.trials.getter._trial_protocol") as mock_p,
            patch("biomcp.trials.getter._trial_locations") as mock_l,
            patch("biomcp.trials.getter._trial_outcomes") as mock_o,
            patch("biomcp.trials.getter._trial_references") as mock_r,
        ):
            mock_p.return_value = json.dumps(mock_protocol)
            mock_l.return_value = json.dumps(mock_locations)
            mock_o.return_value = json.dumps({"outcomes": {}})
            mock_r.return_value = json.dumps({"references": []})

            from biomcp.router import fetch

            result = await fetch(
                call_benefit="Testing trial fetch",
                domain="trial",
                id_="NCT123",
                detail="all",
            )

            # Verify all sections are included
            assert result["id"] == "NCT123"
            assert "locations" in result["metadata"]
            assert "outcomes" in result["metadata"]
            assert "references" in result["metadata"]

    async def test_mcp_parameter_parsing(self):
        """Test parameter parsing through MCP."""
        mock_result = json.dumps([])

        with patch("biomcp.articles.search.search_articles") as mock_search:
            mock_search.return_value = mock_result

            from biomcp.router import search

            # Test with various parameter formats
            await search(
                call_benefit="Testing parameters",
                domain="article",
                genes='["BRAF", "KRAS"]',  # JSON string
                diseases="cancer,melanoma",  # Comma-separated
                keywords=["test1", "test2"],  # Already a list
            )

            # Verify parameters were parsed correctly
            call_args = mock_search.call_args[0][0]
            assert call_args.genes == ["BRAF", "KRAS"]
            assert call_args.diseases == ["cancer", "melanoma"]
            assert call_args.keywords == ["test1", "test2"]
