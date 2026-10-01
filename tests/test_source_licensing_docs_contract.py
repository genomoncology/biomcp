from __future__ import annotations

from datetime import date
import json
import re

import pytest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SOURCE_REVIEW_MAX_AGE_DAYS = 365

DIRECT_SOURCE_MODULES = {
    "alphagenome": "AlphaGenome",
    "cancerhotspots": "Cancerhotspots.org",
    "cbioportal": "cBioPortal",
    "cellosaurus": "Cellosaurus",
    "chembl": "ChEMBL",
    "civic": "CIViC",
    "clingen": "ClinGen",
    "clingen_allele_registry": "ClinGen Allele Registry",
    "clingen_cspec": "ClinGen CSpec",
    "clingen_erepo": "ClinGen ERepo",
    "clingen_ldh": "ClinGen LDH",
    "orcid": "ORCID",
    "gencc": "GenCC",
    "clinicaltrials": "ClinicalTrials.gov",
    "complexportal": "ComplexPortal",
    "cpic": "CPIC",
    "cvx": "CDC CVX/MVX",
    "dbsnp": "dbSNP",
    "vaers": "CDC WONDER VAERS",
    "ddinter": "DDInter",
    "dgidb": "DGIdb",
    "disgenet": "DisGeNET",
    "ema": "EMA",
    "who_pq": "WHO Prequalification",
    "who_ivd": "WHO Prequalified IVD",
    "enrichr": "Enrichr",
    "europepmc": "Europe PMC",
    "fhir": "FHIR (operator server)",
    "figshare": "Figshare",
    "fda_orphan": "FDA Orphan Drug Designations and Approvals",
    "gnomad": "gnomAD",
    "gprofiler": "g:Profiler",
    "gtex": "GTEx",
    "gwas": "GWAS Catalog",
    "hpa": "Human Protein Atlas",
    "hpo": "HPO JAX API",
    "interpro": "InterPro",
    "kegg": "KEGG",
    "litsense2": "LitSense2",
    "medlineplus": "MedlinePlus",
    "monarch": "Monarch Initiative",
    "mutalyzer": "Mutalyzer",
    "mychem": "MyChem.info",
    "mydisease": "MyDisease.info",
    "mygene": "MyGene.info",
    "myvariant": "MyVariant.info",
    "gtr": "NCBI Genetic Testing Registry",
    "ncbi_efetch": "NCBI E-utilities",
    "ncbi_idconv": "NCBI ID Converter",
    "nci_cts": "NCI CTS",
    "nih_reporter": "NIH Reporter",
    "ols4": "OLS4",
    "oncokb": "OncoKB",
    "opencitations": "OpenCitations",
    "openfda": "OpenFDA",
    "opentargets": "OpenTargets",
    "pharmacodb": "PharmacoDB",
    "pharmgkb": "PharmGKB",
    "pmc_oa": "PMC OA",
    "pubmed": "PubMed",
    "pubtator": "PubTator3",
    "quickgo": "QuickGO",
    "reactome": "Reactome",
    "seer": "SEER Explorer",
    "semantic_scholar": "Semantic Scholar",
    "string": "STRING",
    "umls": "UMLS",
    "uniprot": "UniProt",
    "variantvalidator": "VariantValidator",
    "wikipathways": "WikiPathways",
}

NESTED_DIRECT_SOURCES = {"ClinVar"}

INDIRECT_ONLY_ROWS = {
    "AlphaFold DB": "UniProt",
    "Cell Model Passports": "cellosaurus",
    "DepMap": "cellosaurus",
    "Cancer Genome Interpreter": "MyVariant.info",
    "COSMIC": "MyVariant.info",
    "Disease Ontology": "MyDisease.info",
    "DrugBank": "MyChem.info",
    "Drugs@FDA": "OpenFDA",
    "MONDO": "MyDisease.info",
    "PDB": "UniProt",
}

EXPECTED_NAMES = sorted(
    [
        *DIRECT_SOURCE_MODULES.values(),
        *NESTED_DIRECT_SOURCES,
        *INDIRECT_ONLY_ROWS.keys(),
    ]
)


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _markdown_section_block(text: str, heading: str, next_heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    end = remainder.find(next_heading)
    if end == -1:
        return remainder
    return remainder[:end]


def _source_inventory() -> list[dict[str, object]]:
    raw = _read("docs/reference/sources.json")
    data = json.loads(raw)
    assert isinstance(data, list)
    return data


def _inventory_item(name: str) -> dict[str, object]:
    for item in _source_inventory():
        if item["name"] == name:
            return item
    raise AssertionError(f"missing inventory item {name!r}")


def test_sources_inventory_is_complete_and_schema_conformant() -> None:
    source_mod = _read("src/sources/mod.rs")
    discovered_modules = re.findall(r"pub\(crate\) mod ([a-z0-9_]+);", source_mod)
    discovered_modules = [
        module
        for module in discovered_modules
        if module
        not in {
            "rate_limit",
            "cbioportal_download",
            "cbioportal_study",
            "pmc_article",
            "ordinary_url_policy",
            "provider_url_policy",
            "ca_bundle",
        }
    ]
    assert sorted(discovered_modules) == sorted(DIRECT_SOURCE_MODULES)

    inventory = _source_inventory()
    ids = [item["id"] for item in inventory]
    assert len(ids) == len(set(ids)), (
        f"duplicate id values: {[id for id in ids if ids.count(id) > 1]}"
    )
    names = sorted(item["name"] for item in inventory)
    assert names == EXPECTED_NAMES

    allowed_auth = {"none", "optional_env", "required_env", "not_applicable"}
    allowed_modes = {"direct_api", "indirect_only"}
    for item in inventory:
        assert set(item) == {
            "id",
            "name",
            "tier",
            "integration_mode",
            "via",
            "bioMcp_surfaces",
            "bioMcp_auth",
            "env_var",
            "provider_access",
            "license_summary",
            "redistribution_summary",
            "terms_url",
            "key_url",
            "reviewed_on",
            "notes",
        }
        assert item["tier"] in {1, 2, 3}
        assert item["integration_mode"] in allowed_modes
        assert item["bioMcp_auth"] in allowed_auth
        # Ticket 1205: a provider that publishes no terms page gets a null
        # terms_url rather than a link to something that is not terms, and its
        # licence summary has to say so.
        if item["terms_url"] is None:
            assert "publishes no licence or terms page" in item["license_summary"]
        else:
            assert item["terms_url"].startswith("https://")
        assert re.fullmatch(r"\d{4}-\d{2}-\d{2}", str(item["reviewed_on"]))
        assert isinstance(item["bioMcp_surfaces"], list)
        assert item["bioMcp_surfaces"]
        if item["bioMcp_auth"] in {"optional_env", "required_env"}:
            assert item["env_var"]
            assert str(item["key_url"]).startswith("https://")
        if item["integration_mode"] == "indirect_only":
            assert item["name"] in INDIRECT_ONLY_ROWS
            assert item["via"] == INDIRECT_ONLY_ROWS[item["name"]]
            assert item["bioMcp_auth"] == "not_applicable"
        else:
            assert item["name"] in {
                *DIRECT_SOURCE_MODULES.values(),
                *NESTED_DIRECT_SOURCES,
            }


def _review_age_days(reviewed_on: str, today: date) -> int:
    reviewed = date.fromisoformat(reviewed_on)
    return (today - reviewed).days


# Reviews this close to the limit warn without failing, so a batch of
# dates nearing the limit is visible in CI logs before it turns red.
SOURCE_REVIEW_WARN_AGE_DAYS = 300


def _split_review_entries(
    entries: list[dict[str, object]], today: date
) -> tuple[list[str], list[str]]:
    """Split review entries into (nearing-limit, expired) lines."""
    warned = [
        f"{item['id']} reviewed_on={item['reviewed_on']} ({_review_age_days(str(item['reviewed_on']), today)} days)"
        for item in entries
        if SOURCE_REVIEW_WARN_AGE_DAYS
        < _review_age_days(str(item["reviewed_on"]), today)
        <= SOURCE_REVIEW_MAX_AGE_DAYS
    ]
    expired = [
        f"{item['id']} reviewed_on={item['reviewed_on']} ({_review_age_days(str(item['reviewed_on']), today)} days)"
        for item in entries
        if _review_age_days(str(item["reviewed_on"]), today)
        > SOURCE_REVIEW_MAX_AGE_DAYS
    ]
    return warned, expired


def _stale_review_entries(today: date) -> tuple[list[str], list[str]]:
    return _split_review_entries(_source_inventory(), today)


def test_source_review_dates_warn_then_fail() -> None:
    # A print is invisible for passing tests (pytest hides captured
    # output), so approaching staleness warns and true staleness fails
    # (ticket 1244, batch 2). The canonical test gate runs this file,
    # so the assert fails CI exactly when a review passes 365 days.
    import warnings

    warned, expired = _stale_review_entries(date.today())
    for entry in warned:
        warnings.warn(
            f"source licensing review nearing the limit; re-read the "
            f"provider's terms and refresh reviewed_on: {entry}",
            UserWarning,
            stacklevel=2,
        )
    assert not expired, (
        "source licensing reviews older than 12 months must be "
        "re-read and refreshed before the release gate passes: "
        f"{expired}"
    )


def test_a_review_exactly_365_days_old_is_warned_not_failed() -> None:
    """The boundary: 365 days warns (and passes); 366 days fails.

    Drives the same splitter the main test reads the real inventory
    through, with synthetic entries pinned to the boundary so the
    behavior does not depend on any real date aging into range.
    """
    from datetime import timedelta

    today = date.today()
    entries = [
        {"id": "boundary", "reviewed_on": (today - timedelta(days=365)).isoformat()},
        {"id": "one-more", "reviewed_on": (today - timedelta(days=366)).isoformat()},
        {"id": "fresh", "reviewed_on": (today - timedelta(days=10)).isoformat()},
    ]
    warned, expired = _split_review_entries(entries, today)
    assert [line.split()[0] for line in warned] == ["boundary"]
    assert [line.split()[0] for line in expired] == ["one-more"]
    # And the whole real inventory, viewed a year ahead, all expires.
    _, real_expired = _stale_review_entries(today + timedelta(days=366))
    assert real_expired, "every real entry is older than 366 days from a year ahead"


def test_orcid_is_a_direct_exact_record_source() -> None:
    assert (REPO_ROOT / "src/sources/orcid.rs").exists()
    assert "pub(crate) mod orcid;" in _read("src/sources/mod.rs")

    orcid = _inventory_item("ORCID")
    assert orcid["integration_mode"] == "direct_api"
    assert orcid["bioMcp_auth"] == "required_env"
    assert orcid["env_var"] == "ORCID_ACCESS_TOKEN"
    assert orcid["bioMcp_surfaces"] == [
        "get author orcid:<id>",
        "author papers orcid:<id>",
    ]
    assert "public" in orcid["license_summary"].lower()
    assert "identically licensed" in orcid["redistribution_summary"]

    pubmed = _inventory_item("PubMed")
    assert "get article <id> indexing" in pubmed["bioMcp_surfaces"]
    assert "ORCID" in pubmed["notes"]


def test_source_licensing_reference_matches_inventory_and_required_sections() -> None:
    licensing = _read("docs/reference/source-licensing.md")

    assert "# Source Licensing and Terms" in licensing
    assert "## How to read this page" in licensing
    assert "## Summary table" in licensing
    assert "## Tier 1" in licensing
    assert "## Tier 2" in licensing
    assert "## Tier 3" in licensing
    assert "## Indirect-only providers surfaced through aggregators" in licensing
    assert "## Source notes" in licensing
    assert "BioMCP itself is MIT-licensed" in licensing
    assert (
        "BioMCP does not vendor, mirror, or ship upstream datasets in the repository."
        in licensing
    )
    assert (
        "BioMCP performs on-demand read-only queries against upstream services."
        in licensing
    )
    assert (
        "Returned records, downloaded full text, saved output, and downstream reuse"
        in licensing
    )
    assert "COSMIC" in licensing
    assert "MyVariant.info as the direct carrier" in licensing
    assert "SnpEff `ann`" in licensing
    assert "licensing risk" in licensing
    assert "PubMed" in licensing
    assert "Drugs@FDA" in licensing

    for name in EXPECTED_NAMES:
        assert name in licensing, f"missing source row or note for {name}"


def test_article_fulltext_source_inventory_matches_resolver_contract() -> None:
    licensing = _read("docs/reference/source-licensing.md")
    ncbi_eutilities = _inventory_item("NCBI E-utilities")
    pmc_oa = _inventory_item("PMC OA")
    pubmed = _inventory_item("PubMed")
    europe_pmc = _inventory_item("Europe PMC")
    semantic_scholar = _inventory_item("Semantic Scholar")

    assert "get article <id> fulltext" in europe_pmc["bioMcp_surfaces"]
    assert "get article <id> fulltext --pdf" in semantic_scholar["bioMcp_surfaces"]
    assert "PMC article HTML" in ncbi_eutilities["notes"]
    assert "PMC article HTML is a separate derived fallback" in pmc_oa["notes"]
    assert "PMC article HTML is documented as a PMC web fallback" in pubmed["notes"]
    assert "openAccessPdf" in semantic_scholar["notes"]
    assert "explicit PDF opt-in" in semantic_scholar["notes"]
    assert "Semantic Scholar/CDN allowlist" in semantic_scholar["notes"]
    assert "article-level reuse terms remain separate" in semantic_scholar["notes"]

    europe_pmc_section = _markdown_section_block(
        licensing, "### Europe PMC\n", "\n### g:Profiler"
    )
    semantic_scholar_section = _markdown_section_block(
        licensing, "### Semantic Scholar\n", "\n### UMLS"
    )
    source_notes = licensing.split("## Source notes\n", 1)[1]
    assert "get article <id> fulltext" in europe_pmc_section
    assert "get article <id> fulltext --pdf" in semantic_scholar_section
    assert "PMC article HTML" in source_notes
    assert "PMC web fallback" in source_notes
    assert "openAccessPdf" in semantic_scholar_section
    assert "explicit PDF opt-in" in semantic_scholar_section
    assert "Semantic Scholar/CDN allowlist" in semantic_scholar_section
    assert "article-level reuse terms remain separate" in semantic_scholar_section


def test_readme_and_docs_index_have_consistent_licensing_section() -> None:
    readme = _read("README.md")
    docs_index = _read("docs/index.md")

    for text in (readme, docs_index):
        section = _markdown_section_block(
            text,
            "## Data Sources and Licensing",
            "\n## License" if text is readme else "\n## Skills",
        )
        assert "MIT-licensed" in section
        assert "on-demand queries against upstream providers" in section
        assert "upstream terms govern reuse of retrieved results" in section
        assert "source-licensing.md" in section
        assert "api-keys.md" in section
        assert "KEGG" in section
        assert "COSMIC" in section


def test_api_keys_page_policies_data_sources_and_nav_link_to_licensing_reference() -> (
    None
):
    api_keys = _read("docs/getting-started/api-keys.md")
    policies = _read("docs/policies.md")
    data_sources = _read("docs/reference/data-sources.md")
    mkdocs = _read("mkdocs.yml")

    assert "source-licensing.md" in api_keys
    for env_var in (
        "ALPHAGENOME_API_KEY",
        "ONCOKB_TOKEN",
        "NCI_API_KEY",
        "DISGENET_API_KEY",
        "UMLS_API_KEY",
        "NCBI_API_KEY",
        "S2_API_KEY",
        "OPENFDA_API_KEY",
    ):
        assert env_var in api_keys

    assert "[Source licensing reference](reference/source-licensing.md)" in policies
    assert (
        "| NCBI E-utilities | `NCBI_API_KEY` | Optional; improves ClinVar EFetch, PubTator3, PubMed/efetch, PMC OA, and NCBI ID Converter quota headroom |"
        in data_sources
    )
    assert "      - Source Licensing: reference/source-licensing.md" in mkdocs


def test_docs_index_documentation_section_links_new_reference() -> None:
    docs_index = _read("docs/index.md")
    documentation = _markdown_section_block(
        docs_index,
        "## Documentation",
        "\n## Citation",
    )

    assert (
        "[Source Licensing and Terms](reference/source-licensing.md)" in documentation
    )


def test_the_licensing_page_tier_table_agrees_with_the_registry() -> None:
    """2026-09-29 review: the page still listed Enrichr as tier 1
    after sources.json moved it to tier 3, and nothing compared the
    two. This test fails the moment they disagree again.
    """
    page = _read("docs/reference/source-licensing.md")
    rows = {}
    for line in page.splitlines():
        m = re.match(r"^\|\s*([A-Za-z0-9 .&/-]+?)\s*\|\s*(\d)\s*\|", line)
        if m:
            rows[m.group(1).strip().lower()] = int(m.group(2))
    sections: dict[str, int] = {}
    current_tier: int | None = None
    for line in page.splitlines():
        m = re.match(r"^## Tier (\d)", line)
        if m:
            current_tier = int(m.group(1))
            continue
        m = re.match(r"^### (.+)$", line)
        if m and current_tier is not None:
            sections[m.group(1).strip().lower()] = current_tier
    inventory = _source_inventory()
    assert inventory, "the registry must parse"
    mismatches = []
    for entry in inventory:
        name = str(entry.get("name") or "")
        tier = int(entry.get("tier", 0))
        row = rows.get(name.lower())
        if row is not None and row != tier:
            mismatches.append(
                f"{name}: table says tier {row}, registry says tier {tier}"
            )
        # Every source with a detail section must sit under the
        # heading tier that matches the registry (2026-09-29 second
        # review: eight tier-1 sections sat under Tier 3 and Enrichr
        # sat under Tier 1, and no test looked at headings).
        if name.lower() in sections and sections[name.lower()] != tier:
            mismatches.append(
                f"{name}: heading tier {sections[name.lower()]}, registry tier {tier}"
            )
        if row is None and name.lower() not in sections:
            mismatches.append(f"{name}: absent from the page tier table and sections")
    assert not mismatches, "\n".join(mismatches)


def test_the_evidence_table_keeps_one_row_per_line() -> None:
    """2026-09-30 review: an unwrap pass collapsed the 49-row table
    onto a single physical line, which markdown renders as a wall of
    text. Each table row must start its own line.
    """
    text = _read("docs/reference/source-licensing-evidence-2026-09-27.md")
    table_lines = [row for row in text.splitlines() if row.startswith("|")]
    assert len(table_lines) == 52, (
        f"the evidence table lost its rows: {len(table_lines)} pipe-prefixed lines"
    )
    assert all(row.count("|") >= 4 for row in table_lines), (
        "a table row is malformed (fewer than four cells)"
    )
    # Exact coverage (second go-request review: a one-row collapse
    # passed the >= 49 floor). The pass documented 48 rows (45
    # verified, 2 changed, 1 restricted), so the table is exactly 50
    # pipe lines plus the two 2026-10-01 follow-up rows for the
    # identifier-only entries (depmap, cell-model-passports), every
    # row key is unique, and no row names a source the registry does
    # not know (the registry has grown past the pass, so
    # registry-to-row is not one-to-one).
    def _normalize(value: str) -> str:
        return value.strip().lower().replace(" ", "-").replace(".", "")

    row_keys = {_normalize(row.split("|")[1]) for row in table_lines[2:]}
    assert len(table_lines) == 52, (
        f"the pass documented 48 rows plus header, separator and two follow-up rows; "
        f"found {len(table_lines)}"
    )
    assert len(row_keys) == 50, f"duplicate or missing row keys: {len(row_keys)}"
    assert {"depmap", "cell-model-passports"} <= row_keys, "the 2026-10-01 follow-up rows are missing"
    # Every row names a source the registry knows: strip punctuation
    # and spaces so "ClinicalTrials.gov", "g:Profiler" and
    # "Cancerhotspots.org" match their registry keys.
    inventory = _source_inventory()

    def _squash(value: str) -> str:
        return re.sub(r"[^a-z0-9]", "", value.lower())

    known = {
        _squash(str(entry.get("id") or entry.get("key") or "")) for entry in inventory
    } | {_squash(str(entry.get("name") or "")) for entry in inventory}
    squashed = {_squash(key) for key in row_keys}
    assert squashed <= known, f"rows naming no registry source: {sorted(squashed - known)[:5]}"

    # The separator row must be its own line, not glued to the header.
    separator = table_lines[1]
    assert set(separator.replace("|", "").replace("-", "").strip()) == set(), (
        "the second table line must be the pure separator row"
    )


def test_licensing_contract_rejects_a_misplaced_restricted_section(monkeypatch) -> None:
    page = _read("docs/reference/source-licensing.md")
    start = page.index("### Enrichr\n")
    end = page.find("\n### ", start + 1)
    if end == -1:
        end = len(page)
    section = page[start:end]
    without = page[:start] + page[end:]
    tier_two = without.index("## Tier 2")
    misplaced = without[:tier_two] + section + "\n" + without[tier_two:]
    original_read = _read
    monkeypatch.setattr(
        __import__(__name__, fromlist=["_read"]), "_read",
        lambda path: misplaced if path == "docs/reference/source-licensing.md" else original_read(path),
    )
    with pytest.raises(AssertionError, match="Enrichr: heading tier 1, registry tier 3"):
        test_the_licensing_page_tier_table_agrees_with_the_registry()


def test_evidence_table_contract_rejects_one_missing_row(monkeypatch) -> None:
    original_read = _read
    path = "docs/reference/source-licensing-evidence-2026-09-27.md"
    lines = original_read(path).splitlines()
    first_data = next(i for i, line in enumerate(lines) if line.startswith("|") ) + 2
    del lines[first_data]
    monkeypatch.setattr(
        __import__(__name__, fromlist=["_read"]), "_read",
        lambda name: "\n".join(lines) if name == path else original_read(name),
    )
    with pytest.raises(AssertionError, match="lost its rows"):
        test_the_evidence_table_keeps_one_row_per_line()
