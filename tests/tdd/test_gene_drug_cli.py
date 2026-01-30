"""Unit tests for gene and drug CLI commands."""

import re
from unittest.mock import patch

from typer.testing import CliRunner

from biomcp.cli.main import app

runner = CliRunner()


def strip_ansi(text: str) -> str:
    """Remove ANSI escape codes from text."""
    ansi_escape = re.compile(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])")
    return ansi_escape.sub("", text)


class TestGeneCLI:
    """Test gene CLI commands."""

    def test_gene_help(self):
        """Test gene command shows help."""
        result = runner.invoke(app, ["gene", "--help"])
        assert result.exit_code == 0
        assert "gene" in result.stdout.lower()
        assert "get" in result.stdout.lower()

    def test_gene_get_help(self):
        """Test gene get command shows help."""
        result = runner.invoke(app, ["gene", "get", "--help"])
        assert result.exit_code == 0
        output = strip_ansi(result.stdout)
        assert "gene" in output.lower()
        assert "--json" in output

    @patch("biomcp.cli.genes.get_gene")
    def test_gene_get_success(self, mock_get_gene):
        """Test successful gene retrieval."""
        mock_get_gene.return_value = "## Gene: TP53\n- **Symbol**: TP53"

        result = runner.invoke(app, ["gene", "get", "TP53"])

        assert result.exit_code == 0
        assert "TP53" in result.stdout
        mock_get_gene.assert_called_once()

    @patch("biomcp.cli.genes.get_gene")
    def test_gene_get_json_output(self, mock_get_gene):
        """Test gene retrieval with JSON output."""
        mock_get_gene.return_value = (
            '{"symbol": "TP53", "name": "tumor protein p53"}'
        )

        result = runner.invoke(app, ["gene", "get", "TP53", "--json"])

        assert result.exit_code == 0
        mock_get_gene.assert_called_once()
        call_args = mock_get_gene.call_args
        assert call_args[0][0] == "TP53"
        assert call_args[1]["output_json"] is True

    @patch("biomcp.cli.genes.get_gene")
    def test_gene_get_not_found(self, mock_get_gene):
        """Test gene not found response."""
        mock_get_gene.return_value = '{"error": "Gene \'INVALID\' not found"}'

        result = runner.invoke(app, ["gene", "get", "INVALID"])

        assert result.exit_code == 0
        assert (
            "error" in result.stdout.lower()
            or "not found" in result.stdout.lower()
        )

    def test_gene_no_args_shows_help(self):
        """Test that running gene without subcommand shows help."""
        result = runner.invoke(app, ["gene"])
        assert result.exit_code == 0
        assert "get" in result.stdout.lower()


class TestDrugCLI:
    """Test drug CLI commands."""

    def test_drug_help(self):
        """Test drug command shows help."""
        result = runner.invoke(app, ["drug", "--help"])
        assert result.exit_code == 0
        assert "drug" in result.stdout.lower()
        assert "get" in result.stdout.lower()

    def test_drug_get_help(self):
        """Test drug get command shows help."""
        result = runner.invoke(app, ["drug", "get", "--help"])
        assert result.exit_code == 0
        output = strip_ansi(result.stdout)
        assert "drug" in output.lower()
        assert "--json" in output

    @patch("biomcp.cli.drugs.get_drug")
    def test_drug_get_success(self, mock_get_drug):
        """Test successful drug retrieval."""
        mock_get_drug.return_value = (
            "## Drug: Imatinib\n- **DrugBank ID**: DB00619"
        )

        result = runner.invoke(app, ["drug", "get", "imatinib"])

        assert result.exit_code == 0
        assert (
            "Imatinib" in result.stdout or "imatinib" in result.stdout.lower()
        )
        mock_get_drug.assert_called_once()

    @patch("biomcp.cli.drugs.get_drug")
    def test_drug_get_json_output(self, mock_get_drug):
        """Test drug retrieval with JSON output."""
        mock_get_drug.return_value = (
            '{"name": "imatinib", "drugbank_id": "DB00619"}'
        )

        result = runner.invoke(app, ["drug", "get", "imatinib", "--json"])

        assert result.exit_code == 0
        mock_get_drug.assert_called_once()
        call_args = mock_get_drug.call_args
        assert call_args[0][0] == "imatinib"
        assert call_args[1]["output_json"] is True

    @patch("biomcp.cli.drugs.get_drug")
    def test_drug_get_by_drugbank_id(self, mock_get_drug):
        """Test drug retrieval by DrugBank ID."""
        mock_get_drug.return_value = (
            "## Drug: Imatinib\n- **DrugBank ID**: DB00619"
        )

        result = runner.invoke(app, ["drug", "get", "DB00619"])

        assert result.exit_code == 0
        mock_get_drug.assert_called_once_with("DB00619", output_json=False)

    @patch("biomcp.cli.drugs.get_drug")
    def test_drug_get_not_found(self, mock_get_drug):
        """Test drug not found response."""
        mock_get_drug.return_value = "Drug 'INVALID' not found in MyChem.info"

        result = runner.invoke(app, ["drug", "get", "INVALID"])

        assert result.exit_code == 0
        assert "not found" in result.stdout.lower()

    def test_drug_no_args_shows_help(self):
        """Test that running drug without subcommand shows help."""
        result = runner.invoke(app, ["drug"])
        assert result.exit_code == 0
        assert "get" in result.stdout.lower()


class TestCLIRegistration:
    """Test that gene and drug commands are registered in main CLI."""

    def test_main_help_shows_gene(self):
        """Test that main help includes gene command."""
        result = runner.invoke(app, ["--help"])
        assert result.exit_code == 0
        assert "gene" in result.stdout.lower()

    def test_main_help_shows_drug(self):
        """Test that main help includes drug command."""
        result = runner.invoke(app, ["--help"])
        assert result.exit_code == 0
        assert "drug" in result.stdout.lower()
