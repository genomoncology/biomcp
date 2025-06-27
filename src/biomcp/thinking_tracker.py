"""Track thinking tool usage within MCP sessions.

This module provides a simple mechanism to track whether the think tool
has been used in the current session, encouraging AI clients to follow
best practices.
"""

from contextvars import ContextVar

# Track if thinking has been used in current context
thinking_used: ContextVar[bool] = ContextVar("thinking_used", default=False)


def mark_thinking_used() -> None:
    """Mark that the thinking tool has been used."""
    thinking_used.set(True)


def has_thinking_been_used() -> bool:
    """Check if thinking tool has been used in current context."""
    return thinking_used.get()


def reset_thinking_tracker() -> None:
    """Reset the thinking tracker (for testing)."""
    thinking_used.set(False)


def get_thinking_reminder() -> str:
    """Get a reminder message if thinking hasn't been used."""
    if not has_thinking_been_used():
        return (
            "\n\n⚠️ **REMINDER**: You haven't used the 'think' tool yet! "
            "For optimal results, please use 'think' BEFORE searching to plan "
            "your research strategy and ensure comprehensive analysis."
        )
    return ""
