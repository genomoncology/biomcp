from __future__ import annotations

from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[2]
URL_RE = re.compile(r"`(https?://[^`]+)`")
DOC_ENV_RE = re.compile(r"`([A-Z][A-Z0-9_]+)`")
RUST_BIOMCP_CONST_RE = re.compile(
    r'(?m)\b(?:pub\(crate\)\s+)?const\s+([A-Z][A-Z0-9_]*)\s*:\s*&str\s*=\s*"(BIOMCP_[A-Z0-9_]+)"'
)
RUST_DIRECT_ENV_READ_RE = re.compile(
    r'(?:std::)?env::var\(\s*"(BIOMCP_[A-Z0-9_]+)"\s*\)'
)
RUST_INDIRECT_ENV_READ_RE = re.compile(r"(?:std::)?env::var\(\s*([A-Z][A-Z0-9_]*)\s*\)")
RUST_OPTION_ENV_RE = re.compile(r'option_env!\(\s*"(BIOMCP_[A-Z0-9_]+)"\s*\)')
RUST_ENV_BASE_RE = re.compile(r"env_base\([^)]*,\s*([A-Z][A-Z0-9_]*)\s*\)", re.S)

PUBLIC_BIOMCP_SECTIONS = {
    "Operator Data and Cache Knobs",
    "Observability and Degradation",
}

PRODUCTION_READ_ENV_ALLOWLIST = {
    "BIOMCP_BUILD_DATE": "compile-time build metadata, not runtime operator configuration",
    "BIOMCP_BUILD_GIT_SHA": "compile-time build metadata, not runtime operator configuration",
    "BIOMCP_BUILD_GIT_TAG": "compile-time build metadata, not runtime operator configuration",
}


def _read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def _configuration_sections() -> dict[str, set[str]]:
    sections: dict[str, set[str]] = {}
    current: str | None = None
    for line in _read("docs/reference/configuration.md").splitlines():
        if line.startswith("## "):
            current = line.removeprefix("## ").strip()
            sections.setdefault(current, set())
            continue
        if current is not None:
            sections[current].update(DOC_ENV_RE.findall(line))
    return sections


def _production_rust_sources() -> list[str]:
    sources: list[str] = []
    for path in (ROOT / "src").rglob("*.rs"):
        relative = path.relative_to(ROOT)
        if "tests" in relative.parts or "test_support" in path.name:
            continue
        sources.append(path.read_text(encoding="utf-8"))
    return sources


def _production_biomcp_env_names() -> set[str]:
    sources = _production_rust_sources()
    constants = {
        ident: env_name
        for source in sources
        for ident, env_name in RUST_BIOMCP_CONST_RE.findall(source)
    }
    names: set[str] = set()
    for source in sources:
        names.update(RUST_DIRECT_ENV_READ_RE.findall(source))
        names.update(RUST_OPTION_ENV_RE.findall(source))
        names.update(
            constants[ident]
            for ident in RUST_INDIRECT_ENV_READ_RE.findall(source)
            if ident in constants
        )
        names.update(
            constants[ident]
            for ident in RUST_ENV_BASE_RE.findall(source)
            if ident in constants
        )
    return names


def test_source_versioning_covers_data_source_urls() -> None:
    data_sources = _read("docs/reference/data-sources.md")
    source_versioning = _read("docs/reference/source-versioning.md")
    urls = sorted(set(URL_RE.findall(data_sources)))
    missing = [url for url in urls if url not in source_versioning]
    assert not missing, "source-versioning.md missing documented URLs: " + ", ".join(
        missing
    )


def test_configuration_reference_classifies_env_var_families() -> None:
    config = _read("docs/reference/configuration.md")
    for heading in [
        "## Operator API Keys",
        "## Operator Data and Cache Knobs",
        "## Test and Fixture Override Seams",
        "## Release and Install Variables",
        "## Observability and Degradation",
    ]:
        assert heading in config

    for env_var in [
        "ALPHAGENOME_API_KEY",
        "DISGENET_API_KEY",
        "NCBI_API_KEY",
        "NCI_API_KEY",
        "ONCOKB_TOKEN",
        "OPENFDA_API_KEY",
        "S2_API_KEY",
        "UMLS_API_KEY",
        "BIOMCP_CACHE_DIR",
        "BIOMCP_STUDY_DIR",
        "BIOMCP_BIN",
    ]:
        assert f"`{env_var}`" in config


def test_observability_policy_names_public_status_surfaces() -> None:
    config = _read("docs/reference/configuration.md")
    assert "stderr" in config
    assert "`_meta.source_status`" in config
    assert "biomcp health --apis-only" in config
    assert "SourceUnavailable" in config


def test_biomcp_env_docs_match_runtime_reads() -> None:
    sections = _configuration_sections()
    production_names = _production_biomcp_env_names()
    classified_names: dict[str, list[str]] = {}
    for section, names in sections.items():
        for name in names:
            if name.startswith("BIOMCP_"):
                classified_names.setdefault(name, []).append(section)

    public_names = set().union(*(sections[name] for name in PUBLIC_BIOMCP_SECTIONS))
    unread_public = sorted(
        name
        for name in public_names
        if name.startswith("BIOMCP_") and name not in production_names
    )
    assert not unread_public, (
        "public BIOMCP env vars documented but not read: " + ", ".join(unread_public)
    )

    unclassified = sorted(
        production_names - set(classified_names) - set(PRODUCTION_READ_ENV_ALLOWLIST)
    )
    assert not unclassified, (
        "production BIOMCP env vars missing docs classification: "
        + ", ".join(unclassified)
    )

    duplicates = {
        name: sorted(locations)
        for name, locations in classified_names.items()
        if len(locations) > 1
    }
    assert not duplicates, (
        f"BIOMCP env vars classified in multiple sections: {duplicates}"
    )

    assert all(PRODUCTION_READ_ENV_ALLOWLIST.values())
