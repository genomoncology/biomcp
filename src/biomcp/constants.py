"""
Central constants file for BioMCP.

This module contains all constants used throughout the BioMCP codebase,
including API URLs, default values, limits, and domain configurations.
"""

# ============================================================================
# API Base URLs
# ============================================================================

# PubTator3 API
# https://www.ncbi.nlm.nih.gov/research/pubtator3/api
PUBTATOR3_BASE_URL = "https://www.ncbi.nlm.nih.gov/research/pubtator3-api"
PUBTATOR3_SEARCH_URL = f"{PUBTATOR3_BASE_URL}/search/"
PUBTATOR3_FULLTEXT_URL = f"{PUBTATOR3_BASE_URL}/publications/export/biocjson"
PUBTATOR3_AUTOCOMPLETE_URL = f"{PUBTATOR3_BASE_URL}/entity/autocomplete/"

# ClinicalTrials.gov API
# https://clinicaltrials.gov/data-api/api
CLINICAL_TRIALS_BASE_URL = "https://clinicaltrials.gov/api/v2/studies"
CLINICAL_TRIALS_STUDY_URL = "https://clinicaltrials.gov/study/"

# MyVariant.info API
# https://docs.myvariant.info/
MYVARIANT_BASE_URL = "https://myvariant.info/v1"
MYVARIANT_QUERY_URL = f"{MYVARIANT_BASE_URL}/query"
MYVARIANT_GET_URL = f"{MYVARIANT_BASE_URL}/variant"

# Preprint Server APIs
BIORXIV_BASE_URL = "https://api.biorxiv.org/details/biorxiv"
MEDRXIV_BASE_URL = "https://api.biorxiv.org/details/medrxiv"
EUROPE_PMC_BASE_URL = "https://www.ebi.ac.uk/europepmc/webservices/rest/search"

# External Variant APIs
GDC_BASE_URL = "https://api.gdc.cancer.gov"
GDC_SSMS_ENDPOINT_URL = f"{GDC_BASE_URL}/ssms"  # Simple Somatic Mutations
GDC_SSM_OCCURRENCES_URL = f"{GDC_BASE_URL}/ssm_occurrences"
ENSEMBL_REST_BASE_URL = "https://rest.ensembl.org"
ENSEMBL_VARIATION_URL = f"{ENSEMBL_REST_BASE_URL}/variation/human"

# External Resource URLs
PUBMED_BASE_URL = "https://pubmed.ncbi.nlm.nih.gov/"
PMC_BASE_URL = "https://www.ncbi.nlm.nih.gov/pmc/articles/"
DOI_BASE_URL = "https://doi.org/"
DBSNP_BASE_URL = "https://www.ncbi.nlm.nih.gov/snp/"
CLINVAR_BASE_URL = "https://www.ncbi.nlm.nih.gov/clinvar/variation/"
COSMIC_BASE_URL = "https://cancer.sanger.ac.uk/cosmic/mutation/overview?id="
CIVIC_BASE_URL = "https://civicdb.org/variants/"
ENSEMBL_VARIANT_BASE_URL = "https://ensembl.org/Homo_sapiens/Variation/Explore?v="
GENENAMES_BASE_URL = "https://www.genenames.org/data/gene-symbol-report/#!/symbol/"
UCSC_GENOME_BROWSER_URL = "https://genome.ucsc.edu/cgi-bin/hgTracks?db=hg19&"

# ============================================================================
# Default Values and Limits
# ============================================================================

# Caching
DEFAULT_CACHE_TIMEOUT = 60 * 60 * 24 * 7  # 1 week in seconds

# Pagination
SYSTEM_PAGE_SIZE = 40  # Default page size for all searches
DEFAULT_PAGE_SIZE = 10  # Default page size for unified search
MIN_PAGE_SIZE = 1
MAX_PAGE_SIZE = 100
DEFAULT_PAGE_NUMBER = 1

# Search limits
MAX_RESULTS_PER_DOMAIN_DEFAULT = 10  # Default max results per domain in unified search
ESTIMATED_ADDITIONAL_RESULTS = 100  # Estimate for additional results when full page returned
DEFAULT_AUTOCOMPLETE_LIMIT = 1
MAX_AUTOCOMPLETE_LIMIT = 100

# Text display
MAX_WIDTH = 72  # Maximum width for text wrapping in console output
SNIPPET_LENGTH = 200  # Maximum length for text snippets in search results

# ============================================================================
# Domain Configuration
# ============================================================================

# Valid domains for search
VALID_DOMAINS = ["article", "trial", "variant", "thinking"]
VALID_DOMAINS_PLURAL = ["articles", "trials", "variants"]

# Domain mappings for unified search
DOMAIN_TO_PLURAL = {
    "article": "articles",
    "trial": "trials",
    "variant": "variants"
}

PLURAL_TO_DOMAIN = {
    "articles": "article",
    "trials": "trial",
    "variants": "variant"
}

# Trial detail sections
TRIAL_DETAIL_SECTIONS = ["protocol", "locations", "outcomes", "references", "all", "full"]

# ============================================================================
# Field Names and Enums
# ============================================================================

# Autocomplete concept types
AUTOCOMPLETE_CONCEPTS = ["variant", "chemical", "disease", "gene"]

# HTTP methods
VALID_HTTP_METHODS = ["GET", "POST"]

# Trial search defaults
DEFAULT_TRIAL_FORMAT = "csv"
DEFAULT_TRIAL_MARKUP = "markdown"

# ============================================================================
# Error Messages
# ============================================================================

ERROR_THOUGHT_NUMBER_MIN = "Error: thoughtNumber must be >= 1"
ERROR_TOTAL_THOUGHTS_MIN = "Error: totalThoughts must be >= 1"
ERROR_DOMAIN_REQUIRED = "Either 'query' or 'domain' parameter must be provided"
ERROR_THOUGHT_REQUIRED = "'thought' parameter is required when domain='thinking'"
ERROR_THOUGHT_NUMBER_REQUIRED = "'thoughtNumber' parameter is required when domain='thinking'"
ERROR_TOTAL_THOUGHTS_REQUIRED = "'totalThoughts' parameter is required when domain='thinking'"
ERROR_NEXT_THOUGHT_REQUIRED = "'nextThoughtNeeded' parameter is required when domain='thinking'"

# ============================================================================
# API Response Formatting
# ============================================================================

# Metadata field names
METADATA_YEAR = "year"
METADATA_JOURNAL = "journal"
METADATA_AUTHORS = "authors"
METADATA_STATUS = "status"
METADATA_PHASE = "phase"
METADATA_START_DATE = "start_date"
METADATA_COMPLETION_DATE = "primary_completion_date"
METADATA_GENE = "gene"
METADATA_RSID = "rsid"
METADATA_SIGNIFICANCE = "clinical_significance"
METADATA_CONSEQUENCE = "consequence"
METADATA_SOURCE = "source"

# Result field names
RESULT_ID = "id"
RESULT_TITLE = "title"
RESULT_SNIPPET = "snippet"  # Internal use for domain handlers
RESULT_TEXT = "text"  # OpenAI MCP compliant field name
RESULT_URL = "url"
RESULT_METADATA = "metadata"
RESULT_DATA = "data"
RESULT_PAGE = "page"
RESULT_PAGE_SIZE = "page_size"
RESULT_TOTAL = "total"
RESULT_NEXT_PAGE = "next_page"
