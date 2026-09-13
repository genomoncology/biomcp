#!/usr/bin/env python3
from pathlib import Path
import re

root = Path(__file__).resolve().parent
dist = root / "dist"
required = [
    "biodata/models/clinical-trial/index.html",
    "biodata/models/clinical-trial.md",
    "llms.txt",
    "llms-full.txt",
    "downloads/biodata/clinical-trial.schema.json",
    "downloads/biodata/clinical-trial-projection.schema.json",
    "downloads/biodata/ctgov-clinical-trial-projection.json",
    "downloads/biodata/clinical-trial-v1.bundle.json",
]
for relative in required:
    if not (dist / relative).is_file():
        raise SystemExit(f"missing built route: /{relative}")
for name in ("llms.txt", "llms-full.txt"):
    for url in re.findall(r"https://biomcp\.org(/[^\s)]+)", (dist / name).read_text()):
        target = dist / url.lstrip("/")
        if url.endswith("/"):
            target /= "index.html"
        if not target.is_file():
            raise SystemExit(f"unresolved {name} URL: {url}")
