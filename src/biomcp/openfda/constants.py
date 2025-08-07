"""
Constants for OpenFDA API integration.
"""

# OpenFDA API Base
OPENFDA_BASE_URL = "https://api.fda.gov"

# Drug endpoints
OPENFDA_DRUG_EVENTS_URL = f"{OPENFDA_BASE_URL}/drug/event.json"
OPENFDA_DRUG_LABELS_URL = f"{OPENFDA_BASE_URL}/drug/label.json"
OPENFDA_DRUG_ENFORCEMENT_URL = f"{OPENFDA_BASE_URL}/drug/enforcement.json"
OPENFDA_DRUGSFDA_URL = f"{OPENFDA_BASE_URL}/drug/drugsfda.json"

# Device endpoints
OPENFDA_DEVICE_EVENTS_URL = f"{OPENFDA_BASE_URL}/device/event.json"
OPENFDA_DEVICE_CLASSIFICATION_URL = (
    f"{OPENFDA_BASE_URL}/device/classification.json"
)
OPENFDA_DEVICE_RECALL_URL = f"{OPENFDA_BASE_URL}/device/recall.json"

# API limits
OPENFDA_DEFAULT_LIMIT = 25
OPENFDA_MAX_LIMIT = 100
OPENFDA_RATE_LIMIT_NO_KEY = 40  # requests per minute without key
OPENFDA_RATE_LIMIT_WITH_KEY = 240  # requests per minute with key

# Genomic device filters - product codes for genomic/diagnostic devices
GENOMIC_DEVICE_PRODUCT_CODES = [
    "OOI",  # Next Generation Sequencing Oncology Panel Test System
    "PQP",  # Nucleic Acid Based In Vitro Diagnostic Devices
    "OYD",  # Gene Mutation Detection System
    "NYE",  # DNA Sequencer
    "OEO",  # Hereditary or Somatic Variant Detection System
    "QIN",  # Tumor Profiling Test
    "QDI",  # Companion Diagnostic
    "PTA",  # Cancer Predisposition Risk Assessment System
]

# Common adverse event search fields
ADVERSE_EVENT_FIELDS = [
    "patient.drug.medicinalproduct",
    "patient.drug.openfda.brand_name",
    "patient.drug.openfda.generic_name",
    "patient.drug.drugindication",
    "patient.reaction.reactionmeddrapt",
]

# Label search fields
LABEL_FIELDS = [
    "openfda.brand_name",
    "openfda.generic_name",
    "indications_and_usage",
    "boxed_warning",
    "warnings_and_precautions",
    "adverse_reactions",
    "drug_interactions",
]

# Device event search fields
DEVICE_FIELDS = [
    "device.brand_name",
    "device.generic_name",
    "device.manufacturer_d_name",
    "device.openfda.device_name",
    "device.openfda.medical_specialty_description",
]

# Disclaimer text
OPENFDA_DISCLAIMER = (
    "⚠️ **FDA Data Notice**: Information from openFDA API. "
    "Not for clinical decision-making. Adverse events don't prove causation. "
    "Data may be incomplete or delayed. Consult healthcare professionals and "
    "official FDA sources at fda.gov for medical decisions."
)

OPENFDA_SHORTAGE_DISCLAIMER = (
    "🚨 **Critical Warning**: Drug shortage information is time-sensitive. "
    "Always verify current availability with FDA Drug Shortages Database at "
    "https://www.accessdata.fda.gov/scripts/drugshortages/ before making "
    "supply chain or treatment decisions."
)
