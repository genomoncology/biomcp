"""Tests for unified search/fetch with BioThings domains."""

import json

import pytest

from biomcp.router import fetch, search


class TestUnifiedBioThingsSearch:
    """Test unified search with BioThings domains."""

    @pytest.mark.asyncio
    async def test_search_gene_domain(self, monkeypatch):
        """Test searching genes through unified search."""
        # Mock the BioThingsClient
        mock_gene_query = [{"_id": "673", "symbol": "BRAF"}]
        mock_gene_details = {
            "_id": "673",
            "symbol": "BRAF",
            "name": "B-Raf proto-oncogene, serine/threonine kinase",
            "summary": "This gene encodes a protein belonging to the RAF family...",
            "entrezgene": 673,
        }

        class MockBioThingsClient:
            async def _query_gene(self, query):
                return mock_gene_query

            async def _get_gene_by_id(self, gene_id):
                from biomcp.integrations.biothings_client import GeneInfo

                return GeneInfo(**mock_gene_details)

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        # Test gene search
        results = await search(query="", domain="gene", keywords=["BRAF"])

        assert "results" in results
        # Skip thinking reminder if present
        actual_results = [
            r for r in results["results"] if r["id"] != "thinking-reminder"
        ]
        assert len(actual_results) == 1
        assert actual_results[0]["id"] == "673"
        assert "BRAF" in actual_results[0]["title"]

    @pytest.mark.asyncio
    async def test_search_drug_domain(self, monkeypatch):
        """Test searching drugs through unified search."""
        # Mock the BioThingsClient
        mock_drug_query = [{"_id": "CHEMBL941"}]
        mock_drug_details = {
            "_id": "CHEMBL941",
            "name": "Imatinib",
            "drugbank_id": "DB00619",
            "description": "Imatinib is a tyrosine kinase inhibitor...",
            "indication": "Treatment of chronic myeloid leukemia...",
        }

        class MockBioThingsClient:
            async def _query_drug(self, query):
                return mock_drug_query

            async def _get_drug_by_id(self, drug_id):
                from biomcp.integrations.biothings_client import DrugInfo

                return DrugInfo(**mock_drug_details)

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        # Test drug search
        results = await search(query="", domain="drug", keywords=["imatinib"])

        assert "results" in results
        # Skip thinking reminder if present
        actual_results = [
            r for r in results["results"] if r["id"] != "thinking-reminder"
        ]
        assert len(actual_results) == 1
        assert actual_results[0]["id"] == "CHEMBL941"
        assert "Imatinib" in actual_results[0]["title"]

    @pytest.mark.asyncio
    async def test_search_disease_domain(self, monkeypatch):
        """Test searching diseases through unified search."""
        # Mock the BioThingsClient
        mock_disease_query = [{"_id": "MONDO:0005105"}]
        mock_disease_details = {
            "_id": "MONDO:0005105",
            "name": "melanoma",
            "definition": "A malignant neoplasm composed of melanocytes.",
            "mondo": {"id": "MONDO:0005105"},
            "phenotypes": [],
        }

        class MockBioThingsClient:
            async def _query_disease(self, query):
                return mock_disease_query

            async def _get_disease_by_id(self, disease_id):
                from biomcp.integrations.biothings_client import DiseaseInfo

                return DiseaseInfo(**mock_disease_details)

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        # Test disease search
        results = await search(
            query="", domain="disease", keywords=["melanoma"]
        )

        assert "results" in results
        # Skip thinking reminder if present
        actual_results = [
            r for r in results["results"] if r["id"] != "thinking-reminder"
        ]
        assert len(actual_results) == 1
        assert actual_results[0]["id"] == "MONDO:0005105"
        assert "melanoma" in actual_results[0]["title"]


class TestUnifiedBioThingsFetch:
    """Test unified fetch with BioThings domains."""

    @pytest.mark.asyncio
    async def test_fetch_gene(self, monkeypatch):
        """Test fetching gene information."""
        mock_gene_info = {
            "_id": "673",
            "symbol": "BRAF",
            "name": "B-Raf proto-oncogene, serine/threonine kinase",
            "summary": "This gene encodes a protein belonging to the RAF family...",
            "entrezgene": 673,
            "type_of_gene": "protein-coding",
            "alias": ["BRAF1", "B-RAF1"],
        }

        class MockBioThingsClient:
            async def get_gene_info(self, gene_id):
                from biomcp.integrations.biothings_client import GeneInfo

                return GeneInfo(**mock_gene_info)

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        # Test gene fetch
        result = await fetch(id="BRAF", domain="gene")

        assert result["id"] == "673"
        assert "BRAF" in result["title"]
        assert "B-Raf proto-oncogene" in result["title"]
        assert "Entrez ID: 673" in result["text"]
        assert "Type: protein-coding" in result["text"]

    @pytest.mark.asyncio
    async def test_fetch_drug(self, monkeypatch):
        """Test fetching drug information."""
        mock_drug_info = {
            "_id": "CHEMBL941",
            "name": "Imatinib",
            "drugbank_id": "DB00619",
            "description": "Imatinib is a tyrosine kinase inhibitor...",
            "indication": "Treatment of chronic myeloid leukemia...",
            "mechanism_of_action": "Inhibits BCR-ABL tyrosine kinase...",
            "tradename": ["Gleevec", "Glivec"],
            "formula": "C29H31N7O",
        }

        class MockBioThingsClient:
            async def get_drug_info(self, drug_id):
                from biomcp.integrations.biothings_client import DrugInfo

                return DrugInfo(**mock_drug_info)

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        # Test drug fetch
        result = await fetch(id="imatinib", domain="drug")

        assert result["id"] == "CHEMBL941"
        assert "Imatinib" in result["title"]
        assert "DrugBank ID: DB00619" in result["text"]
        assert "Formula: C29H31N7O" in result["text"]
        assert "Trade Names: Gleevec, Glivec" in result["text"]

    @pytest.mark.asyncio
    async def test_fetch_disease(self, monkeypatch):
        """Test fetching disease information."""
        mock_disease_info = {
            "_id": "MONDO:0005105",
            "name": "melanoma",
            "definition": "A malignant neoplasm composed of melanocytes.",
            "mondo": {"id": "MONDO:0005105"},
            "synonyms": [
                "malignant melanoma",
                "melanoma, malignant",
                "melanosarcoma",
            ],
            "phenotypes": [{"hp": "HP:0002861"}],
        }

        class MockBioThingsClient:
            async def get_disease_info(self, disease_id):
                from biomcp.integrations.biothings_client import DiseaseInfo

                return DiseaseInfo(**mock_disease_info)

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        # Test disease fetch
        result = await fetch(id="melanoma", domain="disease")

        assert result["id"] == "MONDO:0005105"
        assert "melanoma" in result["title"]
        assert "MONDO ID: MONDO:0005105" in result["text"]
        assert "Definition:" in result["text"]
        assert "Synonyms:" in result["text"]
        assert "Associated Phenotypes: 1" in result["text"]


class TestUnifiedQueryLanguage:
    """Test unified query language with BioThings domains."""

    @pytest.mark.asyncio
    async def test_cross_domain_gene_search(self, monkeypatch):
        """Test that gene searches include gene domain."""
        # Mock multiple domain searches
        searched_domains = []

        async def mock_execute_routing_plan(plan, output_json=True):
            searched_domains.extend(plan.tools_to_call)
            return {
                "articles": json.dumps([]),
                "variants": json.dumps([]),
                "genes": json.dumps([]),
                "trials": json.dumps([]),
            }

        monkeypatch.setattr(
            "biomcp.router.execute_routing_plan", mock_execute_routing_plan
        )

        # Test cross-domain gene search
        await search(query="gene:BRAF")

        assert "gene_searcher" in searched_domains
        assert "article_searcher" in searched_domains
        assert "variant_searcher" in searched_domains

    @pytest.mark.asyncio
    async def test_cross_domain_disease_search(self, monkeypatch):
        """Test that disease searches include disease domain."""
        # Mock multiple domain searches
        searched_domains = []

        async def mock_execute_routing_plan(plan, output_json=True):
            searched_domains.extend(plan.tools_to_call)
            return {
                "articles": json.dumps([]),
                "trials": json.dumps([]),
                "diseases": json.dumps([]),
            }

        monkeypatch.setattr(
            "biomcp.router.execute_routing_plan", mock_execute_routing_plan
        )

        # Test cross-domain disease search
        await search(query="disease:melanoma")

        assert "disease_searcher" in searched_domains
        assert "article_searcher" in searched_domains
        assert "trial_searcher" in searched_domains

    @pytest.mark.asyncio
    async def test_domain_specific_query(self, monkeypatch):
        """Test domain-specific query language."""
        # Mock execute routing plan
        searched_domains = []

        async def mock_execute_routing_plan(plan, output_json=True):
            searched_domains.extend(plan.tools_to_call)
            return {"genes": json.dumps([])}

        monkeypatch.setattr(
            "biomcp.router.execute_routing_plan", mock_execute_routing_plan
        )

        # Test gene-specific search
        await search(query="genes.symbol:BRAF")

        assert "gene_searcher" in searched_domains
        assert len(searched_domains) == 1  # Only gene domain searched


class TestBioThingsErrorCases:
    """Test error handling for BioThings integration."""

    @pytest.mark.asyncio
    async def test_gene_api_failure(self, monkeypatch):
        """Test handling of API failures for gene search."""

        class MockBioThingsClient:
            async def _query_gene(self, query):
                raise Exception("API connection failed")

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        # Test that search handles the error gracefully
        with pytest.raises(Exception) as exc_info:
            await search(query="", domain="gene", keywords=["BRAF"])

        assert "API connection failed" in str(exc_info.value)

    @pytest.mark.asyncio
    async def test_drug_not_found(self, monkeypatch):
        """Test handling when drug is not found."""

        class MockBioThingsClient:
            async def _query_drug(self, query):
                return []  # No results

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        results = await search(
            query="", domain="drug", keywords=["nonexistent"]
        )
        assert "results" in results
        actual_results = [
            r for r in results["results"] if r["id"] != "thinking-reminder"
        ]
        assert len(actual_results) == 0

    @pytest.mark.asyncio
    async def test_disease_invalid_id(self, monkeypatch):
        """Test handling of invalid disease ID in fetch."""

        class MockBioThingsClient:
            async def get_disease_info(self, disease_id):
                return None  # Not found

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        result = await fetch(id="INVALID:12345", domain="disease")
        assert "error" in result
        assert "not found" in result["error"].lower()

    @pytest.mark.asyncio
    async def test_gene_partial_data(self, monkeypatch):
        """Test handling of incomplete gene data."""
        mock_gene_query = [{"_id": "673"}]  # Missing symbol
        mock_gene_details = {
            "_id": "673",
            # Missing symbol, name, summary
            "entrezgene": 673,
        }

        class MockBioThingsClient:
            async def _query_gene(self, query):
                return mock_gene_query

            async def _get_gene_by_id(self, gene_id):
                from biomcp.integrations.biothings_client import GeneInfo

                return GeneInfo(**mock_gene_details)

        monkeypatch.setattr(
            "biomcp.router.BioThingsClient", MockBioThingsClient
        )

        results = await search(query="", domain="gene", keywords=["673"])
        assert "results" in results
        actual_results = [
            r for r in results["results"] if r["id"] != "thinking-reminder"
        ]
        assert len(actual_results) == 1
        # Should handle missing data gracefully
        assert actual_results[0]["id"] == "673"
