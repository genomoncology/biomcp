from __future__ import annotations

import re
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def _reqwest_client_constructions(text: str) -> int:
    qualified = re.compile(r"reqwest::Client::(?:builder|new)\s*\(")
    client_import = re.compile(r"use\s+reqwest::Client(?:\s+as\s+(\w+))?\s*;")
    crate_import = re.compile(r"use\s+reqwest(?:\s+as\s+(\w+))?\s*;")
    grouped_import = re.compile(r"use\s+reqwest::\{([^}]*)\}\s*;", re.DOTALL)
    count = len(qualified.findall(text))
    for match in client_import.finditer(text):
        name = match.group(1) or "Client"
        count += len(re.findall(rf"\b{re.escape(name)}::(?:builder|new)\s*\(", text))
    for match in crate_import.finditer(text):
        name = match.group(1) or "reqwest"
        count += len(
            re.findall(rf"\b{re.escape(name)}::Client::(?:builder|new)\s*\(", text)
        )
    for match in grouped_import.finditer(text):
        client = re.search(r"\bClient(?:\s+as\s+(\w+))?\b", match.group(1))
        if client:
            name = client.group(1) or "Client"
            count += len(re.findall(rf"\b{re.escape(name)}::(?:builder|new)\s*\(", text))
    return count


def test_reqwest_transport_construction_has_a_fail_closed_inventory() -> None:
    found: Counter[str] = Counter()
    for root in (ROOT / "src/sources", ROOT / "src/entities"):
        for path in root.rglob("*.rs"):
            text = path.read_text()
            count = _reqwest_client_constructions(text)
            if count:
                found[str(path.relative_to(ROOT))] = count

    # ordinary_url_policy.rs owns the two production builders. The remaining
    # entries are either test fixtures or provider-returned downloads which
    # install the stronger ProviderUrlPolicy directly.
    assert found == Counter(
        {
            "src/sources/mod.rs": 3,
            "src/sources/ordinary_url_policy.rs": 3,
            "src/sources/clingen_cspec.rs": 1,
            "src/sources/provider_url_policy.rs": 1,
            "src/sources/pubmed/tests/parsing.rs": 1,
            "src/entities/trial/documents.rs": 1,
            "src/entities/trial/search/ctgov/tests.rs": 1,
        }
    )


def test_inventory_recognizes_common_reqwest_alias_spellings() -> None:
    for sample in [
        "use reqwest::Client as HttpClient; HttpClient::builder()",
        "use reqwest as net; net::Client::builder()",
        "use reqwest::{Client as HttpClient, StatusCode}; HttpClient::new()",
    ]:
        assert _reqwest_client_constructions(sample) == 1


def test_production_http_builders_disable_proxies_and_own_redirects_and_dns() -> None:
    policy = (ROOT / "src/sources/ordinary_url_policy.rs").read_text()
    ordinary = policy.split("pub(crate) fn ordinary_http_client_builder", 1)[1].split(
        "pub(crate) fn provider_policy_client_builder", 1
    )[0]
    strict = policy.split("pub(crate) fn provider_policy_client_builder", 1)[1].split(
        "fn redirect_target_is_allowed", 1
    )[0]
    for builder in (ordinary, strict):
        assert ".no_proxy()" in builder
        assert ".dns_resolver(" in builder
        assert ".redirect(" in builder

    trial_download = (ROOT / "src/entities/trial/documents.rs").read_text()
    assert ".no_proxy()" in trial_download
    assert ".dns_resolver(policy.dns_resolver())" in trial_download
    assert ".redirect(policy.redirect_policy())" in trial_download


def test_alphagenome_is_the_single_documented_non_reqwest_provider_transport() -> None:
    uses: list[str] = []
    for path in (ROOT / "src/sources").rglob("*.rs"):
        if "tonic::transport::Endpoint::" in path.read_text():
            uses.append(str(path.relative_to(ROOT)))
    assert uses == ["src/sources/alphagenome.rs"]

    reference = (ROOT / "docs/reference/data-sources.md").read_text()
    assert "authenticated gRPC/Tonic provider transport" in reference
    assert "not part of this ordinary Reqwest boundary" in " ".join(reference.split())
