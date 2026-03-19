from __future__ import annotations

import os

import pytest

OPENFDA_SPEC_NODEIDS = {
    "spec/05-drug.md::Drug to Adverse Events",
    "spec/05-drug.md::Adverse Event Search",
    "spec/17-cross-entity-pivots.md::Drug to Adverse Events",
    "spec/11-evidence-urls.md::Markdown Evidence Links",
    "spec/11-evidence-urls.md::JSON Metadata Contract",
    "spec/12-search-positionals.md::Adverse-event Positional Query",
}

S2_SPEC_NODEIDS = {
    "spec/06-article.md::Semantic Scholar TLDR Section",
    "spec/06-article.md::Semantic Scholar Citations",
    "spec/06-article.md::Semantic Scholar References",
    "spec/06-article.md::Semantic Scholar Recommendations (Single Seed)",
    "spec/06-article.md::Semantic Scholar Recommendations (Multi Seed)",
}

DISGENET_SPEC_NODEIDS = {
    "spec/02-gene.md::Gene DisGeNET Associations",
    "spec/07-disease.md::Disease DisGeNET Associations",
}


def _has_openfda_api_key() -> bool:
    return bool(os.environ.get("OPENFDA_API_KEY", "").strip())


def _has_s2_api_key() -> bool:
    return bool(os.environ.get("S2_API_KEY", "").strip())


def _has_disgenet_api_key() -> bool:
    return bool(os.environ.get("DISGENET_API_KEY", "").strip())


def pytest_collection_modifyitems(
    config: pytest.Config, items: list[pytest.Item]
) -> None:
    del config

    openfda_skip = None
    if not _has_openfda_api_key():
        openfda_skip = pytest.mark.skip(
            reason=(
                "requires OPENFDA_API_KEY for stable live-spec quota against OpenFDA"
            )
        )

    s2_skip = None
    if not _has_s2_api_key():
        s2_skip = pytest.mark.skip(
            reason=(
                "requires S2_API_KEY for Semantic Scholar article enrichment/live-spec coverage"
            )
        )

    disgenet_skip = None
    if not _has_disgenet_api_key():
        disgenet_skip = pytest.mark.skip(
            reason=(
                "requires DISGENET_API_KEY for DisGeNET scored association live-spec coverage"
            )
        )

    for item in items:
        if openfda_skip and any(
            item.nodeid.startswith(nodeid) for nodeid in OPENFDA_SPEC_NODEIDS
        ):
            item.add_marker(openfda_skip)
        if s2_skip and any(item.nodeid.startswith(nodeid) for nodeid in S2_SPEC_NODEIDS):
            item.add_marker(s2_skip)
        if disgenet_skip and any(
            item.nodeid.startswith(nodeid) for nodeid in DISGENET_SPEC_NODEIDS
        ):
            item.add_marker(disgenet_skip)
