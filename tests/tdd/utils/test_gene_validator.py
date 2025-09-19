"""Tests for gene validation utilities."""

from biomcp.utils.gene_validator import (
    is_valid_gene_symbol,
    sanitize_gene_symbol,
)


class TestGeneValidator:
    """Test gene symbol validation."""

    def test_valid_gene_symbols(self):
        """Test that valid gene symbols are accepted."""
        valid_genes = [
            "BRAF",
            "TP53",
            "KRAS",
            "EGFR",
            "PIK3CA",
            "BRCA1",
            "BRCA2",
            "MYC",
            "ERBB2",
            "CDKN2A",
            "VHL",
            "RB1",
            "PTEN",
            "APC",
            "MLH1",
            "MSH2",
            "MSH6",
            "PMS2",
            "ATM",
            "CHEK2",
            "PALB2",
            "RAD51C",
            "RAD51D",
            "BRIP1",
            "CDH1",
            "STK11",
            "MUTYH",
            "BMPR1A",
            "SMAD4",
            "ALK",
            "ROS1",
            "RET",
            "MET",
            "HER2",
            "FGFR1",
            "FGFR2",
            "FGFR3",
            "FGFR4",
            "IDH1",
            "IDH2",
            "TERT",
            "ATRX",
            "H3F3A",
            "HIST1H3B",
            "BRAFV600E",  # With mutation
            "KRASG12D",  # With mutation
            "EGFRL858R",  # With mutation
        ]

        for gene in valid_genes:
            assert is_valid_gene_symbol(
                gene
            ), f"Should accept valid gene: {gene}"

    def test_invalid_gene_symbols(self):
        """Test that invalid gene symbols are rejected."""
        invalid_genes = [
            None,
            "",
            " ",
            "  ",
            "123",  # Starts with number
            "A",  # Too short
            "INVALID_GENE_XYZ",  # Known invalid
            "TEST",
            "NULL",
            "NONE",
            "UNKNOWN",
            "gene",  # Lowercase
            "Braf",  # Mixed case
            "GENE-WITH-SPECIAL-CHARS!",
            "GENE WITH SPACES",
            "GENE/WITH/SLASHES",
            "GENE.WITH.DOTS",
            "VERYLONGGENENAMETHATEXCEEDSLIMIT",  # Too long
            "_GENE",  # Starts with underscore
            "-GENE",  # Starts with hyphen
        ]

        for gene in invalid_genes:
            assert not is_valid_gene_symbol(
                gene
            ), f"Should reject invalid gene: {gene}"

    def test_gene_symbols_with_version(self):
        """Test gene symbols with version suffixes."""
        versioned_genes = [
            "MT-CO1",
            "MT-CO2",
            "MT-CO3",
            "HLA-A",
            "HLA-B",
            "HLA-C",
            "HLA-DRB1",
            "HLA-DQB1",
            "HLA-DPB1",
        ]

        for gene in versioned_genes:
            assert is_valid_gene_symbol(
                gene
            ), f"Should accept versioned gene: {gene}"

    def test_sanitize_gene_symbol(self):
        """Test gene symbol sanitization."""
        # Test uppercase conversion
        assert sanitize_gene_symbol("braf") == "BRAF"
        assert sanitize_gene_symbol("Tp53") == "TP53"
        assert sanitize_gene_symbol("kRaS") == "KRAS"

        # Test whitespace stripping
        assert sanitize_gene_symbol(" BRAF ") == "BRAF"
        assert sanitize_gene_symbol("\tTP53\n") == "TP53"
        assert sanitize_gene_symbol("  KRAS  ") == "KRAS"

        # Test combination
        assert sanitize_gene_symbol("  braf  ") == "BRAF"
        assert sanitize_gene_symbol("\ttp53\n") == "TP53"
