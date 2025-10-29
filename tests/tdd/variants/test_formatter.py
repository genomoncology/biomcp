"""Tests for variant formatter module."""

from biomcp.variants.formatter import (
    _consolidate_clinical,
    _consolidate_databases,
    _consolidate_external,
    _consolidate_frequencies,
    _consolidate_predictions,
    _extract_alleles,
    _extract_summary,
    consolidate_multi_allelic_variants,
)


class TestConsolidateMultiAllelicVariants:
    """Test the main consolidation function."""

    def test_empty_list(self):
        """Test with empty variant list."""
        result = consolidate_multi_allelic_variants([])
        assert result == {}

    def test_single_variant(self):
        """Test consolidation with single variant."""
        variant = {
            "_id": "chr7:g.140453136A>T",
            "dbsnp": {
                "rsid": "rs113488022",
                "gene": {"symbol": "BRAF", "geneid": 673},
            },
            "hg19": {"start": 140453136, "end": 140453136},
            "chrom": "7",
        }

        result = consolidate_multi_allelic_variants([variant])

        assert "summary" in result
        assert "alleles" in result
        assert "predictions" in result
        assert "clinical_significance" in result

    def test_multi_allelic_variants(self):
        """Test consolidation with multiple alleles."""
        variant1 = {
            "_id": "chr7:g.140453136A>G",
            "dbsnp": {
                "rsid": "rs113488022",
                "gene": {"symbol": "BRAF", "geneid": 673},
            },
            "vcf": {"ref": "A", "alt": "G"},
        }
        variant2 = {
            "_id": "chr7:g.140453136A>T",
            "dbsnp": {
                "rsid": "rs113488022",
                "gene": {"symbol": "BRAF", "geneid": 673},
            },
            "vcf": {"ref": "A", "alt": "T"},
        }

        result = consolidate_multi_allelic_variants([variant1, variant2])

        assert len(result["alleles"]) == 2
        assert result["alleles"][0]["alt"] == "G"
        assert result["alleles"][1]["alt"] == "T"


class TestExtractSummary:
    """Test summary extraction."""

    def test_extract_gene_info(self):
        """Test extracting gene information."""
        variants = [
            {
                "dbsnp": {
                    "rsid": "rs113488022",
                    "gene": {
                        "symbol": "BRAF",
                        "geneid": 673,
                        "name": "B-Raf proto-oncogene",
                    },
                },
                "hg19": {"start": 140453136, "end": 140453136},
                "chrom": "7",
            }
        ]

        result = _extract_summary(variants)

        assert result["gene"]["symbol"] == "BRAF"
        assert result["gene"]["id"] == 673
        assert result["rsid"] == "rs113488022"
        assert result["position"]["chromosome"] == "7"

    def test_extract_with_dbnsfp_gene(self):
        """Test extracting gene from dbnsfp when dbsnp not available."""
        variants = [{"dbnsfp": {"genename": "BRAF"}}]

        result = _extract_summary(variants)

        assert result["gene"]["symbol"] == "BRAF"


class TestExtractAlleles:
    """Test allele extraction."""

    def test_extract_basic_allele_info(self):
        """Test extracting basic allele information."""
        variants = [
            {
                "_id": "chr7:g.140453136A>T",
                "vcf": {"ref": "A", "alt": "T"},
                "dbnsfp": {
                    "aa": {"ref": "V", "alt": "E", "pos": 600},
                    "hgvsp": "p.Val600Glu",
                },
            }
        ]

        result = _extract_alleles(variants)

        assert len(result) == 1
        assert result[0]["id"] == "chr7:g.140453136A>T"
        assert result[0]["ref"] == "A"
        assert result[0]["alt"] == "T"
        assert result[0]["aa_change"] == "V600E"

    def test_extract_multiple_hgvs(self):
        """Test extracting and deduplicating HGVS notations."""
        variants = [
            {
                "_id": "chr7:g.140453136A>T",
                "vcf": {"ref": "A", "alt": "T"},
                "dbnsfp": {
                    "hgvsp": ["p.Val600Glu", "p.Val640Glu", "p.Val600Glu"],
                    "hgvsc": "c.1799T>A",
                },
            }
        ]

        result = _extract_alleles(variants)

        # Should deduplicate and limit to 3
        assert len(result[0]["hgvs"]) <= 3
        # Should not have duplicates
        assert len(set(result[0]["hgvs"])) == len(result[0]["hgvs"])


class TestConsolidatePredictions:
    """Test prediction score consolidation."""

    def test_consolidate_cadd_scores(self):
        """Test consolidating CADD scores."""
        variants = [
            {"cadd": {"phred": 21.2}},
            {"cadd": {"phred": 32.0}},
        ]

        result = _consolidate_predictions(variants)

        assert "CADD" in result
        assert result["CADD"] == [21.2, 32.0]

    def test_consolidate_revel_scores(self):
        """Test consolidating REVEL scores."""
        variants = [
            {"dbnsfp": {"revel": {"score": 0.672}}},
            {"dbnsfp": {"revel": {"score": 0.931}}},
        ]

        result = _consolidate_predictions(variants)

        assert "REVEL" in result
        assert result["REVEL"] == [0.672, 0.931]

    def test_handle_missing_scores(self):
        """Test handling variants with missing prediction scores."""
        variants = [
            {"cadd": {"phred": 21.2}},
            {"dbnsfp": {}},  # Missing CADD
        ]

        result = _consolidate_predictions(variants)

        assert "CADD" in result
        assert result["CADD"] == [21.2, None]


class TestConsolidateClinical:
    """Test clinical data consolidation."""

    def test_consolidate_clinvar_data(self):
        """Test consolidating ClinVar submissions."""
        variants = [
            {
                "clinvar": {
                    "rcv": {
                        "clinical_significance": "Pathogenic",
                        "conditions": {"name": "Melanoma"},
                    }
                }
            },
            {
                "clinvar": {
                    "rcv": [
                        {
                            "clinical_significance": "Likely pathogenic",
                            "conditions": {"name": "Thyroid cancer"},
                        }
                    ]
                }
            },
        ]

        result = _consolidate_clinical(variants)

        assert "clinvar" in result
        assert result["clinvar"]["variant_count"] == 2
        assert "Pathogenic" in result["clinvar"]["clinical_significances"]
        assert "Melanoma" in result["clinvar"]["associated_conditions"]

    def test_consolidate_cosmic_data(self):
        """Test consolidating COSMIC data."""
        variants = [
            {"cosmic": {"tumor_site": "melanoma"}},
            {"cosmic": [{"tumor_site": "thyroid"}, {"tumor_site": "colon"}]},
        ]

        result = _consolidate_clinical(variants)

        assert "cosmic" in result
        assert result["cosmic"]["total_entries"] == 3
        assert set(result["cosmic"]["tumor_sites"]) == {
            "melanoma",
            "thyroid",
            "colon",
        }

    def test_consolidate_civic_data(self):
        """Test consolidating CIViC data."""
        variants = [
            {"civic": {"id": 12}},
            {},  # No CIViC data
            {"civic": {"id": 13}},
        ]

        result = _consolidate_clinical(variants)

        assert "civic" in result
        assert result["civic"]["variant_count"] == 2


class TestConsolidateFrequencies:
    """Test population frequency consolidation."""

    def test_consolidate_gnomad_frequencies(self):
        """Test consolidating gnomAD frequencies."""
        variants = [
            {"gnomad_exome": {"af": {"af": 1.65e-05}}},
            {"gnomad_exome": {"af": {"af": 3.98e-06}}},
        ]

        result = _consolidate_frequencies(variants)

        assert "gnomad_exome" in result
        assert result["gnomad_exome"]["allele_1"] == 1.65e-05
        assert result["gnomad_exome"]["allele_2"] == 3.98e-06

    def test_consolidate_exac_frequencies(self):
        """Test consolidating ExAC frequencies."""
        variants = [
            {"exac": {"af": 1.647e-05}},
            {},  # Missing ExAC data
        ]

        result = _consolidate_frequencies(variants)

        assert "exac" in result
        assert result["exac"]["allele_1"] == 1.647e-05
        assert "allele_2" not in result["exac"]


class TestConsolidateDatabases:
    """Test database cross-reference consolidation."""

    def test_consolidate_cosmic_ids(self):
        """Test consolidating COSMIC IDs."""
        variants = [
            {"cosmic": {"cosmic_id": "COSM476"}},
            {
                "cosmic": [
                    {"cosmic_id": "COSM18443"},
                    {"cosmic_id": "COSM6137"},
                ]
            },
        ]

        result = _consolidate_databases(variants)

        assert "cosmic_ids" in result
        assert len(result["cosmic_ids"]) <= 5  # Top 5
        assert "COSM476" in result["cosmic_ids"]

    def test_consolidate_clinvar_ids(self):
        """Test consolidating ClinVar IDs."""
        variants = [
            {"clinvar": {"variant_id": 13961}},
            {"clinvar": {"variant_id": 40389}},
        ]

        result = _consolidate_databases(variants)

        assert "clinvar_ids" in result
        assert 13961 in result["clinvar_ids"]
        assert 40389 in result["clinvar_ids"]


class TestConsolidateExternal:
    """Test external annotation consolidation."""

    def test_consolidate_cbioportal_data(self):
        """Test consolidating cBioPortal annotations."""
        variants = [
            {
                "_id": "chr7:g.140453136A>T",
                "cbioportal": {
                    "total_cases": 838,
                    "hotspot_samples": 870,
                    "cancer_types": {
                        "Melanoma": 197,
                        "Thyroid": 284,
                        "Colorectal": 71,
                    },
                },
            }
        ]

        result = _consolidate_external(variants)

        assert "cbioportal" in result
        assert len(result["cbioportal"]) == 1
        assert result["cbioportal"][0]["total_cases"] == 838
        assert len(result["cbioportal"][0]["cancer_types"]) <= 5

    def test_consolidate_oncokb_data(self):
        """Test consolidating OncoKB annotations."""
        variants = [
            {
                "_id": "chr7:g.140453136A>T",
                "oncokb": {
                    "oncogenic": "Oncogenic",
                    "mutation_effect": "Gain-of-function",
                    "is_hotspot": True,
                },
            }
        ]

        result = _consolidate_external(variants)

        assert "oncokb" in result
        assert result["oncokb"][0]["oncogenic"] == "Oncogenic"
        assert result["oncokb"][0]["is_hotspot"] is True

    def test_consolidate_tcga_data(self):
        """Test consolidating TCGA annotations."""
        variants = [
            {
                "_id": "chr7:g.140453136A>T",
                "tcga": {"case_count": 150},
            }
        ]

        result = _consolidate_external(variants)

        assert "tcga" in result
        assert result["tcga"][0]["case_count"] == 150
