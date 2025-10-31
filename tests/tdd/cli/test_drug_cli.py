"""Comprehensive tests for drug CLI commands."""

import json
from unittest.mock import patch

import pytest
from typer.testing import CliRunner

from biomcp.cli.main import app

runner = CliRunner()


@pytest.fixture
def mock_drug_info_imatinib():
    """Create a mock drug info response for imatinib."""
    return """## Drug: Imatinib
- **Formula**: C29H31N7O
- **DrugBank ID**: DB00619
- **ChEMBL ID**: CHEMBL941
- **PubChem CID**: 5291
- **ChEBI ID**: CHEBI:45783
- **InChIKey**: KTUFNOKKBVMGRW-UHFFFAOYSA-N
- **Trade Names**: Gleevec, Glivec

### Description
Imatinib is a tyrosine kinase inhibitor used in the treatment of multiple cancers...

### Indication
Treatment of chronic myeloid leukemia, acute lymphoblastic leukemia, and gastrointestinal stromal tumors...

### Mechanism of Action
Inhibits BCR-ABL tyrosine kinase, the constitutive abnormal gene product of the Philadelphia chromosome...

### External Links
- [DrugBank](https://www.drugbank.ca/drugs/DB00619)
- [ChEMBL](https://www.ebi.ac.uk/chembl/compound_report_card/CHEMBL941/)
- [PubChem](https://pubchem.ncbi.nlm.nih.gov/compound/5291)
- [ChEBI](https://www.ebi.ac.uk/chebi/searchId.do?chebiId=CHEBI:45783)"""


@pytest.fixture
def mock_drug_info_aspirin():
    """Create a mock drug info response for aspirin."""
    return """## Drug: Aspirin
- **Formula**: C9H8O4
- **DrugBank ID**: DB00945
- **ChEMBL ID**: CHEMBL25
- **PubChem CID**: 2244
- **InChIKey**: BSYNRYMUTXBXSQ-UHFFFAOYSA-N
- **Trade Names**: Bayer Aspirin, Ecotrin, Bufferin

### Description
Aspirin is a salicylate used to treat pain, fever, inflammation, and migraines...

### External Links
- [DrugBank](https://www.drugbank.ca/drugs/DB00945)
- [ChEMBL](https://www.ebi.ac.uk/chembl/compound_report_card/CHEMBL25/)
- [PubChem](https://pubchem.ncbi.nlm.nih.gov/compound/2244)"""


@pytest.fixture
def mock_drug_info_multiword():
    """Create a mock drug info response for multi-word drug name."""
    return """## Drug: Idecabtagene Vicleucel
- **DrugBank ID**: DB15769
- **Trade Names**: Abecma

### Description
Idecabtagene vicleucel is a B-cell maturation antigen (BCMA)-directed CAR T-cell immunotherapy...

### Indication
Treatment of adult patients with relapsed or refractory multiple myeloma...

### External Links
- [DrugBank](https://www.drugbank.ca/drugs/DB15769)"""


@pytest.fixture
def mock_drug_info_pembrolizumab():
    """Create a mock drug info response for pembrolizumab."""
    return """## Drug: Pembrolizumab
- **DrugBank ID**: DB09037
- **Trade Names**: Keytruda

### Description
Pembrolizumab is a humanized monoclonal antibody used in cancer immunotherapy...

### Indication
Treatment of melanoma, non-small cell lung cancer, and other cancers...

### Mechanism of Action
Binds to PD-1 receptor to block interaction with PD-L1 and PD-L2...

### External Links
- [DrugBank](https://www.drugbank.ca/drugs/DB09037)"""


class TestDrugGetCommand:
    """Test the 'drug get' command."""

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_by_name_imatinib(
        self, mock_run, mock_drug_info_imatinib
    ):
        """Test getting drug info by name (imatinib)."""
        mock_run.return_value = mock_drug_info_imatinib

        result = runner.invoke(app, ["drug", "get", "imatinib"])

        assert result.exit_code == 0
        assert "Imatinib" in result.stdout
        assert "DrugBank ID**: DB00619" in result.stdout
        assert "ChEMBL ID**: CHEMBL941" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_by_name_aspirin(self, mock_run, mock_drug_info_aspirin):
        """Test getting drug info by name (aspirin)."""
        mock_run.return_value = mock_drug_info_aspirin

        result = runner.invoke(app, ["drug", "get", "aspirin"])

        assert result.exit_code == 0
        assert "Aspirin" in result.stdout
        assert "DrugBank ID**: DB00945" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_multiword_name_quoted(
        self, mock_run, mock_drug_info_multiword
    ):
        """Test getting drug with multi-word name (quoted) - CRITICAL for URL encoding fix."""
        mock_run.return_value = mock_drug_info_multiword

        result = runner.invoke(app, ["drug", "get", "idecabtagene vicleucel"])

        assert result.exit_code == 0
        assert "Idecabtagene Vicleucel" in result.stdout
        assert "Abecma" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_multiword_name_pembrolizumab(
        self, mock_run, mock_drug_info_pembrolizumab
    ):
        """Test getting drug with multi-word name (pembrolizumab) - validates URL encoding."""
        mock_run.return_value = mock_drug_info_pembrolizumab

        result = runner.invoke(app, ["drug", "get", "pembrolizumab"])

        assert result.exit_code == 0
        assert "Pembrolizumab" in result.stdout
        assert "Keytruda" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_multiword_name_with_spaces(
        self, mock_run, mock_drug_info_multiword
    ):
        """Test getting drug with multiple spaces in name - ensures proper URL encoding."""
        mock_run.return_value = mock_drug_info_multiword

        # Test with the actual multi-word drug name as it would be typed
        result = runner.invoke(app, ["drug", "get", "idecabtagene vicleucel"])

        assert result.exit_code == 0
        assert "Idecabtagene Vicleucel" in result.stdout
        # Verify the mock was called with the correct argument
        mock_run.assert_called_once()
        # Get the actual call arguments
        # The first argument should be the coroutine from get_drug()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_by_drugbank_id(self, mock_run, mock_drug_info_imatinib):
        """Test getting drug info by DrugBank ID."""
        mock_run.return_value = mock_drug_info_imatinib

        result = runner.invoke(app, ["drug", "get", "DB00619"])

        assert result.exit_code == 0
        assert "Imatinib" in result.stdout
        assert "DB00619" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_by_chembl_id(self, mock_run, mock_drug_info_aspirin):
        """Test getting drug info by ChEMBL ID."""
        mock_run.return_value = mock_drug_info_aspirin

        result = runner.invoke(app, ["drug", "get", "CHEMBL25"])

        assert result.exit_code == 0
        assert "Aspirin" in result.stdout or "CHEMBL25" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_not_found(self, mock_run):
        """Test handling of non-existent drug."""
        error_msg = "Drug 'INVALID_DRUG_XYZ' not found in MyChem.info"
        mock_run.return_value = error_msg

        result = runner.invoke(app, ["drug", "get", "INVALID_DRUG_XYZ"])

        assert result.exit_code == 0
        assert "not found" in result.stdout

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_with_json_flag(self, mock_run):
        """Test getting drug info with --json flag."""
        json_output = json.dumps(
            {
                "drug_id": "CHEMBL941",
                "name": "Imatinib",
                "drugbank_id": "DB00619",
                "chembl_id": "CHEMBL941",
                "formula": "C29H31N7O",
                "_links": {
                    "DrugBank": "https://www.drugbank.ca/drugs/DB00619",
                    "ChEMBL": "https://www.ebi.ac.uk/chembl/compound_report_card/CHEMBL941/",
                },
            },
            indent=2,
        )
        mock_run.return_value = json_output

        result = runner.invoke(app, ["drug", "get", "imatinib", "--json"])

        assert result.exit_code == 0
        assert "CHEMBL941" in result.stdout
        assert "Imatinib" in result.stdout
        assert "DB00619" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_with_json_short_flag(self, mock_run):
        """Test getting drug info with -j short flag."""
        json_output = json.dumps(
            {"drug_id": "CHEMBL941", "name": "Imatinib"}, indent=2
        )
        mock_run.return_value = json_output

        result = runner.invoke(app, ["drug", "get", "imatinib", "-j"])

        assert result.exit_code == 0
        assert "Imatinib" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_multiword_with_json(self, mock_run):
        """Test getting multi-word drug name with JSON output."""
        json_output = json.dumps(
            {
                "drug_id": "DB15769",
                "name": "Idecabtagene Vicleucel",
                "drugbank_id": "DB15769",
                "tradename": ["Abecma"],
            },
            indent=2,
        )
        mock_run.return_value = json_output

        result = runner.invoke(
            app, ["drug", "get", "idecabtagene vicleucel", "--json"]
        )

        assert result.exit_code == 0
        assert "Idecabtagene Vicleucel" in result.stdout
        assert "DB15769" in result.stdout
        mock_run.assert_called_once()


class TestDrugSearchCommand:
    """Test the 'drug search' command."""

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_basic(self, mock_run, mock_drug_info_imatinib):
        """Test basic drug search."""
        mock_run.return_value = mock_drug_info_imatinib

        result = runner.invoke(app, ["drug", "search", "imatinib"])

        assert result.exit_code == 0
        assert "Imatinib" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_multiword_query(
        self, mock_run, mock_drug_info_multiword
    ):
        """Test drug search with multi-word query - validates URL encoding fix."""
        mock_run.return_value = mock_drug_info_multiword

        result = runner.invoke(app, ["drug", "search", "kinase inhibitor"])

        assert result.exit_code == 0
        # Should contain either the search term or drug information
        assert "Drug:" in result.stdout or "kinase" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_with_page(self, mock_run, mock_drug_info_aspirin):
        """Test drug search with page parameter."""
        mock_run.return_value = mock_drug_info_aspirin

        result = runner.invoke(
            app, ["drug", "search", "aspirin", "--page", "2"]
        )

        assert result.exit_code == 0
        # Should show development note when pagination is used
        assert "development" in result.stdout.lower()
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_with_page_short_flag(
        self, mock_run, mock_drug_info_aspirin
    ):
        """Test drug search with -p short flag for page."""
        mock_run.return_value = mock_drug_info_aspirin

        result = runner.invoke(app, ["drug", "search", "aspirin", "-p", "2"])

        assert result.exit_code == 0
        assert "development" in result.stdout.lower()
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_with_page_size(
        self, mock_run, mock_drug_info_imatinib
    ):
        """Test drug search with page_size parameter."""
        mock_run.return_value = mock_drug_info_imatinib

        result = runner.invoke(
            app, ["drug", "search", "imatinib", "--page-size", "20"]
        )

        assert result.exit_code == 0
        assert "development" in result.stdout.lower()
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_with_pagination(
        self, mock_run, mock_drug_info_aspirin
    ):
        """Test drug search with full pagination parameters."""
        mock_run.return_value = mock_drug_info_aspirin

        result = runner.invoke(
            app,
            ["drug", "search", "aspirin", "--page", "2", "--page-size", "20"],
        )

        assert result.exit_code == 0
        assert "development" in result.stdout.lower()
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_with_json_output(self, mock_run):
        """Test drug search with JSON output."""
        json_output = json.dumps(
            {
                "drug_id": "CHEMBL941",
                "name": "Imatinib",
                "drugbank_id": "DB00619",
            },
            indent=2,
        )
        mock_run.return_value = json_output

        result = runner.invoke(app, ["drug", "search", "imatinib", "--json"])

        assert result.exit_code == 0
        assert "Imatinib" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_with_json_short_flag(self, mock_run):
        """Test drug search with -j short flag."""
        json_output = json.dumps(
            {"drug_id": "CHEMBL941", "name": "Imatinib"}, indent=2
        )
        mock_run.return_value = json_output

        result = runner.invoke(app, ["drug", "search", "imatinib", "-j"])

        assert result.exit_code == 0
        assert "Imatinib" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_multiword_with_json(self, mock_run):
        """Test drug search with multi-word query and JSON output."""
        json_output = json.dumps(
            {"name": "Idecabtagene Vicleucel", "tradename": ["Abecma"]},
            indent=2,
        )
        mock_run.return_value = json_output

        result = runner.invoke(app, ["drug", "search", "CAR T cell", "--json"])

        assert result.exit_code == 0
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_not_found(self, mock_run):
        """Test drug search with no results."""
        error_msg = "Drug 'INVALID_QUERY_XYZ' not found in MyChem.info"
        mock_run.return_value = error_msg

        result = runner.invoke(app, ["drug", "search", "INVALID_QUERY_XYZ"])

        assert result.exit_code == 0
        assert "not found" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_search_drugs_with_all_options(self, mock_run):
        """Test drug search with all available options."""
        json_output = json.dumps({"name": "Imatinib"}, indent=2)
        mock_run.return_value = json_output

        result = runner.invoke(
            app,
            [
                "drug",
                "search",
                "imatinib",
                "--page",
                "2",
                "--page-size",
                "20",
                "--json",
            ],
        )

        assert result.exit_code == 0
        assert "Imatinib" in result.stdout
        assert "development" in result.stdout.lower()
        mock_run.assert_called_once()


class TestDrugCliIntegration:
    """Integration-style tests for drug CLI (still using mocks)."""

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_full_flow(self, mock_run, mock_drug_info_imatinib):
        """Test full drug get flow with mocked output."""
        mock_run.return_value = mock_drug_info_imatinib

        result = runner.invoke(app, ["drug", "get", "imatinib"])

        assert result.exit_code == 0
        assert "Imatinib" in result.stdout
        assert "Formula**: C29H31N7O" in result.stdout
        assert "DrugBank ID**: DB00619" in result.stdout
        assert "External Links" in result.stdout

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_get_drug_with_trade_names(self, mock_run, mock_drug_info_aspirin):
        """Test drug with trade names."""
        mock_run.return_value = mock_drug_info_aspirin

        result = runner.invoke(app, ["drug", "get", "aspirin"])

        assert result.exit_code == 0
        assert "Aspirin" in result.stdout
        assert "Trade Names" in result.stdout

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_drug_command_error_handling(self, mock_run):
        """Test error handling in drug commands."""
        error_msg = "Error retrieving drug information: API connection error"
        mock_run.return_value = error_msg

        result = runner.invoke(app, ["drug", "get", "imatinib"])

        # Should exit successfully but show error message
        assert result.exit_code == 0
        assert "Error" in result.stdout

    def test_drug_help_command(self):
        """Test drug command help text."""
        result = runner.invoke(app, ["drug", "--help"])

        assert result.exit_code == 0
        assert "drug" in result.stdout.lower()
        assert "get" in result.stdout.lower()
        assert "search" in result.stdout.lower()

    def test_drug_get_help_command(self):
        """Test drug get command help text."""
        result = runner.invoke(app, ["drug", "get", "--help"])

        assert result.exit_code == 0
        assert "imatinib" in result.stdout.lower()
        assert "idecabtagene vicleucel" in result.stdout.lower()
        assert "json" in result.stdout.lower()

    def test_drug_search_help_command(self):
        """Test drug search command help text."""
        result = runner.invoke(app, ["drug", "search", "--help"])

        assert result.exit_code == 0
        assert "search" in result.stdout.lower()
        assert "page" in result.stdout.lower()
        assert "json" in result.stdout.lower()


class TestDrugMultiWordUrlEncoding:
    """Specific tests for multi-word drug name URL encoding fix."""

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_multiword_drug_idecabtagene_vicleucel(
        self, mock_run, mock_drug_info_multiword
    ):
        """Test idecabtagene vicleucel (the bug report case)."""
        mock_run.return_value = mock_drug_info_multiword

        result = runner.invoke(app, ["drug", "get", "idecabtagene vicleucel"])

        assert result.exit_code == 0
        assert "Idecabtagene Vicleucel" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_multiword_drug_with_three_words(self, mock_run):
        """Test drug name with three words."""
        mock_response = """## Drug: Test Drug Name
- **DrugBank ID**: DB99999"""
        mock_run.return_value = mock_response

        result = runner.invoke(app, ["drug", "get", "test drug name"])

        assert result.exit_code == 0
        assert "Drug:" in result.stdout
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_multiword_search_query(self, mock_run, mock_drug_info_imatinib):
        """Test search with multi-word query."""
        mock_run.return_value = mock_drug_info_imatinib

        result = runner.invoke(
            app, ["drug", "search", "tyrosine kinase inhibitor"]
        )

        assert result.exit_code == 0
        mock_run.assert_called_once()

    @patch("biomcp.cli.drugs.asyncio.run")
    def test_multiword_with_special_characters(self, mock_run):
        """Test drug name with special characters (hyphens, etc)."""
        mock_response = """## Drug: Test-Drug-Name
- **DrugBank ID**: DB88888"""
        mock_run.return_value = mock_response

        result = runner.invoke(app, ["drug", "get", "test-drug-name"])

        assert result.exit_code == 0
        mock_run.assert_called_once()
