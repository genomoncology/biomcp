"""AlphaGenome integration for variant effect prediction."""

import logging
import os
import re
from typing import Any, TypedDict

from ..utils.request_cache import request_cache

logger = logging.getLogger(__name__)

# Default threshold for significant changes
DEFAULT_SIGNIFICANCE_THRESHOLD = 0.5

# Chromosome pattern for validation
CHROMOSOME_PATTERN = re.compile(r"^chr([1-9]|1[0-9]|2[0-2]|X|Y|M|MT)$")

# Valid nucleotide characters
VALID_NUCLEOTIDES = set("ACGT")


class VariantPrediction(TypedDict):
    """Type definition for variant prediction results."""

    gene_expression: dict[str, float]
    chromatin_accessibility: dict[str, float]
    splicing_effects: list[str]
    summary_stats: dict[str, int]


@request_cache(ttl=1800)  # Cache for 30 minutes
async def predict_variant_effects(
    chromosome: str,
    position: int,
    reference: str,
    alternate: str,
    interval_size: int = 131_072,
    tissue_types: list[str] | None = None,
    significance_threshold: float = DEFAULT_SIGNIFICANCE_THRESHOLD,
    api_key: str | None = None,
) -> str:
    """
    Predict variant effects using AlphaGenome.

    Args:
        chromosome: Chromosome (e.g., 'chr7')
        position: 1-based genomic position
        reference: Reference allele(s)
        alternate: Alternate allele(s)
        interval_size: Size of genomic context window (max 1,000,000)
        tissue_types: Optional UBERON ontology terms for tissue-specific predictions
        significance_threshold: Threshold for significant changes (default 0.5)
        api_key: Optional API key (if not provided, uses ALPHAGENOME_API_KEY env var)

    Returns:
        Formatted markdown string with predictions

    Raises:
        ValueError: If input parameters are invalid
    """
    # Validate inputs
    _validate_inputs(chromosome, position, reference, alternate)

    # Check for API key (prefer parameter over environment variable)
    if not api_key:
        api_key = os.getenv("ALPHAGENOME_API_KEY")

    if not api_key:
        return (
            "❌ **AlphaGenome API key required**\n\n"
            "I need an API key to use AlphaGenome. Please provide it by either:\n\n"
            "**Option 1: Include your key in your request**\n"
            'Say: "My AlphaGenome API key is YOUR_KEY_HERE" and I\'ll use it for this prediction.\n\n'
            "**Option 2: Set it as an environment variable (for persistent use)**\n"
            "```bash\n"
            "export ALPHAGENOME_API_KEY='your-key'\n"
            "```\n\n"
            "Get a free API key at: https://deepmind.google.com/science/alphagenome\n\n"
            "**ACTION REQUIRED**: Please provide your API key using Option 1 above to continue."
        )

    # Try to import AlphaGenome
    try:
        from alphagenome.data import genome
        from alphagenome.models import dna_client, variant_scorers
    except ImportError:
        return (
            "❌ **AlphaGenome not installed**\n\n"
            "To install:\n"
            "```bash\n"
            "git clone https://github.com/google-deepmind/alphagenome.git\n"
            "cd alphagenome && pip install .\n"
            "```\n\n"
            "Standard variant annotations are still available via `variant_searcher`."
        )

    try:
        # Create client
        model = dna_client.create(api_key)

        # Calculate interval boundaries (ensure within supported sizes)
        # Supported sizes: 2048, 16384, 131072, 524288, 1048576
        supported_sizes = [2048, 16384, 131072, 524288, 1048576]

        # Find smallest supported size that's >= requested size
        valid_sizes = [s for s in supported_sizes if s >= interval_size]
        if not valid_sizes:
            # If requested size is larger than max, use max
            interval_size = supported_sizes[-1]
        else:
            interval_size = min(valid_sizes)

        half_size = interval_size // 2
        interval_start = max(0, position - half_size - 1)  # Convert to 0-based
        interval_end = interval_start + interval_size

        # Create interval and variant objects
        interval = genome.Interval(
            chromosome=chromosome, start=interval_start, end=interval_end
        )

        variant = genome.Variant(
            chromosome=chromosome,
            position=position,
            reference_bases=reference,
            alternate_bases=alternate,
        )

        # Get recommended scorers for human
        scorers = variant_scorers.get_recommended_scorers(organism="human")

        # Make prediction
        scores = model.score_variant(
            interval=interval, variant=variant, variant_scorers=scorers
        )

        # Format results
        return _format_predictions(
            variant, scores, interval_size, significance_threshold
        )

    except Exception as e:
        logger.error(f"AlphaGenome prediction failed: {e}", exc_info=True)
        error_context = (
            f"❌ **AlphaGenome prediction failed**\n\n"
            f"Error: {e!s}\n\n"
            f"**Context:**\n"
            f"- Variant: {chromosome}:{position} {reference}>{alternate}\n"
            f"- Interval size: {interval_size:,} bp\n"
            f"- Tissue types: {tissue_types or 'None specified'}"
        )
        return error_context


def _format_predictions(
    variant: Any,
    scores: list[Any],
    interval_size: int,
    significance_threshold: float = DEFAULT_SIGNIFICANCE_THRESHOLD,
) -> str:
    """Format AlphaGenome predictions into markdown.

    Args:
        variant: The variant object from AlphaGenome
        scores: List of prediction scores
        interval_size: Size of the genomic context window
        significance_threshold: Threshold for significant changes

    Returns:
        Formatted markdown string
    """
    try:
        from alphagenome.models import variant_scorers

        # Convert scores to DataFrame
        scores_df = variant_scorers.tidy_scores(scores)

        # Start building the output
        lines = [
            "## AlphaGenome Variant Effect Predictions\n",
            f"**Variant**: {variant.chromosome}:{variant.position} {variant.reference_bases}>{variant.alternate_bases}",
            f"**Analysis window**: {interval_size:,} bp\n",
        ]

        # Group scores by output type
        if not scores_df.empty:
            # Gene expression effects
            expr_scores = scores_df[
                scores_df["output_type"].str.contains("RNA_SEQ", na=False)
            ]
            if not expr_scores.empty:
                top_expr = expr_scores.loc[
                    expr_scores["raw_score"].abs().idxmax()
                ]
                gene = top_expr.get("gene_name", "Unknown")
                score = top_expr["raw_score"]
                direction = "↓ decreases" if score < 0 else "↑ increases"
                lines.append("\n### Gene Expression")
                lines.append(
                    f"- **{gene}**: {score:+.2f} log₂ fold change ({direction} expression)"
                )

            # Chromatin accessibility
            chrom_scores = scores_df[
                scores_df["output_type"].str.contains("ATAC|DNASE", na=False)
            ]
            if not chrom_scores.empty:
                top_chrom = chrom_scores.loc[
                    chrom_scores["raw_score"].abs().idxmax()
                ]
                score = top_chrom["raw_score"]
                track = top_chrom.get("track_name", "tissue")
                direction = "↓ decreases" if score < 0 else "↑ increases"
                lines.append("\n### Chromatin Accessibility")
                lines.append(
                    f"- **{track}**: {score:+.2f} log₂ change ({direction} accessibility)"
                )

            # Splicing effects
            splice_scores = scores_df[
                scores_df["output_type"].str.contains("SPLICE", na=False)
            ]
            if not splice_scores.empty:
                lines.append("\n### Splicing")
                lines.append("- Potential splicing alterations detected")

            # Summary statistics
            total_tracks = len(scores_df)
            significant = len(
                scores_df[
                    scores_df["raw_score"].abs() > significance_threshold
                ]
            )
            lines.append("\n### Summary")
            lines.append(f"- Analyzed {total_tracks} regulatory tracks")
            lines.append(
                f"- {significant} tracks show substantial changes (|log₂| > {significance_threshold})"
            )
        else:
            lines.append("\n*No significant regulatory effects predicted*")

        return "\n".join(lines)

    except Exception as e:
        logger.error(f"Failed to format predictions: {e}")
        return f"## AlphaGenome Results\n\nPrediction completed but formatting failed: {e!s}"


def _validate_inputs(
    chromosome: str, position: int, reference: str, alternate: str
) -> None:
    """Validate input parameters for variant prediction.

    Args:
        chromosome: Chromosome identifier
        position: Genomic position
        reference: Reference allele(s)
        alternate: Alternate allele(s)

    Raises:
        ValueError: If any input is invalid
    """
    # Validate chromosome format
    if not CHROMOSOME_PATTERN.match(chromosome):
        raise ValueError(
            f"Invalid chromosome format: {chromosome}. "
            "Expected format: chr1-22, chrX, chrY, chrM, or chrMT"
        )

    # Validate position
    if position < 1:
        raise ValueError(f"Position must be >= 1, got {position}")

    # Validate nucleotides
    ref_upper = reference.upper()
    alt_upper = alternate.upper()

    if not ref_upper:
        raise ValueError("Reference allele cannot be empty")

    if not alt_upper:
        raise ValueError("Alternate allele cannot be empty")

    invalid_ref = set(ref_upper) - VALID_NUCLEOTIDES
    if invalid_ref:
        raise ValueError(
            f"Invalid nucleotides in reference allele: {invalid_ref}. "
            f"Only A, C, G, T are allowed"
        )

    invalid_alt = set(alt_upper) - VALID_NUCLEOTIDES
    if invalid_alt:
        raise ValueError(
            f"Invalid nucleotides in alternate allele: {invalid_alt}. "
            f"Only A, C, G, T are allowed"
        )
