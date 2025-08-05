"""Domain-specific result handlers for BioMCP.

This module contains formatting functions for converting raw API responses
from different biomedical data sources into a standardized format.
"""

import logging
from typing import Any

from biomcp.constants import (
    DEFAULT_SIGNIFICANCE,
    DEFAULT_TITLE,
    METADATA_AUTHORS,
    METADATA_COMPLETION_DATE,
    METADATA_CONSEQUENCE,
    METADATA_GENE,
    METADATA_JOURNAL,
    METADATA_PHASE,
    METADATA_RSID,
    METADATA_SIGNIFICANCE,
    METADATA_SOURCE,
    METADATA_START_DATE,
    METADATA_STATUS,
    METADATA_YEAR,
    RESULT_ID,
    RESULT_METADATA,
    RESULT_SNIPPET,
    RESULT_TITLE,
    RESULT_URL,
    SNIPPET_LENGTH,
)

logger = logging.getLogger(__name__)


class ArticleHandler:
    """Handles formatting for article/publication results."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single article result.

        Args:
            result: Raw article data from PubTator3 or preprint APIs

        Returns:
            Standardized article result with id, title, snippet, url, and metadata
        """
        if "pmid" in result:
            # PubMed article
            # Clean up title - remove extra spaces
            title = result.get("title", "").strip()
            title = " ".join(title.split())  # Normalize whitespace

            # Use default if empty
            if not title:
                title = DEFAULT_TITLE

            return {
                RESULT_ID: result["pmid"],
                RESULT_TITLE: title,
                RESULT_SNIPPET: result.get("abstract", "")[:SNIPPET_LENGTH]
                + "..."
                if result.get("abstract")
                else "",
                RESULT_URL: f"https://pubmed.ncbi.nlm.nih.gov/{result['pmid']}/",
                RESULT_METADATA: {
                    METADATA_YEAR: result.get("pub_year")
                    or (
                        result.get("date", "")[:4]
                        if result.get("date")
                        else None
                    ),
                    METADATA_JOURNAL: result.get("journal", ""),
                    METADATA_AUTHORS: result.get("authors", [])[:3],
                },
            }
        else:
            # Preprint result
            return {
                RESULT_ID: result.get("doi", result.get("id", "")),
                RESULT_TITLE: result.get("title", ""),
                RESULT_SNIPPET: result.get("abstract", "")[:SNIPPET_LENGTH]
                + "..."
                if result.get("abstract")
                else "",
                RESULT_URL: result.get("url", ""),
                RESULT_METADATA: {
                    METADATA_YEAR: result.get("pub_year"),
                    METADATA_SOURCE: result.get("source", ""),
                    METADATA_AUTHORS: result.get("authors", [])[:3],
                },
            }


class TrialHandler:
    """Handles formatting for clinical trial results."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single trial result.

        Handles both ClinicalTrials.gov API v2 nested structure and legacy formats.

        Args:
            result: Raw trial data from ClinicalTrials.gov API

        Returns:
            Standardized trial result with id, title, snippet, url, and metadata
        """
        # Handle ClinicalTrials.gov API v2 nested structure
        if "protocolSection" in result:
            # API v2 format - extract from nested modules
            protocol = result.get("protocolSection", {})
            identification = protocol.get("identificationModule", {})
            status = protocol.get("statusModule", {})
            description = protocol.get("descriptionModule", {})

            nct_id = identification.get("nctId", "")
            brief_title = identification.get("briefTitle", "")
            official_title = identification.get("officialTitle", "")
            brief_summary = description.get("briefSummary", "")
            overall_status = status.get("overallStatus", "")
            start_date = status.get("startDateStruct", {}).get("date", "")
            completion_date = status.get(
                "primaryCompletionDateStruct", {}
            ).get("date", "")

            # Extract phase from designModule
            design = protocol.get("designModule", {})
            phases = design.get("phases", [])
            phase = phases[0] if phases else ""
        elif "NCT Number" in result:
            # Legacy flat format from search results
            nct_id = result.get("NCT Number", "")
            brief_title = result.get("Study Title", "")
            official_title = ""  # Not available in this format
            brief_summary = result.get("Brief Summary", "")
            overall_status = result.get("Study Status", "")
            phase = result.get("Phases", "")
            start_date = result.get("Start Date", "")
            completion_date = result.get("Completion Date", "")
        else:
            # Original legacy format or simplified structure
            nct_id = result.get("nct_id", "")
            brief_title = result.get("brief_title", "")
            official_title = result.get("official_title", "")
            brief_summary = result.get("brief_summary", "")
            overall_status = result.get("overall_status", "")
            phase = result.get("phase", "")
            start_date = result.get("start_date", "")
            completion_date = result.get("primary_completion_date", "")

        return {
            RESULT_ID: nct_id,
            RESULT_TITLE: brief_title or official_title or DEFAULT_TITLE,
            RESULT_SNIPPET: brief_summary[:SNIPPET_LENGTH] + "..."
            if brief_summary
            else "",
            RESULT_URL: f"https://clinicaltrials.gov/study/{nct_id}",
            RESULT_METADATA: {
                METADATA_STATUS: overall_status,
                METADATA_PHASE: phase,
                METADATA_START_DATE: start_date,
                METADATA_COMPLETION_DATE: completion_date,
            },
        }


class VariantHandler:
    """Handles formatting for genetic variant results."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single variant result.

        Args:
            result: Raw variant data from MyVariant.info API

        Returns:
            Standardized variant result with id, title, snippet, url, and metadata
        """
        # Extract gene symbol - MyVariant.info stores this in multiple locations
        gene = (
            result.get("dbnsfp", {}).get("genename", "")
            or result.get("dbsnp", {}).get("gene", {}).get("symbol", "")
            or ""
        )
        # Handle case where gene is a list
        if isinstance(gene, list):
            gene = gene[0] if gene else ""

        # Extract rsid
        rsid = result.get("dbsnp", {}).get("rsid", "") or ""

        # Extract clinical significance
        clinvar = result.get("clinvar", {})
        significance = ""
        if isinstance(clinvar.get("rcv"), dict):
            significance = clinvar["rcv"].get("clinical_significance", "")
        elif isinstance(clinvar.get("rcv"), list) and clinvar["rcv"]:
            significance = clinvar["rcv"][0].get("clinical_significance", "")

        # Build a meaningful title
        hgvs = ""
        if "dbnsfp" in result and "hgvsp" in result["dbnsfp"]:
            hgvs = result["dbnsfp"]["hgvsp"]
            if isinstance(hgvs, list):
                hgvs = hgvs[0] if hgvs else ""

        title = f"{gene} {hgvs}".strip() or result.get("_id", DEFAULT_TITLE)

        return {
            RESULT_ID: result.get("_id", ""),
            RESULT_TITLE: title,
            RESULT_SNIPPET: f"Clinical significance: {significance or DEFAULT_SIGNIFICANCE}",
            RESULT_URL: f"https://www.ncbi.nlm.nih.gov/snp/{rsid}"
            if rsid
            else "",
            RESULT_METADATA: {
                METADATA_GENE: gene,
                METADATA_RSID: rsid,
                METADATA_SIGNIFICANCE: significance,
                METADATA_CONSEQUENCE: result.get("cadd", {}).get(
                    "consequence", ""
                ),
            },
        }


class GeneHandler:
    """Handles formatting for gene information results from MyGene.info."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single gene result.

        Args:
            result: Raw gene data from MyGene.info API

        Returns:
            Standardized gene result with id, title, snippet, url, and metadata
        """
        # Extract gene information
        gene_id = result.get("_id", result.get("entrezgene", ""))
        symbol = result.get("symbol", "")
        name = result.get("name", "")
        summary = result.get("summary", "")

        # Build title
        title = (
            f"{symbol}: {name}"
            if symbol and name
            else symbol or name or DEFAULT_TITLE
        )

        # Create snippet from summary
        snippet = (
            summary[:SNIPPET_LENGTH] + "..."
            if summary and len(summary) > SNIPPET_LENGTH
            else summary
        )

        return {
            RESULT_ID: str(gene_id),
            RESULT_TITLE: title,
            RESULT_SNIPPET: snippet or "No summary available",
            RESULT_URL: f"https://www.genenames.org/data/gene-symbol-report/#!/symbol/{symbol}"
            if symbol
            else "",
            RESULT_METADATA: {
                "entrezgene": result.get("entrezgene"),
                "symbol": symbol,
                "name": name,
                "type_of_gene": result.get("type_of_gene", ""),
                "ensembl": result.get("ensembl", {}).get("gene")
                if isinstance(result.get("ensembl"), dict)
                else None,
                "refseq": result.get("refseq", {}),
            },
        }


class DrugHandler:
    """Handles formatting for drug/chemical information results from MyChem.info."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single drug result.

        Args:
            result: Raw drug data from MyChem.info API

        Returns:
            Standardized drug result with id, title, snippet, url, and metadata
        """
        # Extract drug information
        drug_id = result.get("_id", "")
        name = result.get("name", "")
        drugbank_id = result.get("drugbank_id", "")
        description = result.get("description", "")
        indication = result.get("indication", "")

        # Build title
        title = name or drug_id or DEFAULT_TITLE

        # Create snippet from description or indication
        snippet_text = indication or description
        snippet = (
            snippet_text[:SNIPPET_LENGTH] + "..."
            if snippet_text and len(snippet_text) > SNIPPET_LENGTH
            else snippet_text
        )

        # Determine URL based on available IDs
        url = ""
        if drugbank_id:
            url = f"https://www.drugbank.ca/drugs/{drugbank_id}"
        elif result.get("pubchem_cid"):
            url = f"https://pubchem.ncbi.nlm.nih.gov/compound/{result['pubchem_cid']}"

        return {
            RESULT_ID: drug_id,
            RESULT_TITLE: title,
            RESULT_SNIPPET: snippet or "No description available",
            RESULT_URL: url,
            RESULT_METADATA: {
                "drugbank_id": drugbank_id,
                "chembl_id": result.get("chembl_id", ""),
                "pubchem_cid": result.get("pubchem_cid", ""),
                "chebi_id": result.get("chebi_id", ""),
                "formula": result.get("formula", ""),
                "tradename": result.get("tradename", []),
            },
        }


class DiseaseHandler:
    """Handles formatting for disease information results from MyDisease.info."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single disease result.

        Args:
            result: Raw disease data from MyDisease.info API

        Returns:
            Standardized disease result with id, title, snippet, url, and metadata
        """
        # Extract disease information
        disease_id = result.get("_id", "")
        name = result.get("name", "")
        definition = result.get("definition", "")
        mondo_info = result.get("mondo", {})

        # Build title
        title = name or disease_id or DEFAULT_TITLE

        # Create snippet from definition
        snippet = (
            definition[:SNIPPET_LENGTH] + "..."
            if definition and len(definition) > SNIPPET_LENGTH
            else definition
        )

        # Extract MONDO ID for URL
        mondo_id = mondo_info.get("id") if isinstance(mondo_info, dict) else ""
        url = (
            f"https://monarchinitiative.org/disease/{mondo_id}"
            if mondo_id
            else ""
        )

        return {
            RESULT_ID: disease_id,
            RESULT_TITLE: title,
            RESULT_SNIPPET: snippet or "No definition available",
            RESULT_URL: url,
            RESULT_METADATA: {
                "mondo_id": mondo_id,
                "definition": definition,
                "synonyms": result.get("synonyms", []),
                "xrefs": result.get("xrefs", {}),
                "phenotypes": len(result.get("phenotypes", [])),
            },
        }


class NCIOrganizationHandler:
    """Handles formatting for NCI organization results."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single NCI organization result.

        Args:
            result: Raw organization data from NCI CTS API

        Returns:
            Standardized organization result with id, title, snippet, url, and metadata
        """
        org_id = result.get("id", result.get("org_id", ""))
        name = result.get("name", "Unknown Organization")
        org_type = result.get("type", result.get("category", ""))
        city = result.get("city", "")
        state = result.get("state", "")

        # Build location string
        location_parts = [p for p in [city, state] if p]
        location = ", ".join(location_parts) if location_parts else ""

        # Create snippet
        snippet_parts = []
        if org_type:
            snippet_parts.append(f"Type: {org_type}")
        if location:
            snippet_parts.append(f"Location: {location}")
        snippet = " | ".join(snippet_parts) or "No details available"

        return {
            RESULT_ID: org_id,
            RESULT_TITLE: name,
            RESULT_SNIPPET: snippet,
            RESULT_URL: "",  # NCI doesn't provide direct URLs to organizations
            RESULT_METADATA: {
                "type": org_type,
                "city": city,
                "state": state,
                "country": result.get("country", ""),
            },
        }


class NCIInterventionHandler:
    """Handles formatting for NCI intervention results."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single NCI intervention result.

        Args:
            result: Raw intervention data from NCI CTS API

        Returns:
            Standardized intervention result with id, title, snippet, url, and metadata
        """
        int_id = result.get("id", result.get("intervention_id", ""))
        name = result.get("name", "Unknown Intervention")
        int_type = result.get("type", result.get("category", ""))
        synonyms = result.get("synonyms", [])

        # Create snippet
        snippet_parts = []
        if int_type:
            snippet_parts.append(f"Type: {int_type}")
        if synonyms:
            if isinstance(synonyms, list) and synonyms:
                snippet_parts.append(
                    f"Also known as: {', '.join(synonyms[:3])}"
                )
            elif isinstance(synonyms, str):
                snippet_parts.append(f"Also known as: {synonyms}")
        snippet = " | ".join(snippet_parts) or "No details available"

        return {
            RESULT_ID: int_id,
            RESULT_TITLE: name,
            RESULT_SNIPPET: snippet,
            RESULT_URL: "",  # NCI doesn't provide direct URLs to interventions
            RESULT_METADATA: {
                "type": int_type,
                "synonyms": synonyms,
                "description": result.get("description", ""),
            },
        }


class NCIBiomarkerHandler:
    """Handles formatting for NCI biomarker results."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single NCI biomarker result.

        Args:
            result: Raw biomarker data from NCI CTS API

        Returns:
            Standardized biomarker result with id, title, snippet, url, and metadata
        """
        bio_id = result.get("id", result.get("biomarker_id", ""))
        name = result.get("name", "Unknown Biomarker")
        gene = result.get("gene", result.get("gene_symbol", ""))
        bio_type = result.get("type", result.get("category", ""))
        assay_type = result.get("assay_type", "")

        # Build title
        title = name
        if gene and gene not in name:
            title = f"{gene} - {name}"

        # Create snippet
        snippet_parts = []
        if bio_type:
            snippet_parts.append(f"Type: {bio_type}")
        if assay_type:
            snippet_parts.append(f"Assay: {assay_type}")
        snippet = (
            " | ".join(snippet_parts) or "Biomarker for trial eligibility"
        )

        return {
            RESULT_ID: bio_id,
            RESULT_TITLE: title,
            RESULT_SNIPPET: snippet,
            RESULT_URL: "",  # NCI doesn't provide direct URLs to biomarkers
            RESULT_METADATA: {
                "gene": gene,
                "type": bio_type,
                "assay_type": assay_type,
                "trial_count": result.get("trial_count", 0),
            },
        }


class NCIDiseaseHandler:
    """Handles formatting for NCI disease vocabulary results."""

    @staticmethod
    def format_result(result: dict[str, Any]) -> dict[str, Any]:
        """Format a single NCI disease result.

        Args:
            result: Raw disease data from NCI CTS API

        Returns:
            Standardized disease result with id, title, snippet, url, and metadata
        """
        disease_id = result.get("id", result.get("disease_id", ""))
        name = result.get(
            "name", result.get("preferred_name", "Unknown Disease")
        )
        category = result.get("category", result.get("type", ""))
        synonyms = result.get("synonyms", [])

        # Create snippet
        snippet_parts = []
        if category:
            snippet_parts.append(f"Category: {category}")
        if synonyms:
            if isinstance(synonyms, list) and synonyms:
                snippet_parts.append(
                    f"Also known as: {', '.join(synonyms[:3])}"
                )
                if len(synonyms) > 3:
                    snippet_parts.append(f"and {len(synonyms) - 3} more")
            elif isinstance(synonyms, str):
                snippet_parts.append(f"Also known as: {synonyms}")
        snippet = " | ".join(snippet_parts) or "NCI cancer vocabulary term"

        return {
            RESULT_ID: disease_id,
            RESULT_TITLE: name,
            RESULT_SNIPPET: snippet,
            RESULT_URL: "",  # NCI doesn't provide direct URLs to disease terms
            RESULT_METADATA: {
                "category": category,
                "synonyms": synonyms,
                "codes": result.get("codes", {}),
            },
        }


def get_domain_handler(
    domain: str,
) -> (
    type[ArticleHandler]
    | type[TrialHandler]
    | type[VariantHandler]
    | type[GeneHandler]
    | type[DrugHandler]
    | type[DiseaseHandler]
    | type[NCIOrganizationHandler]
    | type[NCIInterventionHandler]
    | type[NCIBiomarkerHandler]
    | type[NCIDiseaseHandler]
):
    """Get the appropriate handler class for a domain.

    Args:
        domain: The domain name ('article', 'trial', 'variant', 'gene', 'drug', 'disease',
                               'nci_organization', 'nci_intervention', 'nci_biomarker', 'nci_disease')

    Returns:
        The handler class for the domain

    Raises:
        ValueError: If domain is not recognized
    """
    handlers: dict[
        str,
        type[ArticleHandler]
        | type[TrialHandler]
        | type[VariantHandler]
        | type[GeneHandler]
        | type[DrugHandler]
        | type[DiseaseHandler]
        | type[NCIOrganizationHandler]
        | type[NCIInterventionHandler]
        | type[NCIBiomarkerHandler]
        | type[NCIDiseaseHandler],
    ] = {
        "article": ArticleHandler,
        "trial": TrialHandler,
        "variant": VariantHandler,
        "gene": GeneHandler,
        "drug": DrugHandler,
        "disease": DiseaseHandler,
        "nci_organization": NCIOrganizationHandler,
        "nci_intervention": NCIInterventionHandler,
        "nci_biomarker": NCIBiomarkerHandler,
        "nci_disease": NCIDiseaseHandler,
    }

    handler = handlers.get(domain)
    if handler is None:
        raise ValueError(f"Unknown domain: {domain}")

    return handler
