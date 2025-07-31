"""BioThings API client for unified access to the BioThings suite.

The BioThings suite (https://biothings.io) provides high-performance biomedical
data APIs including:
- MyGene.info - Gene annotations and information
- MyVariant.info - Genetic variant annotations (existing integration enhanced)
- MyDisease.info - Disease ontology and synonyms
- MyChem.info - Drug/chemical annotations and information

This module provides a centralized client for interacting with all BioThings APIs,
handling common concerns like error handling, rate limiting, and response parsing.
While MyVariant.info has specialized modules for complex variant operations, this
client provides the base layer for all BioThings API interactions.
"""

import logging
from typing import Any
from urllib.parse import quote

from pydantic import BaseModel, Field

from .. import http_client
from ..constants import (
    MYVARIANT_GET_URL,
)

logger = logging.getLogger(__name__)

# BioThings API endpoints
MYGENE_BASE_URL = "https://mygene.info/v3"
MYGENE_QUERY_URL = f"{MYGENE_BASE_URL}/query"
MYGENE_GET_URL = f"{MYGENE_BASE_URL}/gene"

MYDISEASE_BASE_URL = "https://mydisease.info/v1"
MYDISEASE_QUERY_URL = f"{MYDISEASE_BASE_URL}/query"
MYDISEASE_GET_URL = f"{MYDISEASE_BASE_URL}/disease"

MYCHEM_BASE_URL = "https://mychem.info/v1"
MYCHEM_QUERY_URL = f"{MYCHEM_BASE_URL}/query"
MYCHEM_GET_URL = f"{MYCHEM_BASE_URL}/chem"


class GeneInfo(BaseModel):
    """Gene information from MyGene.info."""

    gene_id: str = Field(alias="_id")
    symbol: str | None = None
    name: str | None = None
    summary: str | None = None
    alias: list[str] | None = Field(default_factory=list)
    entrezgene: int | str | None = None
    ensembl: dict[str, Any] | None = None
    refseq: dict[str, Any] | None = None
    type_of_gene: str | None = None
    taxid: int | None = None


class DiseaseInfo(BaseModel):
    """Disease information from MyDisease.info."""

    disease_id: str = Field(alias="_id")
    name: str | None = None
    mondo: dict[str, Any] | None = None
    definition: str | None = None
    synonyms: list[str] | None = Field(default_factory=list)
    xrefs: dict[str, Any] | None = None
    phenotypes: list[dict[str, Any]] | None = None


class DrugInfo(BaseModel):
    """Drug/chemical information from MyChem.info."""

    drug_id: str = Field(alias="_id")
    name: str | None = None
    tradename: list[str] | None = Field(default_factory=list)
    drugbank_id: str | None = None
    chebi_id: str | None = None
    chembl_id: str | None = None
    pubchem_cid: str | None = None
    unii: str | dict[str, Any] | None = None
    inchikey: str | None = None
    formula: str | None = None
    description: str | None = None
    indication: str | None = None
    pharmacology: dict[str, Any] | None = None
    mechanism_of_action: str | None = None


class BioThingsClient:
    """Unified client for BioThings APIs (MyGene, MyVariant, MyDisease, MyChem)."""

    def __init__(self):
        """Initialize the BioThings client."""
        self.logger = logger

    async def get_gene_info(
        self, gene_id_or_symbol: str, fields: list[str] | None = None
    ) -> GeneInfo | None:
        """Get gene information from MyGene.info.

        Args:
            gene_id_or_symbol: Gene ID (Entrez, Ensembl) or symbol (e.g., "TP53")
            fields: Optional list of fields to return

        Returns:
            GeneInfo object or None if not found
        """
        try:
            # First, try direct GET (works for Entrez IDs)
            if gene_id_or_symbol.isdigit():
                return await self._get_gene_by_id(gene_id_or_symbol, fields)

            # For symbols, we need to query first
            query_result = await self._query_gene(gene_id_or_symbol)
            if not query_result:
                return None

            # Get the best match
            gene_id = query_result[0].get("_id")
            if not gene_id:
                return None

            # Now get full details
            return await self._get_gene_by_id(gene_id, fields)

        except Exception as e:
            self.logger.warning(
                f"Failed to get gene info for {gene_id_or_symbol}: {e}"
            )
            return None

    async def _query_gene(self, symbol: str) -> list[dict[str, Any]] | None:
        """Query MyGene.info for a gene symbol."""
        params = {
            "q": f"symbol:{quote(symbol)}",
            "species": "human",
            "fields": "_id,symbol,name,taxid",
            "size": 5,
        }

        response, error = await http_client.request_api(
            url=MYGENE_QUERY_URL,
            request=params,
            method="GET",
            domain="mygene",
        )

        if error or not response:
            return None

        hits = response.get("hits", [])
        # Filter for human genes (taxid 9606)
        human_hits = [h for h in hits if h.get("taxid") == 9606]
        return human_hits if human_hits else hits

    async def _get_gene_by_id(
        self, gene_id: str, fields: list[str] | None = None
    ) -> GeneInfo | None:
        """Get gene details by ID from MyGene.info."""
        if fields is None:
            fields = [
                "symbol",
                "name",
                "summary",
                "alias",
                "type_of_gene",
                "ensembl",
                "refseq",
                "entrezgene",
            ]

        params = {"fields": ",".join(fields)}

        response, error = await http_client.request_api(
            url=f"{MYGENE_GET_URL}/{gene_id}",
            request=params,
            method="GET",
            domain="mygene",
        )

        if error or not response:
            return None

        try:
            return GeneInfo(**response)
        except Exception as e:
            self.logger.warning(f"Failed to parse gene response: {e}")
            return None

    async def batch_get_genes(
        self, gene_ids: list[str], fields: list[str] | None = None
    ) -> list[GeneInfo]:
        """Get multiple genes in a single request.

        Args:
            gene_ids: List of gene IDs or symbols
            fields: Optional list of fields to return

        Returns:
            List of GeneInfo objects
        """
        if not gene_ids:
            return []

        if fields is None:
            fields = ["symbol", "name", "summary", "alias", "type_of_gene"]

        # MyGene supports POST for batch queries
        data = {
            "ids": ",".join(gene_ids),
            "fields": ",".join(fields),
            "species": "human",
        }

        response, error = await http_client.request_api(
            url=MYGENE_GET_URL,
            request=data,
            method="POST",
            domain="mygene",
        )

        if error or not response:
            return []

        results = []
        for item in response:
            try:
                if "notfound" not in item:
                    results.append(GeneInfo(**item))
            except Exception as e:
                self.logger.warning(f"Failed to parse gene in batch: {e}")
                continue

        return results

    async def get_disease_info(
        self, disease_id_or_name: str, fields: list[str] | None = None
    ) -> DiseaseInfo | None:
        """Get disease information from MyDisease.info.

        Args:
            disease_id_or_name: Disease ID (MONDO, DOID) or name
            fields: Optional list of fields to return

        Returns:
            DiseaseInfo object or None if not found
        """
        try:
            # Check if it's an ID (starts with known prefixes)
            if any(
                disease_id_or_name.upper().startswith(prefix)
                for prefix in ["MONDO:", "DOID:", "OMIM:", "MESH:"]
            ):
                return await self._get_disease_by_id(
                    disease_id_or_name, fields
                )

            # Otherwise, query by name
            query_result = await self._query_disease(disease_id_or_name)
            if not query_result:
                return None

            # Get the best match
            disease_id = query_result[0].get("_id")
            if not disease_id:
                return None

            # Now get full details
            return await self._get_disease_by_id(disease_id, fields)

        except Exception as e:
            self.logger.warning(
                f"Failed to get disease info for {disease_id_or_name}: {e}"
            )
            return None

    async def _query_disease(self, name: str) -> list[dict[str, Any]] | None:
        """Query MyDisease.info for a disease name."""
        params = {
            "q": quote(name),
            "fields": "_id,name,mondo",
            "size": 10,
        }

        response, error = await http_client.request_api(
            url=MYDISEASE_QUERY_URL,
            request=params,
            method="GET",
            domain="mydisease",
        )

        if error or not response:
            return None

        return response.get("hits", [])

    async def _get_disease_by_id(
        self, disease_id: str, fields: list[str] | None = None
    ) -> DiseaseInfo | None:
        """Get disease details by ID from MyDisease.info."""
        if fields is None:
            fields = [
                "name",
                "mondo",
                "definition",
                "synonyms",
                "xrefs",
                "phenotypes",
            ]

        params = {"fields": ",".join(fields)}

        response, error = await http_client.request_api(
            url=f"{MYDISEASE_GET_URL}/{quote(disease_id, safe='')}",
            request=params,
            method="GET",
            domain="mydisease",
        )

        if error or not response:
            return None

        try:
            # Extract definition from mondo if available
            if "mondo" in response and isinstance(response["mondo"], dict):
                if (
                    "definition" in response["mondo"]
                    and "definition" not in response
                ):
                    response["definition"] = response["mondo"]["definition"]
                # Extract synonyms from mondo
                if "synonym" in response["mondo"]:
                    mondo_synonyms = response["mondo"]["synonym"]
                    if isinstance(mondo_synonyms, dict):
                        # Handle exact synonyms
                        exact = mondo_synonyms.get("exact", [])
                        if isinstance(exact, list):
                            response["synonyms"] = exact
                    elif isinstance(mondo_synonyms, list):
                        response["synonyms"] = mondo_synonyms

            return DiseaseInfo(**response)
        except Exception as e:
            self.logger.warning(f"Failed to parse disease response: {e}")
            return None

    async def get_disease_synonyms(self, disease_id_or_name: str) -> list[str]:
        """Get disease synonyms for query expansion.

        Args:
            disease_id_or_name: Disease ID or name

        Returns:
            List of synonyms including the original term
        """
        disease_info = await self.get_disease_info(disease_id_or_name)
        if not disease_info:
            return [disease_id_or_name]

        synonyms = [disease_id_or_name]
        if disease_info.name and disease_info.name != disease_id_or_name:
            synonyms.append(disease_info.name)

        if disease_info.synonyms:
            synonyms.extend(disease_info.synonyms)

        # Remove duplicates while preserving order
        seen = set()
        unique_synonyms = []
        for syn in synonyms:
            if syn.lower() not in seen:
                seen.add(syn.lower())
                unique_synonyms.append(syn)

        return unique_synonyms[
            :5
        ]  # Limit to top 5 to avoid overly broad searches

    async def get_drug_info(
        self, drug_id_or_name: str, fields: list[str] | None = None
    ) -> DrugInfo | None:
        """Get drug/chemical information from MyChem.info.

        Args:
            drug_id_or_name: Drug ID (DrugBank, ChEMBL, etc.) or name
            fields: Optional list of fields to return

        Returns:
            DrugInfo object or None if not found
        """
        try:
            # Check if it's an ID (starts with known prefixes)
            if any(
                drug_id_or_name.upper().startswith(prefix)
                for prefix in ["DRUGBANK:", "DB", "CHEMBL", "CHEBI:", "CID"]
            ):
                return await self._get_drug_by_id(drug_id_or_name, fields)

            # Otherwise, query by name
            query_result = await self._query_drug(drug_id_or_name)
            if not query_result:
                return None

            # Get the best match
            drug_id = query_result[0].get("_id")
            if not drug_id:
                return None

            # Now get full details
            return await self._get_drug_by_id(drug_id, fields)

        except Exception as e:
            self.logger.warning(
                f"Failed to get drug info for {drug_id_or_name}: {e}"
            )
            return None

    async def _query_drug(self, name: str) -> list[dict[str, Any]] | None:
        """Query MyChem.info for a drug name."""
        params = {
            "q": quote(name),
            "fields": "_id,name,drugbank.name,chebi.name,chembl.pref_name,unii.display_name",
            "size": 10,
        }

        response, error = await http_client.request_api(
            url=MYCHEM_QUERY_URL,
            request=params,
            method="GET",
            domain="mychem",
        )

        if error or not response:
            return None

        hits = response.get("hits", [])

        # Sort hits to prioritize those with actual drug names
        def score_hit(hit):
            score = hit.get("_score", 0)
            # Boost score if hit has drug name fields
            if hit.get("drugbank", {}).get("name"):
                score += 10
            if hit.get("chembl", {}).get("pref_name"):
                score += 5
            if hit.get("unii", {}).get("display_name"):
                score += 3
            return score

        hits.sort(key=score_hit, reverse=True)
        return hits

    async def _get_drug_by_id(
        self, drug_id: str, fields: list[str] | None = None
    ) -> DrugInfo | None:
        """Get drug details by ID from MyChem.info."""
        if fields is None:
            fields = [
                "name",
                "drugbank",
                "chebi",
                "chembl",
                "pubchem",
                "unii",
                "inchikey",
                "formula",
                "description",
                "indication",
                "pharmacology",
                "mechanism_of_action",
            ]

        params = {"fields": ",".join(fields)}

        response, error = await http_client.request_api(
            url=f"{MYCHEM_GET_URL}/{quote(drug_id, safe='')}",
            request=params,
            method="GET",
            domain="mychem",
        )

        if error or not response:
            return None

        try:
            # Handle array response (multiple results)
            if isinstance(response, list):
                if not response:
                    return None
                # Take the first result
                response = response[0]

            # Extract fields from nested structures
            self._extract_drugbank_fields(response)
            self._extract_chebi_fields(response)
            self._extract_chembl_fields(response)
            self._extract_pubchem_fields(response)
            self._extract_unii_fields(response)

            return DrugInfo(**response)
        except Exception as e:
            self.logger.warning(f"Failed to parse drug response: {e}")
            return None

    def _extract_drugbank_fields(self, response: dict[str, Any]) -> None:
        """Extract DrugBank fields from response."""
        if "drugbank" in response and isinstance(response["drugbank"], dict):
            db = response["drugbank"]
            response["drugbank_id"] = db.get("id")
            response["name"] = response.get("name") or db.get("name")
            response["tradename"] = db.get("products", {}).get("name", [])
            if isinstance(response["tradename"], str):
                response["tradename"] = [response["tradename"]]
            response["indication"] = db.get("indication")
            response["mechanism_of_action"] = db.get("mechanism_of_action")
            response["description"] = db.get("description")

    def _extract_chebi_fields(self, response: dict[str, Any]) -> None:
        """Extract ChEBI fields from response."""
        if "chebi" in response and isinstance(response["chebi"], dict):
            response["chebi_id"] = response["chebi"].get("id")
            if not response.get("name"):
                response["name"] = response["chebi"].get("name")

    def _extract_chembl_fields(self, response: dict[str, Any]) -> None:
        """Extract ChEMBL fields from response."""
        if "chembl" in response and isinstance(response["chembl"], dict):
            response["chembl_id"] = response["chembl"].get(
                "molecule_chembl_id"
            )
            if not response.get("name"):
                response["name"] = response["chembl"].get("pref_name")

    def _extract_pubchem_fields(self, response: dict[str, Any]) -> None:
        """Extract PubChem fields from response."""
        if "pubchem" in response and isinstance(response["pubchem"], dict):
            response["pubchem_cid"] = str(response["pubchem"].get("cid", ""))

    def _extract_unii_fields(self, response: dict[str, Any]) -> None:
        """Extract UNII fields from response."""
        if "unii" in response and isinstance(response["unii"], dict):
            unii_data = response["unii"]
            # Set UNII code
            response["unii"] = unii_data.get("unii", "")
            # Use display name as drug name if not already set
            if not response.get("name") and unii_data.get("display_name"):
                response["name"] = unii_data["display_name"]
            # Use NCIT description if no description
            if not response.get("description") and unii_data.get(
                "ncit_description"
            ):
                response["description"] = unii_data["ncit_description"]

    async def get_variant_info(
        self, variant_id: str, fields: list[str] | None = None
    ) -> dict[str, Any] | None:
        """Get variant information from MyVariant.info.

        This is a wrapper around the existing MyVariant integration.

        Args:
            variant_id: Variant ID (rsID, HGVS)
            fields: Optional list of fields to return

        Returns:
            Variant data dictionary or None if not found
        """
        params = {"fields": "all" if fields is None else ",".join(fields)}

        response, error = await http_client.request_api(
            url=f"{MYVARIANT_GET_URL}/{variant_id}",
            request=params,
            method="GET",
            domain="myvariant",
        )

        if error or not response:
            return None

        return response
