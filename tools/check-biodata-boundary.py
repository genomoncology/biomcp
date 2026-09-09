#!/usr/bin/env python3
"""Enforce the BioMCP 1.0 boundary with its exact BioData dependency."""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path

import tomllib

URL = "https://github.com/genomoncology/biodata"
REVISION = "cfafc69d27c9a2fc74909f21692a418a8b17db83"
EXPECTED_DEPENDENCY = {"git": URL, "rev": REVISION}


def require(condition: bool, message: str, failures: list[str]) -> None:
    if not condition:
        failures.append(message)


def tracked_files(root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=root,
        check=True,
        capture_output=True,
    )
    return [root / value.decode() for value in result.stdout.split(b"\0") if value]


def dependency_entries(
    document: object, path: tuple[str, ...] = ()
) -> list[tuple[tuple[str, ...], object]]:
    if not isinstance(document, dict):
        return []
    found: list[tuple[tuple[str, ...], object]] = []
    for key, value in document.items():
        next_path = (*path, key)
        if key == "biodata" and path and path[-1].endswith("dependencies"):
            found.append((next_path, value))
        found.extend(dependency_entries(value, next_path))
    return found


def read(root: Path, relative: str, failures: list[str]) -> str:
    path = root / relative
    if not path.is_file():
        failures.append(f"missing boundary file: {relative}")
        return ""
    return path.read_text(encoding="utf-8")


def check(root: Path) -> list[str]:
    failures: list[str] = []
    manifest_text = read(root, "Cargo.toml", failures)
    try:
        manifest = tomllib.loads(manifest_text)
    except tomllib.TOMLDecodeError as error:
        return [f"Cargo.toml is invalid: {error}"]

    entries = dependency_entries(manifest)
    require(
        entries == [(("dependencies", "biodata"), EXPECTED_DEPENDENCY)],
        "Cargo.toml must contain exactly one exact Git-pinned BioData dependency",
        failures,
    )
    require(
        "path" not in repr(entries).lower(),
        "BioData must not use a path or sibling-checkout dependency",
        failures,
    )
    require(
        "[patch." not in manifest_text.lower(),
        "Cargo.toml must not patch the BioData dependency",
        failures,
    )

    lock = read(root, "Cargo.lock", failures)
    require(
        lock.count('name = "biodata"') == 1,
        "Cargo.lock must contain exactly one BioData package",
        failures,
    )
    exact_source = f'source = "git+{URL}?rev={REVISION}#{REVISION}"'
    require(
        lock.count(exact_source) == 1,
        "Cargo.lock must pin BioData to the reviewed source and revision",
        failures,
    )

    for path in tracked_files(root):
        if path.name not in {"Cargo.toml", "config", "config.toml"}:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        relative = path.relative_to(root)
        if relative == Path("Cargo.toml"):
            continue
        require(
            not re.search(r"(?i)(?:\.\./|path\s*=).*biodata", text),
            f"{relative} refers to a BioData checkout or path dependency",
            failures,
        )

    design = read(root, "src/entities/trial/design.rs", failures)
    eligibility = read(root, "src/entities/trial/eligibility.rs", failures)
    trial = read(root, "src/entities/trial/mod.rs", failures)
    detail = read(root, "src/entities/trial/get.rs", failures)
    ctgov = read(root, "src/sources/clinicaltrials.rs", failures)
    nci = read(root, "src/sources/nci_cts.rs", failures)

    for symbol in (
        "ClinicalTrialArm",
        "ClinicalTrialIntervention",
        "ClinicalTrialArms",
        "ClinicalTrialArmInterventionAssignment",
        "ClinicalTrialArmRelationshipError",
    ):
        require(
            symbol in design, f"BioData design ownership is missing {symbol}", failures
        )
        require(
            not re.search(rf"(?:pub\s+)?(?:struct|enum)\s+{symbol}\b", design),
            f"BioMCP duplicates BioData design value {symbol}",
            failures,
        )
    require(
        "use biodata::" in design, "trial design must import BioData values", failures
    )

    require(
        "ClinicalTrialEligibility::from_json_bytes" in eligibility
        and ".to_json()" in eligibility,
        "eligibility must use BioData's standalone value codec",
        failures,
    )
    require(
        "ClinicalTrialReference::from_json_bytes" in trial and ".to_json()" in trial,
        "references must use BioData's standalone value codec",
        failures,
    )
    for duplicate in ("EligibilityOwned", "CriterionOwned", "ClassificationOwned"):
        require(
            duplicate not in eligibility,
            f"BioMCP duplicates migrated eligibility value {duplicate}",
            failures,
        )

    require(
        "ClinicalTrialSection" in detail,
        "trial detail must consume BioData request-aware section states",
        failures,
    )
    for source, plan, response in (
        (ctgov, "ClinicalTrialsGovApiV2DetailPlan", "ClinicalTrialsGovApiV2Response"),
        (nci, "NciCtsV2DetailPlan", "NciCtsV2DetailResponse"),
    ):
        require(
            "use biodata::" in source, f"provider adapter must import {plan}", failures
        )
        require(
            plan in source and response in source,
            f"provider adapter must delegate request planning and parsing to {plan}",
            failures,
        )
        require(
            not re.search(rf"(?:pub\s+)?struct\s+{plan}\b", source),
            f"BioMCP duplicates BioData request plan {plan}",
            failures,
        )
        require(
            not re.search(rf"(?:pub\s+)?struct\s+{response}\b", source),
            f"BioMCP duplicates BioData response parser {response}",
            failures,
        )

    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    args = parser.parse_args()
    failures = check(args.root.resolve())
    if failures:
        for failure in failures:
            print(f"BioData boundary violation: {failure}")
        return 1
    print("BioData 1.0 boundary check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
