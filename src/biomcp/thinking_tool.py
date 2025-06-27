"""Sequential thinking tool for structured problem-solving.

This module provides a dedicated MCP tool for sequential thinking,
separate from the main search functionality.
"""

from typing import Annotated

from pydantic import Field

from biomcp.core import mcp_app
from biomcp.metrics import track_performance
from biomcp.thinking.sequential import _sequential_thinking
from biomcp.thinking_tracker import mark_thinking_used


@mcp_app.tool()
@track_performance("biomcp.think")
async def think(
    thought: Annotated[
        str,
        Field(description="Current thinking step for analysis"),
    ],
    thoughtNumber: Annotated[
        int,
        Field(
            description="Current thought number, starting at 1",
            ge=1,
        ),
    ],
    totalThoughts: Annotated[
        int,
        Field(
            description="Estimated total thoughts needed for complete analysis",
            ge=1,
        ),
    ],
    nextThoughtNeeded: Annotated[
        bool,
        Field(
            description="Whether more thinking steps are needed after this one",
        ),
    ] = True,
) -> dict:
    """REQUIRED FIRST STEP: Perform structured sequential thinking for ANY biomedical research task.

    🚨 IMPORTANT: You MUST use this tool BEFORE any search or fetch operations when:
    - Researching ANY biomedical topic (genes, diseases, variants, trials)
    - Planning to use multiple BioMCP tools
    - Answering questions that require analysis or synthesis
    - Comparing information from different sources
    - Making recommendations or drawing conclusions

    ⚠️ FAILURE TO USE THIS TOOL FIRST will result in:
    - Incomplete or poorly structured analysis
    - Missing important connections between data
    - Suboptimal search strategies
    - Overlooked critical information

    Sequential thinking ensures you:
    1. Fully understand the research question
    2. Plan an optimal search strategy
    3. Identify all relevant data sources
    4. Structure your analysis properly
    5. Deliver comprehensive, well-reasoned results

    ## Usage Pattern:
    1. Start with thoughtNumber=1 to initiate analysis
    2. Progress through numbered thoughts sequentially
    3. Adjust totalThoughts estimate as understanding develops
    4. Set nextThoughtNeeded=False only when analysis is complete

    ## Example:
    ```python
    # Initial analysis
    await think(
        thought="Breaking down the relationship between BRAF mutations and melanoma treatment resistance...",
        thoughtNumber=1,
        totalThoughts=5,
        nextThoughtNeeded=True
    )

    # Continue analysis
    await think(
        thought="Examining specific BRAF V600E mutation mechanisms...",
        thoughtNumber=2,
        totalThoughts=5,
        nextThoughtNeeded=True
    )

    # Final thought
    await think(
        thought="Synthesizing findings and proposing research directions...",
        thoughtNumber=5,
        totalThoughts=5,
        nextThoughtNeeded=False
    )
    ```

    ## Important Notes:
    - Each thought builds on previous ones within a session
    - State is maintained throughout the MCP session
    - Use thoughtful, detailed analysis in each step
    - Revisions and branching are supported through the underlying implementation
    """
    # Mark that thinking has been used
    mark_thinking_used()

    result = await _sequential_thinking(
        thought=thought,
        thoughtNumber=thoughtNumber,
        totalThoughts=totalThoughts,
        nextThoughtNeeded=nextThoughtNeeded,
    )

    return {
        "domain": "thinking",
        "result": result,
        "thoughtNumber": thoughtNumber,
        "nextThoughtNeeded": nextThoughtNeeded,
    }
