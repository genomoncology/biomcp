"""Sequential thinking module for BioMCP."""

from typing import Annotated

from .session import ThoughtEntry, _session_manager


def get_current_timestamp() -> str:
    """Get current timestamp in ISO format."""
    from datetime import datetime

    return datetime.now().isoformat()


async def _sequential_thinking(
    thought: Annotated[
        str, "Current thinking step - be detailed and thorough"
    ],
    nextThoughtNeeded: Annotated[
        bool, "True if more thinking needed, False only when completely done"
    ],
    thoughtNumber: Annotated[int, "Current thought number (start at 1)"],
    totalThoughts: Annotated[
        int, "Best estimate of total thoughts (adjust as needed)"
    ],
    isRevision: Annotated[
        bool, "True when correcting/improving a previous thought"
    ] = False,
    revisesThought: Annotated[
        int | None, "The thought number being revised"
    ] = None,
    branchFromThought: Annotated[
        int | None, "Create alternative path from this thought number"
    ] = None,
    needsMoreThoughts: Annotated[
        bool | None,
        "True when problem is significantly larger than initially estimated",
    ] = None,
) -> str:
    """
    ALWAYS use this tool for complex reasoning, analysis, or problem-solving. This facilitates a detailed, step-by-step thinking process that helps break down problems systematically.

    Use this tool when:
    - Analyzing complex problems or questions
    - Planning multi-step solutions
    - Breaking down tasks into components
    - Reasoning through uncertainties
    - Exploring alternative approaches

    Start with thoughtNumber=1 and totalThoughts as your best estimate. Set nextThoughtNeeded=true to continue thinking, or false when done. You can revise earlier thoughts or branch into alternative paths as needed.

    This is your primary reasoning tool - USE IT LIBERALLY for any non-trivial thinking task.
    """

    # Validate inputs
    if thoughtNumber < 1:
        return "Error: thoughtNumber must be >= 1"

    if totalThoughts < 1:
        return "Error: totalThoughts must be >= 1"

    if isRevision and not revisesThought:
        return "Error: revisesThought must be specified when isRevision=True"

    # Get or create session
    session = _session_manager.get_or_create_session()

    # Create thought entry
    branch_id = f"branch_{branchFromThought}" if branchFromThought else None

    entry = ThoughtEntry(
        thought=thought,
        thought_number=thoughtNumber,
        total_thoughts=totalThoughts,
        next_thought_needed=nextThoughtNeeded,
        is_revision=isRevision,
        revises_thought=revisesThought,
        branch_from_thought=branchFromThought,
        branch_id=branch_id,
        metadata={"needsMoreThoughts": needsMoreThoughts}
        if needsMoreThoughts
        else {},
    )

    # Add thought to session
    session.add_thought(entry)

    # Generate status message
    if branchFromThought:
        status_msg = f"Added thought {thoughtNumber} to branch '{branch_id}'"
    elif isRevision and revisesThought:
        status_msg = (
            f"Revised thought {revisesThought} (now thought {thoughtNumber})"
        )
    else:
        status_msg = f"Added thought {thoughtNumber} to main sequence"

    # Generate progress information
    progress_msg = f"Progress: {thoughtNumber}/{totalThoughts} thoughts"
    next_msg = (
        "Next thought needed"
        if nextThoughtNeeded
        else "Thinking sequence complete"
    )

    return f"{status_msg}. {progress_msg}. {next_msg}."
