"""Constants for variant modules."""

import os

# cBioPortal API endpoints
CBIO_BASE_URL = os.getenv("CBIO_BASE_URL", "https://www.cbioportal.org/api")
CBIO_TOKEN = os.getenv("CBIO_TOKEN")
