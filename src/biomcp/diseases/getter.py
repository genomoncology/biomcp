"""Disease information retrieval from MyDisease.info."""

import json
import logging
from typing import Annotated

from pydantic import Field

from ..integrations import BioThingsClient
from ..render import to_markdown

logger = logging.getLogger(__name__)


def _add_disease_links(disease_info, result: dict) -> None:
    """Add helpful links to disease result."""
    links = {}

    # Add MONDO browser link if available
    if (
        disease_info.mondo
        and isinstance(disease_info.mondo, dict)
        and (mondo_id := disease_info.mondo.get("mondo"))
        and isinstance(mondo_id, str)
        and mondo_id.startswith("MONDO:")
    ):
        links["MONDO Browser"] = (
            f"https://www.ebi.ac.uk/ols/ontologies/mondo/terms?iri=http://purl.obolibrary.org/obo/{mondo_id.replace(':', '_')}"
        )

    # Add Disease Ontology link if available
    if (
        disease_info.xrefs
        and isinstance(disease_info.xrefs, dict)
        and (doid := disease_info.xrefs.get("doid"))
    ):
        if isinstance(doid, list) and doid:
            doid_id = doid[0] if isinstance(doid[0], str) else str(doid[0])
            links["Disease Ontology"] = (
                f"https://www.disease-ontology.org/?id={doid_id}"
            )
        elif isinstance(doid, str):
            links["Disease Ontology"] = (
                f"https://www.disease-ontology.org/?id={doid}"
            )

    # Add PubMed search link
    if disease_info.name:
        links["PubMed Search"] = (
            f"https://pubmed.ncbi.nlm.nih.gov/?term={disease_info.name.replace(' ', '+')}"
        )

    if links:
        result["_links"] = links


def _format_disease_output(disease_info, result: dict) -> None:
    """Format disease output for display."""
    # Format synonyms nicely
    if disease_info.synonyms:
        result["synonyms"] = ", ".join(
            disease_info.synonyms[:10]
        )  # Limit to first 10
        if len(disease_info.synonyms) > 10:
            result["synonyms"] += (
                f" (and {len(disease_info.synonyms) - 10} more)"
            )

    # Format phenotypes if present
    if disease_info.phenotypes:
        # Just show count and first few phenotypes
        phenotype_names = []
        for pheno in disease_info.phenotypes[:5]:
            if isinstance(pheno, dict) and "phenotype" in pheno:
                phenotype_names.append(pheno["phenotype"])
        if phenotype_names:
            result["associated_phenotypes"] = ", ".join(phenotype_names)
            if len(disease_info.phenotypes) > 5:
                result["associated_phenotypes"] += (
                    f" (and {len(disease_info.phenotypes) - 5} more)"
                )
        # Remove the raw phenotypes data for cleaner output
        result.pop("phenotypes", None)


async def get_disease(
    disease_id_or_name: str,
    output_json: bool = False,
) -> str:
    """
    Get disease information from MyDisease.info.

    Args:
        disease_id_or_name: Disease ID (MONDO, DOID) or name (e.g., "melanoma", "MONDO:0016575")
        output_json: Return as JSON instead of markdown

    Returns:
        Disease information as markdown or JSON string
    """
    client = BioThingsClient()

    try:
        disease_info = await client.get_disease_info(disease_id_or_name)

        if not disease_info:
            error_data = {
                "error": f"Disease '{disease_id_or_name}' not found",
                "suggestion": "Please check the disease name or ID (MONDO:, DOID:, OMIM:, MESH:)",
            }
            return (
                json.dumps(error_data, indent=2)
                if output_json
                else to_markdown([error_data])
            )

        # Convert to dict for rendering
        result = disease_info.model_dump(exclude_none=True)

        # Add helpful links
        _add_disease_links(disease_info, result)

        # Format output for display
        _format_disease_output(disease_info, result)

        if output_json:
            return json.dumps(result, indent=2)
        else:
            return to_markdown([result])

    except Exception as e:
        logger.error(
            f"Error fetching disease info for {disease_id_or_name}: {e}"
        )
        error_data = {
            "error": "Failed to retrieve disease information",
            "details": str(e),
        }
        return (
            json.dumps(error_data, indent=2)
            if output_json
            else to_markdown([error_data])
        )


async def _disease_details(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    disease_id_or_name: Annotated[
        str,
        Field(
            description="Disease name (e.g., melanoma, GIST) or ID (e.g., MONDO:0016575, DOID:1909)"
        ),
    ],
) -> str:
    """
    Retrieves detailed information for a disease from MyDisease.info.

    This tool provides real-time disease annotations including:
    - Official disease name and definition
    - Disease synonyms and alternative names
    - Ontology mappings (MONDO, DOID, OMIM, etc.)
    - Associated phenotypes
    - Links to disease databases

    Parameters:
    - call_benefit: Define why this function is being called
    - disease_id_or_name: Disease name or ontology ID

    Process: Queries MyDisease.info API for up-to-date disease information
    Output: Markdown formatted disease information with definition and metadata

    Note: For clinical trials about diseases, use trial_searcher. For articles about diseases, use article_searcher.
    """
    return await get_disease(disease_id_or_name, output_json=False)
