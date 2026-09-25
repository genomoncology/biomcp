#!/usr/bin/env python3
"""Assert shipped ELF artifacts reference no glibc symbol above a floor.

Usage: check-wheel-glibc-floor.py WHEEL [FLOOR]
       check-wheel-glibc-floor.py --elf FILE [FLOOR]

FLOOR defaults to 2.28. Versions compare as integer (major, minor)
pairs, the way the dynamic loader sees them, so 2.30 outranks 2.9
instead of reading as the decimal 2.3. The wheel mode scans every
member for ``GLIBC_x.y`` byte markers (the dynamic symbol versions the
binary actually imports); the ``--elf`` mode scans one raw ELF file,
so the pre-tar release binary can carry the same floor. Runs inside
the manylinux build container so the floor is checked against the
artifacts that ship.
"""

from __future__ import annotations

import re
import sys
import zipfile
from pathlib import Path

MARKER = re.compile(rb"GLIBC_([0-9]+)\.([0-9]+)")
FLOOR_PATTERN = re.compile(r"([0-9]+)\.([0-9]+)")
NO_VERSION = (0, 0)
DEFAULT_FLOOR = (2, 28)


def member_versions(data: bytes) -> list[tuple[int, int]]:
    return [
        (int(major.decode()), int(minor.decode()))
        for major, minor in MARKER.findall(data)
    ]


def parse_floor(text: str) -> tuple[int, int]:
    match = FLOOR_PATTERN.fullmatch(text)
    if match is None:
        raise ValueError(f"floor must be MAJOR.MINOR integers, got {text!r}")
    return (int(match.group(1)), int(match.group(2)))


def render(version: tuple[int, int]) -> str:
    return f"{version[0]}.{version[1]}"


def wheel_floor(wheel: Path) -> tuple[int, int]:
    top = NO_VERSION
    scanned = 0
    with zipfile.ZipFile(wheel) as archive:
        for name in archive.namelist():
            if (
                not name.endswith(".so")
                and not name.endswith("/biomcp")
                and name != "biomcp"
            ):
                continue
            scanned += 1
            versions = member_versions(archive.read(name))
            if versions:
                top = max(top, *versions)
    if scanned == 0:
        raise SystemExit(f"FAIL {wheel.name}: no ELF member matched the scan filter")
    return top


def elf_floor(path: Path) -> tuple[int, int]:
    data = sys.stdin.buffer.read() if str(path) == "-" else path.read_bytes()
    versions = member_versions(data)
    if not versions:
        label = "stdin" if str(path) == "-" else path.name
        raise SystemExit(
            f"FAIL {label}: no GLIBC marker found; not a dynamically linked ELF?"
        )
    return max(NO_VERSION, *versions)


def _report(name: str, top: tuple[int, int], floor: tuple[int, int]) -> int:
    if top > floor:
        print(
            f"FAIL {name}: highest glibc reference {render(top)} exceeds floor {render(floor)}"
        )
        return 1
    print(
        f"OK {name}: highest glibc reference {render(top)} within floor {render(floor)}"
    )
    return 0


def main(argv: list[str]) -> int:
    elf_mode = bool(argv[1:]) and argv[1] == "--elf"
    rest = argv[2:] if elf_mode else argv[1:]
    if len(rest) not in (1, 2):
        print(__doc__, file=sys.stderr)
        return 2
    if len(rest) == 2:
        try:
            floor = parse_floor(rest[1])
        except ValueError as error:
            print(error, file=sys.stderr)
            print(__doc__, file=sys.stderr)
            return 2
    else:
        floor = DEFAULT_FLOOR
    target = Path(rest[0])
    if elf_mode:
        top = elf_floor(target)
        name = "stdin" if str(target) == "-" else target.name
    else:
        top = wheel_floor(target)
        name = target.name
    return _report(name, top, floor)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
