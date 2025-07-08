"""Constants for variant tests."""

# API retry settings
API_RETRY_DELAY_SECONDS = 1.0
MAX_RETRY_ATTEMPTS = 2

# Test data settings
DEFAULT_MAX_STUDIES = 10  # Number of studies to query in integration tests
STRUCTURE_CHECK_LIMIT = (
    3  # Number of items to check when verifying data structures
)

# Timeout settings
INTEGRATION_TEST_TIMEOUT = 30.0  # Maximum time for integration tests
