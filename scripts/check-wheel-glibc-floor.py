#!/usr/bin/env python3
"""Assert a wheel's ELF members reference no glibc symbol above a floor.

Usage: check-wheel-glibc-floor.py WHEEL [FLOOR]

FLOOR defaults to 2.28. Scans every member of the wheel for
``GLIBC_x.y`` byte markers (the dynamic symbol versions the binary
actually imports) and fails when the highest exceeds the floor. Runs
inside the manylinux build container so the floor is checked against
the artifact that ships.
"""

from __future__ import annotations

import re
import sys
import zipfile
from pathlib import Path

MARKER = re.compile(rb"GLIBC_([0-9]+)\.([0-9]+)")


def member_versions(data: bytes) -> list[float]:
    return [float(f"{major.decode()}.{minor.decode()}") for major, minor in MARKER.findall(data)]


def wheel_floor(wheel: Path) -> float:
    top = 0.0
    scanned = 0
    with zipfile.ZipFile(wheel) as archive:
        for name in archive.namelist():
            if not name.endswith(".so") and not name.endswith("/biomcp") and name != "biomcp":
                continue
            scanned += 1
            versions = member_versions(archive.read(name))
            if versions:
                top = max(top, *versions)
    if scanned == 0:
        raise SystemExit(f"FAIL {wheel.name}: no ELF member matched the scan filter")
    return top


def main(argv: list[str]) -> int:
    if len(argv) not in (2, 3):
        print(__doc__, file=sys.stderr)
        return 2
    wheel = Path(argv[1])
    floor = float(argv[2]) if len(argv) == 3 else 2.28
    top = wheel_floor(wheel)
    if top > floor:
        print(f"FAIL {wheel.name}: highest glibc reference {top:g} exceeds floor {floor:g}")
        return 1
    print(f"OK {wheel.name}: highest glibc reference {top:g} within floor {floor:g}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
