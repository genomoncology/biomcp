"""Tests for domain handlers module."""

import pytest

from biomcp.constants import DEFAULT_TITLE
from biomcp.domain_handlers import (
    ArticleHandler,
    TrialHandler,
    VariantHandler,
    get_domain_handler,
)


class TestArticleHandler:
    """Test ArticleHandler class."""

    def test_format_pubmed_article(self):
        """Test formatting a PubMed article."""
        article = {
            "pmid": "12345",
            "title": "Test Article Title",
            "abstract": "This is a test abstract that is longer than 200 characters. "
            * 5,
            "pub_year": "2023",
            "journal": "Test Journal",
            "authors": ["Smith J", "Doe J", "Johnson A", "Williams B"],
        }

        result = ArticleHandler.format_result(article)

        assert result["id"] == "12345"
        assert result["title"] == "Test Article Title"
        assert len(result["snippet"]) == 203  # 200 + "..."
        assert result["snippet"].endswith("...")
        assert result["url"] == "https://pubmed.ncbi.nlm.nih.gov/12345/"
        assert result["metadata"]["year"] == "2023"
        assert result["metadata"]["journal"] == "Test Journal"
        assert len(result["metadata"]["authors"]) == 3  # Only first 3

    def test_format_preprint_article(self):
        """Test formatting a preprint article."""
        preprint = {
            "doi": "10.1101/2023.01.01.12345",
            "id": "biorxiv-123",
            "title": "Preprint Title",
            "abstract": "Short abstract",
            "url": "https://www.biorxiv.org/content/10.1101/2023.01.01.12345",
            "pub_year": "2023",
            "source": "bioRxiv",
            "authors": ["Author A", "Author B"],
        }

        result = ArticleHandler.format_result(preprint)

        assert result["id"] == "10.1101/2023.01.01.12345"
        assert result["title"] == "Preprint Title"
        assert result["snippet"] == "Short abstract..."
        assert (
            result["url"]
            == "https://www.biorxiv.org/content/10.1101/2023.01.01.12345"
        )
        assert result["metadata"]["source"] == "bioRxiv"

    def test_format_article_missing_fields(self):
        """Test formatting article with missing fields."""
        article = {
            "pmid": "67890",
            # Missing title, abstract, etc.
        }

        result = ArticleHandler.format_result(article)

        assert result["id"] == "67890"
        assert (
            result["title"] == DEFAULT_TITLE
        )  # Should use default for missing title
        assert result["snippet"] == ""  # Empty when no abstract
        assert result["url"] == "https://pubmed.ncbi.nlm.nih.gov/67890/"

    def test_format_article_with_date_field(self):
        """Test formatting article with date field instead of pub_year."""
        article = {
            "pmid": "123",
            "title": "Test",
            "date": "2023-05-15",
        }

        result = ArticleHandler.format_result(article)

        assert result["metadata"]["year"] == "2023"

    def test_format_article_title_normalization(self):
        """Test that article title whitespace is normalized."""
        article = {
            "pmid": "123",
            "title": "  Test   Article\n\nWith  Extra   Spaces  ",
        }

        result = ArticleHandler.format_result(article)

        assert result["title"] == "Test Article With Extra Spaces"


class TestTrialHandler:
    """Test TrialHandler class."""

    def test_format_trial_api_v2(self):
        """Test formatting trial with API v2 structure."""
        trial = {
            "protocolSection": {
                "identificationModule": {
                    "nctId": "NCT12345",
                    "briefTitle": "Brief Title",
                    "officialTitle": "Official Title",
                },
                "statusModule": {
                    "overallStatus": "RECRUITING",
                    "startDateStruct": {"date": "2023-01-01"},
                    "primaryCompletionDateStruct": {"date": "2024-12-31"},
                },
                "descriptionModule": {
                    "briefSummary": "This is a brief summary of the trial."
                },
                "designModule": {
                    "phases": ["PHASE3"],
                },
            }
        }

        result = TrialHandler.format_result(trial)

        assert result["id"] == "NCT12345"
        assert result["title"] == "Brief Title"
        assert "brief summary" in result["snippet"]
        assert result["url"] == "https://clinicaltrials.gov/study/NCT12345"
        assert result["metadata"]["status"] == "RECRUITING"
        assert result["metadata"]["phase"] == "PHASE3"
        assert result["metadata"]["start_date"] == "2023-01-01"
        assert result["metadata"]["primary_completion_date"] == "2024-12-31"

    def test_format_trial_legacy_flat(self):
        """Test formatting trial with legacy flat structure."""
        trial = {
            "NCT Number": "NCT67890",
            "Study Title": "Legacy Trial Title",
            "Brief Summary": "Legacy summary",
            "Study Status": "COMPLETED",
            "Phases": "Phase 2",
            "Start Date": "2022-01-01",
            "Completion Date": "2023-12-31",
        }

        result = TrialHandler.format_result(trial)

        assert result["id"] == "NCT67890"
        assert result["title"] == "Legacy Trial Title"
        assert result["snippet"].startswith("Legacy summary")
        assert result["url"] == "https://clinicaltrials.gov/study/NCT67890"
        assert result["metadata"]["status"] == "COMPLETED"
        assert result["metadata"]["phase"] == "Phase 2"

    def test_format_trial_legacy_simple(self):
        """Test formatting trial with legacy simple structure."""
        trial = {
            "nct_id": "NCT11111",
            "brief_title": "Simple Trial",
            "overall_status": "ACTIVE",
            "phase": "PHASE1",
        }

        result = TrialHandler.format_result(trial)

        assert result["id"] == "NCT11111"
        assert result["title"] == "Simple Trial"
        assert result["metadata"]["status"] == "ACTIVE"
        assert result["metadata"]["phase"] == "PHASE1"

    def test_format_trial_missing_title(self):
        """Test formatting trial with missing brief title."""
        trial = {
            "protocolSection": {
                "identificationModule": {
                    "nctId": "NCT99999",
                    "officialTitle": "Only Official Title",
                },
            }
        }

        result = TrialHandler.format_result(trial)

        assert result["id"] == "NCT99999"
        assert result["title"] == "Only Official Title"

    def test_format_trial_empty_phases(self):
        """Test formatting trial with empty phases array."""
        trial = {
            "protocolSection": {
                "identificationModule": {"nctId": "NCT123"},
                "designModule": {"phases": []},
            }
        }

        result = TrialHandler.format_result(trial)

        assert result["metadata"]["phase"] == ""


class TestVariantHandler:
    """Test VariantHandler class."""

    def test_format_variant_complete(self):
        """Test formatting variant with complete data."""
        variant = {
            "_id": "chr7:g.140453136A>T",
            "dbnsfp": {
                "genename": "BRAF",
                "hgvsp": ["BRAF:p.V600E"],
            },
            "dbsnp": {
                "rsid": "rs121913529",
                "gene": {"symbol": "BRAF"},
            },
            "clinvar": {
                "rcv": {
                    "clinical_significance": "Pathogenic",
                }
            },
            "cadd": {
                "consequence": "missense_variant",
            },
        }

        result = VariantHandler.format_result(variant)

        assert result["id"] == "chr7:g.140453136A>T"
        assert result["title"] == "BRAF BRAF:p.V600E"
        assert "Pathogenic" in result["snippet"]
        assert "rs121913529" in result["url"]
        assert result["metadata"]["gene"] == "BRAF"
        assert result["metadata"]["rsid"] == "rs121913529"
        assert result["metadata"]["clinical_significance"] == "Pathogenic"
        assert result["metadata"]["consequence"] == "missense_variant"

    def test_format_variant_gene_list(self):
        """Test formatting variant when gene is a list."""
        variant = {
            "_id": "rs123",
            "dbnsfp": {"genename": ["GENE1", "GENE2"]},
        }

        result = VariantHandler.format_result(variant)

        assert result["metadata"]["gene"] == "GENE1"

    def test_format_variant_clinvar_list(self):
        """Test formatting variant when clinvar RCV is a list."""
        variant = {
            "_id": "rs456",
            "clinvar": {
                "rcv": [
                    {"clinical_significance": "Pathogenic"},
                    {"clinical_significance": "Likely pathogenic"},
                ]
            },
        }

        result = VariantHandler.format_result(variant)

        assert result["metadata"]["clinical_significance"] == "Pathogenic"

    def test_format_variant_minimal(self):
        """Test formatting variant with minimal data."""
        variant = {
            "_id": "chr1:g.12345A>G",
        }

        result = VariantHandler.format_result(variant)

        assert result["id"] == "chr1:g.12345A>G"
        assert result["title"] == "chr1:g.12345A>G"
        assert "Unknown" in result["snippet"]
        assert result["url"] == ""

    def test_format_variant_hgvsp_list(self):
        """Test formatting variant when HGVS protein is a list."""
        variant = {
            "_id": "rs789",
            "dbnsfp": {
                "genename": "TP53",
                "hgvsp": ["TP53:p.R175H", "TP53:p.R175C"],
            },
        }

        result = VariantHandler.format_result(variant)

        assert result["title"] == "TP53 TP53:p.R175H"

    def test_format_variant_no_rsid_url(self):
        """Test variant URL generation without rsID."""
        variant = {
            "_id": "chr2:g.234567C>T",
        }

        result = VariantHandler.format_result(variant)

        assert result["url"] == ""


class TestGetDomainHandler:
    """Test get_domain_handler function."""

    def test_get_article_handler(self):
        """Test getting article handler."""
        handler = get_domain_handler("article")
        assert handler == ArticleHandler

    def test_get_trial_handler(self):
        """Test getting trial handler."""
        handler = get_domain_handler("trial")
        assert handler == TrialHandler

    def test_get_variant_handler(self):
        """Test getting variant handler."""
        handler = get_domain_handler("variant")
        assert handler == VariantHandler

    def test_get_invalid_handler(self):
        """Test getting handler for invalid domain."""
        with pytest.raises(ValueError) as exc_info:
            get_domain_handler("invalid")

        assert "Unknown domain: invalid" in str(exc_info.value)

    def test_get_handler_case_sensitive(self):
        """Test that domain names are case sensitive."""
        # Should work with lowercase
        handler = get_domain_handler("article")
        assert handler == ArticleHandler

        # Should fail with uppercase
        with pytest.raises(ValueError):
            get_domain_handler("ARTICLE")
