"""Drug information retrieval from MyChem.info."""

import json
import logging

from ..integrations import BioThingsClient

logger = logging.getLogger(__name__)


def _add_drug_links(drug_info, result: dict) -> None:
    """Add external database links for the drug."""
    links = {}

    if drug_info.drugbank_id:
        links["DrugBank"] = (
            f"https://www.drugbank.ca/drugs/{drug_info.drugbank_id}"
        )

    if drug_info.chembl_id:
        links["ChEMBL"] = (
            f"https://www.ebi.ac.uk/chembl/compound_report_card/{drug_info.chembl_id}/"
        )

    if drug_info.pubchem_cid:
        links["PubChem"] = (
            f"https://pubchem.ncbi.nlm.nih.gov/compound/{drug_info.pubchem_cid}"
        )

    if drug_info.chebi_id:
        chebi_id = drug_info.chebi_id.replace("CHEBI:", "")
        links["ChEBI"] = (
            f"https://www.ebi.ac.uk/chebi/searchId.do?chebiId=CHEBI:{chebi_id}"
        )

    if links:
        result["_links"] = links


def _format_basic_info(drug_info, output_lines: list[str]) -> None:
    """Format basic drug information."""
    if drug_info.formula:
        output_lines.append(f"- **Formula**: {drug_info.formula}")

    if drug_info.drugbank_id:
        output_lines.append(f"- **DrugBank ID**: {drug_info.drugbank_id}")

    if drug_info.chembl_id:
        output_lines.append(f"- **ChEMBL ID**: {drug_info.chembl_id}")

    if drug_info.pubchem_cid:
        output_lines.append(f"- **PubChem CID**: {drug_info.pubchem_cid}")

    if drug_info.chebi_id:
        output_lines.append(f"- **ChEBI ID**: {drug_info.chebi_id}")

    if drug_info.inchikey:
        output_lines.append(f"- **InChIKey**: {drug_info.inchikey}")


def _format_clinical_info(drug_info, output_lines: list[str]) -> None:
    """Format clinical drug information."""
    if drug_info.tradename:
        names = drug_info.tradename[:5]  # Limit to first 5
        output_lines.append(f"- **Trade Names**: {', '.join(names)}")
        if len(drug_info.tradename) > 5:
            output_lines.append(f"  (and {len(drug_info.tradename) - 5} more)")

    if drug_info.description:
        desc = drug_info.description[:500]
        if len(drug_info.description) > 500:
            desc += "..."
        output_lines.append(f"\n### Description\n{desc}")

    if drug_info.indication:
        ind = drug_info.indication[:500]
        if len(drug_info.indication) > 500:
            ind += "..."
        output_lines.append(f"\n### Indication\n{ind}")

    if drug_info.mechanism_of_action:
        moa = drug_info.mechanism_of_action[:500]
        if len(drug_info.mechanism_of_action) > 500:
            moa += "..."
        output_lines.append(f"\n### Mechanism of Action\n{moa}")


def _format_drug_output(drug_info, result: dict) -> None:
    """Format drug information for text output."""
    output_lines = [f"## Drug: {drug_info.name or 'Unknown'}"]

    _format_basic_info(drug_info, output_lines)
    _format_clinical_info(drug_info, output_lines)

    if result.get("_links"):
        output_lines.append("\n### External Links")
        for name, url in result["_links"].items():
            output_lines.append(f"- [{name}]({url})")

    result["_formatted"] = "\n".join(output_lines)


async def get_drug(drug_id_or_name: str, output_json: bool = False) -> str:
    """Get drug information from MyChem.info.

    Args:
        drug_id_or_name: Drug ID (DrugBank, ChEMBL, etc.) or name
        output_json: Return JSON instead of formatted text

    Returns:
        Formatted drug information or JSON string
    """
    try:
        client = BioThingsClient()
        drug_info = await client.get_drug_info(drug_id_or_name)

        if not drug_info:
            error_msg = f"Drug '{drug_id_or_name}' not found in MyChem.info"
            if output_json:
                return json.dumps({"error": error_msg}, indent=2)
            return error_msg

        # Build result dictionary
        result = drug_info.model_dump(by_alias=False, exclude_none=True)

        # Add external links
        _add_drug_links(drug_info, result)

        if output_json:
            return json.dumps(result, indent=2)

        # Format for text output
        _format_drug_output(drug_info, result)
        return result["_formatted"]

    except Exception as e:
        logger.error(f"Error getting drug info: {e}")
        error_msg = f"Error retrieving drug information: {e!s}"
        if output_json:
            return json.dumps({"error": error_msg}, indent=2)
        return error_msg


# MCP tool function
async def _drug_details(drug_id_or_name: str) -> str:
    """Get drug/chemical information from MyChem.info.

    This tool retrieves comprehensive drug information including:
    - Drug identifiers (DrugBank, ChEMBL, PubChem, etc.)
    - Chemical properties (formula, InChIKey)
    - Trade names and synonyms
    - Clinical indications
    - Mechanism of action
    - Links to external databases

    Args:
        drug_id_or_name: Drug name (e.g., "aspirin") or ID (e.g., "DB00945", "CHEMBL25")

    Returns:
        Formatted drug information with external database links
    """
    return await get_drug(drug_id_or_name, output_json=False)
