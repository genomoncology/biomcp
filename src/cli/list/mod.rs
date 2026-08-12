//! Stable facade for the `biomcp list` command-reference pages.

use crate::error::BioMcpError;

mod catalog;
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
        Some("author") => Ok(literature::list_author()),
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
        Some("discover") => Ok(helpers::list_discover()),
        Some("batch") => Ok(helpers::list_batch()),
        Some("enrich") => Ok(helpers::list_enrich()),
        Some("skill") => Ok(crate::cli::skill::list_use_cases()?),
        Some(_) => unreachable!("normalize_entity only returns known entities"),
    }
}

pub fn render_json(entity: Option<&str>) -> Result<String, BioMcpError> {
    #[derive(serde::Serialize)]
    struct ListJson {
        kind: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        entity: Option<&'static str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        entities: Option<Vec<catalog::EntityCapability>>,
        entries: Vec<catalog::CatalogEntry>,
    }

    let entity = normalize_entity(entity)?;
    let entries = catalog::entries(entity);
    catalog::validate(&entries)?;
    crate::render::json::to_pretty(&ListJson {
        kind: if entity.is_some() {
            "list_entity"
        } else {
            "list"
        },
        entity,
        entities: entity.is_none().then(catalog::entities),
        entries,
    })
}

fn normalize_entity(entity: Option<&str>) -> Result<Option<&'static str>, BioMcpError> {
    let Some(raw) = entity.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };

    match raw.to_ascii_lowercase().as_str() {
        "gene" => Ok(Some("gene")),
        "variant" => Ok(Some("variant")),
        "article" => Ok(Some("article")),
        "author" => Ok(Some("author")),
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
        "discover" => Ok(Some("discover")),
        "batch" => Ok(Some("batch")),
        "enrich" => Ok(Some("enrich")),
        "skill" | "skills" => Ok(Some("skill")),
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unknown entity: {other}\n\nValid entities:\n- gene\n- variant\n- article\n- author\n- trial\n- diagnostic\n- drug\n- disease\n- phenotype\n- pgx\n- gwas\n- pathway\n- protein\n- study\n- adverse-event\n- search-all\n- discover\n- batch\n- enrich\n- skill"
        ))),
    }
}

#[cfg(test)]
mod tests {
    mod pages;
    mod router;
}
