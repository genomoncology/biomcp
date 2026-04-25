//! Markdown renderers exposed through the stable markdown facade.

mod adverse_event;
mod article;
mod diagnostic;
mod discovery;
mod disease;
mod drug;
mod drug_regulatory;
mod evidence;
mod funding;
mod gene;
mod pathway;
mod pgx;
mod protein;
mod related;
#[cfg(test)]
mod root_tests;
mod sections;
mod study;
mod support;
#[cfg(test)]
mod test_support;
#[cfg(test)]
pub(crate) mod tests;
mod trial;
mod variant;

#[allow(unused_imports)]
use self::{evidence::*, funding::*, related::*, sections::*, support::*};

#[allow(unused_imports)]
pub use self::adverse_event::{
    adverse_event_count_markdown, adverse_event_markdown, adverse_event_search_markdown,
    adverse_event_search_markdown_with_context, adverse_event_search_markdown_with_footer,
    device_event_markdown, device_event_search_markdown, device_event_search_markdown_with_footer,
    recall_search_markdown, recall_search_markdown_with_footer,
};
#[allow(unused_imports)]
pub use self::article::{
    ArticleSearchRenderContext, article_batch_markdown, article_entities_markdown,
    article_graph_markdown, article_markdown, article_recommendations_markdown,
    article_search_markdown_with_footer_and_context,
};
#[allow(unused_imports)]
pub use self::diagnostic::{
    diagnostic_markdown, diagnostic_search_markdown, diagnostic_search_markdown_with_footer,
};
#[allow(unused_imports)]
pub use self::discovery::{render_discover, search_all_markdown};
#[allow(unused_imports)]
pub use self::disease::{
    disease_markdown, disease_search_markdown, disease_search_markdown_with_footer,
};
#[allow(unused_imports)]
pub use self::drug::{
    drug_interaction_report_markdown, drug_markdown, drug_markdown_with_region,
    drug_search_markdown, drug_search_markdown_with_footer, drug_search_markdown_with_region,
};
#[allow(unused_imports)]
pub use self::gene::{gene_markdown, gene_search_markdown, gene_search_markdown_with_footer};
#[allow(unused_imports)]
pub use self::pathway::{
    pathway_markdown, pathway_search_markdown, pathway_search_markdown_with_footer,
};
#[allow(unused_imports)]
pub use self::pgx::{pgx_markdown, pgx_search_markdown, pgx_search_markdown_with_footer};
#[allow(unused_imports)]
pub use self::protein::{
    protein_markdown, protein_search_markdown, protein_search_markdown_with_footer,
};
#[allow(unused_imports)]
pub use self::study::{
    study_co_occurrence_markdown, study_cohort_markdown, study_compare_expression_markdown,
    study_compare_mutations_markdown, study_download_catalog_markdown, study_download_markdown,
    study_filter_markdown, study_list_markdown, study_query_markdown, study_survival_markdown,
    study_top_mutated_markdown,
};
#[allow(unused_imports)]
pub use self::trial::{trial_markdown, trial_search_markdown, trial_search_markdown_with_footer};
#[allow(unused_imports)]
pub use self::variant::{
    gwas_search_markdown, gwas_search_markdown_with_footer, phenotype_search_markdown,
    phenotype_search_markdown_with_footer, variant_markdown, variant_oncokb_markdown,
    variant_search_markdown, variant_search_markdown_with_context,
    variant_search_markdown_with_footer,
};
use std::collections::HashSet;
use std::fmt::Write as _;
use std::sync::OnceLock;

use minijinja::{Environment, context};

use crate::cli::debug_plan::DebugPlan;
use crate::cli::search_all::SearchAllResults;
use crate::entities::adverse_event::{
    AdverseEvent, AdverseEventCountBucket, AdverseEventSearchResult, AdverseEventSearchSummary,
    DeviceEvent, DeviceEventSearchResult, RecallSearchResult,
};
use crate::entities::article::{
    AnnotationCount, Article, ArticleAnnotations, ArticleBatchEntitySummary, ArticleBatchItem,
    ArticleGraphResult, ArticleRankingMetadata, ArticleRankingMode, ArticleRecommendationsResult,
    ArticleRelatedPaper, ArticleSearchFilters, ArticleSearchResult, ArticleSort, ArticleSource,
};
use crate::entities::diagnostic::{Diagnostic, DiagnosticRegulatoryRecord, DiagnosticSearchResult};
use crate::entities::discover::{DiscoverResult, DiscoverType};
use crate::entities::disease::{
    Disease, DiseaseAssociationScoreSummary, DiseaseSearchResult, PhenotypeSearchResult,
};
use crate::entities::drug::{
    Drug, DrugApproval, DrugInteractionReport, DrugRegion, DrugSearchResult, EmaDrugSearchResult,
    EmaRegulatoryRow, EmaSafetyInfo, EmaShortageEntry, WhoPrequalificationEntry,
    WhoPrequalificationSearchResult,
};
use crate::entities::gene::{Gene, GeneSearchResult};
use crate::entities::pathway::{Pathway, PathwaySearchResult};
use crate::entities::pgx::{Pgx, PgxSearchResult};
use crate::entities::protein::{
    Protein, ProteinComplex, ProteinComplexComponent, ProteinComplexCuration, ProteinSearchResult,
};
use crate::entities::study::{
    CoOccurrenceResult as StudyCoOccurrenceResult, CohortResult as StudyCohortResult,
    ExpressionComparisonResult as StudyExpressionComparisonResult,
    FilterResult as StudyFilterResult, MutationComparisonResult as StudyMutationComparisonResult,
    SampleUniverseBasis as StudySampleUniverseBasis, StudyDownloadCatalog, StudyDownloadResult,
    StudyInfo, StudyQueryResult, SurvivalResult as StudySurvivalResult,
    TopMutatedGenesResult as StudyTopMutatedGenesResult,
};
use crate::entities::trial::{Trial, TrialSearchResult};
use crate::entities::variant::{
    Variant, VariantGwasAssociation, VariantOncoKbResult, VariantPrediction, VariantSearchResult,
    gnomad_variant_slug,
};
use crate::error::BioMcpError;
use crate::sources::nih_reporter::{NihReporterFundingSection, NihReporterGrant};

pub(crate) fn adverse_event_evidence_urls(event: &AdverseEvent) -> Vec<(&'static str, String)> {
    evidence::adverse_event_evidence_urls(event)
}

pub(crate) fn alias_fallback_suggestion(
    decision: &crate::entities::discover::AliasFallbackDecision,
) -> String {
    support::alias_fallback_suggestion(decision)
}

pub(crate) fn article_evidence_urls(article: &Article) -> Vec<(&'static str, String)> {
    evidence::article_evidence_urls(article)
}

pub(crate) fn device_event_evidence_urls(event: &DeviceEvent) -> Vec<(&'static str, String)> {
    evidence::device_event_evidence_urls(event)
}

pub(crate) fn diagnostic_evidence_urls(_diagnostic: &Diagnostic) -> Vec<(&'static str, String)> {
    Vec::new()
}

pub(crate) fn discover_evidence_urls(result: &DiscoverResult) -> Vec<(&'static str, String)> {
    evidence::discover_evidence_urls(result)
}

pub(crate) fn disease_evidence_urls(disease: &Disease) -> Vec<(String, String)> {
    evidence::disease_evidence_urls(disease)
}

pub(crate) fn drug_evidence_urls(drug: &Drug) -> Vec<(&'static str, String)> {
    evidence::drug_evidence_urls(drug)
}

pub(crate) fn drug_interaction_report_evidence_urls(
    report: &DrugInteractionReport,
) -> Vec<(String, String)> {
    evidence::drug_interaction_report_evidence_urls(report)
}

pub(crate) fn gene_evidence_urls(gene: &Gene) -> Vec<(&'static str, String)> {
    evidence::gene_evidence_urls(gene)
}

pub(crate) fn pathway_evidence_urls(pathway: &Pathway) -> Vec<(&'static str, String)> {
    evidence::pathway_evidence_urls(pathway)
}

pub(crate) fn pgx_evidence_urls(pgx: &Pgx) -> Vec<(&'static str, String)> {
    evidence::pgx_evidence_urls(pgx)
}

pub(crate) fn protein_evidence_urls(protein: &Protein) -> Vec<(&'static str, String)> {
    evidence::protein_evidence_urls(protein)
}

pub(crate) fn quote_arg(value: &str) -> String {
    support::quote_arg(value)
}

pub(crate) fn shell_quote_arg(value: &str) -> String {
    support::shell_quote_arg(value)
}

pub(crate) fn preferred_drug_name<'a>(
    names: impl IntoIterator<Item = &'a str>,
    preferred: Option<&str>,
) -> Option<String> {
    related::preferred_drug_name(names, preferred)
}

pub(crate) fn drug_parent_match_rank(name: &str, preferred_lower: &str) -> Option<u8> {
    related::drug_parent_match_rank(name, preferred_lower)
}

pub(crate) fn related_adverse_event(event: &AdverseEvent) -> Vec<String> {
    related::related_adverse_event(event)
}

pub(crate) fn related_article(article: &Article) -> Vec<String> {
    related::related_article(article)
}

pub(crate) fn related_device_event(event: &DeviceEvent) -> Vec<String> {
    related::related_device_event(event)
}

pub(crate) fn diagnostic_next_commands(
    diagnostic: &Diagnostic,
    requested_sections: &[String],
) -> Vec<String> {
    sections::diagnostic_next_commands(diagnostic, requested_sections)
}

pub(crate) fn disease_next_commands(
    disease: &Disease,
    requested_sections: &[String],
) -> Vec<String> {
    sections::disease_next_commands(disease, requested_sections)
}

pub(crate) fn related_diagnostic(diagnostic: &Diagnostic) -> Vec<String> {
    related::related_diagnostic(diagnostic)
}

pub(crate) fn related_disease(disease: &Disease) -> Vec<String> {
    related::related_disease(disease)
}

pub(crate) fn related_drug(drug: &Drug) -> Vec<String> {
    related::related_drug(drug)
}

pub(crate) fn related_drug_interactions(name: &str) -> Vec<String> {
    related::related_drug_interactions(name)
}

pub(crate) fn gene_next_commands(gene: &Gene, requested_sections: &[String]) -> Vec<String> {
    sections::gene_next_commands(gene, requested_sections)
}

pub(crate) fn related_gene(gene: &Gene) -> Vec<String> {
    related::related_gene(gene)
}

pub(crate) fn related_pathway(pathway: &Pathway) -> Vec<String> {
    related::related_pathway(pathway)
}

pub(crate) fn related_pgx(pgx: &Pgx) -> Vec<String> {
    related::related_pgx(pgx)
}

pub(crate) fn related_phenotype_search_results(results: &[PhenotypeSearchResult]) -> Vec<String> {
    related::related_phenotype_search_results(results)
}

pub(crate) fn related_protein(protein: &Protein, requested_sections: &[String]) -> Vec<String> {
    related::related_protein(protein, requested_sections)
}

pub(crate) fn related_trial(trial: &Trial) -> Vec<String> {
    related::related_trial(trial)
}

pub(crate) fn related_variant(variant: &Variant) -> Vec<String> {
    related::related_variant(variant)
}

pub(crate) fn related_variant_search_results(
    results: &[VariantSearchResult],
    gene_filter: Option<&str>,
    condition_filter: Option<&str>,
) -> Vec<String> {
    related::related_variant_search_results(results, gene_filter, condition_filter)
}

#[cfg(test)]
pub(crate) fn related_article_search_results(
    results: &[ArticleSearchResult],
    filters: &ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    exact_entity_commands: &[String],
) -> Vec<String> {
    related::related_article_search_results(results, filters, source_filter, exact_entity_commands)
}

pub(crate) fn markdown_related_article_search_results(
    results: &[ArticleSearchResult],
    filters: &ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    exact_entity_commands: &[String],
) -> Vec<String> {
    related::markdown_related_article_search_results(
        results,
        filters,
        source_filter,
        exact_entity_commands,
    )
}

pub(crate) fn search_next_commands_article(
    results: &[ArticleSearchResult],
    filters: &ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    exact_entity_commands: &[String],
) -> Vec<String> {
    related::search_next_commands_article(results, filters, source_filter, exact_entity_commands)
}

pub(crate) fn search_next_commands_trial(results: &[TrialSearchResult]) -> Vec<String> {
    related::search_next_commands_trial(results)
}

pub(crate) fn search_next_commands_variant(
    results: &[VariantSearchResult],
    gene_filter: Option<&str>,
    condition_filter: Option<&str>,
) -> Vec<String> {
    related::search_next_commands_variant(results, gene_filter, condition_filter)
}

pub(crate) fn search_next_commands_gene(results: &[GeneSearchResult]) -> Vec<String> {
    related::search_next_commands_gene(results)
}

pub(crate) fn search_next_commands_disease(results: &[DiseaseSearchResult]) -> Vec<String> {
    related::search_next_commands_disease(results)
}

pub(crate) fn search_next_commands_diagnostic(results: &[DiagnosticSearchResult]) -> Vec<String> {
    related::search_next_commands_diagnostic(results)
}

pub(crate) fn diagnostic_zero_result_recovery_commands() -> Vec<String> {
    related::diagnostic_zero_result_recovery_commands()
}

pub(crate) fn search_next_commands_drug_regions(
    requested_name: Option<&str>,
    us_results: Option<&[DrugSearchResult]>,
    eu_results: Option<&[EmaDrugSearchResult]>,
    who_results: Option<&[WhoPrequalificationSearchResult]>,
) -> Vec<String> {
    related::search_next_commands_drug_regions(requested_name, us_results, eu_results, who_results)
}

pub(crate) fn search_next_commands_pgx(
    results: &[PgxSearchResult],
    gene_filter: Option<&str>,
    drug_filter: Option<&str>,
) -> Vec<String> {
    related::search_next_commands_pgx(results, gene_filter, drug_filter)
}

pub(crate) fn search_next_commands_pathway(results: &[PathwaySearchResult]) -> Vec<String> {
    related::search_next_commands_pathway(results)
}

pub(crate) fn search_next_commands_faers(results: &[AdverseEventSearchResult]) -> Vec<String> {
    related::search_next_commands_faers(results)
}

pub(crate) use adverse_event::{
    adverse_event_search_markdown_with_source_label, combined_adverse_event_search_markdown,
    vaers_only_markdown,
};

pub(crate) fn search_next_commands_device_events(
    results: &[DeviceEventSearchResult],
) -> Vec<String> {
    related::search_next_commands_device_events(results)
}

pub(crate) fn search_next_commands_recalls(results: &[RecallSearchResult]) -> Vec<String> {
    related::search_next_commands_recalls(results)
}

pub(crate) fn search_next_commands_gwas(results: &[VariantGwasAssociation]) -> Vec<String> {
    related::search_next_commands_gwas(results)
}

pub(crate) fn trial_evidence_urls(trial: &Trial) -> Vec<(&'static str, String)> {
    evidence::trial_evidence_urls(trial)
}

pub(crate) fn variant_evidence_urls(variant: &Variant) -> Vec<(&'static str, String)> {
    evidence::variant_evidence_urls(variant)
}

pub(crate) fn variant_guidance_suggestion(
    guidance: &crate::entities::variant::VariantGuidance,
) -> String {
    support::variant_guidance_suggestion(guidance)
}

static ENV: OnceLock<Environment<'static>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaginationFooterMode {
    Offset,
    Cursor,
}

fn offset_pagination_footer(
    offset: usize,
    limit: usize,
    returned: usize,
    total: Option<usize>,
) -> String {
    let next_offset = offset.saturating_add(returned.max(limit.max(1)));
    if let Some(total) = total {
        if returned == 0 {
            return format!("Showing 0 of {total} results.");
        }
        let start = offset.saturating_add(1);
        let end = offset.saturating_add(returned);
        if end < total {
            format!(
                "Showing {start}-{end} of {total} results. Use --offset {next_offset} for more."
            )
        } else if start == end {
            format!("Showing {end} of {total} results.")
        } else {
            format!("Showing {start}-{end} of {total} results.")
        }
    } else {
        format!("Showing {returned} results (total unknown). Use --offset {next_offset} for more.")
    }
}

pub fn pagination_footer(
    mode: PaginationFooterMode,
    offset: usize,
    limit: usize,
    returned: usize,
    total: Option<usize>,
    next_page_token: Option<&str>,
) -> String {
    match mode {
        PaginationFooterMode::Offset => offset_pagination_footer(offset, limit, returned, total),
        PaginationFooterMode::Cursor => {
            let mut footer = offset_pagination_footer(offset, limit, returned, total);
            let has_token = next_page_token
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_some();
            if has_token && footer.contains("Use --offset") {
                footer.push_str(" (--next-page is also supported.)");
            }
            footer
        }
    }
}

fn with_pagination_footer(mut body: String, pagination_footer: &str) -> String {
    let footer = pagination_footer.trim();
    if footer.is_empty() || body.contains(footer) {
        return body;
    }
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body.push('\n');
    body.push_str(footer);
    body.push('\n');
    body
}

fn env() -> Result<&'static Environment<'static>, BioMcpError> {
    if let Some(env) = ENV.get() {
        return Ok(env);
    }

    let mut env = Environment::new();
    env.add_filter("truncate", |s: String, max_bytes: usize| -> String {
        if s.len() <= max_bytes {
            return s;
        }
        if max_bytes == 0 {
            return "…".to_string();
        }
        let mut boundary = max_bytes;
        while boundary > 0 && !s.is_char_boundary(boundary) {
            boundary -= 1;
        }
        let mut out = s[..boundary].trim_end().to_string();
        out.push('…');
        out
    });
    env.add_filter("phase_short", |phase: String| -> String {
        let p = phase.trim();
        if p.is_empty() || p == "-" {
            return "-".to_string();
        }

        let up = p.to_ascii_uppercase();
        let mut parts: Vec<String> = Vec::new();
        for raw in up.split('/') {
            let seg = raw.trim();
            if seg.is_empty() {
                continue;
            }
            let seg = seg.strip_prefix("PHASE").unwrap_or(seg);
            let seg = seg.trim_matches(|c: char| c == '_' || c.is_whitespace());
            if !seg.is_empty() {
                parts.push(seg.to_string());
            }
        }

        if parts.is_empty() {
            "-".to_string()
        } else {
            parts.join("/")
        }
    });
    env.add_filter("conditions_short", |conditions: Vec<String>| -> String {
        crate::transform::trial::format_conditions(&conditions)
    });
    env.add_filter("pval", |v: f64| -> String {
        if v == 0.0 {
            return "0".to_string();
        }
        if v < 0.001 {
            format!("{v:.2e}")
        } else if v < 0.01 {
            format!("{v:.4}")
        } else {
            format!("{v:.3}")
        }
    });
    env.add_filter("score", |v: f64| -> String { format!("{v:.3}") });
    env.add_filter("af", |v: f64| -> String {
        let mut out = format!("{v:.6}");
        while out.contains('.') && out.ends_with('0') {
            out.pop();
        }
        if out.ends_with('.') {
            out.pop();
        }
        if out.is_empty() { "0".to_string() } else { out }
    });
    env.add_template("gene.md.j2", include_str!("../../../templates/gene.md.j2"))?;
    env.add_template(
        "gene_search.md.j2",
        include_str!("../../../templates/gene_search.md.j2"),
    )?;
    env.add_template(
        "article.md.j2",
        include_str!("../../../templates/article.md.j2"),
    )?;
    env.add_template(
        "article_entities.md.j2",
        include_str!("../../../templates/article_entities.md.j2"),
    )?;
    env.add_template(
        "article_search.md.j2",
        include_str!("../../../templates/article_search.md.j2"),
    )?;
    env.add_template(
        "diagnostic.md.j2",
        include_str!("../../../templates/diagnostic.md.j2"),
    )?;
    env.add_template(
        "diagnostic_search.md.j2",
        include_str!("../../../templates/diagnostic_search.md.j2"),
    )?;
    env.add_template(
        "disease.md.j2",
        include_str!("../../../templates/disease.md.j2"),
    )?;
    env.add_template(
        "disease_search.md.j2",
        include_str!("../../../templates/disease_search.md.j2"),
    )?;
    env.add_template("pgx.md.j2", include_str!("../../../templates/pgx.md.j2"))?;
    env.add_template(
        "pgx_search.md.j2",
        include_str!("../../../templates/pgx_search.md.j2"),
    )?;
    env.add_template(
        "trial.md.j2",
        include_str!("../../../templates/trial.md.j2"),
    )?;
    env.add_template(
        "trial_search.md.j2",
        include_str!("../../../templates/trial_search.md.j2"),
    )?;
    env.add_template(
        "variant.md.j2",
        include_str!("../../../templates/variant.md.j2"),
    )?;
    env.add_template(
        "variant_search.md.j2",
        include_str!("../../../templates/variant_search.md.j2"),
    )?;
    env.add_template(
        "phenotype_search.md.j2",
        include_str!("../../../templates/phenotype_search.md.j2"),
    )?;
    env.add_template(
        "gwas_search.md.j2",
        include_str!("../../../templates/gwas_search.md.j2"),
    )?;
    env.add_template("drug.md.j2", include_str!("../../../templates/drug.md.j2"))?;
    env.add_template(
        "drug_search.md.j2",
        include_str!("../../../templates/drug_search.md.j2"),
    )?;
    env.add_template(
        "pathway.md.j2",
        include_str!("../../../templates/pathway.md.j2"),
    )?;
    env.add_template(
        "pathway_search.md.j2",
        include_str!("../../../templates/pathway_search.md.j2"),
    )?;
    env.add_template(
        "protein.md.j2",
        include_str!("../../../templates/protein.md.j2"),
    )?;
    env.add_template(
        "protein_search.md.j2",
        include_str!("../../../templates/protein_search.md.j2"),
    )?;
    env.add_template(
        "adverse_event.md.j2",
        include_str!("../../../templates/adverse_event.md.j2"),
    )?;
    env.add_template(
        "adverse_event_search.md.j2",
        include_str!("../../../templates/adverse_event_search.md.j2"),
    )?;
    env.add_template(
        "device_event.md.j2",
        include_str!("../../../templates/device_event.md.j2"),
    )?;
    env.add_template(
        "device_event_search.md.j2",
        include_str!("../../../templates/device_event_search.md.j2"),
    )?;
    env.add_template(
        "recall_search.md.j2",
        include_str!("../../../templates/recall_search.md.j2"),
    )?;
    env.add_template(
        "search_all.md.j2",
        include_str!("../../../templates/search_all.md.j2"),
    )?;
    env.add_template(
        "discover.md.j2",
        include_str!("../../../templates/discover.md.j2"),
    )?;

    let _ = ENV.set(env);
    Ok(ENV
        .get()
        .expect("ENV should be initialized by the time this is reached"))
}
