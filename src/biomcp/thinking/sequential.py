"""Sequential thinking module for BioMCP."""

from typing import Annotated, Any, Optional

from .. import mcp_app

# Global state for thought management
thought_history: list[dict[str, Any]] = []
thought_branches: dict[str, list[dict[str, Any]]] = {}


def get_current_timestamp() -> str:
    """Get current timestamp in ISO format."""
    from datetime import datetime

    return datetime.now().isoformat()


def add_thought_to_history(entry: dict[str, Any]) -> None:
    """Add a thought entry to the main history."""
    global thought_history

    # If this is a revision, replace the original thought
    if entry.get("isRevision") and entry.get("revisesThought"):
        revised_thought_num = entry["revisesThought"]
        for i, thought in enumerate(thought_history):
            if thought["thoughtNumber"] == revised_thought_num:
                thought_history[i] = entry
                return

    # Otherwise, add to history
    thought_history.append(entry)


def add_thought_to_branch(entry: dict[str, Any]) -> None:
    """Add a thought entry to a specific branch."""
    global thought_branches

    branch_id = entry.get("branchId")
    if not branch_id:
        return

    if branch_id not in thought_branches:
        thought_branches[branch_id] = []

    thought_branches[branch_id].append(entry)


@mcp_app.tool()
async def sequential_thinking(
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
        Optional[int], "The thought number being revised"
    ] = None,
    branchFromThought: Annotated[
        Optional[int], "Create alternative path from this thought number"
    ] = None,
    needsMoreThoughts: Annotated[
        Optional[bool],
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

    # Create thought entry
    entry: dict[str, Any] = {
        "thought": thought,
        "thoughtNumber": thoughtNumber,
        "totalThoughts": totalThoughts,
        "nextThoughtNeeded": nextThoughtNeeded,
        "isRevision": isRevision,
        "revisesThought": revisesThought,
        "branchFromThought": branchFromThought,
        "branchId": f"branch_{branchFromThought}"
        if branchFromThought
        else None,
        "needsMoreThoughts": needsMoreThoughts,
        "timestamp": get_current_timestamp(),
    }

    # Store in appropriate location
    if branchFromThought:
        branch_id = f"branch_{branchFromThought}"
        if branch_id not in thought_branches:
            thought_branches[branch_id] = []
        thought_branches[branch_id].append(entry)
        status_msg = f"Added thought {thoughtNumber} to branch '{branch_id}'"
    else:
        # If this is a revision, replace the original thought
        if isRevision and revisesThought:
            for i, thought_item in enumerate(thought_history):
                if thought_item["thoughtNumber"] == revisesThought:
                    thought_history[i] = entry
                    status_msg = f"Revised thought {revisesThought} (now thought {thoughtNumber})"
                    break
            else:
                # If original thought not found, just append
                thought_history.append(entry)
                status_msg = f"Added thought {thoughtNumber} to main sequence (revision target not found)"
        else:
            thought_history.append(entry)
            status_msg = f"Added thought {thoughtNumber} to main sequence"

    # Generate progress information
    progress_msg = f"Progress: {thoughtNumber}/{totalThoughts} thoughts"
    next_msg = (
        "Next thought needed"
        if nextThoughtNeeded
        else "Thinking sequence complete"
    )

    return f"{status_msg}. {progress_msg}. {next_msg}."
