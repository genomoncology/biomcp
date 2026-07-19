//! Disease enrichment orchestration and non-association section handlers.

use super::*;

use super::associations::{
    add_civic_variants, add_genes_section, add_monarch_gene_section, add_monarch_models,
    add_monarch_phenotypes, add_pathways_section, add_phenotypes_section,
    attach_opentargets_scores, augment_genes_with_civic, augment_genes_with_opentargets,
    disease_query_value,
};
use super::get::DiseaseSections;
use super::resolution::{DiseaseLookupInput, normalize_disease_id, parse_disease_lookup_input};
use crate::entities::SearchPage;
use crate::entities::diagnostic::{DiagnosticSearchFilters, DiagnosticSearchResult};
use crate::entities::section_outcome::SectionOutcome;

const OPTIONAL_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(8);
const DIAGNOSTIC_PIVOT_LIMIT: usize = 10;
const SURVIVAL_NO_DATA_NOTE: &str = "SEER survival data not available for this condition.";
const SURVIVAL_UNAVAILABLE_NOTE: &str = "SEER survival data is temporarily unavailable.";
const FUNDING_NO_DATA_NOTE: &str = "No NIH funding data found for this query.";
const FUNDING_UNAVAILABLE_NOTE: &str = "NIH Reporter funding data is temporarily unavailable.";
const DISEASE_DIAGNOSTICS_UNAVAILABLE_NOTE: &str = "Diagnostic local data is unavailable. Run `biomcp gtr sync` and `biomcp who-ivd sync` to enable disease diagnostic pivots.";
const GENES_UNAVAILABLE_NOTE: &str = "Disease gene sources are temporarily unavailable.";
const GENES_DEGRADED_NOTE: &str =
    "Disease gene evidence is incomplete because an additive source is unavailable.";
const PATHWAYS_UNAVAILABLE_NOTE: &str = "Reactome pathway data is temporarily unavailable.";
const PHENOTYPES_UNAVAILABLE_NOTE: &str = "Disease phenotype sources are temporarily unavailable.";
const PHENOTYPES_DEGRADED_NOTE: &str =
    "Disease phenotype evidence is incomplete because an additive source is unavailable.";
const VARIANTS_UNAVAILABLE_NOTE: &str = "CIViC variant data is temporarily unavailable.";
const MODELS_UNAVAILABLE_NOTE: &str = "Monarch model data is temporarily unavailable.";
const PREVALENCE_UNAVAILABLE_NOTE: &str =
    "Open Targets prevalence data is temporarily unavailable.";
const CIVIC_UNAVAILABLE_NOTE: &str = "CIViC disease context is temporarily unavailable.";
const TREATMENTS_INAPPLICABLE_NOTE: &str =
    "A disease name or synonym is required for treatment lookup.";
const TREATMENTS_UNAVAILABLE_NOTE: &str = "Disease treatment data is temporarily unavailable.";
const RECRUITING_TRIALS_INAPPLICABLE_NOTE: &str =
    "A disease name or synonym is required for recruiting trial lookup.";
const RECRUITING_TRIALS_UNAVAILABLE_NOTE: &str =
    "Disease recruiting trial data is temporarily unavailable.";

fn normalize_ols_disease_id(value: &str) -> Option<String> {
    normalize_disease_id(value).or_else(|| normalize_disease_id(&value.replace('_', ":")))
}

pub(super) async fn enrich_sparse_disease_identity(
    disease: &mut Disease,
) -> Result<(), BioMcpError> {
    let name = disease.name.trim();
    let id = disease.id.trim();
    if !name.eq_ignore_ascii_case(id) || !disease.synonyms.is_empty() {
        return Ok(());
    }

    let canonical_id = match normalize_disease_id(id) {
        Some(id) => id,
        None => return Ok(()),
    };

    let client = OlsClient::new()?;
    apply_sparse_disease_identity_docs(disease, &canonical_id, client.search(&canonical_id).await?);
    Ok(())
}

fn apply_sparse_disease_identity_docs(
    disease: &mut Disease,
    canonical_id: &str,
    docs: Vec<crate::sources::ols4::OlsDoc>,
) {
    let exact = docs.into_iter().find(|doc| {
        doc.obo_id
            .as_deref()
            .and_then(normalize_ols_disease_id)
            .is_some_and(|value| value == canonical_id)
            || doc
                .short_form
                .as_deref()
                .and_then(normalize_ols_disease_id)
                .is_some_and(|value| value == canonical_id)
    });
    let Some(doc) = exact else {
        return;
    };

    let label = doc.label.trim();
    if !label.is_empty() {
        disease.name = label.to_string();
    }

    let mut seen = disease
        .synonyms
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    seen.insert(disease.name.to_ascii_lowercase());
    for synonym in doc.exact_synonyms {
        let synonym = synonym.trim();
        if synonym.is_empty() {
            continue;
        }
        let key = synonym.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        disease.synonyms.push(synonym.to_string());
        if disease.synonyms.len() >= 10 {
            break;
        }
    }
}

fn disease_funding_query_value(
    disease: &Disease,
    requested_lookup: Option<&str>,
) -> Option<String> {
    if let Some(requested_lookup) = requested_lookup {
        let requested_lookup = requested_lookup.trim();
        if !requested_lookup.is_empty()
            && matches!(
                parse_disease_lookup_input(requested_lookup),
                DiseaseLookupInput::FreeText
            )
        {
            return Some(requested_lookup.to_string());
        }
    }

    if !disease.name.trim().is_empty() {
        return Some(disease.name.trim().to_string());
    }

    disease.synonyms.iter().find_map(|synonym| {
        let synonym = synonym.trim();
        (!synonym.is_empty()).then(|| synonym.to_string())
    })
}

async fn add_treatment_landscape(disease: &mut Disease) -> Result<(), BioMcpError> {
    let Some(query) = disease_query_value(disease) else {
        disease.treatment_landscape.clear();
        disease.section_outcomes.complete(
            DISEASE_SECTION_TREATMENTS,
            SectionOutcome::inapplicable(TREATMENTS_INAPPLICABLE_NOTE),
        );
        return Ok(());
    };

    let filters = DrugSearchFilters {
        indication: Some(query),
        ..Default::default()
    };
    let rows = match drug::search(&filters, 5).await {
        Ok(rows) => rows,
        Err(err) => {
            disease.treatment_landscape.clear();
            disease.section_outcomes.complete(
                DISEASE_SECTION_TREATMENTS,
                SectionOutcome::unavailable(TREATMENTS_UNAVAILABLE_NOTE),
            );
            return Err(err);
        }
    };

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for row in rows {
        let name = row.name.trim();
        if name.is_empty() {
            continue;
        }
        let key = name.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(name.to_string());
        if out.len() >= 5 {
            break;
        }
    }

    let outcome = if out.is_empty() {
        SectionOutcome::empty("MyChem.info indication search")
    } else {
        SectionOutcome::data("MyChem.info indication search")
    };
    disease.treatment_landscape = out;
    disease
        .section_outcomes
        .complete(DISEASE_SECTION_TREATMENTS, outcome);
    Ok(())
}

async fn add_recruiting_trial_count(disease: &mut Disease) -> Result<(), BioMcpError> {
    let Some(query) = disease_query_value(disease) else {
        disease.recruiting_trial_count = None;
        disease.section_outcomes.complete(
            DISEASE_SECTION_RECRUITING_TRIALS,
            SectionOutcome::inapplicable(RECRUITING_TRIALS_INAPPLICABLE_NOTE),
        );
        return Ok(());
    };

    let filters = TrialSearchFilters {
        condition: Some(query),
        status: Some("recruiting".to_string()),
        source: TrialSource::ClinicalTrialsGov,
        ..Default::default()
    };

    let (rows, total) = match trial::search(&filters, 5, 0).await {
        Ok(result) => result,
        Err(err) => {
            disease.recruiting_trial_count = None;
            disease.section_outcomes.complete(
                DISEASE_SECTION_RECRUITING_TRIALS,
                SectionOutcome::unavailable(RECRUITING_TRIALS_UNAVAILABLE_NOTE),
            );
            return Err(err);
        }
    };
    disease.recruiting_trial_count = total.or(Some(rows.len() as u32));
    disease.section_outcomes.complete(
        DISEASE_SECTION_RECRUITING_TRIALS,
        SectionOutcome::data("ClinicalTrials.gov"),
    );
    Ok(())
}

async fn add_prevalence_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    let mut queries: Vec<String> = Vec::new();
    for query in [disease.id.trim(), disease.name.trim()] {
        if query.is_empty() {
            continue;
        }
        if queries.iter().any(|q| q.eq_ignore_ascii_case(query)) {
            continue;
        }
        queries.push(query.to_string());
    }
    if queries.is_empty() {
        disease.prevalence.clear();
        disease.prevalence_note = Some("No prevalence data available from OpenTargets.".into());
        return Ok(());
    }

    let client = OpenTargetsClient::new()?;
    for query in queries {
        let rows = client.disease_prevalence(&query, 8).await?;
        if rows.is_empty() {
            continue;
        }
        disease.prevalence = rows
            .into_iter()
            .map(|row| DiseasePrevalenceEvidence {
                estimate: row.estimate,
                context: row.context,
                source: row.source,
            })
            .collect();
        disease.prevalence_note = None;
        return Ok(());
    }

    disease.prevalence.clear();
    disease.prevalence_note = Some("No prevalence data available from OpenTargets.".into());
    Ok(())
}

fn map_survival_payload(payload: SeerSurvivalPayload) -> DiseaseSurvival {
    DiseaseSurvival {
        site_code: payload.site_code,
        site_label: payload.site_label,
        series: payload
            .series
            .into_iter()
            .map(map_survival_series)
            .collect(),
    }
}

fn map_survival_series(series: crate::sources::seer::SeerSurvivalSeries) -> DiseaseSurvivalSeries {
    let points = series
        .points
        .into_iter()
        .map(map_survival_point)
        .collect::<Vec<_>>();
    let latest_observed = points
        .iter()
        .rev()
        .find(|point| point.relative_survival_rate.is_some())
        .cloned();
    let latest_observed_year = latest_observed.as_ref().map(|point| point.year);
    let latest_modeled = points
        .iter()
        .rev()
        .find(|point| {
            point.modeled_relative_survival_rate.is_some()
                && latest_observed_year.is_none_or(|year| point.year > year)
        })
        .cloned();

    DiseaseSurvivalSeries {
        sex: series.sex_label,
        latest_observed,
        latest_modeled,
        points,
    }
}

fn map_survival_point(point: crate::sources::seer::SeerSurvivalPoint) -> DiseaseSurvivalPoint {
    DiseaseSurvivalPoint {
        year: point.year,
        relative_survival_rate: point.relative_survival_rate,
        standard_error: point.standard_error,
        lower_ci: point.lower_ci,
        upper_ci: point.upper_ci,
        modeled_relative_survival_rate: point.modeled_relative_survival_rate,
        case_count: point.case_count,
    }
}

async fn add_survival_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    let client = match SeerClient::new() {
        Ok(client) => client,
        Err(_) => {
            disease.survival = None;
            disease.survival_note = Some(SURVIVAL_UNAVAILABLE_NOTE.into());
            disease.section_outcomes.complete(
                DISEASE_SECTION_SURVIVAL,
                SectionOutcome::unavailable(SURVIVAL_UNAVAILABLE_NOTE),
            );
            return Ok(());
        }
    };

    let Some((site, catalog)) =
        resolve_survival_site_from_catalog_result(disease, client.site_catalog().await)
    else {
        return Ok(());
    };

    match client.fetch_survival(site.site_code, &catalog).await {
        Ok(payload) => {
            let survival = map_survival_payload(payload);
            let outcome = if survival.series.is_empty() {
                SectionOutcome::empty("SEER Explorer")
            } else {
                SectionOutcome::data("SEER Explorer")
            };
            disease.survival = Some(survival);
            disease.survival_note = None;
            disease
                .section_outcomes
                .complete(DISEASE_SECTION_SURVIVAL, outcome);
        }
        Err(_) => {
            disease.survival = None;
            disease.survival_note = Some(SURVIVAL_UNAVAILABLE_NOTE.into());
            disease.section_outcomes.complete(
                DISEASE_SECTION_SURVIVAL,
                SectionOutcome::unavailable(SURVIVAL_UNAVAILABLE_NOTE),
            );
        }
    }

    Ok(())
}

fn resolve_survival_site_from_catalog_result(
    disease: &mut Disease,
    catalog: Result<SeerSiteCatalog, BioMcpError>,
) -> Option<(ResolvedSeerSite, SeerSiteCatalog)> {
    let catalog = match catalog {
        Ok(catalog) => catalog,
        Err(_) => {
            disease.survival = None;
            disease.survival_note = Some(SURVIVAL_UNAVAILABLE_NOTE.into());
            disease.section_outcomes.complete(
                DISEASE_SECTION_SURVIVAL,
                SectionOutcome::unavailable(SURVIVAL_UNAVAILABLE_NOTE),
            );
            return None;
        }
    };

    let Some(site) = resolve_site(disease, &catalog) else {
        disease.survival = None;
        disease.survival_note = Some(SURVIVAL_NO_DATA_NOTE.into());
        disease.section_outcomes.complete(
            DISEASE_SECTION_SURVIVAL,
            SectionOutcome::empty("SEER Explorer"),
        );
        return None;
    };

    Some((site, catalog))
}

async fn add_civic_section(disease: &mut Disease) -> SectionOutcome {
    let Some(query) = disease_query_value(disease) else {
        disease.civic = Some(CivicContext::default());
        return SectionOutcome::empty("CIViC");
    };

    let civic_fut = async {
        let client = CivicClient::new()?;
        client.by_disease(&query, 10).await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, civic_fut).await {
        Ok(Ok(context)) => {
            let has_data = context.evidence_total_count > 0
                || context.assertion_total_count > 0
                || !context.evidence_items.is_empty()
                || !context.assertions.is_empty();
            disease.civic = Some(context);
            if has_data {
                SectionOutcome::data("CIViC")
            } else {
                SectionOutcome::empty("CIViC")
            }
        }
        Ok(Err(_)) | Err(_) => {
            disease.civic = Some(CivicContext::default());
            SectionOutcome::unavailable(CIVIC_UNAVAILABLE_NOTE)
        }
    }
}

async fn add_diagnostics_section(disease: &mut Disease) -> SectionOutcome {
    let Some(query) = disease_query_value(disease) else {
        disease.diagnostics = Some(Vec::new());
        disease.diagnostics_note = None;
        return SectionOutcome::empty_sources([
            "NCBI Genetic Testing Registry",
            "WHO Prequalified IVD",
        ]);
    };

    let filters = DiagnosticSearchFilters {
        disease: Some(query),
        ..Default::default()
    };
    let result =
        crate::entities::diagnostic::search_page(&filters, DIAGNOSTIC_PIVOT_LIMIT, 0).await;
    apply_diagnostics_section_result(disease, result)
}

fn apply_diagnostics_section_result(
    disease: &mut Disease,
    result: Result<SearchPage<DiagnosticSearchResult>, BioMcpError>,
) -> SectionOutcome {
    match result {
        Ok(page) => {
            let shown = page.results.len();
            let capped = match page.total {
                Some(total) => total > shown,
                None => shown >= DIAGNOSTIC_PIVOT_LIMIT,
            };
            let note = if capped {
                Some(match page.total {
                    Some(total) => format!(
                        "Showing {shown} of {total} diagnostic matches in this disease card. Use diagnostic search with --limit and --offset for the larger result set."
                    ),
                    None => format!(
                        "Showing first {shown} diagnostic matches in this disease card. Use diagnostic search with --limit and --offset for the larger result set."
                    ),
                })
            } else {
                None
            };
            let mut sources = Vec::new();
            for source in page
                .results
                .iter()
                .map(|row| crate::entities::diagnostic::diagnostic_source_label(&row.source))
            {
                if !sources.contains(&source) {
                    sources.push(source);
                }
            }
            disease.diagnostics = Some(page.results);
            disease.diagnostics_note = note;
            if shown == 0 {
                SectionOutcome::empty_sources([
                    "NCBI Genetic Testing Registry",
                    "WHO Prequalified IVD",
                ])
            } else {
                SectionOutcome::data_sources(sources)
            }
        }
        Err(_) => {
            disease.diagnostics = None;
            disease.diagnostics_note = Some(DISEASE_DIAGNOSTICS_UNAVAILABLE_NOTE.into());
            SectionOutcome::unavailable(DISEASE_DIAGNOSTICS_UNAVAILABLE_NOTE)
        }
    }
}

fn empty_funding_section(query: String) -> NihReporterFundingSection {
    NihReporterFundingSection {
        query,
        fiscal_years: Vec::new(),
        matching_project_years: 0,
        grants: Vec::new(),
    }
}

async fn add_funding_section(
    disease: &mut Disease,
    requested_lookup: Option<&str>,
) -> SectionOutcome {
    let Some(query) = disease_funding_query_value(disease, requested_lookup) else {
        disease.funding = Some(empty_funding_section(String::new()));
        disease.funding_note = Some(FUNDING_NO_DATA_NOTE.into());
        return SectionOutcome::empty("NIH Reporter");
    };

    let funding_fut = async {
        let client = NihReporterClient::new()?;
        client.funding(&query).await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, funding_fut).await {
        Ok(Ok(section)) => {
            let no_hits = section.matching_project_years == 0 && section.grants.is_empty();
            disease.funding = Some(section);
            disease.funding_note = if no_hits {
                Some(FUNDING_NO_DATA_NOTE.into())
            } else {
                None
            };
            if no_hits {
                SectionOutcome::empty("NIH Reporter")
            } else {
                SectionOutcome::data("NIH Reporter")
            }
        }
        Ok(Err(_)) | Err(_) => {
            disease.funding = None;
            disease.funding_note = Some(FUNDING_UNAVAILABLE_NOTE.into());
            SectionOutcome::unavailable(FUNDING_UNAVAILABLE_NOTE)
        }
    }
}

fn map_disgenet_disease_association(row: DisgenetAssociationRecord) -> DiseaseDisgenetAssociation {
    DiseaseDisgenetAssociation {
        symbol: row.gene_symbol,
        entrez_id: row.gene_ncbi_id,
        score: row.score,
        publication_count: row.publication_count,
        clinical_trial_count: row.clinical_trial_count,
        evidence_index: row.evidence_index,
        evidence_level: row.evidence_level,
    }
}

async fn add_disgenet_section(disease: &mut Disease) -> Result<(), BioMcpError> {
    let client = DisgenetClient::new()?;
    let associations = client
        .fetch_disease_associations(disease, 10)
        .await?
        .into_iter()
        .map(map_disgenet_disease_association)
        .collect();
    disease.disgenet = Some(DiseaseDisgenet { associations });
    Ok(())
}

pub(super) async fn enrich_base_context(disease: &mut Disease) {
    let _ = add_genes_section(disease).await;

    disease.top_genes = if disease.top_gene_scores.is_empty() {
        disease.associated_genes.iter().take(5).cloned().collect()
    } else {
        disease
            .top_gene_scores
            .iter()
            .take(5)
            .map(|row| row.symbol.clone())
            .collect()
    };

    if let Err(err) = add_treatment_landscape(disease).await {
        warn!("Drug lookup unavailable for disease treatment landscape: {err}");
    }

    if let Err(err) = add_recruiting_trial_count(disease).await {
        warn!("Trial lookup unavailable for disease recruiting count: {err}");
    }
}

pub(super) async fn apply_requested_sections(
    disease: &mut Disease,
    sections: DiseaseSections,
    requested_lookup: Option<&str>,
) -> Result<(), BioMcpError> {
    if sections.include_genes {
        let had_opentargets_data = !disease.top_gene_scores.is_empty();
        let monarch_result = add_monarch_gene_section(disease).await;
        let civic_result = augment_genes_with_civic(disease).await;
        let opentargets_result = augment_genes_with_opentargets(disease).await;
        attach_opentargets_scores(disease);

        let mut contributors = Vec::new();
        if had_opentargets_data {
            contributors.push("Open Targets");
        }
        if monarch_result.is_ok()
            && disease.gene_associations.iter().any(|row| {
                row.source
                    .as_deref()
                    .is_some_and(|source| source.to_ascii_lowercase().contains("monarch"))
            })
        {
            contributors.push("Monarch Initiative");
        }
        if civic_result.is_ok()
            && disease.gene_associations.iter().any(|row| {
                row.source
                    .as_deref()
                    .is_some_and(|source| source.to_ascii_lowercase().contains("civic"))
            })
        {
            contributors.push("CIViC");
        }
        let failed =
            monarch_result.is_err() || civic_result.is_err() || opentargets_result.is_err();
        let outcome = if contributors.is_empty() && failed {
            SectionOutcome::unavailable(GENES_UNAVAILABLE_NOTE)
        } else if failed {
            SectionOutcome::degraded(contributors, GENES_DEGRADED_NOTE)
        } else if contributors.is_empty() {
            SectionOutcome::empty_sources(["Monarch Initiative", "CIViC", "Open Targets"])
        } else {
            SectionOutcome::data_sources(contributors)
        };
        disease
            .section_outcomes
            .complete(DISEASE_SECTION_GENES, outcome);
    }
    if sections.include_pathways {
        let outcome = match add_pathways_section(disease).await {
            Ok(()) if disease.pathways.is_empty() => SectionOutcome::empty("Reactome"),
            Ok(()) => SectionOutcome::data("Reactome"),
            Err(_) => SectionOutcome::unavailable(PATHWAYS_UNAVAILABLE_NOTE),
        };
        disease
            .section_outcomes
            .complete(DISEASE_SECTION_PATHWAYS, outcome);
    }
    let needs_backend_phenotypes =
        sections.include_phenotypes || sections.include_clinical_features;
    if needs_backend_phenotypes {
        let had_mydisease_data = !disease.phenotypes.is_empty();
        let monarch_result = add_monarch_phenotypes(disease).await;
        let hpo_result = add_phenotypes_section(disease).await;
        let failed = monarch_result.is_err() || hpo_result.is_err();
        let mut contributors = Vec::new();
        if had_mydisease_data {
            contributors.push("MyDisease.info");
        }
        if monarch_result.is_ok()
            && disease.phenotypes.iter().any(|row| {
                row.source
                    .as_deref()
                    .is_some_and(|source| source.to_ascii_lowercase().contains("monarch"))
            })
        {
            contributors.push("Monarch Initiative");
        }
        if hpo_result.is_ok() && disease.phenotypes.iter().any(|row| row.name.is_some()) {
            contributors.push("HPO");
        }
        let outcome = if contributors.is_empty() && failed {
            SectionOutcome::unavailable(PHENOTYPES_UNAVAILABLE_NOTE)
        } else if failed {
            SectionOutcome::degraded(contributors, PHENOTYPES_DEGRADED_NOTE)
        } else if contributors.is_empty() {
            SectionOutcome::empty_sources(["Monarch Initiative", "HPO"])
        } else {
            SectionOutcome::data_sources(contributors)
        };
        disease
            .section_outcomes
            .complete(DISEASE_SECTION_PHENOTYPES, outcome);
    }
    if sections.include_variants {
        let outcome = match add_civic_variants(disease).await {
            Ok(()) if disease.variants.is_empty() => SectionOutcome::empty("CIViC"),
            Ok(()) => SectionOutcome::data("CIViC"),
            Err(_) => SectionOutcome::unavailable(VARIANTS_UNAVAILABLE_NOTE),
        };
        disease
            .section_outcomes
            .complete(DISEASE_SECTION_VARIANTS, outcome);
    }
    if sections.include_models {
        let outcome = match add_monarch_models(disease).await {
            Ok(()) if disease.models.is_empty() => SectionOutcome::empty("Monarch Initiative"),
            Ok(()) => SectionOutcome::data("Monarch Initiative"),
            Err(_) => SectionOutcome::unavailable(MODELS_UNAVAILABLE_NOTE),
        };
        disease
            .section_outcomes
            .complete(DISEASE_SECTION_MODELS, outcome);
    }
    if sections.include_prevalence {
        let outcome = match add_prevalence_section(disease).await {
            Ok(()) if disease.prevalence.is_empty() => SectionOutcome::empty("Open Targets"),
            Ok(()) => SectionOutcome::data("Open Targets"),
            Err(_) => {
                disease.prevalence.clear();
                disease.prevalence_note = Some(PREVALENCE_UNAVAILABLE_NOTE.into());
                SectionOutcome::unavailable(PREVALENCE_UNAVAILABLE_NOTE)
            }
        };
        disease
            .section_outcomes
            .complete(DISEASE_SECTION_PREVALENCE, outcome);
    }
    if sections.include_survival {
        add_survival_section(disease).await?;
    }
    if sections.include_funding {
        let outcome = add_funding_section(disease, requested_lookup).await;
        disease
            .section_outcomes
            .complete(DISEASE_SECTION_FUNDING, outcome);
    }
    if sections.include_diagnostics {
        let outcome = add_diagnostics_section(disease).await;
        disease
            .section_outcomes
            .complete(DISEASE_SECTION_DIAGNOSTICS, outcome);
    }
    if sections.include_civic {
        let outcome = add_civic_section(disease).await;
        disease
            .section_outcomes
            .complete(DISEASE_SECTION_CIVIC, outcome);
    }
    if sections.include_disgenet {
        add_disgenet_section(disease).await?;
    }
    if !sections.include_genes && !sections.include_pathways {
        disease.associated_genes.clear();
        disease.gene_associations.clear();
    }
    if !sections.include_variants {
        disease.variants.clear();
        disease.top_variant = None;
    }
    if !sections.include_models {
        disease.models.clear();
    }
    if !sections.include_prevalence {
        disease.prevalence.clear();
        disease.prevalence_note = None;
    }
    if !sections.include_survival {
        disease.survival = None;
        disease.survival_note = None;
    }
    if !sections.include_funding {
        disease.funding = None;
        disease.funding_note = None;
    }
    if !sections.include_diagnostics {
        disease.diagnostics = None;
        disease.diagnostics_note = None;
    }
    if !sections.include_civic {
        disease.civic = None;
    }
    if !sections.include_disgenet {
        disease.disgenet = None;
    }
    if sections.include_clinical_features {
        disease.clinical_features = disease.phenotypes.clone();
    } else {
        disease.clinical_features.clear();
    }
    if !sections.include_phenotypes {
        disease.phenotypes.clear();
    }

    disease.key_features = transform::disease::derive_key_features(disease);

    Ok(())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use self::tests::proof_enrich_sparse_disease_identity_prefers_exact_ols4_match;
