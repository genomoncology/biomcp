from __future__ import annotations

from typing import Any

import model_reference as shared

EXPECTED_SOURCE_DIGEST = (
    "4ab5cb9a467342d0e7c902a5f6328652c36f67d255c5d3c147b04cfebf0e2f40"
)
safe_text = shared.safe_text


def render(catalog: dict[str, Any]) -> str:
    root = next(model for model in catalog["models"] if model["id"] == catalog["root_model"])
    receipt = catalog["example_receipt"]
    lines = [
        "# ScientificPublication",
        "",
        safe_text(root["explanation"]),
        "",
        f"The stable catalog identity is `{safe_text(root['id'])}`.",
        "",
        "Title and abstract values preserve source-observed states without invented text. Authorship assertions stay opaque and completeness stays unknown. Named-date assertions keep their source names; equal raw text does not assert date equivalence.",
        "",
        *shared.support_lines(catalog),
        "",
        "The PubTator3 PMID projection and the Europe PMC LITE PMID projection render as implemented only because the adopted catalog carries executed proof. Europe PMC CORE is unsupported.",
        "",
        "## Fields and absence rules",
        "",
        *shared.field_rows(root),
        "",
        "Every listed member follows its stated absence rule. Required nullable members distinguish an explicit null from a missing member. Rust validation remains authoritative.",
        "",
        *shared.component_lines(catalog, root),
        "## Direct relationship diagram",
        "",
        "![ScientificPublication fields connect to six component models in adopted catalog order.](/downloads/biodata/scientific-publication-relationships.svg)",
        "",
        "[Download the accessible relationship diagram](/downloads/biodata/scientific-publication-relationships.svg).",
        "",
        "## Direct relationship text",
        "",
        "This text is equivalent to the diagram and remains available without images.",
        "",
    ]
    for item in shared.direct_relationships(catalog):
        lines.append(shared.relationship_line(item))
    lines += ["", "## Component relationships", ""]
    for item in catalog["relationships"]:
        lines.append(shared.relationship_line(item))
    lines += [
        "",
        "## BioMCP product behavior",
        "",
        "BioMCP resolves article detail through its existing tested command surfaces. A PMID normally uses the PubTator detail path. DOI and PMCID inputs use Europe PMC resolution and may reuse a resolved PMID. Europe PMC also supplies enrichment and HTTP fallback. BioMCP owns transport, fallback, reconciliation, and display.",
        "",
        "Identifier authorities remain distinct: a `pmid`, `pmcid`, or `doi` value does not prove cross-provider equivalence.",
        "",
        "Retained variant, graph, search, annotation, and compatibility paths are outside the shared model.",
        "",
        "For example, `biomcp get article 39325770 --json` invokes the tested generic PMID detail surface. That invocation is illustrative, not a recorded offline case, and this reference neither freezes nor claims its live output.",
        "",
        "## Provenance",
        "",
        f"This reference comes from catalog format {catalog['format_version']} and the recorded example `{safe_text(receipt['fixture'])}`. {safe_text(receipt['attribution'])}.",
        "",
        f"The receipt records the request `{safe_text(receipt['request'])}` under `{safe_text(receipt['license'])}` with attribution to {safe_text(receipt['attribution'])}. Origin: {safe_text(receipt['origin'])}. Transformation: {safe_text(receipt['transformation'])}.",
        "",
        f"The recorded input SHA\\-256 is `{EXPECTED_SOURCE_DIGEST}`. The adapter and catalog are pinned to BioData revision `{shared.EXPECTED_REVISION}`.",
        "",
        "The raw recorded PubTator3 response remains provider evidence retained by BioData and is not republished here. The only publication example download is the strict adapter-generated `biodata/scientific-publication` document embedded in the bundle.",
        "",
        "## Reviewed crosswalks",
        "",
        "No reviewed crosswalks are recorded for ScientificPublication. The empty crosswalk list is an explicit absence, not an invented mapping.",
        "",
        "## Downloads",
        "",
        "- [ScientificPublication schema](/downloads/biodata/scientific-publication.schema.json)",
        "- [PubTator3 ScientificPublication example](/downloads/biodata/pubtator3-scientific-publication.json)",
        "- [ScientificPublication relationship diagram](/downloads/biodata/scientific-publication-relationships.svg)",
        "- [Adopted catalog bundle](/downloads/biodata/scientific-publication-v1.bundle.json)",
        "- [ScientificPublication discovery contract](/biodata/discovery/scientific-publication.json)",
        "",
        "Artifact digests from the adopted catalog:",
        "",
    ]
    for artifact in catalog["artifacts"]:
        lines.append(f"- `{safe_text(artifact['id'])}`: `{safe_text(artifact['sha256'])}`")
    lines += [
        "",
        "## Scope",
        "",
        "This page renders only the claims and exclusions in the validated adopted catalog.",
        "",
    ]
    return "\n".join(lines)


def discovery_contract(catalog: dict[str, Any]) -> dict[str, Any]:
    routes = {
        "html": "https://biomcp.org/biodata/models/scientific-publication/",
        "raw_markdown": "https://biomcp.org/biodata/models/scientific-publication.md",
        "diagram": "https://biomcp.org/downloads/biodata/scientific-publication-relationships.svg",
        "schema": "https://biomcp.org/downloads/biodata/scientific-publication.schema.json",
        "example": "https://biomcp.org/downloads/biodata/pubtator3-scientific-publication.json",
        "catalog": "https://biomcp.org/downloads/biodata/scientific-publication-v1.bundle.json",
    }
    support_summary = "; ".join(f"{item['id']}: {item['label']}" for item in catalog["support"])
    return {
        "format_version": 1,
        "model": "model:ScientificPublication",
        "routes": routes,
        "diagram_relationships": shared.direct_relationships(catalog),
        "support": catalog["support"],
        "crosswalks": catalog["crosswalks"],
        "tasks": [
            {
                "question": "What fields describe a scientific publication?",
                "route": routes["html"],
                "answer": "The generated model reference lists every ScientificPublication field and absence rule.",
            },
            {
                "question": "How is a scientific publication related to its component records?",
                "route": routes["diagram"],
                "answer": "The diagram and adjacent text describe six direct relationships; the model page describes all six catalog relationships.",
            },
            {
                "question": "How does BioMCP resolve a publication identifier?",
                "route": routes["html"],
                "answer": "A PMID normally uses the PubTator detail path; DOI and PMCID inputs use Europe PMC resolution and may reuse a resolved PMID. Identifiers do not prove cross-provider equivalence.",
            },
            {
                "question": "Which publication projections and crosswalks are supported?",
                "route": routes["html"],
                "answer": f"Catalog support: {support_summary}. No reviewed crosswalks are recorded.",
            },
        ],
    }
