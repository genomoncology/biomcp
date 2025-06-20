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


def get_domain_handler(
    domain: str,
) -> type[ArticleHandler] | type[TrialHandler] | type[VariantHandler]:
    """Get the appropriate handler class for a domain.

    Args:
        domain: The domain name ('article', 'trial', or 'variant')

    Returns:
        The handler class for the domain

    Raises:
        ValueError: If domain is not recognized
    """
    handlers: dict[
        str, type[ArticleHandler] | type[TrialHandler] | type[VariantHandler]
    ] = {
        "article": ArticleHandler,
        "trial": TrialHandler,
        "variant": VariantHandler,
    }

    handler = handlers.get(domain)
    if handler is None:
        raise ValueError(f"Unknown domain: {domain}")

    return handler
