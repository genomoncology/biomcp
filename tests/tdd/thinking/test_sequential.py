"""Tests for sequential thinking functionality."""

from datetime import datetime

import pytest

from biomcp.thinking import sequential
from biomcp.thinking.session import ThoughtEntry, _session_manager


@pytest.fixture(autouse=True)
def clear_thinking_state():
    """Clear thinking state before each test."""
    _session_manager.clear_all_sessions()
    yield
    _session_manager.clear_all_sessions()


class TestSequentialThinking:
    """Test the sequential thinking MCP tool."""

    @pytest.mark.anyio
    async def test_basic_sequential_thinking(self):
        """Test basic sequential thinking flow."""
        result = await sequential._sequential_thinking(
            thought="First step: analyze the problem",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=3,
        )

        assert "Added thought 1 to main sequence" in result
        assert "Progress: 1/3 thoughts" in result
        assert "Next thought needed" in result

        # Get current session
        session = _session_manager.get_session()
        assert session is not None
        assert len(session.thought_history) == 1

        # Verify thought structure
        thought = session.thought_history[0]
        assert thought.thought == "First step: analyze the problem"
        assert thought.thought_number == 1
        assert thought.total_thoughts == 3
        assert thought.next_thought_needed is True
        assert thought.is_revision is False

    @pytest.mark.anyio
    async def test_multiple_sequential_thoughts(self):
        """Test adding multiple thoughts in sequence."""
        # Add first thought
        await sequential._sequential_thinking(
            thought="First step",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=3,
        )

        # Add second thought
        await sequential._sequential_thinking(
            thought="Second step",
            nextThoughtNeeded=True,
            thoughtNumber=2,
            totalThoughts=3,
        )

        # Add final thought
        result = await sequential._sequential_thinking(
            thought="Final step",
            nextThoughtNeeded=False,
            thoughtNumber=3,
            totalThoughts=3,
        )

        assert "Added thought 3 to main sequence" in result
        assert "Thinking sequence complete" in result
        session = _session_manager.get_session()
        assert len(session.thought_history) == 3

    @pytest.mark.anyio
    async def test_thought_revision(self):
        """Test revising a previous thought."""
        # Add initial thought
        await sequential._sequential_thinking(
            thought="Initial analysis",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=2,
        )

        # Revise the thought
        result = await sequential._sequential_thinking(
            thought="Better analysis",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=2,
            isRevision=True,
            revisesThought=1,
        )

        assert "Revised thought 1" in result
        session = _session_manager.get_session()
        assert len(session.thought_history) == 1
        assert session.thought_history[0].thought == "Better analysis"
        assert session.thought_history[0].is_revision is True

    @pytest.mark.anyio
    async def test_branching_logic(self):
        """Test creating thought branches."""
        # Add main sequence thoughts
        await sequential._sequential_thinking(
            thought="Main thought 1",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=3,
        )

        await sequential._sequential_thinking(
            thought="Main thought 2",
            nextThoughtNeeded=True,
            thoughtNumber=2,
            totalThoughts=3,
        )

        # Create a branch
        result = await sequential._sequential_thinking(
            thought="Alternative approach",
            nextThoughtNeeded=True,
            thoughtNumber=3,
            totalThoughts=3,
            branchFromThought=2,
        )

        assert "Added thought 3 to branch 'branch_2'" in result
        session = _session_manager.get_session()
        assert len(session.thought_history) == 2
        assert len(session.thought_branches) == 1
        assert "branch_2" in session.thought_branches
        assert len(session.thought_branches["branch_2"]) == 1

    @pytest.mark.anyio
    async def test_validation_errors(self):
        """Test input validation errors."""
        # Test invalid thought number
        result = await sequential._sequential_thinking(
            thought="Test",
            nextThoughtNeeded=False,
            thoughtNumber=0,
            totalThoughts=1,
        )
        assert "thoughtNumber must be >= 1" in result

        # Test invalid total thoughts
        result = await sequential._sequential_thinking(
            thought="Test",
            nextThoughtNeeded=False,
            thoughtNumber=1,
            totalThoughts=0,
        )
        assert "totalThoughts must be >= 1" in result

        # Test revision without specifying which thought
        result = await sequential._sequential_thinking(
            thought="Test",
            nextThoughtNeeded=False,
            thoughtNumber=1,
            totalThoughts=1,
            isRevision=True,
        )
        assert (
            "revisesThought must be specified when isRevision=True" in result
        )

    @pytest.mark.anyio
    async def test_needs_more_thoughts(self):
        """Test the needsMoreThoughts parameter."""
        result = await sequential._sequential_thinking(
            thought="This problem is more complex than expected",
            nextThoughtNeeded=True,
            thoughtNumber=3,
            totalThoughts=3,
            needsMoreThoughts=True,
        )

        assert "Added thought 3 to main sequence" in result
        session = _session_manager.get_session()
        assert len(session.thought_history) == 1
        assert (
            session.thought_history[0].metadata.get("needsMoreThoughts")
            is True
        )


class TestUtilityFunctions:
    """Test utility functions."""

    def test_get_current_timestamp(self):
        """Test timestamp generation."""
        timestamp = sequential.get_current_timestamp()
        assert isinstance(timestamp, str)
        # Should be able to parse as ISO format
        parsed = datetime.fromisoformat(
            timestamp.replace("Z", "+00:00").replace("T", " ").split(".")[0]
        )
        assert isinstance(parsed, datetime)

    def test_session_management(self):
        """Test session management functionality."""
        # Clear any existing sessions
        _session_manager.clear_all_sessions()

        # Create a new session
        session = _session_manager.create_session()
        assert session is not None
        assert session.session_id is not None

        # Add a thought entry
        entry = ThoughtEntry(
            thought="Test thought",
            thought_number=1,
            total_thoughts=1,
            next_thought_needed=False,
        )
        session.add_thought(entry)
        assert len(session.thought_history) == 1
        assert session.thought_history[0].thought == "Test thought"

        # Test branch creation
        branch_entry = ThoughtEntry(
            thought="Branch thought",
            thought_number=2,
            total_thoughts=2,
            next_thought_needed=False,
            branch_id="test-branch",
            branch_from_thought=1,
        )
        session.add_thought(branch_entry)
        assert len(session.thought_branches) == 1
        assert "test-branch" in session.thought_branches
        assert len(session.thought_branches["test-branch"]) == 1
