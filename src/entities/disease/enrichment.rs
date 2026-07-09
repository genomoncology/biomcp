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

const OPTIONAL_ENRICHMENT_TIMEOUT: Duration = Duration::from_secs(8);
const DIAGNOSTIC_PIVOT_LIMIT: usize = 10;
const SURVIVAL_NO_DATA_NOTE: &str = "SEER survival data not available for this condition.";
const SURVIVAL_UNAVAILABLE_NOTE: &str = "SEER survival data is temporarily unavailable.";
const FUNDING_NO_DATA_NOTE: &str = "No NIH funding data found for this query.";
const FUNDING_UNAVAILABLE_NOTE: &str = "NIH Reporter funding data is temporarily unavailable.";
const DISEASE_DIAGNOSTICS_UNAVAILABLE_NOTE: &str = "Diagnostic local data is unavailable. Run `biomcp gtr sync` and `biomcp who-ivd sync` to enable disease diagnostic pivots.";

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
        return Ok(());
    };

    let filters = DrugSearchFilters {
        indication: Some(query),
        ..Default::default()
    };
    let rows = drug::search(&filters, 5).await?;

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

    disease.treatment_landscape = out;
    Ok(())
}

async fn add_recruiting_trial_count(disease: &mut Disease) -> Result<(), BioMcpError> {
    let Some(query) = disease_query_value(disease) else {
        return Ok(());
    };

    let filters = TrialSearchFilters {
        condition: Some(query),
        status: Some("recruiting".to_string()),
        source: TrialSource::ClinicalTrialsGov,
        ..Default::default()
    };

    let (rows, total) = trial::search(&filters, 5, 0).await?;
    disease.recruiting_trial_count = total.or(Some(rows.len() as u32));
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
        Err(err) => {
            warn!("SEER Explorer unavailable for disease survival section: {err}");
            disease.survival = None;
            disease.survival_note = Some(SURVIVAL_UNAVAILABLE_NOTE.into());
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
            disease.survival = Some(map_survival_payload(payload));
            disease.survival_note = None;
        }
        Err(err) => {
            warn!(
                "SEER Explorer survival unavailable for disease {} at site {}: {err}",
                disease.id, site.site_code
            );
            disease.survival = None;
            disease.survival_note = Some(SURVIVAL_UNAVAILABLE_NOTE.into());
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
        Err(err) => {
            warn!("SEER Explorer catalog unavailable for disease survival section: {err}");
            disease.survival = None;
            disease.survival_note = Some(SURVIVAL_UNAVAILABLE_NOTE.into());
            return None;
        }
    };

    let Some(site) = resolve_site(disease, &catalog) else {
        disease.survival = None;
        disease.survival_note = Some(SURVIVAL_NO_DATA_NOTE.into());
        return None;
    };

    Some((site, catalog))
}

async fn add_civic_section(disease: &mut Disease) {
    let Some(query) = disease_query_value(disease) else {
        disease.civic = Some(CivicContext::default());
        return;
    };

    let civic_fut = async {
        let client = CivicClient::new()?;
        client.by_disease(&query, 10).await
    };

    match tokio::time::timeout(OPTIONAL_ENRICHMENT_TIMEOUT, civic_fut).await {
        Ok(Ok(context)) => disease.civic = Some(context),
        Ok(Err(err)) => {
            warn!(query = %query, "CIViC unavailable for disease section: {err}");
            disease.civic = Some(CivicContext::default());
        }
        Err(_) => {
            warn!(
                query = %query,
                timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                "CIViC disease section timed out"
            );
            disease.civic = Some(CivicContext::default());
        }
    }
}

async fn add_diagnostics_section(disease: &mut Disease) {
    let Some(query) = disease_query_value(disease) else {
        disease.diagnostics = Some(Vec::new());
        disease.diagnostics_note = None;
        return;
    };

    let filters = DiagnosticSearchFilters {
        disease: Some(query.clone()),
        ..Default::default()
    };
    let result =
        crate::entities::diagnostic::search_page(&filters, DIAGNOSTIC_PIVOT_LIMIT, 0).await;
    apply_diagnostics_section_result(disease, &query, result);
}

fn apply_diagnostics_section_result(
    disease: &mut Disease,
    query: &str,
    result: Result<SearchPage<DiagnosticSearchResult>, BioMcpError>,
) {
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
            disease.diagnostics = Some(page.results);
            disease.diagnostics_note = note;
        }
        Err(err) => {
            warn!(query = %query, "Diagnostic local data unavailable for disease diagnostic pivot: {err}");
            disease.diagnostics = None;
            disease.diagnostics_note = Some(DISEASE_DIAGNOSTICS_UNAVAILABLE_NOTE.into());
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

async fn add_funding_section(disease: &mut Disease, requested_lookup: Option<&str>) {
    let Some(query) = disease_funding_query_value(disease, requested_lookup) else {
        disease.funding = Some(empty_funding_section(String::new()));
        disease.funding_note = Some(FUNDING_NO_DATA_NOTE.into());
        return;
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
        }
        Ok(Err(err)) => {
            warn!(query = %query, "NIH Reporter unavailable for disease funding section: {err}");
            disease.funding = None;
            disease.funding_note = Some(FUNDING_UNAVAILABLE_NOTE.into());
        }
        Err(_) => {
            warn!(
                query = %query,
                timeout_secs = OPTIONAL_ENRICHMENT_TIMEOUT.as_secs(),
                "NIH Reporter disease funding section timed out"
            );
            disease.funding = None;
            disease.funding_note = Some(FUNDING_UNAVAILABLE_NOTE.into());
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
    if let Err(err) = add_genes_section(disease).await {
        warn!("OpenTargets unavailable for disease genes context: {err}");
    }

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
        if let Err(err) = add_monarch_gene_section(disease).await {
            warn!("Monarch unavailable for disease genes section: {err}");
        }
        if let Err(err) = augment_genes_with_civic(disease).await {
            warn!("CIViC unavailable for disease gene augmentation: {err}");
        }
        if let Err(err) = augment_genes_with_opentargets(disease).await {
            warn!("OpenTargets unavailable for disease gene augmentation: {err}");
        }
        attach_opentargets_scores(disease);
    }
    if sections.include_pathways
        && let Err(err) = add_pathways_section(disease).await
    {
        warn!("Reactome unavailable for disease pathways section: {err}");
    }
    let needs_backend_phenotypes =
        sections.include_phenotypes || sections.include_clinical_features;
    if needs_backend_phenotypes {
        if let Err(err) = add_monarch_phenotypes(disease).await {
            warn!("Monarch unavailable for disease phenotypes section: {err}");
        }
        if let Err(err) = add_phenotypes_section(disease).await {
            warn!("HPO unavailable for disease phenotypes section: {err}");
        }
    }
    if sections.include_variants
        && let Err(err) = add_civic_variants(disease).await
    {
        warn!("CIViC unavailable for disease variants section: {err}");
    }
    if sections.include_models
        && let Err(err) = add_monarch_models(disease).await
    {
        warn!("Monarch unavailable for disease models section: {err}");
    }
    if sections.include_prevalence
        && let Err(err) = add_prevalence_section(disease).await
    {
        warn!("OpenTargets unavailable for disease prevalence section: {err}");
        disease.prevalence.clear();
        disease.prevalence_note = Some("No prevalence data available from OpenTargets.".into());
    }
    if sections.include_survival {
        add_survival_section(disease).await?;
    }
    if sections.include_funding {
        add_funding_section(disease, requested_lookup).await;
    }
    if sections.include_diagnostics {
        add_diagnostics_section(disease).await;
    }
    if sections.include_civic {
        add_civic_section(disease).await;
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
