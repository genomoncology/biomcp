"""cBioPortal mutation-specific search functionality."""

import logging
from collections import Counter, defaultdict
from typing import Any, cast

from pydantic import BaseModel, Field

from ..utils.cancer_types_api import get_cancer_type_client
from ..utils.cbio_http_adapter import CBioHTTPAdapter
from ..utils.gene_validator import is_valid_gene_symbol, sanitize_gene_symbol
from ..utils.metrics import track_api_call
from ..utils.mutation_filter import MutationFilter
from ..utils.request_cache import request_cache

logger = logging.getLogger(__name__)


class MutationHit(BaseModel):
    """A specific mutation occurrence in a study."""

    study_id: str
    molecular_profile_id: str
    protein_change: str
    mutation_type: str
    start_position: int | None = None
    end_position: int | None = None
    reference_allele: str | None = None
    variant_allele: str | None = None
    sample_id: str | None = None


class StudyMutationSummary(BaseModel):
    """Summary of mutations in a specific study."""

    study_id: str
    study_name: str
    cancer_type: str
    mutation_count: int
    sample_count: int = 0
    mutations: list[str] = Field(default_factory=list)


class MutationSearchResult(BaseModel):
    """Result of a mutation-specific search."""

    gene: str
    specific_mutation: str | None = None
    pattern: str | None = None
    total_studies: int = 0
    studies_with_mutation: int = 0
    total_mutations: int = 0
    top_studies: list[StudyMutationSummary] = Field(default_factory=list)
    mutation_types: dict[str, int] = Field(default_factory=dict)


class CBioPortalMutationClient:
    """Client for mutation-specific searches in cBioPortal."""

    def __init__(self):
        """Initialize the mutation search client."""
        self.http_adapter = CBioHTTPAdapter()

    async def __aenter__(self):
        """Async context manager entry."""
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        pass  # No cleanup needed with centralized client

    @request_cache(ttl=1800)  # Cache for 30 minutes
    @track_api_call("cbioportal_mutation_search")
    async def search_specific_mutation(
        self,
        gene: str,
        mutation: str | None = None,
        pattern: str | None = None,
        max_studies: int = 20,
    ) -> MutationSearchResult | None:
        """Search for specific mutations across all cBioPortal studies.

        Args:
            gene: Gene symbol (e.g., "SRSF2")
            mutation: Specific mutation (e.g., "F57Y")
            pattern: Pattern to match (e.g., "F57" for F57*)
            max_studies: Maximum number of top studies to return

        Returns:
            Detailed mutation search results or None if not found
        """
        # Validate gene
        if not is_valid_gene_symbol(gene):
            logger.warning(f"Invalid gene symbol: {gene}")
            return None

        gene = sanitize_gene_symbol(gene)

        try:
            return await self._search_mutations_with_adapter(
                gene, mutation, pattern, max_studies
            )
        except TimeoutError:
            logger.error(f"Timeout searching mutations for {gene}")
            return None
        except Exception as e:
            logger.error(f"Error searching mutations for {gene}: {e}")
            return None

    async def _search_mutations_with_adapter(
        self,
        gene: str,
        mutation: str | None,
        pattern: str | None,
        max_studies: int,
    ) -> MutationSearchResult | None:
        """Perform the actual mutation search with the adapter."""
        # Get gene info
        gene_data, error = await self.http_adapter.get(
            f"/genes/{gene}", endpoint_key="cbioportal_genes"
        )

        if error or not gene_data:
            logger.warning(f"Gene {gene} not found in cBioPortal")
            return None

        entrez_id = gene_data.get("entrezGeneId")

        if not entrez_id:
            logger.warning(f"No Entrez ID found for gene {gene}")
            return None

        # Get all mutation profiles
        logger.info(f"Fetching mutation profiles for {gene}")
        all_profiles, prof_error = await self.http_adapter.get(
            "/molecular-profiles",
            params={"molecularAlterationType": "MUTATION_EXTENDED"},
            endpoint_key="cbioportal_molecular_profiles",
        )

        if prof_error or not all_profiles:
            logger.error("Failed to fetch molecular profiles")
            return None
        profile_ids = [p["molecularProfileId"] for p in all_profiles]

        # Batch fetch mutations (this is the slow part)
        logger.info(
            f"Fetching mutations for {gene} across {len(profile_ids)} profiles"
        )
        mutations = await self._fetch_all_mutations(profile_ids, entrez_id)

        if not mutations:
            logger.info(f"No mutations found for {gene}")
            return MutationSearchResult(gene=gene)

        # Filter mutations based on criteria
        mutation_filter = MutationFilter(mutation, pattern)
        filtered_mutations = mutation_filter.filter_mutations(mutations)

        # Get study information
        studies_info = await self._get_studies_info()

        # Aggregate results by study
        study_mutations = self._aggregate_by_study(
            cast(list[MutationHit], filtered_mutations), studies_info
        )

        # Sort by mutation count and take top studies
        top_studies = sorted(
            study_mutations.values(),
            key=lambda x: x.mutation_count,
            reverse=True,
        )[:max_studies]

        # Count mutation types
        mutation_types = Counter(m.protein_change for m in filtered_mutations)

        return MutationSearchResult(
            gene=gene,
            specific_mutation=mutation,
            pattern=pattern,
            total_studies=len(all_profiles),
            studies_with_mutation=len(study_mutations),
            total_mutations=len(filtered_mutations),
            top_studies=top_studies,
            mutation_types=dict(mutation_types.most_common(10)),
        )

    @track_api_call("cbioportal_fetch_mutations")
    async def _fetch_all_mutations(
        self,
        profile_ids: list[str],
        entrez_id: int,
    ) -> list[MutationHit]:
        """Fetch all mutations for a gene across all profiles."""

        try:
            raw_mutations, error = await self.http_adapter.post(
                "/mutations/fetch",
                data={
                    "molecularProfileIds": profile_ids,
                    "entrezGeneIds": [entrez_id],
                },
                endpoint_key="cbioportal_mutations",
                cache_ttl=1800,  # Cache for 30 minutes
            )

            if error or not raw_mutations:
                logger.error(f"Failed to fetch mutations: {error}")
                return []

            # Convert to MutationHit objects
            mutations = []
            for mut in raw_mutations:
                try:
                    # Extract study ID from molecular profile ID
                    study_id = mut.get("molecularProfileId", "").replace(
                        "_mutations", ""
                    )

                    mutations.append(
                        MutationHit(
                            study_id=study_id,
                            molecular_profile_id=mut.get(
                                "molecularProfileId", ""
                            ),
                            protein_change=mut.get("proteinChange", ""),
                            mutation_type=mut.get("mutationType", ""),
                            start_position=mut.get("startPosition"),
                            end_position=mut.get("endPosition"),
                            reference_allele=mut.get("referenceAllele"),
                            variant_allele=mut.get("variantAllele"),
                            sample_id=mut.get("sampleId"),
                        )
                    )
                except Exception as e:
                    logger.debug(f"Failed to parse mutation: {e}")
                    continue

            return mutations

        except Exception as e:
            logger.error(f"Error fetching mutations: {e}")
            return []

    async def _get_studies_info(self) -> dict[str, dict[str, Any]]:
        """Get information about all studies."""

        try:
            studies, error = await self.http_adapter.get(
                "/studies",
                endpoint_key="cbioportal_studies",
                cache_ttl=3600,  # Cache for 1 hour
            )

            if error or not studies:
                return {}
            study_info = {}
            cancer_type_client = get_cancer_type_client()

            for s in studies:
                cancer_type_id = s.get("cancerTypeId", "")
                if cancer_type_id and cancer_type_id != "unknown":
                    # Use the API to get the proper display name
                    cancer_type = (
                        await cancer_type_client.get_cancer_type_name(
                            cancer_type_id
                        )
                    )
                else:
                    # Try to get from full study info
                    cancer_type = (
                        await cancer_type_client.get_study_cancer_type(
                            s["studyId"]
                        )
                    )

                study_info[s["studyId"]] = {
                    "name": s.get("name", ""),
                    "cancer_type": cancer_type,
                }
            return study_info
        except Exception as e:
            logger.error(f"Error fetching studies: {e}")
            return {}

    def _aggregate_by_study(
        self,
        mutations: list[MutationHit],
        studies_info: dict[str, dict[str, Any]],
    ) -> dict[str, StudyMutationSummary]:
        """Aggregate mutations by study."""
        study_mutations = defaultdict(list)
        study_samples = defaultdict(set)

        for mut in mutations:
            study_id = mut.study_id
            study_mutations[study_id].append(mut.protein_change)
            if mut.sample_id:
                study_samples[study_id].add(mut.sample_id)

        # Create summaries
        summaries = {}
        for study_id, mutations_list in study_mutations.items():
            info = studies_info.get(study_id, {})
            summaries[study_id] = StudyMutationSummary(
                study_id=study_id,
                study_name=info.get("name", study_id),
                cancer_type=info.get("cancer_type", "unknown"),
                mutation_count=len(mutations_list),
                sample_count=len(study_samples[study_id]),
                mutations=list(set(mutations_list))[
                    :5
                ],  # Top 5 unique mutations
            )

        return summaries


def format_mutation_search_result(result: MutationSearchResult) -> str:
    """Format mutation search results as markdown."""
    lines = [f"### cBioPortal Mutation Search: {result.gene}"]

    if result.specific_mutation:
        lines.append(f"**Specific Mutation**: {result.specific_mutation}")
    elif result.pattern:
        lines.append(f"**Pattern**: {result.pattern}")

    lines.extend([
        f"- **Total Studies**: {result.total_studies}",
        f"- **Studies with Mutation**: {result.studies_with_mutation}",
        f"- **Total Mutations Found**: {result.total_mutations}",
    ])

    if result.top_studies:
        lines.append("\n**Top Studies by Mutation Count:**")
        lines.append("| Count | Study ID | Cancer Type | Study Name |")
        lines.append("|-------|----------|-------------|------------|")

        for study in result.top_studies[:10]:
            study_id = (
                study.study_id[:20] + "..."
                if len(study.study_id) > 20
                else study.study_id
            )
            study_name = (
                study.study_name[:40] + "..."
                if len(study.study_name) > 40
                else study.study_name
            )
            lines.append(
                f"| {study.mutation_count:5d} | {study_id:<20} | "
                f"{study.cancer_type:<11} | {study_name} |"
            )

    if result.mutation_types and len(result.mutation_types) > 1:
        lines.append("\n**Mutation Types Found:**")
        for mut_type, count in list(result.mutation_types.items())[:5]:
            lines.append(f"- {mut_type}: {count} occurrences")

    return "\n".join(lines)
