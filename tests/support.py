"""Shared wait helpers for pytest: scaled watchdogs and procfs liveness.

`wait_until` replaces hand-rolled poll loops: the poll interval is
fixed and short, the window is a watchdog stretched once per process
by BIOMCP_TEST_TIMEOUT_SCALE (default 1.0). `proc_alive` reads
/proc/<pid>/stat, so liveness assertions observe the kernel's view
(alive and not a zombie) instead of sampling side-effect files.
"""

from __future__ import annotations

import os
import time

_SCALE: float | None = None


def _timeout_scale() -> float:
    global _SCALE
    if _SCALE is None:
        try:
            factor = float(os.environ.get("BIOMCP_TEST_TIMEOUT_SCALE", "1.0"))
        except ValueError:
            factor = 1.0
        _SCALE = factor if factor > 0 else 1.0
    return _SCALE


def wait_until(
    predicate: object,
    watchdog_secs: float = 10.0,
    poll_secs: float = 0.05,
) -> None:
    """Assert `predicate()` becomes true inside a scaled watchdog window."""
    deadline = time.monotonic() + watchdog_secs * _timeout_scale()
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(poll_secs)  # watchdog: shared poll interval inside wait_until
    assert predicate(), (
        f"condition not met within {watchdog_secs}s watchdog "
        f"(BIOMCP_TEST_TIMEOUT_SCALE={_timeout_scale()})"
    )


def proc_alive(pid: int) -> bool:
    """True when /proc/<pid>/stat exists and the process is not a zombie."""
    try:
        with open(f"/proc/{pid}/stat", encoding="utf-8") as handle:
            stat = handle.read()
    except OSError:
        return False
    try:
        state = stat[stat.rindex(")") + 2 :].split(maxsplit=1)[0]
    except (ValueError, IndexError):
        return False
    return state not in ("Z", "X", "x")
