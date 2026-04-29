//! Stable facade for the `biomcp list` command-reference pages.

use crate::error::BioMcpError;

mod clinical;
mod helpers;
mod literature;
mod molecular;

pub fn render(entity: Option<&str>) -> Result<String, BioMcpError> {
    match normalize_entity(entity)? {
        None => Ok(helpers::list_all()),
        Some("gene") => Ok(molecular::list_gene()),
        Some("variant") => Ok(molecular::list_variant()),
        Some("article") => Ok(literature::list_article()),
        Some("trial") => Ok(clinical::list_trial()),
        Some("diagnostic") => Ok(clinical::list_diagnostic()),
        Some("drug") => Ok(clinical::list_drug()),
        Some("disease") => Ok(clinical::list_disease()),
        Some("phenotype") => Ok(clinical::list_phenotype()),
        Some("pgx") => Ok(molecular::list_pgx()),
        Some("gwas") => Ok(molecular::list_gwas()),
        Some("pathway") => Ok(molecular::list_pathway()),
        Some("protein") => Ok(molecular::list_protein()),
        Some("study") => Ok(literature::list_study()),
        Some("adverse-event") => Ok(clinical::list_adverse_event()),
        Some("search-all") => Ok(helpers::list_search_all()),
        Some("suggest") => Ok(helpers::list_suggest()),
        Some("discover") => Ok(helpers::list_discover()),
        Some("batch") => Ok(helpers::list_batch()),
        Some("enrich") => Ok(helpers::list_enrich()),
        Some("skill") => Ok(crate::cli::skill::list_use_cases()?),
        Some(_) => unreachable!("normalize_entity only returns known entities"),
    }
}

pub fn render_json(entity: Option<&str>) -> Result<String, BioMcpError> {
    match normalize_entity(entity)? {
        None => {
            #[derive(serde::Serialize)]
            struct ListJson {
                kind: &'static str,
                entities: Vec<String>,
                commands: Vec<String>,
                patterns: Vec<String>,
            }

            let page = helpers::list_all();
            crate::render::json::to_pretty(&ListJson {
                kind: "list",
                entities: section_plain_items(&page, "## Entities"),
                commands: section_code_items(&page, "## Quickstart"),
                patterns: section_code_items(&page, "## Patterns"),
            })
        }
        Some(entity) => {
            #[derive(serde::Serialize)]
            struct EntityListJson {
                kind: &'static str,
                entity: &'static str,
                commands: Vec<String>,
            }

            let page = render(Some(entity))?;
            let mut commands = section_code_items(&page, "## Commands");
            commands.extend(section_code_items(&page, "## Command"));
            crate::render::json::to_pretty(&EntityListJson {
                kind: "list_entity",
                entity,
                commands,
            })
        }
    }
}

fn normalize_entity(entity: Option<&str>) -> Result<Option<&'static str>, BioMcpError> {
    let Some(raw) = entity.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };

    match raw.to_ascii_lowercase().as_str() {
        "gene" => Ok(Some("gene")),
        "variant" => Ok(Some("variant")),
        "article" => Ok(Some("article")),
        "trial" => Ok(Some("trial")),
        "diagnostic" => Ok(Some("diagnostic")),
        "drug" => Ok(Some("drug")),
        "disease" => Ok(Some("disease")),
        "phenotype" => Ok(Some("phenotype")),
        "pgx" => Ok(Some("pgx")),
        "gwas" => Ok(Some("gwas")),
        "pathway" => Ok(Some("pathway")),
        "protein" => Ok(Some("protein")),
        "study" => Ok(Some("study")),
        "adverse-event" | "adverse_event" | "adverseevent" => Ok(Some("adverse-event")),
        "search-all" | "search_all" | "searchall" => Ok(Some("search-all")),
        "suggest" => Ok(Some("suggest")),
        "discover" => Ok(Some("discover")),
        "batch" => Ok(Some("batch")),
        "enrich" => Ok(Some("enrich")),
        "skill" | "skills" => Ok(Some("skill")),
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unknown entity: {other}\n\nValid entities:\n- gene\n- variant\n- article\n- trial\n- diagnostic\n- drug\n- disease\n- phenotype\n- pgx\n- gwas\n- pathway\n- protein\n- study\n- adverse-event\n- search-all\n- suggest\n- discover\n- batch\n- enrich\n- skill"
        ))),
    }
}

fn section_plain_items(page: &str, heading: &str) -> Vec<String> {
    section(page, heading)
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- "))
        .map(str::to_string)
        .collect()
}

fn section_code_items(page: &str, heading: &str) -> Vec<String> {
    section(page, heading)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("- ")?;
            let start = line.find('`')?;
            let rest = &line[start + 1..];
            let end = rest.find('`')?;
            Some(rest[..end].to_string())
        })
        .collect()
}

fn section<'a>(page: &'a str, heading: &str) -> &'a str {
    let marker = format!("{heading}\n");
    let Some(start) = page.find(&marker) else {
        return "";
    };
    let rest = &page[start + marker.len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    &rest[..end]
}

#[cfg(test)]
mod tests {
    mod pages;
    mod router;
}
