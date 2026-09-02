//! Disease markdown renderers and disease-specific view helpers.

use super::*;
#[cfg(test)]
pub(crate) mod tests;

#[derive(serde::Serialize)]
struct XrefRow {
    source: String,
    id: String,
}

#[derive(serde::Serialize)]
struct DiseaseGeneAssociationRenderRow {
    gene: String,
    relationship: Option<String>,
    source: Option<String>,
    source_url: Option<String>,
    opentargets: Option<String>,
}

#[derive(serde::Serialize)]
struct DiseasePhenotypeRenderRow {
    hpo_id: String,
    name: Option<String>,
    evidence: Option<String>,
    frequency: Option<String>,
    frequency_qualifier: Option<String>,
    onset_qualifier: Option<String>,
    sex_qualifier: Option<String>,
    stage_qualifier: Option<String>,
    qualifiers: Vec<String>,
    source: Option<String>,
    source_url: Option<String>,
}

#[derive(serde::Serialize)]
struct DiseaseModelAssociationRenderRow {
    model: String,
    organism: Option<String>,
    relationship: Option<String>,
    source: Option<String>,
    source_url: Option<String>,
    evidence_count: Option<u32>,
}

#[derive(serde::Serialize)]
struct DiseaseSurvivalSummaryRenderRow {
    sex: String,
    latest_observed_year: Option<u16>,
    relative_survival: Option<String>,
    ci_95: Option<String>,
    cases: Option<u32>,
    latest_modeled: Option<String>,
}

#[derive(serde::Serialize)]
struct DiseaseSurvivalHistoryRenderRow {
    sex: String,
    year: u16,
    relative_survival: String,
    ci_95: Option<String>,
    cases: Option<u32>,
}

fn format_disease_association_score(summary: &DiseaseAssociationScoreSummary) -> String {
    let mut parts = vec![format!("overall {:.3}", summary.overall_score)];
    if let Some(score) = summary.gwas_score {
        parts.push(format!("GWAS {:.3}", score));
    }
    if let Some(score) = summary.rare_variant_score {
        parts.push(format!("rare {:.3}", score));
    }
    if let Some(score) = summary.somatic_mutation_score {
        parts.push(format!("somatic {:.3}", score));
    }
    parts.join("; ")
}

fn disease_top_gene_score_labels(disease: &Disease) -> Vec<String> {
    disease
        .top_gene_scores
        .iter()
        .take(5)
        .map(|row| format!("{} (OT {:.3})", row.symbol, row.summary.overall_score))
        .collect()
}

fn disease_gene_association_rows(disease: &Disease) -> Vec<DiseaseGeneAssociationRenderRow> {
    disease
        .gene_associations
        .iter()
        .map(|row| DiseaseGeneAssociationRenderRow {
            gene: row.gene.clone(),
            relationship: row.relationship.clone(),
            source: row.source.clone(),
            source_url: disease_source_url(disease, row.source.as_deref(), None),
            opentargets: row
                .opentargets_score
                .as_ref()
                .map(format_disease_association_score),
        })
        .collect()
}

fn disease_phenotype_rows(
    disease: &Disease,
    rows: &[DiseasePhenotype],
) -> Vec<DiseasePhenotypeRenderRow> {
    rows.iter()
        .map(|row| DiseasePhenotypeRenderRow {
            hpo_id: row.hpo_id.clone(),
            name: row.name.clone(),
            evidence: row.evidence.clone(),
            frequency: row.frequency.clone(),
            frequency_qualifier: row.frequency_qualifier.clone(),
            onset_qualifier: row.onset_qualifier.clone(),
            sex_qualifier: row.sex_qualifier.clone(),
            stage_qualifier: row.stage_qualifier.clone(),
            qualifiers: row.qualifiers.clone(),
            source: row.source.clone(),
            source_url: disease_source_url(disease, row.source.as_deref(), None),
        })
        .collect()
}

fn disease_diagnostic_search_command(disease: &Disease) -> Option<String> {
    let query = if disease.name.trim().is_empty() {
        disease.id.trim()
    } else {
        disease.name.trim()
    };
    if query.is_empty() {
        return None;
    }
    Some(format!(
        "biomcp search diagnostic --disease {} --source all --limit 50",
        shell_quote_arg(query)
    ))
}

fn disease_model_rows(disease: &Disease) -> Vec<DiseaseModelAssociationRenderRow> {
    disease
        .models
        .iter()
        .map(|row| DiseaseModelAssociationRenderRow {
            model: row.model.clone(),
            organism: row.organism.clone(),
            relationship: row.relationship.clone(),
            source: row.source.clone(),
            source_url: disease_source_url(disease, row.source.as_deref(), row.model_id.as_deref())
                .or_else(|| disease_source_url(disease, row.source.as_deref(), Some(&row.model))),
            evidence_count: row.evidence_count,
        })
        .collect()
}

fn format_survival_percent(value: Option<f64>) -> Option<String> {
    value.map(|value| format!("{value:.1}%"))
}

fn format_survival_ci(lower_ci: Option<f64>, upper_ci: Option<f64>) -> Option<String> {
    match (lower_ci, upper_ci) {
        (Some(lower), Some(upper)) => Some(format!("{lower:.1}%-{upper:.1}%")),
        _ => None,
    }
}

fn disease_survival_source_line(disease: &Disease) -> Option<String> {
    disease.survival.as_ref().map(|survival| {
        format!(
            "{} (site code {}) · All Ages · All Races / Ethnicities",
            survival.site_label, survival.site_code
        )
    })
}

fn disease_survival_summary_rows(disease: &Disease) -> Vec<DiseaseSurvivalSummaryRenderRow> {
    let Some(survival) = disease.survival.as_ref() else {
        return Vec::new();
    };

    survival
        .series
        .iter()
        .map(|series| {
            let latest_observed = series.latest_observed.as_ref();
            let latest_modeled = series.latest_modeled.as_ref().and_then(|point| {
                point
                    .modeled_relative_survival_rate
                    .map(|value| format!("{}: {value:.1}%", point.year))
            });

            DiseaseSurvivalSummaryRenderRow {
                sex: series.sex.clone(),
                latest_observed_year: latest_observed.map(|point| point.year),
                relative_survival: format_survival_percent(
                    latest_observed.and_then(|point| point.relative_survival_rate),
                ),
                ci_95: latest_observed
                    .and_then(|point| format_survival_ci(point.lower_ci, point.upper_ci)),
                cases: latest_observed.and_then(|point| point.case_count),
                latest_modeled,
            }
        })
        .collect()
}

fn disease_survival_history_rows(disease: &Disease) -> Vec<DiseaseSurvivalHistoryRenderRow> {
    let Some(survival) = disease.survival.as_ref() else {
        return Vec::new();
    };

    let mut rows = Vec::new();
    for series in &survival.series {
        for point in series
            .points
            .iter()
            .rev()
            .filter(|point| point.relative_survival_rate.is_some())
            .take(10)
        {
            rows.push(DiseaseSurvivalHistoryRenderRow {
                sex: series.sex.clone(),
                year: point.year,
                relative_survival: format!("{:.1}%", point.relative_survival_rate.unwrap_or(0.0)),
                ci_95: format_survival_ci(point.lower_ci, point.upper_ci),
                cases: point.case_count,
            });
        }
    }

    rows
}

pub fn disease_markdown(
    disease: &Disease,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    disease_markdown_with_identity_notice(disease, requested_sections, None)
}

pub(crate) fn disease_markdown_with_identity_notice(
    disease: &Disease,
    requested_sections: &[String],
    identity_notice: Option<&str>,
) -> Result<String, BioMcpError> {
    let mut xrefs: Vec<XrefRow> = disease
        .xrefs
        .iter()
        .map(|(k, v)| XrefRow {
            source: k.clone(),
            id: v.clone(),
        })
        .collect();
    xrefs.sort_by(|a, b| a.source.cmp(&b.source));

    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_genes_section = include_all || has_requested("genes");
    let show_pathways_section = include_all || has_requested("pathways");
    let show_phenotypes_section = include_all || has_requested("phenotypes");
    let show_variants_section = include_all || has_requested("variants");
    let show_models_section = include_all || has_requested("models");
    let show_prevalence_section = include_all || has_requested("prevalence");
    let show_survival_section = include_all || has_requested("survival");
    let show_funding_section = has_requested("funding");
    let show_diagnostics_section = has_requested("diagnostics");
    let show_clinical_features_section = has_requested("clinical_features");
    let show_civic_section = include_all || has_requested("civic");
    let show_disgenet_section = has_requested("disgenet");
    let disease_label = if disease.name.trim().is_empty() {
        disease.id.as_str()
    } else {
        disease.name.as_str()
    };

    let tmpl = env()?.get_template("disease.md.j2")?;
    let top_gene_score_labels = disease_top_gene_score_labels(disease);
    let gene_association_rows = disease_gene_association_rows(disease);
    let phenotype_rows = disease_phenotype_rows(disease, &disease.phenotypes);
    let clinical_features = disease_phenotype_rows(disease, &disease.clinical_features);
    let model_rows = disease_model_rows(disease);
    let survival_source_line = disease_survival_source_line(disease);
    let survival_summary_rows = disease_survival_summary_rows(disease);
    let survival_history_rows = disease_survival_history_rows(disease);
    let funding_rows = funding_rows(disease.funding.as_ref());
    let funding_summary = funding_summary_line(disease.funding.as_ref());
    let diagnostic_rows =
        super::diagnostic::diagnostic_search_rows(disease.diagnostics.as_deref().unwrap_or(&[]));
    let diagnostic_search_command = disease_diagnostic_search_command(disease);
    let hide_note = |key: &str| {
        disease.section_outcomes.get(key).is_some_and(|outcome| {
            matches!(
                outcome.outcome(),
                crate::entities::section_outcome::SectionOutcomeState::Degraded
                    | crate::entities::section_outcome::SectionOutcomeState::Unavailable
            )
        })
    };
    let prevalence_note = (!hide_note("prevalence"))
        .then_some(disease.prevalence_note.as_ref())
        .flatten();
    let survival_note = (!hide_note("survival"))
        .then_some(disease.survival_note.as_ref())
        .flatten();
    let funding_note = (!hide_note("funding"))
        .then_some(disease.funding_note.as_ref())
        .flatten();
    let diagnostics_note = (!hide_note("diagnostics"))
        .then_some(disease.diagnostics_note.as_ref())
        .flatten();
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(disease_label, requested_sections),
        id => &disease.id,
        name => &disease.name,
        identity_notice => identity_notice,
        definition => &disease.definition,
        synonyms => &disease.synonyms,
        parents => &disease.parents,
        associated_genes => &disease.associated_genes,
        gene_associations => &disease.gene_associations,
        gene_association_rows => gene_association_rows,
        top_genes => &disease.top_genes,
        top_gene_scores => &disease.top_gene_scores,
        top_gene_score_labels => top_gene_score_labels,
        treatment_landscape => &disease.treatment_landscape,
        recruiting_trial_count => &disease.recruiting_trial_count,
        pathways => &disease.pathways,
        phenotypes => &phenotype_rows,
        clinical_features => &clinical_features,
        key_features => &disease.key_features,
        has_definition => disease.definition.is_some(),
        literature_query => disease_literature_query(disease),
        variants => &disease.variants,
        top_variant => &disease.top_variant,
        models => model_rows,
        prevalence => &disease.prevalence,
        prevalence_note => prevalence_note,
        survival => &disease.survival,
        survival_note => survival_note,
        funding => &disease.funding,
        funding_note => funding_note,
        funding_rows => funding_rows,
        funding_summary => funding_summary,
        diagnostics_note => diagnostics_note,
        diagnostic_rows => diagnostic_rows,
        diagnostic_search_command => diagnostic_search_command,
        survival_source_line => survival_source_line,
        survival_summary_rows => survival_summary_rows,
        survival_history_rows => survival_history_rows,
        civic => &disease.civic,
        disgenet => &disease.disgenet,
        show_genes_section => show_genes_section,
        show_pathways_section => show_pathways_section,
        show_phenotypes_section => show_phenotypes_section,
        show_variants_section => show_variants_section,
        show_models_section => show_models_section,
        show_prevalence_section => show_prevalence_section,
        show_survival_section => show_survival_section,
        show_funding_section => show_funding_section,
        show_diagnostics_section => show_diagnostics_section,
        show_clinical_features_section => show_clinical_features_section,
        show_civic_section => show_civic_section,
        show_disgenet_section => show_disgenet_section,
        show_disgenet_empty => section_has_healthy_empty(&disease.section_outcomes, "disgenet"),
        xrefs => xrefs,
        sections_block => format_sections_block("disease", &disease.id, sections_disease(disease, requested_sections)),
        related_block => format_related_block(related_disease(disease)),
        source_states => section_render_contexts("disease", &disease.section_outcomes),
    })?;
    Ok(append_evidence_urls(body, disease_evidence_urls(disease)))
}

// dead-code reason: disease::disease_search_markdown is exercised by native renderer contracts
#[allow(dead_code)]
pub fn disease_search_markdown(
    query: &str,
    results: &[DiseaseSearchResult],
) -> Result<String, BioMcpError> {
    disease_search_markdown_with_footer(query, query, results, false, "")
}

pub fn disease_search_markdown_with_footer(
    raw_query: &str,
    query_summary: &str,
    results: &[DiseaseSearchResult],
    fallback_used: bool,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("disease_search.md.j2")?;
    let discover_hint = discover_try_line(raw_query, "resolve abbreviations and synonyms");
    let body = tmpl.render(context! {
        query => query_summary,
        count => results.len(),
        results => results,
        fallback_used => fallback_used,
        discover_hint => discover_hint,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}
