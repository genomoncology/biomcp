"""Cancer type utilities using cBioPortal API."""

import logging

from ..utils.cbio_http_adapter import CBioHTTPAdapter
from ..utils.request_cache import request_cache

logger = logging.getLogger(__name__)


class CancerTypeAPIClient:
    """Client for fetching cancer types from cBioPortal API."""

    def __init__(self):
        """Initialize the cancer type API client."""
        self.http_adapter = CBioHTTPAdapter()
        # Cache for cancer types
        self._cancer_types_cache: dict[str, str] | None = None

    @request_cache(ttl=86400)  # Cache for 24 hours
    async def get_all_cancer_types(self) -> dict[str, str]:
        """Fetch all cancer types from cBioPortal API.

        Returns:
            Dictionary mapping cancer type IDs to display names
        """
        if self._cancer_types_cache is not None:
            return self._cancer_types_cache

        try:
            cancer_types, error = await self.http_adapter.get(
                "/cancer-types",
                endpoint_key="cbioportal_cancer_types",
                cache_ttl=86400,  # 24 hours
            )

            if error:
                logger.error(f"Failed to fetch cancer types: {error.message}")
                return {}

            if cancer_types:
                # Build mapping from ID to name
                result = {}
                for ct in cancer_types:
                    cancer_type_id = ct.get("cancerTypeId", "")
                    name = ct.get("name", "")

                    if cancer_type_id and name:
                        result[cancer_type_id.lower()] = name

                        # Also add common abbreviations
                        short_name = ct.get("shortName", "")
                        if short_name and short_name != cancer_type_id:
                            result[short_name.lower()] = name

                self._cancer_types_cache = result
                logger.info(f"Loaded {len(result)} cancer types from API")
                return result

            return {}

        except Exception as e:
            logger.error(f"Error fetching cancer types: {e}")
            return {}

    async def get_cancer_type_name(self, cancer_type_id: str) -> str:
        """Get the display name for a cancer type ID.

        Args:
            cancer_type_id: The cancer type identifier

        Returns:
            Display name or the original ID if not found
        """
        if not cancer_type_id:
            return "Unknown"

        cancer_types = await self.get_all_cancer_types()

        # Try exact match (case-insensitive)
        normalized_id = cancer_type_id.lower()
        if normalized_id in cancer_types:
            return cancer_types[normalized_id]

        # If not found, return the original ID with title case
        if cancer_type_id == cancer_type_id.lower():
            return cancer_type_id.title()
        return cancer_type_id

    @request_cache(ttl=3600)  # Cache for 1 hour
    async def get_study_cancer_type(self, study_id: str) -> str:
        """Get cancer type for a specific study.

        Args:
            study_id: The study identifier

        Returns:
            Cancer type name or "Unknown"
        """
        try:
            study_data, error = await self.http_adapter.get(
                f"/studies/{study_id}",
                endpoint_key="cbioportal_studies",
                cache_ttl=3600,  # 1 hour
            )

            if error or not study_data:
                logger.debug(f"Study {study_id} not found")
                return "Unknown"

            cancer_type_id = study_data.get("cancerType", {}).get(
                "cancerTypeId", ""
            )

            if cancer_type_id and cancer_type_id != "unknown":
                return await self.get_cancer_type_name(cancer_type_id)

            # Fallback to the cancer type name directly
            cancer_type_name = study_data.get("cancerType", {}).get("name", "")
            if cancer_type_name:
                return cancer_type_name

            return "Unknown"

        except Exception as e:
            logger.debug(f"Error fetching study {study_id}: {e}")
            return "Unknown"


# Global instance for reuse
_cancer_type_client: CancerTypeAPIClient | None = None


def get_cancer_type_client() -> CancerTypeAPIClient:
    """Get or create the global cancer type client."""
    global _cancer_type_client
    if _cancer_type_client is None:
        _cancer_type_client = CancerTypeAPIClient()
    return _cancer_type_client
