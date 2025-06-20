"""Session management for sequential thinking."""

import uuid
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any


@dataclass
class ThoughtEntry:
    """Represents a single thought in the thinking process."""

    thought: str
    thought_number: int
    total_thoughts: int
    next_thought_needed: bool
    timestamp: datetime = field(default_factory=datetime.now)
    is_revision: bool = False
    revises_thought: int | None = None
    branch_from_thought: int | None = None
    branch_id: str | None = None
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class ThinkingSession:
    """Manages state for a thinking session."""

    session_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    created_at: datetime = field(default_factory=datetime.now)
    thought_history: list[ThoughtEntry] = field(default_factory=list)
    thought_branches: dict[str, list[ThoughtEntry]] = field(
        default_factory=lambda: defaultdict(list)
    )
    metadata: dict[str, Any] = field(default_factory=dict)

    def add_thought(self, entry: ThoughtEntry) -> None:
        """Add a thought to the session."""
        # If this is a revision, replace the original thought
        if entry.is_revision and entry.revises_thought:
            for i, thought in enumerate(self.thought_history):
                if thought.thought_number == entry.revises_thought:
                    self.thought_history[i] = entry
                    return

        # Add to appropriate collection
        if entry.branch_id:
            self.thought_branches[entry.branch_id].append(entry)
        else:
            self.thought_history.append(entry)

    def get_thought(self, thought_number: int) -> ThoughtEntry | None:
        """Get a specific thought by number."""
        for thought in self.thought_history:
            if thought.thought_number == thought_number:
                return thought
        return None

    def get_branch_thoughts(self, branch_id: str) -> list[ThoughtEntry]:
        """Get all thoughts in a specific branch."""
        return self.thought_branches.get(branch_id, [])

    def get_all_thoughts(self) -> list[ThoughtEntry]:
        """Get all thoughts across main history and branches."""
        all_thoughts = list(self.thought_history)
        for branch_thoughts in self.thought_branches.values():
            all_thoughts.extend(branch_thoughts)
        return sorted(all_thoughts, key=lambda t: t.timestamp)


class SessionManager:
    """Manages multiple thinking sessions."""

    def __init__(self):
        self.sessions: dict[str, ThinkingSession] = {}
        self._current_session_id: str | None = None

    def create_session(self) -> ThinkingSession:
        """Create a new thinking session."""
        session = ThinkingSession()
        self.sessions[session.session_id] = session
        self._current_session_id = session.session_id
        return session

    def get_session(
        self, session_id: str | None = None
    ) -> ThinkingSession | None:
        """Get a session by ID or the current session."""
        if session_id:
            return self.sessions.get(session_id)
        elif self._current_session_id:
            return self.sessions.get(self._current_session_id)
        return None

    def get_or_create_session(
        self, session_id: str | None = None
    ) -> ThinkingSession:
        """Get existing session or create new one."""
        if session_id and session_id in self.sessions:
            self._current_session_id = session_id
            return self.sessions[session_id]

        session = self.get_session()
        if not session:
            session = self.create_session()
        return session

    def clear_session(self, session_id: str | None = None) -> None:
        """Clear a specific session or the current session."""
        if session_id:
            self.sessions.pop(session_id, None)
            if self._current_session_id == session_id:
                self._current_session_id = None
        elif self._current_session_id:
            self.sessions.pop(self._current_session_id, None)
            self._current_session_id = None

    def clear_all_sessions(self) -> None:
        """Clear all sessions."""
        self.sessions.clear()
        self._current_session_id = None


# Global session manager instance
_session_manager = SessionManager()
