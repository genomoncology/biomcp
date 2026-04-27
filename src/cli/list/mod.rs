//! Stable facade for the `biomcp list` command-reference pages.

use crate::error::BioMcpError;

mod clinical;
mod helpers;
mod literature;
mod molecular;

pub fn render(entity: Option<&str>) -> Result<String, BioMcpError> {
    match entity.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(helpers::list_all()),
        Some(raw) => match raw.to_ascii_lowercase().as_str() {
            "gene" => Ok(molecular::list_gene()),
            "variant" => Ok(molecular::list_variant()),
            "article" => Ok(literature::list_article()),
            "trial" => Ok(clinical::list_trial()),
            "diagnostic" => Ok(clinical::list_diagnostic()),
            "drug" => Ok(clinical::list_drug()),
            "disease" => Ok(clinical::list_disease()),
            "phenotype" => Ok(clinical::list_phenotype()),
            "pgx" => Ok(molecular::list_pgx()),
            "gwas" => Ok(molecular::list_gwas()),
            "pathway" => Ok(molecular::list_pathway()),
            "protein" => Ok(molecular::list_protein()),
            "study" => Ok(literature::list_study()),
            "adverse-event" | "adverse_event" | "adverseevent" => {
                Ok(clinical::list_adverse_event())
            }
            "search-all" | "search_all" | "searchall" => Ok(helpers::list_search_all()),
            "suggest" => Ok(helpers::list_suggest()),
            "discover" => Ok(helpers::list_discover()),
            "batch" => Ok(helpers::list_batch()),
            "enrich" => Ok(helpers::list_enrich()),
            "skill" | "skills" => Ok(crate::cli::skill::list_use_cases()?),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown entity: {other}\n\nValid entities:\n- gene\n- variant\n- article\n- trial\n- diagnostic\n- drug\n- disease\n- phenotype\n- pgx\n- gwas\n- pathway\n- protein\n- study\n- adverse-event\n- search-all\n- suggest\n- discover\n- batch\n- enrich\n- skill"
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    mod pages;
    mod router;
}
