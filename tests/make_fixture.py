"""Environment ownership for synthetic Make and lint contract fixtures."""
from __future__ import annotations

import os
from pathlib import Path
import sys
from collections.abc import Mapping


def make_fixture_env(
    root: Path, extra: Mapping[str, str] | None = None, *, commands: tuple[Path, ...] = (),
) -> dict[str, str]:
    """Keep host basics, but never inherit Make controls or gate/tool settings."""
    # An allowlist also excludes recipe variables such as SPEC_PROFILE and
    # TEST_LANE_WORKERS, not just GNU Make's own recursive controls.
    basics = ("SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT", "LANG", "LC_ALL", "TZ")
    env = {key: os.environ[key] for key in basics if key in os.environ}
    home = root.parent / (root.name + "-home")
    home.mkdir(exist_ok=True)
    temporary = home / "tmp"
    temporary.mkdir(exist_ok=True)
    env.update({
        "HOME": str(home), "XDG_CONFIG_HOME": str(home / "config"),
        "XDG_CACHE_HOME": str(home / "cache"), "TMPDIR": str(temporary),
        "GIT_CONFIG_NOSYSTEM": "1",
        "BIOMCP_TOOL_DIR": str(root / "tools"),
        "PATH": os.pathsep.join(map(str, (*commands, root / "tools", Path(sys.executable).parent)))
        + os.pathsep + os.defpath,
    })
    env.update(extra or {})
    return env
