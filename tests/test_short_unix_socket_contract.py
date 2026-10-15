from __future__ import annotations

from pathlib import Path

import pytest

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module

# The Rust test at src/cache/clear.rs already runs in the nextest
# archive; this module keeps only the source pin, so the Python lane
# never reruns a Rust test through cargo.


ROOT = Path(__file__).resolve().parents[1]


def test_socket_fixture_uses_unique_short_owned_root() -> None:
    source = (ROOT / "src/cache/clear.rs").read_text()
    assert '.prefix("biomcp-sock-")' in source
    assert '.tempdir_in("/tmp")' in source
    assert "socket_path.as_os_str().len() < 100" in source
