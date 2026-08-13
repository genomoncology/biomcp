use std::borrow::Cow;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{Tool, ToolAnnotations};

pub(super) struct ToolCatalogEntry {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

pub(super) const TOOLS: &[ToolCatalogEntry] = &[
    ToolCatalogEntry {
        name: "biomcp",
        title: "BioMCP command",
        description: "Run one read-only BioMCP command. Prefer typed tools. Use `biomcp list <entity>` for discovery or `biomcp skill list` for workflows. Set `json` for structured output. Binary downloads and mutations are rejected.",
    },
    ToolCatalogEntry {
        name: "search",
        title: "BioMCP search",
        description: "Search one biomedical entity with typed, bounded inputs.",
    },
    ToolCatalogEntry {
        name: "get",
        title: "BioMCP get",
        description: "Get one biomedical record with typed inputs. Binary assets are CLI-only.",
    },
    ToolCatalogEntry {
        name: "variant_normalize_car",
        title: "ClinGen Allele Registry normalization",
        description: "Normalize 1-50 RefSeq HGVS values with ClinGen Allele Registry.",
    },
    ToolCatalogEntry {
        name: "variant_erepo",
        title: "ClinGen ERepo assertions",
        description: "Get ClinGen ERepo expert assertions by CAid or gene.",
    },
    ToolCatalogEntry {
        name: "gene_cspec",
        title: "ClinGen CSpec",
        description: "Get ClinGen CSpec manifests, pages, or attachment metadata.",
    },
    ToolCatalogEntry {
        name: "variant_articles",
        title: "Variant literature batch",
        description: "Get compact literature shortlists for 1-10 variant identities.",
    },
];

pub(super) fn apply<S>(router: &mut ToolRouter<S>) {
    let actual = router
        .map
        .keys()
        .map(|name| name.as_ref())
        .collect::<Vec<_>>();
    assert_eq!(actual.len(), TOOLS.len(), "MCP router and catalog differ");
    for entry in TOOLS {
        let route = router
            .map
            .get_mut(entry.name)
            .unwrap_or_else(|| panic!("MCP tool {} is missing from the router", entry.name));
        route.attr.title = Some(entry.title.into());
        route.attr.description = Some(Cow::Borrowed(entry.description));
        route.attr.annotations = Some(ToolAnnotations::from_raw(
            Some(entry.title.into()),
            Some(true),
            Some(false),
            Some(true),
            Some(true),
        ));
    }
}

pub(super) fn list<S>(router: &ToolRouter<S>) -> Vec<Tool> {
    TOOLS
        .iter()
        .map(|entry| {
            router
                .map
                .get(entry.name)
                .unwrap_or_else(|| panic!("MCP tool {} is missing from the router", entry.name))
                .attr
                .clone()
        })
        .collect()
}

pub(super) fn instructions() -> String {
    let names = TOOLS
        .iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "BioMCP provides data from leading public biomedical data sources through seven read-only tools: {names}. Prefer bounded typed tools before the raw `biomcp` escape hatch. Use `search` and `get` for ordinary entity lookup, `variant_articles` for bounded literature batches, and the three ClinGen tools for their named contracts. For long-tail commands, start raw discovery with `biomcp list` or `biomcp list <entity>`; use `biomcp skill list` for worked workflows."
    )
}
