from __future__ import annotations

import re
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]

CURRENT_ARCHITECTURE_DOCS = [
    "architecture/technical/overview.md",
    "architecture/technical/spec-v2.md",
    "architecture/technical/request-contract-test-architecture.md",
    "architecture/technical/semantic-scholar-runtime-contract.md",
    "architecture/technical/cli-module-decomposition.md",
    "architecture/technical/benchmark-cli-ownership-decision.md",
    "architecture/ux/cli-reference.md",
]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _squash(text: str) -> str:
    return re.sub(r"\s+", " ", text)


def _nav_entries(items: object) -> set[str]:
    found: set[str] = set()
    if isinstance(items, list):
        for item in items:
            found.update(_nav_entries(item))
    elif isinstance(items, dict):
        for value in items.values():
            if isinstance(value, str):
                found.add(value)
            else:
                found.update(_nav_entries(value))
    return found


def test_ticket_405_rust_crate_surface_is_internal_not_gene_facade() -> None:
    lib_rs = _read("src/lib.rs")
    current_docs = "\n".join(_read(path) for path in CURRENT_ARCHITECTURE_DOCS)
    docs_lower = current_docs.lower()

    assert "pub mod gene;" not in lib_rs, (
        "ticket 405 decides BioMCP has no supported public Rust library API; "
        "src/lib.rs must not keep the rejected gene facade public"
    )
    for marker in ("internal", "unstable", "no semver", "not for downstream import"):
        assert marker in docs_lower, (
            "current architecture/UX docs must explicitly document the Rust crate API "
            f"as internal/unstable with no downstream semver support; missing {marker!r}"
        )


def test_ticket_405_current_docs_do_not_present_make_check_as_biomcp_gate() -> None:
    stale_claims: list[str] = []
    allowed_context = re.compile(
        r"\b(historical|former|legacy|background|old|not supported|no supported|target state)\b",
        re.IGNORECASE,
    )
    current_gate_claim = re.compile(
        r"\b(canonical|current|local|routine|release|verify|full-blocking|dependency-free|intermediate|gate|fail)\b",
        re.IGNORECASE,
    )

    for path in CURRENT_ARCHITECTURE_DOCS:
        for paragraph in re.split(r"\n\s*\n", _read(path)):
            if "make check" not in paragraph:
                continue
            if allowed_context.search(paragraph) and not current_gate_claim.search(paragraph):
                continue
            stale_claims.append(f"{path}: {_squash(paragraph)}")

    assert not stale_claims, (
        "current architecture/operator docs must not present `make check` as a "
        "current BioMCP gate; use `make lint`, `make test`, and `make spec` "
        "or mark old text as historical/target-only:\n" + "\n".join(stale_claims)
    )


def test_ticket_405_surface_contract_lane_is_documented_for_make_spec_and_make_test() -> None:
    overview = _read("architecture/technical/overview.md")
    runner = _read("scripts/run-specs.sh")
    makefile = _read("Makefile")

    assert "tests/surface/" in overview, (
        "current technical overview must explain that Python/static surface contracts live "
        "under tests/surface/"
    )
    assert "make spec" in overview and "make test" in overview, (
        "current guidance must describe the split between Markdown specs and "
        "Python/static contracts"
    )
    assert "tests/surface/test_parallel_isolation_contract.py" in runner
    assert "uv sync --extra dev --no-install-project" not in runner
    assert "uv run --no-sync pytest" not in runner
    assert "test-contracts" in makefile and "pytest tests/" in makefile


def test_ticket_405_cache_and_logging_operator_contracts_are_inventoried() -> None:
    docs = "\n".join(
        _read(path)
        for path in (
            "architecture/technical/overview.md",
            "architecture/technical/staging-demo.md",
            "docs/user-guide/cli-reference.md",
            "docs/reference/quick-reference.md",
        )
        if (REPO_ROOT / path).exists()
    )

    cache_markers = [
        "BIOMCP_CACHE_DIR",
        "BIOMCP_CACHE_MAX_SIZE",
        "BIOMCP_CACHE_MIN_DISK_FREE",
        "cache.toml",
        "[cache].dir",
        "[cache].max_size",
        "[cache].min_disk_free",
        "[cache].max_age_secs",
        "10_000_000_000",
        "86_400",
        "10%",
        "env",
        "file",
        "default",
    ]
    missing_cache = [marker for marker in cache_markers if marker not in docs]
    assert not missing_cache, (
        "operator-facing docs must inventory cache env vars, cache.toml fields, "
        f"defaults, and env > file > default precedence; missing {missing_cache}"
    )

    logging_markers = [
        "RUST_LOG",
        "stderr",
        "warn",
        "error",
        "ANSI",
        "TTY",
        "redact",
        "JSON",
    ]
    missing_logging = [marker for marker in logging_markers if marker.lower() not in docs.lower()]
    assert not missing_logging, (
        "operator-facing docs must describe default tracing/logging, stderr behavior, "
        f"non-TTY color, redaction, and JSON-mode error expectations; missing {missing_logging}"
    )


def test_ticket_405_dependency_docs_name_article_fulltext_conversion_stack() -> None:
    dependencies = _read("docs/reference/dependencies.md")
    required_markers = [
        "Article fulltext",
        "JATS",
        "HTML",
        "PDF",
        "Europe PMC",
        "NCBI EFetch",
        "NCBI ID Converter",
        "PMC OA",
        "Figshare",
        "Semantic Scholar PDF",
        "built-in Rust transformers",
    ]
    missing = [marker for marker in required_markers if marker not in dependencies]
    assert not missing, (
        "docs/reference/dependencies.md must list the shipped article fulltext and "
        f"conversion stack by capability; missing {missing}"
    )


def test_ticket_405_next_command_ownership_is_ratcheted_or_named_followup() -> None:
    docs = "\n".join(_read(path) for path in CURRENT_ARCHITECTURE_DOCS)
    docs_lower = docs.lower()
    ratchet_exists = (
        "next-command ownership ratchet" in docs_lower
        or "entity code must not depend on markdown quote helpers" in docs_lower
    )
    followup_exists = (
        "next-command ownership follow-up" in docs_lower
        or "next-command ownership debt" in docs_lower
    )

    assert ratchet_exists or followup_exists, (
        "current architecture docs must either name a static ratchet for next-command "
        "ownership or capture next-command ownership debt as a named repo-doc follow-up"
    )


def test_ticket_405_mkdocs_nav_keeps_source_pages_visible() -> None:
    mkdocs = yaml.safe_load(_read("mkdocs.yml"))
    nav_paths = _nav_entries(mkdocs.get("nav", []))

    for source_page in ("sources/vaers.md", "sources/who-ivd.md", "sources/cdc-cvx.md"):
        assert source_page in nav_paths, f"mkdocs nav must include {source_page}"
