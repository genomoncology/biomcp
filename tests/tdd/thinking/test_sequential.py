"""Tests for sequential thinking functionality."""

from datetime import datetime

import pytest

from biomcp.thinking import sequential


@pytest.fixture(autouse=True)
def clear_thinking_state():
    """Clear thinking state before each test."""
    sequential.thought_history.clear()
    sequential.thought_branches.clear()
    yield
    sequential.thought_history.clear()
    sequential.thought_branches.clear()


class TestSequentialThinking:
    """Test the sequential thinking MCP tool."""

    @pytest.mark.anyio
    async def test_basic_sequential_thinking(self):
        """Test basic sequential thinking flow."""
        result = await sequential.sequential_thinking(
            thought="First step: analyze the problem",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=3,
        )

        assert "Added thought 1 to main sequence" in result
        assert "Progress: 1/3 thoughts" in result
        assert "Next thought needed" in result
        assert len(sequential.thought_history) == 1

        # Verify thought structure
        thought = sequential.thought_history[0]
        assert thought["thought"] == "First step: analyze the problem"
        assert thought["thoughtNumber"] == 1
        assert thought["totalThoughts"] == 3
        assert thought["nextThoughtNeeded"] is True
        assert thought["isRevision"] is False

    @pytest.mark.anyio
    async def test_multiple_sequential_thoughts(self):
        """Test adding multiple thoughts in sequence."""
        # Add first thought
        await sequential.sequential_thinking(
            thought="First step",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=3,
        )

        # Add second thought
        await sequential.sequential_thinking(
            thought="Second step",
            nextThoughtNeeded=True,
            thoughtNumber=2,
            totalThoughts=3,
        )

        # Add final thought
        result = await sequential.sequential_thinking(
            thought="Final step",
            nextThoughtNeeded=False,
            thoughtNumber=3,
            totalThoughts=3,
        )

        assert "Added thought 3 to main sequence" in result
        assert "Thinking sequence complete" in result
        assert len(sequential.thought_history) == 3

    @pytest.mark.anyio
    async def test_thought_revision(self):
        """Test revising a previous thought."""
        # Add initial thought
        await sequential.sequential_thinking(
            thought="Initial analysis",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=2,
        )

        # Revise the thought
        result = await sequential.sequential_thinking(
            thought="Better analysis",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=2,
            isRevision=True,
            revisesThought=1,
        )

        assert "Revised thought 1" in result
        assert len(sequential.thought_history) == 1
        assert sequential.thought_history[0]["thought"] == "Better analysis"
        assert sequential.thought_history[0]["isRevision"] is True

    @pytest.mark.anyio
    async def test_branching_logic(self):
        """Test creating thought branches."""
        # Add main sequence thoughts
        await sequential.sequential_thinking(
            thought="Main thought 1",
            nextThoughtNeeded=True,
            thoughtNumber=1,
            totalThoughts=3,
        )

        await sequential.sequential_thinking(
            thought="Main thought 2",
            nextThoughtNeeded=True,
            thoughtNumber=2,
            totalThoughts=3,
        )

        # Create a branch
        result = await sequential.sequential_thinking(
            thought="Alternative approach",
            nextThoughtNeeded=True,
            thoughtNumber=3,
            totalThoughts=3,
            branchFromThought=2,
        )

        assert "Added thought 3 to branch 'branch_2'" in result
        assert len(sequential.thought_history) == 2
        assert len(sequential.thought_branches) == 1
        assert "branch_2" in sequential.thought_branches
        assert len(sequential.thought_branches["branch_2"]) == 1

    @pytest.mark.anyio
    async def test_validation_errors(self):
        """Test input validation errors."""
        # Test invalid thought number
        result = await sequential.sequential_thinking(
            thought="Test",
            nextThoughtNeeded=False,
            thoughtNumber=0,
            totalThoughts=1,
        )
        assert "thoughtNumber must be >= 1" in result

        # Test invalid total thoughts
        result = await sequential.sequential_thinking(
            thought="Test",
            nextThoughtNeeded=False,
            thoughtNumber=1,
            totalThoughts=0,
        )
        assert "totalThoughts must be >= 1" in result

        # Test revision without specifying which thought
        result = await sequential.sequential_thinking(
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
        result = await sequential.sequential_thinking(
            thought="This problem is more complex than expected",
            nextThoughtNeeded=True,
            thoughtNumber=3,
            totalThoughts=3,
            needsMoreThoughts=True,
        )

        assert "Added thought 3 to main sequence" in result
        assert len(sequential.thought_history) == 1
        assert sequential.thought_history[0]["needsMoreThoughts"] is True


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

    def test_helper_functions(self):
        """Test helper functions for CLI support."""
        # Test add_thought_to_history
        entry = {
            "thought": "Test thought",
            "thoughtNumber": 1,
            "totalThoughts": 1,
            "nextThoughtNeeded": False,
            "timestamp": sequential.get_current_timestamp(),
        }
        sequential.add_thought_to_history(entry)
        assert len(sequential.thought_history) == 1
        assert sequential.thought_history[0]["thought"] == "Test thought"

        # Test add_thought_to_branch
        branch_entry = {
            "thought": "Branch thought",
            "thoughtNumber": 2,
            "totalThoughts": 2,
            "nextThoughtNeeded": False,
            "branchId": "test-branch",
            "timestamp": sequential.get_current_timestamp(),
        }
        sequential.add_thought_to_branch(branch_entry)
        assert len(sequential.thought_branches) == 1
        assert "test-branch" in sequential.thought_branches
        assert len(sequential.thought_branches["test-branch"]) == 1
