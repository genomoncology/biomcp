//! Disease-associated gene, phenotype, pathway, model, and CIViC/OpenTargets helpers.

use super::*;

use super::resolution::normalize_disease_id;

pub(super) async fn add_genes_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    let mut queries: Vec<String> = Vec::new();
    let mut push_query = |candidate: &str| {
        let candidate = candidate.trim();
        if candidate.is_empty() {
            return;
        }
        if queries.iter().any(|q| q.eq_ignore_ascii_case(candidate)) {
            return;
        }
        queries.push(candidate.to_string());
    };
    // Single-word synonyms are abbreviations (MF, CML, aCML); querying Open
    // Targets with one resolves the abbreviation on its ontology and can
    // return another disease's genes. Only multi-word, name-shaped synonyms
    // join by identity; the resolved record's own name and ID come first
    // (ticket 2017).
    let synonym_candidates = disease
        .synonyms
        .iter()
        .take(3)
        .map(String::as_str)
        .filter(|candidate| candidate.contains(char::is_whitespace));
    for candidate in std::iter::once(disease.name.as_str())
        .chain(synonym_candidates)
        .chain(std::iter::once(disease.id.as_str()))
    {
        if candidate.contains('/') {
            for segment in candidate.split('/') {
                push_query(segment);
            }
        }
        push_query(candidate);
    }
    if queries.is_empty() {
        return Ok(());
    }

    let client = OpenTargetsClient::new()?;
    for query in queries {
        let rows = client.disease_associated_targets(&query, 20).await?;
        if rows.is_empty() {
            continue;
        }

        let mut associated_genes: Vec<String> = Vec::new();
        let mut top_gene_scores = Vec::new();

        for row in rows {
            let symbol = row.symbol.trim();
            if symbol.is_empty() {
                continue;
            }
            if associated_genes
                .iter()
                .any(|v| v.eq_ignore_ascii_case(symbol))
            {
                continue;
            }
            associated_genes.push(symbol.to_string());
            if let Some(summary) = disease_association_summary(&row) {
                top_gene_scores.push(DiseaseTargetScore {
                    symbol: symbol.to_string(),
                    summary,
                });
            }
        }

        if !associated_genes.is_empty() {
            disease.associated_genes = associated_genes;
            disease.top_gene_scores = top_gene_scores;
            disease.associated_genes.truncate(20);
            disease.top_gene_scores.truncate(20);
            // Open Targets replaced the gene list wholesale, so the seed
            // rows (DisGeNET via the MyDisease hit) describe a list that
            // no longer exists — drop them with it (ticket 1256).
            disease.gene_associations.clear();
            return Ok(());
        }
    }

    disease.associated_genes.truncate(20);
    disease.top_gene_scores.clear();
    Ok(())
}

pub(super) async fn add_pathways_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    if disease.associated_genes.is_empty() {
        add_genes_section(disease).await?;
    }
    if disease.associated_genes.is_empty() {
        return Ok(());
    }

    let reactome = ReactomeClient::new()?;
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out: Vec<DiseasePathway> = Vec::new();

    for gene in disease.associated_genes.iter().take(6) {
        let (rows, _) = reactome.search_pathways(gene, 6).await?;
        for row in rows {
            let id = row.id.trim().to_string();
            let name = row.name.trim().to_string();
            if id.is_empty() || name.is_empty() {
                continue;
            }
            if !seen.insert(id.to_ascii_uppercase()) {
                continue;
            }
            out.push(DiseasePathway { id, name });
            if out.len() >= 10 {
                disease.pathways = out;
                return Ok(());
            }
        }
    }

    disease.pathways = out;
    Ok(())
}

pub(super) fn normalize_hpo_id(value: &str) -> Option<String> {
    let mut id = value.trim().to_ascii_uppercase();
    if id.is_empty() {
        return None;
    }
    id = id.replace('_', ":");
    if !id.starts_with("HP:") {
        return None;
    }
    let suffix = id.trim_start_matches("HP:");
    if suffix.is_empty() || !suffix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("HP:{suffix}"))
}

pub(super) async fn add_phenotypes_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    if disease.phenotypes.is_empty() {
        return Ok(());
    }

    let mut ids: Vec<String> = Vec::new();
    for row in &disease.phenotypes {
        if let Some(id) = normalize_hpo_id(&row.hpo_id) {
            ids.push(id);
        }
        if let Some(freq) = row.frequency.as_deref().and_then(normalize_hpo_id) {
            ids.push(freq);
        }
    }
    if ids.is_empty() {
        return Ok(());
    }

    let client = HpoClient::new()?;
    let names = client.resolve_terms(&ids, 20).await?;
    for row in &mut disease.phenotypes {
        if row.name.is_none()
            && let Some(id) = normalize_hpo_id(&row.hpo_id)
        {
            row.name = names.get(&id).cloned();
        }
        if let Some(freq_id) = row.frequency.as_deref().and_then(normalize_hpo_id)
            && let Some(label) = names.get(&freq_id)
        {
            row.frequency = Some(label.clone());
        }
    }
    disease.phenotypes.truncate(20);
    Ok(())
}

fn push_associated_gene(disease: &mut Disease, symbol: &str) {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return;
    }
    if disease
        .associated_genes
        .iter()
        .any(|v| v.eq_ignore_ascii_case(symbol))
    {
        return;
    }
    disease.associated_genes.push(symbol.to_string());
}

fn map_monarch_gene_association(row: MonarchGeneAssociation) -> Option<DiseaseGeneAssociation> {
    let gene = row.gene.trim();
    if gene.is_empty() {
        return None;
    }
    Some(DiseaseGeneAssociation {
        gene: gene.to_string(),
        relationship: row
            .relationship
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        source: row
            .source
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        opentargets_score: None,
    })
}

fn disease_association_summary(
    row: &crate::sources::opentargets::OpenTargetsAssociatedGene,
) -> Option<DiseaseAssociationScoreSummary> {
    Some(DiseaseAssociationScoreSummary {
        overall_score: row.overall_score?,
        gwas_score: row.gwas_score,
        rare_variant_score: row.rare_variant_score,
        somatic_mutation_score: row.somatic_mutation_score,
    })
}

pub(super) fn attach_opentargets_scores(disease: &mut Disease) {
    let score_map = disease
        .top_gene_scores
        .iter()
        .map(|row| (row.symbol.to_ascii_lowercase(), row.summary.clone()))
        .collect::<HashMap<_, _>>();

    for association in &mut disease.gene_associations {
        association.opentargets_score = score_map
            .get(&association.gene.to_ascii_lowercase())
            .cloned();
    }
}

pub(super) async fn add_monarch_gene_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    let disease_id = match normalize_disease_id(&disease.id) {
        Some(v) => v,
        None => return Ok(()),
    };

    let client = MonarchClient::new()?;
    let rows = client.disease_gene_associations(&disease_id, 50).await?;

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let Some(mapped) = map_monarch_gene_association(row) else {
            continue;
        };

        let key = mapped.gene.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        push_associated_gene(disease, &mapped.gene);
        out.push(mapped);
        if out.len() >= 20 {
            break;
        }
    }

    disease.gene_associations = out;
    disease.associated_genes.truncate(20);
    Ok(())
}

fn normalize_gene_source_label(label: &str) -> Option<String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("monarch") {
        Some("Monarch".to_string())
    } else if lower.contains("civic") {
        Some("CIViC".to_string())
    } else if lower.contains("opentarget") || lower.contains("open targets") {
        Some("OpenTargets".to_string())
    } else {
        Some(trimmed.to_string())
    }
}

fn merge_gene_source(existing: &mut Option<String>, new_source: &str) {
    let mut labels: Vec<String> = existing
        .as_deref()
        .into_iter()
        .flat_map(|value| value.split(';'))
        .filter_map(normalize_gene_source_label)
        .collect();
    if let Some(new_label) = normalize_gene_source_label(new_source)
        && !labels
            .iter()
            .any(|value| value.eq_ignore_ascii_case(&new_label))
    {
        labels.push(new_label);
    }

    let mut merged = Vec::new();
    for preferred in ["Monarch", "CIViC", "OpenTargets"] {
        if labels.iter().any(|value| value == preferred) {
            merged.push(preferred.to_string());
        }
    }
    for label in labels {
        if merged
            .iter()
            .any(|value| value.eq_ignore_ascii_case(&label))
        {
            continue;
        }
        merged.push(label);
    }

    *existing = if merged.is_empty() {
        None
    } else {
        Some(merged.join("; "))
    };
}

pub(super) async fn add_monarch_phenotypes(disease: &mut Disease) -> Result<(), BioMcpError> {
    let disease_id = match normalize_disease_id(&disease.id) {
        Some(v) => v,
        None => return Ok(()),
    };

    let client = MonarchClient::new()?;
    let rows = client.disease_phenotypes(&disease_id, 80).await?;
    if rows.is_empty() {
        return Ok(());
    }

    for row in rows {
        let normalized = normalize_hpo_id(&row.hpo_id);
        let Some(hpo_id) = normalized else { continue };

        if let Some(existing) = disease
            .phenotypes
            .iter_mut()
            .find(|p| normalize_hpo_id(&p.hpo_id).is_some_and(|id| id == hpo_id))
        {
            if existing.name.is_none() {
                existing.name = row
                    .label
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string);
            }
            if existing.frequency_qualifier.is_none() {
                existing.frequency_qualifier = row.frequency_qualifier;
            }
            if existing.onset_qualifier.is_none() {
                existing.onset_qualifier = row.onset_qualifier;
            }
            if existing.sex_qualifier.is_none() {
                existing.sex_qualifier = row.sex_qualifier;
            }
            if existing.stage_qualifier.is_none() {
                existing.stage_qualifier = row.stage_qualifier;
            }
            if existing.source.is_none() {
                existing.source = row.source;
            }
            for qualifier in row.qualifiers {
                if qualifier.trim().is_empty() {
                    continue;
                }
                if existing
                    .qualifiers
                    .iter()
                    .any(|v| v.eq_ignore_ascii_case(&qualifier))
                {
                    continue;
                }
                existing.qualifiers.push(qualifier);
            }
            continue;
        }

        disease.phenotypes.push(DiseasePhenotype {
            hpo_id,
            name: row
                .label
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
            evidence: row.relationship,
            frequency: None,
            frequency_qualifier: row.frequency_qualifier,
            onset_qualifier: row.onset_qualifier,
            sex_qualifier: row.sex_qualifier,
            stage_qualifier: row.stage_qualifier,
            qualifiers: row.qualifiers,
            source: row.source,
        });
    }

    disease.phenotypes.truncate(40);
    Ok(())
}

fn looks_like_protein_change(token: &str) -> bool {
    let chars = token.chars().collect::<Vec<_>>();
    if chars.len() < 3 {
        return false;
    }
    chars.first().is_some_and(char::is_ascii_alphabetic)
        && chars.last().is_some_and(char::is_ascii_alphabetic)
        && chars[1..chars.len() - 1].iter().all(char::is_ascii_digit)
}

fn is_hgnc_symbol_candidate(token: &str) -> bool {
    let token = token.trim();
    if token.len() < 2 || token.len() > 15 {
        return false;
    }
    if !token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return false;
    }
    if !token
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
    {
        return false;
    }
    if looks_like_protein_change(token) {
        return false;
    }

    let upper = token.to_ascii_uppercase();
    let excluded = [
        "MUTATION",
        "MUTATIONS",
        "AMPLIFICATION",
        "DELETION",
        "FUSION",
        "WILD",
        "TYPE",
        "LOSS",
        "GAIN",
    ];
    !excluded.contains(&upper.as_str())
}

fn civic_gene_symbol_from_profile(profile: &str) -> Option<String> {
    for token in profile.split(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
        let token = token.trim();
        if token.is_empty() || !is_hgnc_symbol_candidate(token) {
            continue;
        }
        return Some(token.to_ascii_uppercase());
    }
    None
}

pub(super) async fn augment_genes_with_civic(disease: &mut Disease) -> Result<(), BioMcpError> {
    let Some(query) = disease_query_value(disease) else {
        return Ok(());
    };

    let client = CivicClient::new()?;
    let context = client.by_disease(&query, 25).await?;
    let mut seen = disease
        .gene_associations
        .iter()
        .map(|row| row.gene.to_ascii_lowercase())
        .collect::<HashSet<_>>();

    for symbol in context
        .evidence_items
        .iter()
        .filter_map(|row| civic_gene_symbol_from_profile(&row.molecular_profile))
        .chain(
            context
                .assertions
                .iter()
                .filter_map(|row| civic_gene_symbol_from_profile(&row.molecular_profile)),
        )
    {
        let key = symbol.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        push_associated_gene(disease, &symbol);
        disease.gene_associations.push(DiseaseGeneAssociation {
            gene: symbol,
            relationship: Some("associated with disease".into()),
            source: Some("CIViC".into()),
            opentargets_score: None,
        });
        if disease.gene_associations.len() >= 20 {
            break;
        }
    }

    disease.gene_associations.truncate(20);
    disease.associated_genes.truncate(20);
    Ok(())
}

pub(super) async fn augment_genes_with_opentargets(
    disease: &mut Disease,
) -> Result<(), BioMcpError> {
    for score in disease.top_gene_scores.clone() {
        let existing = disease
            .gene_associations
            .iter_mut()
            .find(|row| row.gene.eq_ignore_ascii_case(&score.symbol));
        if let Some(row) = existing {
            merge_gene_source(&mut row.source, "OpenTargets");
            continue;
        }
        if disease.gene_associations.len() >= 20 {
            break;
        }

        push_associated_gene(disease, &score.symbol);
        disease.gene_associations.push(DiseaseGeneAssociation {
            gene: score.symbol,
            relationship: Some("associated with disease".into()),
            source: Some("OpenTargets".into()),
            opentargets_score: None,
        });
    }

    disease.gene_associations.truncate(20);
    disease.associated_genes.truncate(20);
    Ok(())
}

pub(super) async fn add_civic_variants(disease: &mut Disease) -> Result<(), BioMcpError> {
    let Some(query) = disease_query_value(disease) else {
        return Ok(());
    };

    let client = CivicClient::new()?;
    let context = client.by_disease(&query, 25).await?;

    let mut counts: HashMap<String, (String, u32)> = HashMap::new();
    for profile in context
        .evidence_items
        .iter()
        .map(|row| row.molecular_profile.as_str())
        .chain(
            context
                .assertions
                .iter()
                .map(|row| row.molecular_profile.as_str()),
        )
    {
        let profile = profile.trim();
        if profile.is_empty() {
            continue;
        }
        let key = profile.to_ascii_lowercase();
        let entry = counts
            .entry(key)
            .or_insert_with(|| (profile.to_string(), 0));
        entry.1 += 1;
    }

    let mut rows = counts
        .into_values()
        .map(|(variant, evidence_count)| DiseaseVariantAssociation {
            variant,
            relationship: Some("associated with disease".into()),
            source: Some("CIViC".into()),
            evidence_count: Some(evidence_count),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.evidence_count
            .unwrap_or_default()
            .cmp(&a.evidence_count.unwrap_or_default())
            .then_with(|| a.variant.cmp(&b.variant))
    });
    rows.truncate(20);
    disease.top_variant = rows.first().cloned();
    disease.variants = rows;
    Ok(())
}

fn map_monarch_model(row: MonarchModelAssociation) -> Option<DiseaseModelAssociation> {
    let model = row.model.trim();
    if model.is_empty() {
        return None;
    }
    Some(DiseaseModelAssociation {
        model: model.to_string(),
        model_id: row
            .model_id
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        organism: row
            .organism
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        relationship: row.relationship,
        source: row.source,
        evidence_count: row.evidence_count,
    })
}

pub(super) async fn add_monarch_models(disease: &mut Disease) -> Result<(), BioMcpError> {
    let disease_id = match normalize_disease_id(&disease.id) {
        Some(v) => v,
        None => return Ok(()),
    };

    let client = MonarchClient::new()?;
    let rows = client.disease_models(&disease_id, 50).await?;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let Some(mapped) = map_monarch_model(row) else {
            continue;
        };
        let key = mapped.model.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(mapped);
        if out.len() >= 20 {
            break;
        }
    }
    disease.models = out;
    Ok(())
}

pub(super) fn disease_query_value(disease: &Disease) -> Option<String> {
    if !disease.name.trim().is_empty() {
        Some(disease.name.trim().to_string())
    } else if !disease.id.trim().is_empty() {
        Some(disease.id.trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use self::tests::{
    proof_augment_genes_with_opentargets_merges_sources_without_duplicates,
    proof_augment_genes_with_opentargets_respects_twenty_gene_cap,
};
