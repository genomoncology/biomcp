"""Helper functions for cBioPortal search to reduce complexity."""

import logging
import re
from typing import Any

from .cbioportal_search import GeneHotspot

logger = logging.getLogger(__name__)


async def process_mutation_results(
    mutation_results: list[tuple[Any, str]],
    cancer_types_lookup: dict[str, dict[str, Any]],
    client: Any,
) -> dict[str, Any]:
    """Process mutation results from multiple studies.

    Args:
        mutation_results: List of (result, study_id) tuples
        cancer_types_lookup: Cancer type lookup dictionary
        client: Client instance for API calls

    Returns:
        Dictionary with aggregated mutation data
    """
    total_mutations = 0
    total_samples = 0
    hotspot_counts: dict[str, dict[str, Any]] = {}
    cancer_distribution: dict[str, int] = {}
    studies_with_data = 0

    for result, study_id in mutation_results:
        if isinstance(result, Exception):
            logger.debug(f"Failed to get mutations for {study_id}: {result}")
            continue

        if result and "mutations" in result:
            mutations = result["mutations"]
            sample_count = result["sample_count"]

            if mutations:
                studies_with_data += 1
                # Count unique samples with mutations
                unique_samples = {
                    m.get("sampleId") for m in mutations if m.get("sampleId")
                }
                total_mutations += len(unique_samples)
                total_samples += sample_count

                # Process mutations for hotspots and cancer types
                study_cancer_type = await client._get_study_cancer_type(
                    study_id, cancer_types_lookup
                )
                _update_hotspot_counts(
                    mutations, hotspot_counts, study_cancer_type
                )
                _update_cancer_distribution(
                    mutations, cancer_distribution, study_cancer_type
                )

    return {
        "total_mutations": total_mutations,
        "total_samples": total_samples,
        "studies_with_data": studies_with_data,
        "hotspot_counts": hotspot_counts,
        "cancer_distribution": cancer_distribution,
    }


def _update_hotspot_counts(
    mutations: list[dict[str, Any]],
    hotspot_counts: dict[str, dict[str, Any]],
    cancer_type: str,
) -> None:
    """Update hotspot counts from mutations."""
    for mut in mutations:
        protein_change = mut.get("proteinChange", "")
        if protein_change:
            if protein_change not in hotspot_counts:
                hotspot_counts[protein_change] = {
                    "count": 0,
                    "cancer_types": set(),
                }
            hotspot_counts[protein_change]["count"] += 1
            hotspot_counts[protein_change]["cancer_types"].add(cancer_type)


def _update_cancer_distribution(
    mutations: list[dict[str, Any]],
    cancer_distribution: dict[str, int],
    cancer_type: str,
) -> None:
    """Update cancer type distribution."""
    cancer_distribution[cancer_type] = cancer_distribution.get(
        cancer_type, 0
    ) + len({m.get("sampleId") for m in mutations if m.get("sampleId")})


def format_hotspots(
    hotspot_counts: dict[str, dict[str, Any]], total_mutations: int
) -> list[GeneHotspot]:
    """Format hotspot counts into GeneHotspot objects."""
    hotspots = []

    for protein_change, data in sorted(
        hotspot_counts.items(), key=lambda x: x[1]["count"], reverse=True
    )[:5]:  # Top 5 hotspots
        # Try to extract position from protein change
        position = 0
        try:
            match = re.search(r"(\d+)", protein_change)
            if match:
                position = int(match.group(1))
        except Exception:
            logger.debug("Failed to extract position from protein change")

        hotspots.append(
            GeneHotspot(
                position=position,
                amino_acid_change=protein_change,
                count=data["count"],
                frequency=data["count"] / total_mutations
                if total_mutations > 0
                else 0.0,
                cancer_types=list(data["cancer_types"]),
            )
        )

    return hotspots
