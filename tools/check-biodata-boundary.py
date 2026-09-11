#!/usr/bin/env python3
"""Enforce BioMCP's declared BioData 1.0 ownership boundary."""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path

import tomllib

URL = "https://github.com/genomoncology/biodata"
REVISION = "d9d419eb96bfdf8056db7c71d6b973f17f0c1699"
VERSION = "0.0.16"
EXPECTED_DEPENDENCY = {"git": URL, "rev": REVISION}
DEPENDENCY_TABLES = {"dependencies", "dev-dependencies", "build-dependencies"}
RETIRED_DECLARATIONS = (
    "ClinicalTrialIdentityError",
    "ClinicalTrialCore",
    "ClinicalTrialArmId",
    "ClinicalTrialInterventionId",
    "ClinicalTrialEligibilityCriterionId",
    "ExtensibleCode",
    "ClinicalTrialArm",
    "ClinicalTrialArmId",
    "ClinicalTrialIntervention",
    "ClinicalTrialInterventionId",
    "ClinicalTrialArms",
    "ClinicalTrialArmInterventionAssignment",
    "ClinicalTrialArmRelationshipError",
    "ClinicalTrialIdentityError",
    "ClinicalTrialEligibility",
    "ClinicalTrialEligibilityCriterion",
    "ClinicalTrialEligibilityCriterionId",
    "ClinicalTrialEligibilityClassification",
    "ClinicalTrialReference",
    "ClinicalTrialSection",
    "ExtensibleCode",
    "CtGovDetailPlan",
    "CtGovDetailResponse",
    "CtGovProjection",
    "ClinicalTrialsGovApiV2DetailPlan",
    "ClinicalTrialsGovApiV2Response",
    "CtGovDetailPlan",
    "CtGovDetailResponse",
    "CtGovProjection",
    "NciCtsV2DetailPlan",
    "NciCtsV2DetailResponse",
    "EligibilityOwned",
    "CriterionOwned",
    "ClassificationOwned",
    "EligibilityWire",
    "EligibilityRef",
    "AgeRangeWire",
    "AgeBoundWire",
    "ClassificationWire",
    "CriterionWire",
    "RuleWire",
    "UnitWire",
    "BoundWire",
    "RuleWire",
    "AgeBoundWire",
    "AgeRangeWire",
    "ClassificationWire",
    "CriterionWire",
    "EligibilityWire",
    "EligibilityRef",
    "ReferenceWire",
)


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


def cargo_file(path: Path, root: Path) -> bool:
    relative = path.relative_to(root)
    return path.name == "Cargo.toml" or (
        path.name in {"config", "config.toml"} and ".cargo" in relative.parts
    )


def dependency_entries(
    document: object, path: tuple[str, ...] = ()
) -> list[tuple[tuple[str, ...], str, object]]:
    if not isinstance(document, dict):
        return []
    found: list[tuple[tuple[str, ...], str, object]] = []
    for key, value in document.items():
        next_path = (*path, key)
        is_dependency = path and path[-1] in DEPENDENCY_TABLES
        renamed_biodata = isinstance(value, dict) and value.get("package") == "biodata"
        if is_dependency and (key == "biodata" or renamed_biodata):
            found.append((next_path, key, value))
        found.extend(dependency_entries(value, next_path))
    return found


def has_biodata_local_path(document: object) -> bool:
    if isinstance(document, dict):
        for key, value in document.items():
            if key in {"path", "directory", "local-registry"} and isinstance(
                value, str
            ):
                if "biodata" in value.lower():
                    return True
            if has_biodata_local_path(value):
                return True
    elif isinstance(document, list):
        return any(has_biodata_local_path(value) for value in document)
    return False


def read(root: Path, relative: str, failures: list[str]) -> str:
    path = root / relative
    if not path.is_file():
        failures.append(f"missing boundary file: {relative}")
        return ""
    return path.read_text(encoding="utf-8")


def parse_toml(path: Path, root: Path, failures: list[str]) -> dict[str, object] | None:
    relative = path.relative_to(root)
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        failures.append(f"{relative} is invalid TOML: {error}")
        return None


def check_cargo_files(root: Path, files: list[Path], failures: list[str]) -> None:
    all_entries: list[tuple[Path, tuple[str, ...], str, object]] = []
    for path in files:
        if not cargo_file(path, root):
            continue
        document = parse_toml(path, root, failures)
        if document is None:
            continue
        relative = path.relative_to(root)
        if path.name == "Cargo.toml":
            all_entries.extend(
                (relative, entry_path, key, value)
                for entry_path, key, value in dependency_entries(document)
            )
            patch = document.get("patch")
            replace = document.get("replace")
            require(
                not patch or "biodata" not in repr(patch).lower(),
                f"{relative} must not patch BioData",
                failures,
            )
            require(
                not replace or "biodata" not in repr(replace).lower(),
                f"{relative} must not replace BioData",
                failures,
            )
        else:
            source = document.get("source")
            if isinstance(source, dict):
                require(
                    not any(
                        isinstance(value, dict) and "replace-with" in value
                        for value in source.values()
                    ),
                    f"{relative} must not replace Cargo sources",
                    failures,
                )
        require(
            not has_biodata_local_path(document),
            f"{relative} refers to a BioData sibling checkout or local path",
            failures,
        )

    expected = [
        (
            Path("Cargo.toml"),
            ("dependencies", "biodata"),
            "biodata",
            EXPECTED_DEPENDENCY,
        )
    ]
    require(
        all_entries == expected,
        "tracked Cargo manifests must contain exactly one root exact Git-pinned BioData dependency",
        failures,
    )


def check_lock(root: Path, failures: list[str]) -> None:
    lock_text = read(root, "Cargo.lock", failures)
    try:
        lock = tomllib.loads(lock_text)
    except tomllib.TOMLDecodeError as error:
        failures.append(f"Cargo.lock is invalid: {error}")
        return
    packages = [
        package
        for package in lock.get("package", [])
        if isinstance(package, dict) and package.get("name") == "biodata"
    ]
    exact_source = f"git+{URL}?rev={REVISION}#{REVISION}"
    require(
        packages == [{"name": "biodata", "version": VERSION, "source": exact_source}]
        or (
            len(packages) == 1
            and packages[0].get("name") == "biodata"
            and packages[0].get("version") == VERSION
            and packages[0].get("source") == exact_source
        ),
        "Cargo.lock must contain one BioData package with the reviewed version and source",
        failures,
    )


def check_rust_ownership(root: Path, files: list[Path], failures: list[str]) -> None:
    sources: list[tuple[Path, str]] = []
    for path in files:
        relative = path.relative_to(root)
        if path.suffix == ".rs" and relative.parts and relative.parts[0] == "src":
            sources.append((relative, path.read_text(encoding="utf-8")))
    combined = "\n".join(source for _, source in sources)
    require(
        "use biodata::" in combined, "production Rust must import BioData", failures
    )
    for symbol in RETIRED_DECLARATIONS:
        pattern = re.compile(
            rf"(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|type|trait)\s+{symbol}\b"
        )
        for relative, source in sources:
            require(
                not pattern.search(source),
                f"{relative} redeclares BioData-owned {symbol}",
                failures,
            )

    for marker in ("NO_LIMIT_RULE", "fn eligibility_value", "fn age_bound_value"):
        for relative, source in sources:
            require(
                marker not in source,
                f"{relative} retains retired eligibility codec marker {marker}",
                failures,
            )

    retired_modules = re.compile(r"\bmod\s+strict_json\b|pub\(crate\)\s+mod\s+shared\b")
    for relative, source in sources:
        require(
            not retired_modules.search(source),
            f"{relative} retains a retired migrated codec or value module",
            failures,
        )

    retired_modules = re.compile(
        r"(?:pub\(crate\)\s+mod\s+shared|mod\s+strict_json)\s*[;{]"
    )
    for relative, source in sources:
        require(
            not retired_modules.search(source),
            f"{relative} retains a retired migrated codec/value module",
            failures,
        )

    for symbol in (
        "ClinicalTrialArm",
        "ClinicalTrialCore",
        "ClinicalTrialIntervention",
        "ClinicalTrialArms",
        "ClinicalTrialArmInterventionAssignment",
        "ClinicalTrialArmRelationshipError",
        "ClinicalTrialSection",
        "ClinicalTrialsGovApiV2DetailPlan",
        "ClinicalTrialsGovApiV2Response",
        "NciCtsV2DetailPlan",
        "NciCtsV2DetailResponse",
    ):
        require(
            symbol in combined, f"BioData consumption is missing {symbol}", failures
        )
    require(
        "ClinicalTrialEligibility::from_json_bytes" in combined
        and combined.count(".to_json()") >= 2,
        "eligibility must use BioData's standalone value codec",
        failures,
    )
    require(
        "ClinicalTrialReference::from_json_bytes" in combined,
        "references must use BioData's standalone value codec",
        failures,
    )


def check(root: Path) -> list[str]:
    failures: list[str] = []
    files = tracked_files(root)
    check_cargo_files(root, files, failures)
    check_lock(root, failures)
    check_rust_ownership(root, files, failures)
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
