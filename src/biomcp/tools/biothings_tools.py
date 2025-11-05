"""Consolidated BioThings tools for accessing gene, disease, and drug databases."""

import logging
from typing import Annotated, Literal

from pydantic import Field

from biomcp.core import mcp_app
from biomcp.genes.getter import _gene_details
from biomcp.diseases.getter import _disease_details
from biomcp.drugs.getter import _drug_details
from biomcp.metrics import track_performance

logger = logging.getLogger(__name__)


@mcp_app.tool()
@track_performance("biomcp.gene")
async def gene(
    action: Annotated[
        Literal["get"],
        Field(description="Action to perform: 'get' to retrieve gene information"),
    ],
    gene_id_or_symbol: Annotated[
        str,
        Field(
            description="Gene symbol (e.g., 'TP53', 'BRAF') or Entrez ID (e.g., '7157')"
        ),
    ],
) -> str:
    """Get detailed gene information from MyGene.info.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to understand your research goal!

    Provides real-time gene annotations from MyGene.info including:
    - Official gene name and symbol
    - Gene summary/description
    - Aliases and alternative names
    - Gene type (protein-coding, etc.)
    - Entrez ID and Ensembl IDs
    - RefSeq information
    - Links to external databases

    This tool fetches CURRENT gene information from MyGene.info, ensuring
    you always have the latest annotations and nomenclature.

    ## Actions:
    - get: Retrieve comprehensive gene information

    ## Examples:

    Get TP53 tumor suppressor information:
    ```python
    await gene(action="get", gene_id_or_symbol="TP53")
    ```

    Look up BRAF kinase gene details:
    ```python
    await gene(action="get", gene_id_or_symbol="BRAF")
    ```

    Get gene by Entrez ID:
    ```python
    await gene(action="get", gene_id_or_symbol="673")  # BRAF Entrez ID
    ```

    Note: For genetic variants, use the variant tool. For articles about genes, use the article tool.
    """
    logger.info(f"Gene tool called: action={action}, gene={gene_id_or_symbol}")

    if action == "get":
        return await _gene_details(
            call_benefit="Get up-to-date gene annotations and information",
            gene_id_or_symbol=gene_id_or_symbol,
        )

    return f"Error: Invalid action '{action}'"


@mcp_app.tool()
@track_performance("biomcp.disease")
async def disease(
    action: Annotated[
        Literal["get"],
        Field(description="Action to perform: 'get' to retrieve disease information"),
    ],
    disease_id_or_name: Annotated[
        str,
        Field(
            description="Disease name (e.g., 'melanoma', 'lung cancer') or ontology ID (e.g., 'MONDO:0016575', 'DOID:1909')"
        ),
    ],
) -> str:
    """Get detailed disease information from MyDisease.info.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to understand your research goal!

    Provides real-time disease annotations from MyDisease.info including:
    - Official disease name and definition
    - Disease synonyms and alternative names
    - Ontology mappings (MONDO, DOID, OMIM, etc.)
    - Associated phenotypes
    - Cross-references to disease databases
    - Links to disease resources

    This tool fetches CURRENT disease information from MyDisease.info, ensuring
    you always have the latest ontology mappings and definitions.

    ## Actions:
    - get: Retrieve comprehensive disease information

    ## Examples:

    Get GIST (Gastrointestinal Stromal Tumor) definition:
    ```python
    await disease(action="get", disease_id_or_name="GIST")
    ```

    Look up melanoma synonyms:
    ```python
    await disease(action="get", disease_id_or_name="melanoma")
    ```

    Find disease by MONDO ID:
    ```python
    await disease(action="get", disease_id_or_name="MONDO:0016575")
    ```

    Note: For NCI's cancer-specific disease vocabulary, use the nci tool with resource="disease".
    For clinical trials about diseases, use the trial tool. For articles about diseases, use the article tool.
    """
    logger.info(f"Disease tool called: action={action}, disease={disease_id_or_name}")

    if action == "get":
        return await _disease_details(
            call_benefit="Get up-to-date disease definitions and ontology information",
            disease_id_or_name=disease_id_or_name,
        )

    return f"Error: Invalid action '{action}'"


@mcp_app.tool()
@track_performance("biomcp.drug")
async def drug(
    action: Annotated[
        Literal["get"],
        Field(description="Action to perform: 'get' to retrieve drug information"),
    ],
    drug_id_or_name: Annotated[
        str,
        Field(
            description="Drug name (e.g., 'aspirin', 'imatinib') or ID (e.g., 'DB00945', 'CHEMBL941')"
        ),
    ],
) -> str:
    """Get detailed drug/chemical information from MyChem.info.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to understand your research goal!

    Provides comprehensive drug information from MyChem.info including:
    - Chemical properties (formula, InChIKey)
    - Drug identifiers (DrugBank, ChEMBL, PubChem)
    - Trade names and brand names
    - Clinical indications
    - Mechanism of action
    - Pharmacology details
    - Links to drug databases

    This tool fetches CURRENT drug information from MyChem.info, part of the
    BioThings suite, ensuring you always have the latest drug data.

    ## Actions:
    - get: Retrieve comprehensive drug information

    ## Examples:

    Get imatinib (Gleevec) information:
    ```python
    await drug(action="get", drug_id_or_name="imatinib")
    ```

    Look up drug by DrugBank ID:
    ```python
    await drug(action="get", drug_id_or_name="DB00619")
    ```

    Find mechanism of action for pembrolizumab:
    ```python
    await drug(action="get", drug_id_or_name="pembrolizumab")
    ```

    Note: For FDA drug labels and safety data, use the fda tool.
    For clinical trials about drugs, use the trial tool. For articles about drugs, use the article tool.
    """
    logger.info(f"Drug tool called: action={action}, drug={drug_id_or_name}")

    if action == "get":
        return await _drug_details(drug_id_or_name)

    return f"Error: Invalid action '{action}'"
