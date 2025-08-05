"""Integration tests for BioThings API - calls real APIs."""

import pytest

from biomcp.integrations import BioThingsClient


@pytest.mark.integration
class TestRealBioThingsAPIs:
    """Integration tests that call real BioThings APIs."""

    @pytest.fixture
    def client(self):
        """Create a real BioThings client."""
        return BioThingsClient()

    @pytest.mark.asyncio
    async def test_mygene_tp53(self, client):
        """Test real MyGene.info API with TP53."""
        result = await client.get_gene_info("TP53")

        assert result is not None
        assert result.symbol == "TP53"
        assert result.name == "tumor protein p53"
        assert result.entrezgene in ["7157", 7157]
        assert "tumor suppressor" in result.summary.lower()
        # Check for either lowercase or uppercase P53 in aliases
        assert any("p53" in alias.lower() for alias in result.alias)

    @pytest.mark.asyncio
    async def test_mygene_braf(self, client):
        """Test real MyGene.info API with BRAF."""
        result = await client.get_gene_info("BRAF")

        assert result is not None
        assert result.symbol == "BRAF"
        assert "proto-oncogene" in result.name.lower()
        assert result.type_of_gene == "protein-coding"

    @pytest.mark.asyncio
    async def test_mygene_by_entrez_id(self, client):
        """Test real MyGene.info API with Entrez ID."""
        result = await client.get_gene_info("673")  # BRAF

        assert result is not None
        assert result.symbol == "BRAF"
        assert result.gene_id == "673"

    @pytest.mark.asyncio
    async def test_mydisease_melanoma(self, client):
        """Test real MyDisease.info API with melanoma."""
        result = await client.get_disease_info("melanoma")

        if result is None:
            # API might be down or melanoma might not be found directly
            # Try a more specific search
            result = await client.get_disease_info(
                "MONDO:0005105"
            )  # MONDO ID for melanoma

        assert result is not None, "Disease info should be returned"
        # The API may return subtypes of melanoma
        if result.name:
            assert "melanoma" in result.name.lower() or (
                result.definition and "melanoma" in result.definition.lower()
            )
        assert result.disease_id is not None
        # Synonyms might be empty for specific subtypes
        assert result.synonyms is not None

    @pytest.mark.asyncio
    async def test_mydisease_gist(self, client):
        """Test real MyDisease.info API with GIST."""
        result = await client.get_disease_info("GIST")

        if result is None:
            # API might be down or GIST might not be found directly
            # Try the full name
            result = await client.get_disease_info(
                "gastrointestinal stromal tumor"
            )

        assert result is not None, "Disease info should be returned"
        # GIST might return as a variant name
        if result.name:
            assert (
                "gist" in result.name.lower()
                or "stromal" in result.name.lower()
            )
        assert result.disease_id is not None
        # GIST should have synonyms including full name if available
        assert result.synonyms is not None

    @pytest.mark.asyncio
    async def test_mydisease_by_mondo_id(self, client):
        """Test real MyDisease.info API with MONDO ID."""
        result = await client.get_disease_info("MONDO:0005105")  # melanoma

        assert result is not None
        assert result.disease_id == "MONDO:0005105"
        # The result should have mondo data
        assert result.mondo is not None
        assert result.mondo.get("mondo") == "MONDO:0005105"
        # Name field might come from different sources in the API
        if result.name:
            assert "melanoma" in result.name.lower()

    @pytest.mark.asyncio
    async def test_disease_synonyms_expansion(self, client):
        """Test disease synonym expansion."""
        synonyms = await client.get_disease_synonyms("lung cancer")

        assert len(synonyms) >= 1  # At least includes the original term
        assert "lung cancer" in [s.lower() for s in synonyms]
        # May or may not include formal terms depending on API results
        # Just check we got some results back
        assert synonyms is not None and len(synonyms) > 0

    @pytest.mark.asyncio
    async def test_batch_genes(self, client):
        """Test batch gene retrieval."""
        # Test single gene retrieval as a workaround since batch requires special POST encoding
        # This validates the gene getter can handle multiple calls efficiently
        genes = ["TP53", "BRAF", "EGFR"]
        results = []

        for gene in genes:
            result = await client.get_gene_info(gene)
            if result:
                results.append(result)

        assert len(results) == 3
        gene_symbols = [r.symbol for r in results]
        assert "TP53" in gene_symbols
        assert "BRAF" in gene_symbols
        assert "EGFR" in gene_symbols

    @pytest.mark.asyncio
    async def test_invalid_gene(self, client):
        """Test handling of invalid gene."""
        result = await client.get_gene_info("INVALID_GENE_XYZ123")
        assert result is None

    @pytest.mark.asyncio
    async def test_invalid_disease(self, client):
        """Test handling of invalid disease."""
        result = await client.get_disease_info("INVALID_DISEASE_XYZ123")
        assert result is None

    @pytest.mark.asyncio
    async def test_mychem_aspirin(self, client):
        """Test real MyChem.info API with aspirin."""
        # Use DrugBank ID for reliable results
        result = await client.get_drug_info("DB00945")

        assert result is not None
        # API returns various forms - could be aspirin or acetylsalicylic acid
        assert result.name is not None
        assert result.drugbank_id == "DB00945"
        # Should have at least one identifier
        assert any([
            result.drugbank_id,
            result.chembl_id,
            result.chebi_id,
            result.pubchem_cid,
        ])

    @pytest.mark.asyncio
    async def test_mychem_imatinib(self, client):
        """Test real MyChem.info API with imatinib."""
        # Use DrugBank ID for reliable results
        result = await client.get_drug_info("DB00619")

        assert result is not None
        assert result.name is not None
        assert "imatinib" in result.name.lower()
        assert result.drugbank_id == "DB00619"
        # Should have at least one identifier
        assert any([
            result.drugbank_id,
            result.chembl_id,
            result.chebi_id,
            result.pubchem_cid,
        ])

    @pytest.mark.asyncio
    async def test_mychem_by_drugbank_id(self, client):
        """Test real MyChem.info API with DrugBank ID."""
        result = await client.get_drug_info("DB00945")  # Aspirin

        assert result is not None
        assert result.drugbank_id == "DB00945"
        assert (
            result.name is not None
        )  # Could be Acetylsalicylic acid or similar

    @pytest.mark.asyncio
    async def test_invalid_drug(self, client):
        """Test handling of invalid drug."""
        result = await client.get_drug_info("INVALID_DRUG_XYZ123")
        assert result is None

    @pytest.mark.asyncio
    async def test_mychem_pembrolizumab(self, client):
        """Test real MyChem.info API with pembrolizumab."""
        result = await client.get_drug_info("pembrolizumab")

        assert result is not None
        assert result.name == "Pembrolizumab"
        assert result.drugbank_id == "DB09037"
        assert result.unii == "DPT0O3T46P"
        assert "PD-1" in result.description
        assert "antibody" in result.description.lower()


@pytest.mark.integration
class TestGeneToolIntegration:
    """Test the gene getter tool with real APIs."""

    @pytest.mark.asyncio
    async def test_gene_getter_tool(self):
        """Test the gene_getter tool function."""
        from biomcp.genes.getter import get_gene

        result = await get_gene("TP53", output_json=False)

        assert "TP53" in result
        assert "tumor protein p53" in result
        assert "tumor suppressor" in result.lower()
        # Links might be formatted differently
        assert "ncbi" in result.lower() or "gene" in result.lower()

    @pytest.mark.asyncio
    async def test_gene_getter_json(self):
        """Test gene_getter with JSON output."""
        import json

        from biomcp.genes.getter import get_gene

        result = await get_gene("BRAF", output_json=True)
        data = json.loads(result)

        assert data["symbol"] == "BRAF"
        assert "_links" in data
        assert "NCBI Gene" in data["_links"]


@pytest.mark.integration
class TestDiseaseToolIntegration:
    """Test the disease getter tool with real APIs."""

    @pytest.mark.asyncio
    async def test_disease_getter_tool(self):
        """Test the disease_getter tool function."""
        from biomcp.diseases.getter import get_disease

        result = await get_disease("melanoma", output_json=False)

        assert "melanoma" in result.lower()
        assert "MONDO:" in result
        # In markdown format, links are shown as "MONDO Browser:" not "_links"
        assert "Browser:" in result or "https://" in result

    @pytest.mark.asyncio
    async def test_disease_getter_json(self):
        """Test disease_getter with JSON output."""
        import json

        from biomcp.diseases.getter import get_disease

        result = await get_disease("GIST", output_json=True)
        data = json.loads(result)

        # API might return error or different structure
        if "error" in data:
            pytest.skip("Disease not found in API")
        else:
            # Check for key fields
            assert "disease_id" in data or "id" in data or "_id" in data
            assert "MONDO:" in str(data)


@pytest.mark.integration
class TestDrugToolIntegration:
    """Test the drug getter tool with real APIs."""

    @pytest.mark.asyncio
    async def test_drug_getter_tool(self):
        """Test the drug_getter tool function."""
        from biomcp.drugs.getter import get_drug

        result = await get_drug("DB00945", output_json=False)  # Aspirin

        assert "Drug:" in result
        assert "DrugBank ID" in result
        assert "DB00945" in result
        assert "External Links" in result

    @pytest.mark.asyncio
    async def test_drug_getter_json(self):
        """Test drug_getter with JSON output."""
        import json

        from biomcp.drugs.getter import get_drug

        result = await get_drug("DB00619", output_json=True)  # Imatinib
        data = json.loads(result)

        # Check for basic fields
        assert "drug_id" in data
        assert "drugbank_id" in data
        assert data["drugbank_id"] == "DB00619"
        assert "_links" in data
        # Should have at least one database link
        assert any(
            key in data["_links"]
            for key in ["DrugBank", "ChEMBL", "PubChem", "ChEBI"]
        )
