"""Refactored cBioPortal client for external variant aggregator using centralized HTTP."""

import asyncio
import logging
import re
from typing import Any

from pydantic import BaseModel, Field

from ..utils.cbio_http_adapter import CBioHTTPAdapter
from .cancer_types import MAX_STUDIES_PER_GENE, get_cancer_keywords

logger = logging.getLogger(__name__)


class CBioPortalVariantData(BaseModel):
    """cBioPortal variant annotation data."""

    total_cases: int | None = Field(
        None, description="Total number of cases with this variant"
    )
    studies: list[str] = Field(
        default_factory=list,
        description="List of studies containing this variant",
    )
    cancer_type_distribution: dict[str, int] = Field(
        default_factory=dict,
        description="Distribution of mutation across cancer types",
    )
    mutation_types: dict[str, int] = Field(
        default_factory=dict,
        description="Distribution of mutation types (missense, nonsense, etc)",
    )
    hotspot_count: int = Field(
        0, description="Number of samples where this is a known hotspot"
    )
    mean_vaf: float | None = Field(
        None, description="Mean variant allele frequency across samples"
    )
    sample_types: dict[str, int] = Field(
        default_factory=dict,
        description="Distribution across sample types (primary, metastatic)",
    )


class CBioPortalExternalClient:
    """Refactored cBioPortal client using centralized HTTP."""

    def __init__(self) -> None:
        self.http_adapter = CBioHTTPAdapter()
        self._study_cache: dict[str, dict[str, Any]] = {}

    async def get_variant_data(
        self, gene_aa: str
    ) -> CBioPortalVariantData | None:
        """Fetch variant data from cBioPortal.

        Args:
            gene_aa: Gene and AA change format (e.g., "BRAF V600E")
        """
        logger.info(
            f"CBioPortalExternalClient.get_variant_data called with: {gene_aa}"
        )
        try:
            # Split gene and AA change
            parts = gene_aa.split(" ", 1)
            if len(parts) != 2:
                logger.warning(f"Invalid gene_aa format: {gene_aa}")
                return None

            gene, aa_change = parts
            logger.info(f"Extracted gene={gene}, aa_change={aa_change}")

            # Get gene ID
            gene_id = await self._get_gene_id(gene)
            if not gene_id:
                return None

            # Get relevant mutation profiles
            mutation_profiles = await self._get_mutation_profiles(gene)
            if not mutation_profiles:
                logger.info(f"No relevant mutation profiles found for {gene}")
                return CBioPortalVariantData()

            # Fetch mutations
            mutations_data = await self._fetch_mutations(
                gene_id, mutation_profiles
            )
            if not mutations_data:
                return CBioPortalVariantData()

            # Filter mutations by AA change
            matching_mutations = self._filter_mutations_by_aa_change(
                mutations_data, aa_change
            )
            if not matching_mutations:
                return None

            # Aggregate mutation data
            return await self._aggregate_mutation_data(matching_mutations)

        except Exception as e:
            logger.error(
                f"Error getting cBioPortal data for {gene_aa}: {type(e).__name__}: {e}"
            )
            return None

    async def _get_gene_id(self, gene: str) -> int | None:
        """Get Entrez gene ID from gene symbol.

        Args:
            gene: Gene symbol (e.g., "BRAF")

        Returns:
            Entrez gene ID if found, None otherwise
        """
        gene_data, gene_error = await self.http_adapter.get(
            f"/genes/{gene}",
            endpoint_key="cbioportal_genes",
            cache_ttl=3600,  # 1 hour
        )

        if gene_error or not gene_data:
            logger.warning(f"Failed to fetch gene info for {gene}")
            return None

        gene_id = gene_data.get("entrezGeneId")
        if not gene_id:
            logger.warning(f"No entrezGeneId in gene response: {gene_data}")
            return None

        logger.info(f"Got entrezGeneId: {gene_id}")
        return gene_id

    async def _get_mutation_profiles(self, gene: str) -> list[dict[str, Any]]:
        """Get relevant mutation profiles for a gene.

        Args:
            gene: Gene symbol to find profiles for

        Returns:
            List of mutation profile dictionaries filtered by cancer relevance
        """
        profiles, prof_error = await self.http_adapter.get(
            "/molecular-profiles",
            endpoint_key="cbioportal_molecular_profiles",
            cache_ttl=3600,  # 1 hour
        )

        if prof_error or not profiles:
            logger.warning("Failed to fetch molecular profiles")
            return []

        # Get cancer keywords from configuration
        cancer_keywords = get_cancer_keywords(gene)

        # Collect mutation profiles to query
        mutation_profiles: list[dict[str, Any]] = []
        if not isinstance(profiles, list):
            return []

        for p in profiles:
            if (
                isinstance(p, dict)
                and p.get("molecularAlterationType") == "MUTATION_EXTENDED"
            ):
                study_id = p.get("studyId", "").lower()
                if any(keyword in study_id for keyword in cancer_keywords):
                    mutation_profiles.append(p)
                    if len(mutation_profiles) >= MAX_STUDIES_PER_GENE:
                        break

        logger.info(
            f"Found {len(mutation_profiles)} relevant mutation profiles"
        )
        return mutation_profiles

    async def _fetch_mutations(
        self, gene_id: int, mutation_profiles: list[dict[str, Any]]
    ) -> list[dict[str, Any]]:
        """Fetch mutations for a gene from mutation profiles.

        Args:
            gene_id: Entrez gene ID
            mutation_profiles: List of molecular profile dictionaries

        Returns:
            List of mutation records from cBioPortal
        """
        profile_ids = [p["molecularProfileId"] for p in mutation_profiles]
        logger.info(f"Querying {len(profile_ids)} profiles for mutations")

        mutations_data, mut_error = await self.http_adapter.post(
            "/mutations/fetch",
            data={
                "entrezGeneIds": [gene_id],
                "molecularProfileIds": profile_ids,
            },
            endpoint_key="cbioportal_mutations",
            cache_ttl=1800,  # 30 minutes
        )

        if mut_error or not mutations_data:
            logger.warning(f"Failed to fetch mutations: {mut_error}")
            return []

        if not isinstance(mutations_data, list):
            return []

        return mutations_data

    def _filter_mutations_by_aa_change(
        self, mutations_data: list[dict[str, Any]], aa_change: str
    ) -> list[dict[str, Any]]:
        """Filter mutations by amino acid change.

        Args:
            mutations_data: List of mutation records from cBioPortal
            aa_change: Amino acid change notation (e.g., "V600E")

        Returns:
            Filtered list containing only mutations matching the AA change
        """
        matching_mutations = []
        aa_patterns = self._get_aa_patterns(aa_change)

        for mut in mutations_data:
            protein_change = mut.get("proteinChange", "")
            if any(pattern.match(protein_change) for pattern in aa_patterns):
                matching_mutations.append(mut)

        logger.info(f"Found {len(matching_mutations)} matching mutations")
        return matching_mutations

    async def _aggregate_mutation_data(
        self, matching_mutations: list[dict[str, Any]]
    ) -> CBioPortalVariantData:
        """Aggregate mutation data into summary statistics.

        Args:
            matching_mutations: List of mutations matching the query criteria

        Returns:
            Aggregated variant data with statistics across all samples
        """
        # Get unique study IDs
        study_ids = list({
            mut.get("studyId", "")
            for mut in matching_mutations
            if mut.get("studyId")
        })

        # Fetch study metadata in parallel
        study_cancer_types = await self._fetch_study_metadata_parallel(
            study_ids
        )

        # Aggregate data
        sample_ids: set[str] = set()
        cancer_type_dist: dict[str, int] = {}
        mutation_type_dist: dict[str, int] = {}
        vaf_values: list[float] = []
        sample_type_dist: dict[str, int] = {}

        for mut in matching_mutations:
            # Count samples
            sample_id = mut.get("sampleId")
            if sample_id:
                sample_ids.add(sample_id)

            # Count cancer types
            study_id = mut.get("studyId", "")
            if study_id in study_cancer_types:
                cancer_type = study_cancer_types[study_id]
                cancer_type_dist[cancer_type] = (
                    cancer_type_dist.get(cancer_type, 0) + 1
                )

            # Count mutation types
            mut_type = mut.get("mutationType", "Unknown")
            mutation_type_dist[mut_type] = (
                mutation_type_dist.get(mut_type, 0) + 1
            )

            # Calculate VAF if data available
            tumor_alt = mut.get("tumorAltCount")
            tumor_ref = mut.get("tumorRefCount")
            if (
                tumor_alt is not None
                and tumor_ref is not None
                and (tumor_alt + tumor_ref) > 0
            ):
                vaf = tumor_alt / (tumor_alt + tumor_ref)
                vaf_values.append(vaf)

            # Count sample types
            sample_type = mut.get("sampleType", "Unknown")
            sample_type_dist[sample_type] = (
                sample_type_dist.get(sample_type, 0) + 1
            )

        # Calculate mean VAF
        mean_vaf = None
        if vaf_values:
            mean_vaf = round(sum(vaf_values) / len(vaf_values), 3)

        # Check for hotspots (simplified - just check if it's a common mutation)
        hotspot_count = (
            len(matching_mutations) if len(matching_mutations) > 10 else 0
        )

        return CBioPortalVariantData(
            total_cases=len(sample_ids),
            studies=sorted(study_ids)[:10],  # Top 10 studies
            cancer_type_distribution=cancer_type_dist,
            mutation_types=mutation_type_dist,
            hotspot_count=hotspot_count,
            mean_vaf=mean_vaf,
            sample_types=sample_type_dist,
        )

    def _get_aa_patterns(self, aa_change: str) -> list[re.Pattern]:
        """Get regex patterns to match amino acid changes.

        Handles various notation formats:
        - Direct match (e.g., "V600E")
        - With p. prefix (e.g., "p.V600E")
        - Position wildcards (e.g., "V600*" matches V600E, V600K, etc.)

        Args:
            aa_change: Amino acid change notation

        Returns:
            List of compiled regex patterns for flexible matching
        """
        patterns = []

        # Direct match
        patterns.append(re.compile(re.escape(aa_change)))

        # Handle p. prefix
        if not aa_change.startswith("p."):
            patterns.append(re.compile(f"p\\.{re.escape(aa_change)}"))
        else:
            # Also try without p.
            patterns.append(re.compile(re.escape(aa_change[2:])))

        # Handle special cases like V600E/V600K
        base_match = re.match(r"([A-Z])(\d+)([A-Z])", aa_change)
        if base_match:
            ref_aa, position, _ = base_match.groups()
            # Match any mutation at this position
            patterns.append(re.compile(f"p?\\.?{ref_aa}{position}[A-Z]"))

        return patterns

    async def _fetch_study_metadata_parallel(
        self, study_ids: list[str]
    ) -> dict[str, str]:
        """Fetch study metadata in parallel for cancer type information.

        Args:
            study_ids: List of study IDs to fetch

        Returns:
            Dict mapping study ID to cancer type name
        """
        # Check cache first
        study_cancer_types = {}
        uncached_ids = []

        for study_id in study_ids:
            if study_id in self._study_cache:
                study_data = self._study_cache[study_id]
                cancer_type = study_data.get("cancerType", {})
                study_cancer_types[study_id] = cancer_type.get(
                    "name", "Unknown"
                )
            else:
                uncached_ids.append(study_id)

        if uncached_ids:
            # Fetch uncached studies in parallel
            tasks = []
            for study_id in uncached_ids[:10]:  # Limit parallel requests
                tasks.append(self._fetch_single_study(study_id))

            results = await asyncio.gather(*tasks, return_exceptions=True)

            for study_id, result in zip(
                uncached_ids[:10], results, strict=False
            ):
                if isinstance(result, Exception):
                    logger.debug(
                        f"Failed to fetch study {study_id}: {type(result).__name__}"
                    )
                    study_cancer_types[study_id] = "Unknown"
                elif isinstance(result, dict):
                    # Cache the study data
                    self._study_cache[study_id] = result
                    cancer_type = result.get("cancerType", {})
                    study_cancer_types[study_id] = cancer_type.get(
                        "name", "Unknown"
                    )
                else:
                    study_cancer_types[study_id] = "Unknown"

        return study_cancer_types

    async def _fetch_single_study(
        self, study_id: str
    ) -> dict[str, Any] | None:
        """Fetch metadata for a single study."""
        study_data, error = await self.http_adapter.get(
            f"/studies/{study_id}",
            endpoint_key="cbioportal_studies",
            cache_ttl=3600,  # 1 hour
        )

        if error or not study_data:
            logger.debug(f"Failed to fetch study {study_id}: {error}")
            return None

        return study_data
