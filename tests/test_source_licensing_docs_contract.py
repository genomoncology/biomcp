from __future__ import annotations

import json
import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]

DIRECT_SOURCE_MODULES = {
    "alphagenome": "AlphaGenome",
    "cbioportal": "cBioPortal",
    "chembl": "ChEMBL",
    "civic": "CIViC",
    "clingen": "ClinGen",
    "clinicaltrials": "ClinicalTrials.gov",
    "complexportal": "ComplexPortal",
    "cpic": "CPIC",
    "dgidb": "DGIdb",
    "disgenet": "DisGeNET",
    "enrichr": "Enrichr",
    "europepmc": "Europe PMC",
    "gnomad": "gnomAD",
    "gprofiler": "g:Profiler",
    "gtex": "GTEx",
    "gwas": "GWAS Catalog",
    "hpa": "Human Protein Atlas",
    "hpo": "HPO JAX API",
    "interpro": "InterPro",
    "kegg": "KEGG",
    "medlineplus": "MedlinePlus",
    "monarch": "Monarch Initiative",
    "mychem": "MyChem.info",
    "mydisease": "MyDisease.info",
    "mygene": "MyGene.info",
    "myvariant": "MyVariant.info",
    "ncbi_idconv": "NCBI ID Converter",
    "nci_cts": "NCI CTS",
    "ols4": "OLS4",
    "oncokb": "OncoKB",
    "openfda": "OpenFDA",
    "opentargets": "OpenTargets",
    "pharmgkb": "PharmGKB",
    "pmc_oa": "PMC OA",
    "pubtator": "PubTator3",
    "quickgo": "QuickGO",
    "reactome": "Reactome",
    "semantic_scholar": "Semantic Scholar",
    "string": "STRING",
    "umls": "UMLS",
    "uniprot": "UniProt",
    "wikipathways": "WikiPathways",
}

INDIRECT_ONLY_ROWS = {
    "AlphaFold DB": "UniProt",
    "Cancer Genome Interpreter": "MyVariant.info",
    "ClinVar": "MyVariant.info",
    "COSMIC": "MyVariant.info",
    "Disease Ontology": "MyDisease.info",
    "DrugBank": "MyChem.info",
    "Drugs@FDA": "OpenFDA",
    "MONDO": "MyDisease.info",
    "PDB": "UniProt",
}

EXPECTED_NAMES = sorted([*DIRECT_SOURCE_MODULES.values(), *INDIRECT_ONLY_ROWS.keys()])


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


def test_sources_inventory_is_complete_and_schema_conformant() -> None:
    source_mod = _read("src/sources/mod.rs")
    discovered_modules = re.findall(r"pub\(crate\) mod ([a-z0-9_]+);", source_mod)
    discovered_modules = [
        module
        for module in discovered_modules
        if module not in {"rate_limit", "cbioportal_download", "cbioportal_study"}
    ]
    assert sorted(discovered_modules) == sorted(DIRECT_SOURCE_MODULES)

    inventory = _source_inventory()
    ids = [item["id"] for item in inventory]
    assert len(ids) == len(set(ids)), f"duplicate id values: {[id for id in ids if ids.count(id) > 1]}"
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
            assert item["name"] in DIRECT_SOURCE_MODULES.values()


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
    assert "BioMCP does not vendor, mirror, or ship upstream datasets in the repository." in licensing
    assert "BioMCP performs on-demand read-only queries against upstream services." in licensing
    assert "Returned records, downloaded full text, saved output, and downstream reuse" in licensing
    assert "COSMIC" in licensing
    assert "licensing risk" in licensing
    assert "PubMed" in licensing
    assert "Drugs@FDA" in licensing

    for name in EXPECTED_NAMES:
        assert name in licensing, f"missing source row or note for {name}"


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


def test_api_keys_page_policies_data_sources_and_nav_link_to_licensing_reference() -> None:
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
    assert "| NCBI E-utilities | `NCBI_API_KEY` | Optional; improves PubTator3, PMC OA, and NCBI ID Converter quota headroom |" in data_sources
    assert "      - Source Licensing: reference/source-licensing.md" in mkdocs


def test_docs_index_documentation_section_links_new_reference() -> None:
    docs_index = _read("docs/index.md")
    documentation = _markdown_section_block(
        docs_index,
        "## Documentation",
        "\n## Citation",
    )

    assert "[Source Licensing and Terms](reference/source-licensing.md)" in documentation
