#!/usr/bin/env python3
"""Reject drift in the two accepted BioData discovery artifacts."""

from __future__ import annotations

import hashlib
from pathlib import Path
import sys

PINS = {
    "public/downloads/biodata/clinical-trial-relationships.svg": (
        "899261542e9927ac54a8a6f13084c00823eea9306f879f991c4bd15fdf861e78"
    ),
    "public/biodata/discovery/clinical-trial.json": (
        "31f344dec1469d5309f1f8dd5c1e8680139600b1727270907aba357b2cb31dd6"
    ),
    "public/downloads/biodata/scientific-publication-relationships.svg": (
        "afcb973f9629cf69f0821e9c47d1dc26d885de572cb51f7b8c2c7bd4ba81972a"
    ),
    "public/biodata/discovery/scientific-publication.json": (
        "6da784d7613178ae80b24357f1da3468f7151ff72990b3f9dffbfafb47b71728"
    ),
}


def check(root: Path) -> bool:
    for relative, expected in PINS.items():
        path = root / relative
        try:
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
        except OSError:
            return False
        if actual != expected:
            return False
    return True


def main(argv: list[str]) -> int:
    if len(argv) > 2:
        print("usage: check-biodata-artifact-pins.py [website-root]", file=sys.stderr)
        return 2
    root = Path(argv[1]) if len(argv) == 2 else Path(__file__).resolve().parent
    if not check(root):
        print("accepted BioData artifact digest mismatch", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
