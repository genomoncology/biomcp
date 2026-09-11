#!/usr/bin/env python3
"""Load and validate the reviewed BioData migration test selection."""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
import json
from pathlib import Path
import re
import tomllib


class SelectionError(ValueError):
    """The focused selection is malformed or does not match discovered tests."""


@dataclass(frozen=True)
class FocusedSelection:
    rust: tuple[str, ...]
    python: tuple[str, ...]


FORBIDDEN_FRAGMENTS = (
    "live_provider",
    "live-provider",
    "provider_smoke",
    "provider-smoke",
    "test_credentials.py",
    "test_credential.py",
    "publish",
    "publication",
    "deploy",
    "deployment",
    "release",
)
CREDENTIAL_ASSERTION_SELECTORS = {
    "entities::trial::search::plan_tests::clients_execute_exact_biodata_pairs_and_only_nci_adds_a_credential",
    "sources::tests::request_plan_transport::biodata_nci_search_keeps_one_logical_value_and_adds_one_credential",
}


def _selectors(value: object, label: str) -> tuple[str, ...]:
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        raise SelectionError(f"{label} selectors must be a TOML string array")
    selectors = tuple(value)
    if not selectors:
        raise SelectionError(f"{label} selectors must not be empty")
    if any(not selector.strip() for selector in selectors):
        raise SelectionError("focused selectors must not be empty")
    return selectors


def load_selection(path: Path) -> FocusedSelection:
    try:
        document = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise SelectionError(f"cannot read focused selection: {error}") from error
    if set(document) != {"version", "selection"} or document.get("version") != 1:
        raise SelectionError("focused selection must use manifest version 1")
    table = document.get("selection")
    if not isinstance(table, dict) or set(table) != {"rust", "python"}:
        raise SelectionError("focused selection must contain only Rust and Python selectors")
    selection = FocusedSelection(
        rust=_selectors(table["rust"], "Rust"),
        python=_selectors(table["python"], "Python"),
    )
    duplicates = sorted(
        name for name, count in Counter(selection.rust + selection.python).items() if count != 1
    )
    if duplicates:
        raise SelectionError(f"duplicate focused selector: {duplicates[0]}")
    for selector in selection.rust + selection.python:
        lowered = selector.casefold()
        if any(fragment in lowered for fragment in FORBIDDEN_FRAGMENTS):
            raise SelectionError(f"forbidden focused selector: {selector}")
        if "credential" in lowered and selector not in CREDENTIAL_ASSERTION_SELECTORS:
            raise SelectionError(f"credential-bearing focused selector: {selector}")
    if any("::" not in selector for selector in selection.rust):
        raise SelectionError("Rust selectors must name one exact test")
    for selector in selection.python:
        path_part = selector.split("::", 1)[0]
        if not re.fullmatch(r"tests/test_[a-z0-9_]+\.py", path_part):
            raise SelectionError(f"Python selector must name one focused test file: {selector}")
    return selection


def nextest_names(output: str) -> tuple[str, ...]:
    try:
        document = json.loads(output)
        suites = document["rust-suites"]
        return tuple(name for suite in suites.values() for name in suite["testcases"])
    except (json.JSONDecodeError, KeyError, TypeError, AttributeError) as error:
        raise SelectionError("nextest discovery did not return the expected JSON") from error


def validate_rust_discovery(selection: FocusedSelection, names: tuple[str, ...]) -> None:
    discovered = Counter(names)
    for selector in selection.rust:
        if discovered[selector] != 1:
            raise SelectionError(
                f"Rust selector must match exactly once, matched {discovered[selector]}: {selector}"
            )


def validate_python_collection(selection: FocusedSelection, nodeids: tuple[str, ...]) -> None:
    duplicates = [name for name, count in Counter(nodeids).items() if count != 1]
    if duplicates:
        raise SelectionError(f"Python test collected more than once: {duplicates[0]}")
    for selector in selection.python:
        if "::" in selector:
            count = nodeids.count(selector)
        else:
            prefix = f"{selector}::"
            count = sum(nodeid.startswith(prefix) for nodeid in nodeids)
        if count == 0:
            raise SelectionError(f"Python selector matched no tests: {selector}")
        if "::" in selector and count != 1:
            raise SelectionError(f"Python node identifier must match exactly once: {selector}")


def nextest_filter(selection: FocusedSelection) -> str:
    return " | ".join(f"test(={name})" for name in selection.rust)
