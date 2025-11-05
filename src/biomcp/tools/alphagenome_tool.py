"""AlphaGenome tool for AI-powered variant regulatory effect predictions."""

import logging
from typing import Annotated

from pydantic import Field

from biomcp.core import ensure_list, mcp_app
from biomcp.metrics import track_performance

logger = logging.getLogger(__name__)


@mcp_app.tool()
@track_performance("biomcp.alphagenome")
async def alphagenome(
    chromosome: Annotated[
        str,
        Field(description="Chromosome (e.g., 'chr7', 'chrX')"),
    ],
    position: Annotated[
        int,
        Field(description="1-based genomic position of the variant"),
    ],
    reference: Annotated[
        str,
        Field(description="Reference allele(s) (e.g., 'A', 'ATG')"),
    ],
    alternate: Annotated[
        str,
        Field(description="Alternate allele(s) (e.g., 'T', 'A')"),
    ],
    interval_size: Annotated[
        int,
        Field(
            description="Size of genomic interval to analyze in bp (max 1,000,000)",
            ge=2000,
            le=1000000,
        ),
    ] = 131072,
    tissue_types: Annotated[
        list[str] | str | None,
        Field(
            description="UBERON ontology terms for tissue-specific predictions (e.g., 'UBERON:0002367' for external ear)"
        ),
    ] = None,
    significance_threshold: Annotated[
        float,
        Field(
            description="Threshold for significant log2 fold changes (default: 0.5)",
            ge=0.0,
            le=5.0,
        ),
    ] = 0.5,
    api_key: Annotated[
        str | None,
        Field(
            description="AlphaGenome API key. Check if user mentioned 'my AlphaGenome API key is...' in their message. If not provided here and no env var is set, user will be prompted to provide one."
        ),
    ] = None,
) -> str:
    """Predict variant effects on gene regulation using Google DeepMind's AlphaGenome.

    ⚠️ PREREQUISITE: Use the 'think' tool FIRST to plan your analysis strategy!

    AlphaGenome provides state-of-the-art AI predictions for how genetic variants
    affect gene regulation, including:
    - Gene expression changes (RNA-seq)
    - Chromatin accessibility impacts (ATAC-seq, DNase-seq)
    - Splicing alterations
    - Promoter activity changes (CAGE)

    ## Requirements:
    1. **AlphaGenome package must be installed**: `uv pip install alphagenome`
    2. **API key required** from https://deepmind.google.com/science/alphagenome

    ## API Key Options:
    - Provide directly via the `api_key` parameter
    - Or set ALPHAGENOME_API_KEY environment variable

    ## Use Cases:
    - Predict regulatory effects of coding variants (e.g., BRAF V600E)
    - Assess non-coding variant impact on gene expression
    - Evaluate promoter and enhancer variants
    - Analyze tissue-specific regulatory effects

    ## Examples:

    Predict BRAF V600E regulatory effects:
    ```python
    await alphagenome(
        chromosome="chr7",
        position=140753336,
        reference="A",
        alternate="T",
        api_key="YOUR_KEY"
    )
    ```

    Analyze promoter variant with tissue specificity:
    ```python
    await alphagenome(
        chromosome="chr17",
        position=7577000,
        reference="C",
        alternate="T",
        tissue_types=["UBERON:0002367"],  # external ear
        significance_threshold=1.0,
        api_key="YOUR_KEY"
    )
    ```

    ## Important Notes:
    - This is an **optional tool** requiring separate package installation
    - Predictions are AI-based and should be validated experimentally
    - For standard variant annotations, use the 'variant' tool
    - Processing time depends on interval_size and tissue types
    """
    logger.info(f"AlphaGenome tool called: {chromosome}:{position} {reference}>{alternate}")

    from biomcp.variants.alphagenome import predict_variant_effects

    # Convert tissue_types to list if needed
    tissue_types_list = ensure_list(tissue_types) if tissue_types else None

    return await predict_variant_effects(
        chromosome=chromosome,
        position=position,
        reference=reference,
        alternate=alternate,
        interval_size=interval_size,
        tissue_types=tissue_types_list,
        significance_threshold=significance_threshold,
        api_key=api_key,
    )
