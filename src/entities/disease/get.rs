//! Disease detail retrieval, section parsing, and parent resolution.

use super::*;

use super::enrichment::{
    apply_requested_sections, enrich_base_context, enrich_sparse_disease_identity,
};
use super::resolution::{
    DiseaseLookupInput, parse_disease_lookup_input, preferred_crosswalk_hit,
    resolve_disease_hit_by_name,
};
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct DiseaseSections {
    pub(super) explicit: bool,
    pub(super) include_genes: bool,
    pub(super) include_pathways: bool,
    pub(super) include_phenotypes: bool,
    pub(super) include_diagnostics: bool,
    pub(super) include_variants: bool,
    pub(super) include_models: bool,
    pub(super) include_prevalence: bool,
    pub(super) include_survival: bool,
    pub(super) include_funding: bool,
    pub(super) include_civic: bool,
    pub(super) include_disgenet: bool,
    pub(super) include_clinical_features: bool,
}

#[cfg(test)]
fn parse_sections(sections: &[String]) -> Result<DiseaseSections, BioMcpError> {
    parse_sections_for_name("", sections)
}

fn parse_sections_for_name(
    name_or_id: &str,
    sections: &[String],
) -> Result<DiseaseSections, BioMcpError> {
    let mut out = DiseaseSections::default();
    let mut include_all = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }
        out.explicit = true;

        match section.as_str() {
            DISEASE_SECTION_GENES => out.include_genes = true,
            DISEASE_SECTION_PATHWAYS => out.include_pathways = true,
            DISEASE_SECTION_PHENOTYPES => out.include_phenotypes = true,
            DISEASE_SECTION_DIAGNOSTICS => out.include_diagnostics = true,
            DISEASE_SECTION_VARIANTS => out.include_variants = true,
            DISEASE_SECTION_MODELS => out.include_models = true,
            DISEASE_SECTION_PREVALENCE => out.include_prevalence = true,
            DISEASE_SECTION_SURVIVAL => out.include_survival = true,
            DISEASE_SECTION_FUNDING => out.include_funding = true,
            DISEASE_SECTION_CIVIC => out.include_civic = true,
            DISEASE_SECTION_DISGENET => out.include_disgenet = true,
            DISEASE_SECTION_CLINICAL_FEATURES => out.include_clinical_features = true,
            DISEASE_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(
                    unknown_disease_section_message(name_or_id, sections, raw, &section),
                ));
            }
        }
    }

    if include_all {
        out.include_genes = true;
        out.include_pathways = true;
        out.include_phenotypes = true;
        out.include_variants = true;
        out.include_models = true;
        out.include_prevalence = true;
        out.include_survival = true;
        out.include_civic = true;
    }

    Ok(out)
}

fn unknown_disease_section_message(
    name_or_id: &str,
    sections: &[String],
    raw_section: &str,
    normalized_section: &str,
) -> String {
    let available = DISEASE_SECTION_NAMES.join(", ");
    let tail = sections
        .iter()
        .skip_while(|section| section.as_str() != raw_section)
        .skip(1)
        .collect::<Vec<_>>();
    let section_start = tail
        .iter()
        .position(|section| {
            DISEASE_SECTION_NAMES.contains(&section.trim().to_ascii_lowercase().as_str())
        })
        .unwrap_or(tail.len());
    let name_tail = tail
        .iter()
        .take(section_start)
        .map(|section| section.trim())
        .filter(|section| !section.is_empty());
    let suggestion_name = std::iter::once(name_or_id.trim())
        .chain(std::iter::once(raw_section.trim()))
        .chain(name_tail)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let remaining_sections = tail
        .iter()
        .skip(section_start)
        .map(|section| section.trim())
        .filter(|section| !section.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let next = if remaining_sections.is_empty() {
        format!("biomcp get disease --name \"{suggestion_name}\"")
    } else {
        format!("biomcp get disease --name \"{suggestion_name}\" {remaining_sections}")
    };
    format!(
        "Unknown section \"{normalized_section}\" for disease. Available: {available}. If \"{raw_section}\" is part of a multi-word disease name, use `{next}`."
    )
}

pub async fn get(name_or_id: &str, sections: &[String]) -> Result<Disease, BioMcpError> {
    let parsed_sections = parse_sections_for_name(name_or_id, sections)?;
    let name_or_id = name_or_id.trim();
    if name_or_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Disease name or ID is required. Example: biomcp get disease melanoma".into(),
        ));
    }
    if name_or_id.len() > 512 {
        return Err(BioMcpError::InvalidArgument(
            "Disease name/ID is too long.".into(),
        ));
    }

    let client = MyDiseaseClient::new()?;

    match parse_disease_lookup_input(name_or_id) {
        DiseaseLookupInput::CanonicalOntologyId(id) => {
            let hit = client.get(&id).await?;
            let mut disease = transform::disease::from_mydisease_hit(hit);
            if let Err(err) = enrich_sparse_disease_identity(&mut disease).await {
                warn!("OLS4 unavailable for sparse disease identity repair: {err}");
            }
            disease.parents = resolve_parent_names(&client, &disease.parents).await;
            if !parsed_sections.explicit {
                enrich_base_context(&mut disease).await;
            }
            apply_requested_sections(&mut disease, parsed_sections, Some(name_or_id)).await?;
            return Ok(disease);
        }
        DiseaseLookupInput::CrosswalkId(kind, value) => {
            let resp = client
                .lookup_disease_by_xref(kind.source_key(), &value, 5)
                .await?;
            let best = preferred_crosswalk_hit(resp.hits).ok_or_else(|| BioMcpError::NotFound {
                entity: "disease".into(),
                id: name_or_id.trim().to_string(),
                suggestion: "Try biomcp discover \"<disease name>\" to resolve a supported disease identifier.".into(),
            })?;
            let hit = client.get(&best.id).await?;
            let mut disease = transform::disease::from_mydisease_hit(hit);
            if let Err(err) = enrich_sparse_disease_identity(&mut disease).await {
                warn!("OLS4 unavailable for sparse disease identity repair: {err}");
            }
            disease.parents = resolve_parent_names(&client, &disease.parents).await;
            if !parsed_sections.explicit {
                enrich_base_context(&mut disease).await;
            }
            apply_requested_sections(&mut disease, parsed_sections, Some(name_or_id)).await?;
            return Ok(disease);
        }
        DiseaseLookupInput::FreeText => {}
    }

    let best = resolve_disease_hit_by_name(&client, name_or_id).await?;

    let hit = client.get(&best.id).await?;
    let mut disease = transform::disease::from_mydisease_hit(hit);
    if let Err(err) = enrich_sparse_disease_identity(&mut disease).await {
        warn!("OLS4 unavailable for sparse disease identity repair: {err}");
    }
    disease.parents = resolve_parent_names(&client, &disease.parents).await;
    if !parsed_sections.explicit {
        enrich_base_context(&mut disease).await;
    }
    apply_requested_sections(&mut disease, parsed_sections, Some(name_or_id)).await?;
    Ok(disease)
}

async fn resolve_parent_label(client: &MyDiseaseClient, parent_id: &str) -> String {
    let parent_id = parent_id.trim();
    if parent_id.is_empty() {
        return String::new();
    }

    if let Ok(hit) = client.get(parent_id).await {
        let parent_name = transform::disease::name_from_mydisease_hit(&hit);
        if !parent_name.eq_ignore_ascii_case(parent_id) {
            return format!("{parent_name} ({parent_id})");
        }
    }

    if let Ok(resp) = client.query(parent_id, 1, 0, None, None, None, None).await
        && let Some(hit) = resp.hits.first()
    {
        let parent_name = transform::disease::name_from_mydisease_hit(hit);
        if !parent_name.eq_ignore_ascii_case(parent_id) {
            return format!("{parent_name} ({parent_id})");
        }
    }

    parent_id.to_string()
}

async fn resolve_parent_names(client: &MyDiseaseClient, parents: &[String]) -> Vec<String> {
    let mut lookups = Vec::new();
    for parent in parents {
        let parent_id = parent.trim();
        if parent_id.is_empty() {
            continue;
        }
        lookups.push(async move { resolve_parent_label(client, parent_id).await });
    }
    join_all(lookups)
        .await
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect()
}

#[cfg(test)]
mod tests;
