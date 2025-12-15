"""
Enrichr database name mappings for different enrichment categories.

Inspired by gget enrichr (https://github.com/pachterlab/gget).
Citation: Luebbert & Pachter (2023). Bioinformatics, 39(1), btac836.
BioMCP directly integrates with Enrichr API rather than using gget as a dependency.
"""

from typing import Literal

# Map friendly names to Enrichr database names
ENRICHR_DATABASES = {
    # Pathway databases
    "pathway": "KEGG_2021_Human",
    "kegg": "KEGG_2021_Human",
    "reactome": "Reactome_2022",
    "wikipathways": "WikiPathway_2021_Human",
    # Gene Ontology
    "ontology": "GO_Biological_Process_2021",
    "go_process": "GO_Biological_Process_2021",
    "go_molecular": "GO_Molecular_Function_2021",
    "go_cellular": "GO_Cellular_Component_2021",
    # Cell type databases
    "celltypes": "PanglaoDB_Augmented_2021",
    "tissues": "Human_Gene_Atlas",
    # Disease associations
    "diseases": "GWAS_Catalog_2023",
    "gwas": "GWAS_Catalog_2023",
    # Transcription factors
    "transcription_factors": "ChEA_2022",
    "tf": "ChEA_2022",
}

DatabaseCategory = Literal[
    "pathway",
    "kegg",
    "reactome",
    "wikipathways",
    "ontology",
    "go_process",
    "go_molecular",
    "go_cellular",
    "celltypes",
    "tissues",
    "diseases",
    "gwas",
    "transcription_factors",
    "tf",
]


def get_database_name(category: DatabaseCategory | str) -> str:
    """
    Get the Enrichr database name for a given category.

    Args:
        category: Short name like "pathway", "ontology", or full database name

    Returns:
        Full Enrichr database name

    Raises:
        ValueError: If category is not recognized
    """
    # If it's already a full database name (e.g., contains underscores and year)
    if "_" in category and any(
        year in category for year in ["2021", "2022", "2023"]
    ):
        return category

    # Otherwise look up the mapping
    db_name = ENRICHR_DATABASES.get(category.lower())
    if not db_name:
        raise ValueError(
            f"Unknown database category: {category}. "
            f"Available categories: {', '.join(ENRICHR_DATABASES.keys())}"
        )

    return db_name
