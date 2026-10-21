"""Share the routine runner's timing policy with its test fixtures."""

from pathlib import Path
import tomllib

_CONFIG = tomllib.loads(
    (Path(__file__).resolve().parents[1] / ".config/nextest.toml").read_text()
)["profile"]["default"]["slow-timeout"]
SLOW_WARNING_SECONDS = float(_CONFIG["period"].removesuffix("s"))
HANG_TIMEOUT = SLOW_WARNING_SECONDS * _CONFIG["terminate-after"]
