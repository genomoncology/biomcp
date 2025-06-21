"""Cancer type configuration for gene-specific studies."""

# Gene to cancer type keyword mapping
# These keywords are used to filter relevant studies from cBioPortal
GENE_CANCER_KEYWORDS = {
    "BRAF": [
        "skcm",  # melanoma
        "thca",  # thyroid
        "coad",  # colorectal
        "lung",
        "glioma",  # brain
        "hairy_cell",  # hairy cell leukemia
    ],
    "KRAS": [
        "coad",  # colorectal
        "paad",  # pancreatic
        "lung",
        "stad",  # stomach
        "coadread",  # colorectal adenocarcinoma
        "ampca",  # ampullary carcinoma
    ],
    "TP53": [
        "brca",  # breast
        "ov",  # ovarian
        "lung",
        "hnsc",  # head/neck
        "lgg",  # lower grade glioma
        "gbm",  # glioblastoma
        "blca",  # bladder
        "lihc",  # liver
    ],
    "EGFR": [
        "lung",
        "nsclc",  # non-small cell lung cancer
        "gbm",  # glioblastoma
        "hnsc",  # head/neck
    ],
    "PIK3CA": [
        "brca",  # breast
        "hnsc",  # head/neck
        "coad",  # colorectal
        "ucec",  # endometrial
    ],
    "PTEN": [
        "prad",  # prostate
        "gbm",  # glioblastoma
        "ucec",  # endometrial
        "brca",  # breast
    ],
    "APC": [
        "coad",  # colorectal
        "coadread",
        "stad",  # stomach
    ],
    "VHL": [
        "rcc",  # renal cell carcinoma
        "ccrcc",  # clear cell RCC
        "kirc",  # kidney clear cell
    ],
    "RB1": [
        "rbl",  # retinoblastoma
        "sclc",  # small cell lung cancer
        "blca",  # bladder
    ],
    "BRCA1": [
        "brca",  # breast
        "ov",  # ovarian
        "prad",  # prostate
        "paad",  # pancreatic
    ],
    "BRCA2": [
        "brca",  # breast
        "ov",  # ovarian
        "prad",  # prostate
        "paad",  # pancreatic
    ],
    "ALK": [
        "lung",
        "nsclc",  # non-small cell lung cancer
        "alcl",  # anaplastic large cell lymphoma
        "nbl",  # neuroblastoma
    ],
    "MYC": [
        "burkitt",  # Burkitt lymphoma
        "dlbcl",  # diffuse large B-cell lymphoma
        "mm",  # multiple myeloma
        "nbl",  # neuroblastoma
    ],
    "NRAS": [
        "mel",  # melanoma
        "skcm",
        "thca",  # thyroid
        "aml",  # acute myeloid leukemia
    ],
    "KIT": [
        "gist",  # gastrointestinal stromal tumor
        "mel",  # melanoma
        "aml",  # acute myeloid leukemia
    ],
}

# Default keywords for genes not in the mapping
DEFAULT_CANCER_KEYWORDS = ["msk", "tcga", "metabric", "dfci", "broad"]

# Maximum number of studies to query per gene
MAX_STUDIES_PER_GENE = 20

# Maximum mutations to process per study
MAX_MUTATIONS_PER_STUDY = 5000


def get_cancer_keywords(gene: str) -> list[str]:
    """Get cancer type keywords for a given gene.

    Args:
        gene: Gene symbol (e.g., "BRAF")

    Returns:
        List of cancer type keywords to search for
    """
    return GENE_CANCER_KEYWORDS.get(gene.upper(), DEFAULT_CANCER_KEYWORDS)
