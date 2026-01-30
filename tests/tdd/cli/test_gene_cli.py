"""Comprehensive tests for gene CLI commands."""

import json
from unittest.mock import AsyncMock, patch

import pytest
from typer.testing import CliRunner

from biomcp.cli.main import app
from biomcp.integrations import GeneInfo

runner = CliRunner()


@pytest.fixture
def mock_gene_info():
    """Create a mock GeneInfo object."""
    return GeneInfo(
        gene_id="7157",
        symbol="TP53",
        name="tumor protein p53",
        summary="This gene encodes a tumor suppressor protein containing "
        "transcriptional activation, DNA binding, and oligomerization domains.",
        alias=["p53", "LFS1", "TRP53"],
        type_of_gene="protein-coding",
        entrezgene=7157,
    )


@pytest.fixture
def mock_gene_info_brca1():
    """Create a mock GeneInfo object for BRCA1."""
    return GeneInfo(
        gene_id="672",
        symbol="BRCA1",
        name="BRCA1 DNA repair associated",
        summary="This gene encodes a nuclear phosphoprotein that plays a role "
        "in maintaining genomic stability.",
        alias=["BRCC1", "FANCS", "PNCA4"],
        type_of_gene="protein-coding",
        entrezgene=672,
    )


class TestGeneGetCommand:
    """Test the 'gene get' command."""

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_by_symbol(self, mock_get_gene):
        """Test getting gene info by symbol."""
        mock_get_gene.return_value = "# TP53\n\nTumor protein p53"

        result = runner.invoke(app, ["gene", "get", "TP53"])

        assert result.exit_code == 0
        assert "TP53" in result.stdout
        mock_get_gene.assert_called_once()

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_by_id(self, mock_get_gene):
        """Test getting gene info by Entrez ID."""
        mock_get_gene.return_value = "# TP53\n\nGene ID: 7157"

        result = runner.invoke(app, ["gene", "get", "7157"])

        assert result.exit_code == 0
        assert "7157" in result.stdout
        mock_get_gene.assert_called_once()

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_not_found(self, mock_get_gene):
        """Test handling of non-existent gene."""
        error_msg = json.dumps(
            {
                "error": "Gene 'INVALID_GENE' not found",
                "suggestion": "Please check the gene symbol or ID",
            },
            indent=2,
        )
        mock_get_gene.return_value = error_msg

        result = runner.invoke(app, ["gene", "get", "INVALID_GENE"])

        assert result.exit_code == 0
        assert "not found" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_json_flag(self, mock_get_gene):
        """Test getting gene info with --json flag."""
        json_output = json.dumps(
            {
                "gene_id": "7157",
                "symbol": "TP53",
                "name": "tumor protein p53",
                "type_of_gene": "protein-coding",
            },
            indent=2,
        )
        mock_get_gene.return_value = json_output

        result = runner.invoke(app, ["gene", "get", "TP53", "--json"])

        assert result.exit_code == 0
        assert "7157" in result.stdout
        assert "tumor protein p53" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_json_short_flag(self, mock_get_gene):
        """Test getting gene info with -j short flag."""
        json_output = json.dumps({"symbol": "TP53"}, indent=2)
        mock_get_gene.return_value = json_output

        result = runner.invoke(app, ["gene", "get", "TP53", "-j"])

        assert result.exit_code == 0

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_pathway(self, mock_get_gene):
        """Test getting gene info with --enrich pathway flag."""
        mock_get_gene.return_value = "# TP53\n\nTumor protein p53"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "pathway"]
        )

        assert result.exit_code == 0
        assert "TP53" in result.stdout
        assert "Enrichment Analysis" in result.stdout
        assert "pathway" in result.stdout.lower()

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_ontology(self, mock_get_gene):
        """Test getting gene info with --enrich ontology flag."""
        mock_get_gene.return_value = "# BRCA1\n\nBRCA1 DNA repair associated"

        result = runner.invoke(
            app, ["gene", "get", "BRCA1", "--enrich", "ontology"]
        )

        assert result.exit_code == 0
        assert "BRCA1" in result.stdout
        assert "Enrichment Analysis" in result.stdout
        assert "ontology" in result.stdout.lower()

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_kegg(self, mock_get_gene):
        """Test getting gene info with --enrich kegg flag."""
        mock_get_gene.return_value = (
            "# EGFR\n\nEpidermal growth factor receptor"
        )

        result = runner.invoke(
            app, ["gene", "get", "EGFR", "--enrich", "kegg"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_reactome(self, mock_get_gene):
        """Test getting gene info with --enrich reactome flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "reactome"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_wikipathways(self, mock_get_gene):
        """Test getting gene info with --enrich wikipathways flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "wikipathways"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_go_process(self, mock_get_gene):
        """Test getting gene info with --enrich go_process flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "go_process"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_go_molecular(self, mock_get_gene):
        """Test getting gene info with --enrich go_molecular flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "go_molecular"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_go_cellular(self, mock_get_gene):
        """Test getting gene info with --enrich go_cellular flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "go_cellular"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_celltypes(self, mock_get_gene):
        """Test getting gene info with --enrich celltypes flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "celltypes"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_tissues(self, mock_get_gene):
        """Test getting gene info with --enrich tissues flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "tissues"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_diseases(self, mock_get_gene):
        """Test getting gene info with --enrich diseases flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "diseases"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_gwas(self, mock_get_gene):
        """Test getting gene info with --enrich gwas flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "gwas"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_transcription_factors(self, mock_get_gene):
        """Test getting gene info with --enrich transcription_factors flag."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "transcription_factors"]
        )

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_with_enrich_tf(self, mock_get_gene):
        """Test getting gene info with --enrich tf flag (alias for transcription_factors)."""
        mock_get_gene.return_value = "# TP53\n\nGene info"

        result = runner.invoke(app, ["gene", "get", "TP53", "--enrich", "tf"])

        assert result.exit_code == 0
        assert "Enrichment Analysis" in result.stdout

    def test_get_gene_with_invalid_enrich_type(self):
        """Test getting gene info with invalid enrichment type."""
        result = runner.invoke(
            app, ["gene", "get", "TP53", "--enrich", "invalid_type"]
        )

        assert result.exit_code == 1
        assert "Invalid enrichment type" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_get_gene_combined_json_and_enrich(self, mock_get_gene):
        """Test getting gene info with both --json and --enrich flags."""
        json_output = json.dumps({"symbol": "TP53"}, indent=2)
        mock_get_gene.return_value = json_output

        result = runner.invoke(
            app, ["gene", "get", "TP53", "--json", "--enrich", "pathway"]
        )

        assert result.exit_code == 0
        assert "TP53" in result.stdout
        assert "Enrichment Analysis" in result.stdout


class TestGeneSearchCommand:
    """Test the 'gene search' command."""

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_basic(self, mock_get_gene):
        """Test basic gene search."""
        mock_get_gene.return_value = "# TP53\n\nTumor protein p53"

        result = runner.invoke(app, ["gene", "search", "TP53"])

        assert result.exit_code == 0
        assert "TP53" in result.stdout
        mock_get_gene.assert_called_once()

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_with_page(self, mock_get_gene):
        """Test gene search with page parameter."""
        mock_get_gene.return_value = "# kinase\n\nResults"

        result = runner.invoke(
            app, ["gene", "search", "kinase", "--page", "2"]
        )

        assert result.exit_code == 0
        assert (
            "pagination" in result.stdout.lower()
            or "development" in result.stdout
        )

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_with_page_short_flag(self, mock_get_gene):
        """Test gene search with -p short flag for page."""
        mock_get_gene.return_value = "# kinase\n\nResults"

        result = runner.invoke(app, ["gene", "search", "kinase", "-p", "2"])

        assert result.exit_code == 0

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_with_page_size(self, mock_get_gene):
        """Test gene search with page_size parameter."""
        mock_get_gene.return_value = "# kinase\n\nResults"

        result = runner.invoke(
            app, ["gene", "search", "kinase", "--page-size", "20"]
        )

        assert result.exit_code == 0

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_with_pagination(self, mock_get_gene):
        """Test gene search with full pagination parameters."""
        mock_get_gene.return_value = "# kinase\n\nResults"

        result = runner.invoke(
            app,
            ["gene", "search", "kinase", "--page", "2", "--page-size", "20"],
        )

        assert result.exit_code == 0
        # Should show development note when pagination is used
        assert "development" in result.stdout.lower()

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_with_json_output(self, mock_get_gene):
        """Test gene search with JSON output."""
        json_output = json.dumps(
            {"symbol": "BRCA1", "name": "BRCA1 DNA repair associated"},
            indent=2,
        )
        mock_get_gene.return_value = json_output

        result = runner.invoke(app, ["gene", "search", "BRCA", "--json"])

        assert result.exit_code == 0
        assert "BRCA" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_with_json_short_flag(self, mock_get_gene):
        """Test gene search with -j short flag."""
        json_output = json.dumps({"symbol": "BRCA1"}, indent=2)
        mock_get_gene.return_value = json_output

        result = runner.invoke(app, ["gene", "search", "BRCA", "-j"])

        assert result.exit_code == 0

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_quoted_query(self, mock_get_gene):
        """Test gene search with quoted multi-word query."""
        mock_get_gene.return_value = "# tumor protein\n\nResults"

        result = runner.invoke(app, ["gene", "search", "tumor protein"])

        assert result.exit_code == 0
        assert "tumor protein" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_not_found(self, mock_get_gene):
        """Test gene search with no results."""
        error_msg = json.dumps(
            {
                "error": "Gene 'INVALID_QUERY' not found",
                "suggestion": "Please check the gene symbol or ID",
            },
            indent=2,
        )
        mock_get_gene.return_value = error_msg

        result = runner.invoke(app, ["gene", "search", "INVALID_QUERY"])

        assert result.exit_code == 0
        assert "not found" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_search_genes_with_all_options(self, mock_get_gene):
        """Test gene search with all available options."""
        json_output = json.dumps({"symbol": "kinase"}, indent=2)
        mock_get_gene.return_value = json_output

        result = runner.invoke(
            app,
            [
                "gene",
                "search",
                "kinase",
                "--page",
                "2",
                "--page-size",
                "20",
                "--json",
            ],
        )

        assert result.exit_code == 0


class TestGeneCliIntegration:
    """Integration-style tests for gene CLI (still using mocks)."""

    @patch("biomcp.integrations.biothings_client.http_client")
    def test_get_gene_full_flow(self, mock_http_client):
        """Test full gene get flow with mocked HTTP client."""
        # Mock the API response
        mock_http_client.request_api = AsyncMock(
            return_value=(
                {
                    "_id": "7157",
                    "symbol": "TP53",
                    "name": "tumor protein p53",
                    "summary": "This gene encodes a tumor suppressor protein...",
                    "alias": ["p53", "LFS1"],
                    "type_of_gene": "protein-coding",
                    "entrezgene": 7157,
                },
                None,
            )
        )

        result = runner.invoke(app, ["gene", "get", "7157"])

        assert result.exit_code == 0
        assert "TP53" in result.stdout
        assert "tumor protein p53" in result.stdout

    @patch("biomcp.integrations.biothings_client.http_client")
    def test_get_gene_with_aliases(self, mock_http_client):
        """Test gene with multiple aliases."""
        # Mock both the query (to find BRCA1) and the get (to retrieve details)
        mock_http_client.request_api = AsyncMock(
            side_effect=[
                # First call: query for BRCA1
                (
                    {
                        "hits": [
                            {
                                "_id": "672",
                                "symbol": "BRCA1",
                                "name": "BRCA1 DNA repair associated",
                                "taxid": 9606,
                            }
                        ]
                    },
                    None,
                ),
                # Second call: get full details
                (
                    {
                        "_id": "672",
                        "symbol": "BRCA1",
                        "name": "BRCA1 DNA repair associated",
                        "alias": [
                            "BRCC1",
                            "FANCS",
                            "PNCA4",
                            "RNF53",
                            "BRCAI",
                            "BRCA1/BRCA2-containing complex, subunit 1",
                            "breast cancer 1",
                            "breast cancer 1, early onset",
                            "breast cancer type 1 susceptibility protein",
                            "RING finger protein 53",
                            "PPP1R53",
                        ],
                        "type_of_gene": "protein-coding",
                        "entrezgene": 672,
                    },
                    None,
                ),
            ]
        )

        result = runner.invoke(app, ["gene", "get", "BRCA1"])

        assert result.exit_code == 0
        assert "BRCA1" in result.stdout
        # Should show limited aliases with "and X more" message
        assert "alias" in result.stdout.lower() or "BRCC1" in result.stdout

    @patch("biomcp.cli.genes.get_gene")
    def test_gene_command_error_handling(self, mock_get_gene):
        """Test error handling in gene commands."""
        # Simulate an error response from the API
        error_msg = json.dumps(
            {
                "error": "Failed to retrieve gene information",
                "details": "API connection error",
            },
            indent=2,
        )
        mock_get_gene.return_value = error_msg

        result = runner.invoke(app, ["gene", "get", "TP53"])

        # Should exit successfully but show error message
        assert result.exit_code == 0
        assert "error" in result.stdout.lower()

    def test_gene_help_command(self):
        """Test gene command help text."""
        result = runner.invoke(app, ["gene", "--help"])

        assert result.exit_code == 0
        assert "gene" in result.stdout.lower()
        assert "get" in result.stdout.lower()
        assert "search" in result.stdout.lower()

    def test_gene_get_help_command(self):
        """Test gene get command help text."""
        result = runner.invoke(app, ["gene", "get", "--help"])

        assert result.exit_code == 0
        assert "TP53" in result.stdout
        assert "enrich" in result.stdout.lower()
        assert "json" in result.stdout.lower()

    def test_gene_search_help_command(self):
        """Test gene search command help text."""
        result = runner.invoke(app, ["gene", "search", "--help"])

        assert result.exit_code == 0
        assert "search" in result.stdout.lower()
        assert "page" in result.stdout.lower()
        assert "json" in result.stdout.lower()
