//! Cross-entity follow-up command generation and related-command descriptions.

mod article_support;

use super::*;

pub(super) fn format_related_block(commands: Vec<String>) -> String {
    let commands: Vec<String> = commands
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    if commands.is_empty() {
        return String::new();
    }
    let mut out = String::from("See also:");
    for cmd in &commands {
        if let Some(description) = related_command_description(cmd) {
            out.push_str(&format!("\n  {cmd}   - {description}"));
        } else {
            out.push_str(&format!("\n  {cmd}"));
        }
    }
    out
}

pub(super) fn is_trial_results_search_command(command: &str) -> bool {
    (command.starts_with("biomcp search article -q \"NCT") && command.ends_with(" --limit 5"))
        || (command.starts_with("biomcp search article --drug ")
            && command.contains(" -q \"NCT")
            && command.ends_with(" --limit 5"))
}

pub(super) fn is_variant_literature_follow_up_command(command: &str) -> bool {
    command.starts_with("biomcp search article ")
        && command.contains(" -k ")
        && command.ends_with(" --limit 5")
        && !command.contains(" -q ")
        && !command.contains(" --type ")
}

pub(super) fn related_command_description(command: &str) -> Option<&'static str> {
    if command.starts_with("biomcp article entities ") {
        Some("standardized entity extraction from this article")
    } else if is_trial_results_search_command(command) {
        Some("find publications or conference reports from this completed/terminated trial")
    } else if command.starts_with("biomcp article citations ") {
        Some("later papers that cite this article; use only if the primary paper lacks your answer")
    } else if command.starts_with("biomcp article references ") {
        Some("background evidence this paper builds on; use if the primary paper lacks context")
    } else if command.starts_with("biomcp article recommendations ") {
        Some("related papers to broaden coverage; use only if the primary paper lacks your answer")
    } else if command.starts_with("biomcp search article ")
        && command.contains(" --year-min ")
        && command.contains(" --year-max ")
        && command.ends_with(" --limit 5")
    {
        Some("refine this search to the visible publication-year range")
    } else if command.contains(" --type review --limit 5") {
        Some("supplement sparse structured data with review literature for indication context")
    } else if command.starts_with("biomcp get gene ") && command.ends_with(" clingen constraint") {
        Some("review ClinGen validity and constraint evidence for the top disease gene")
    } else if command.starts_with("biomcp get gene ") && command.ends_with(" protein") {
        Some("deepen into protein function and localization")
    } else if command.starts_with("biomcp get gene ") && command.ends_with(" hpa") {
        Some("deepen into tissue expression and localization")
    } else if command.starts_with("biomcp search trial -c ") && command.ends_with(" -s recruiting")
    {
        Some("recruiting trials for the top ClinGen disease on this gene card")
    } else if command.starts_with("biomcp search diagnostic --gene ") {
        Some("diagnostic tests for this gene")
    } else if command.starts_with("biomcp search diagnostic --disease ") {
        Some("diagnostic tests for this condition")
    } else if command.starts_with("biomcp search pgx -d ")
        || command.starts_with("biomcp search pgx -g ")
    {
        Some("pharmacogenomics interactions")
    } else if command.starts_with("biomcp get disease ") && command.ends_with(" genes phenotypes") {
        Some("open the top phenotype-match disease with genes and phenotypes")
    } else if is_variant_literature_follow_up_command(command) {
        Some("literature follow-up for an uncertain-significance variant")
    } else if command.starts_with("biomcp search drug --indication ") {
        Some("treatment options for this condition")
    } else if command.starts_with("biomcp get pgx ") {
        Some("pharmacogenomics card")
    } else if command.starts_with("biomcp study top-mutated --study ") {
        Some("mutation frequency ranking")
    } else if command == "biomcp study download --list" {
        Some("browse downloadable cancer genomics studies")
    } else if command == "biomcp list diagnostic" {
        Some("source-aware diagnostic filters and local GTR/WHO IVD usage")
    } else if command.starts_with("biomcp drug adverse-events ") {
        Some("inspect safety reports and adverse-event signal")
    } else {
        None
    }
}

pub(super) fn disease_top_gene_symbol(disease: &Disease) -> Option<String> {
    disease
        .top_gene_scores
        .iter()
        .map(|row| row.symbol.trim())
        .chain(disease.top_genes.iter().map(String::as_str).map(str::trim))
        .chain(
            disease
                .associated_genes
                .iter()
                .map(String::as_str)
                .map(str::trim),
        )
        .find(|symbol| !symbol.is_empty())
        .map(str::to_string)
}

pub(super) fn gene_trial_disease_label(gene: &Gene) -> Option<String> {
    gene.clingen
        .as_ref()?
        .validity
        .iter()
        .map(|row| row.disease.trim())
        .find(|label| !label.is_empty())
        .map(str::to_string)
}

pub(super) fn variant_keyword_from_legacy_name(legacy_name: &str, gene: &str) -> String {
    let legacy_name = legacy_name.trim();
    let gene = gene.trim();
    if legacy_name.is_empty() {
        return String::new();
    }
    if gene.is_empty() {
        return legacy_name.to_string();
    }

    let mut tokens = legacy_name.split_whitespace();
    if tokens
        .next()
        .is_some_and(|token| token.eq_ignore_ascii_case(gene))
    {
        let remainder = tokens.collect::<Vec<_>>().join(" ");
        if !remainder.is_empty() {
            return remainder;
        }
    }

    legacy_name.to_string()
}

pub(super) fn variant_literature_keyword_seed(variant: &Variant) -> Option<String> {
    if let Some(seed) = variant
        .legacy_name
        .as_deref()
        .map(|legacy| variant_keyword_from_legacy_name(legacy, &variant.gene))
        .filter(|value| !value.trim().is_empty())
    {
        return Some(seed.trim().to_string());
    }

    if let Some(seed) = variant
        .hgvs_p
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let seed = seed.strip_prefix("p.").unwrap_or(seed).trim();
        if !seed.is_empty() {
            return Some(seed.to_string());
        }
    }

    let id = variant.id.trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

pub(super) fn variant_literature_follow_up(variant: &Variant) -> Option<String> {
    let keyword = force_quote_arg(&variant_literature_keyword_seed(variant)?);
    if keyword.is_empty() {
        return None;
    }

    let gene = variant.gene.trim();
    let disease = variant
        .top_disease
        .as_ref()
        .map(|row| row.condition.trim())
        .filter(|condition| !condition.is_empty())
        .map(force_quote_arg);

    if !gene.is_empty() {
        if let Some(disease) = disease {
            return Some(format!(
                "biomcp search article -g {gene} -d {disease} -k {keyword} --limit 5"
            ));
        }
        return Some(format!(
            "biomcp search article -g {gene} -k {keyword} --limit 5"
        ));
    }

    if let Some(disease) = disease {
        return Some(format!(
            "biomcp search article -d {disease} -k {keyword} --limit 5"
        ));
    }

    Some(format!("biomcp search article -k {keyword} --limit 5"))
}

pub(super) fn related_gene(gene: &Gene) -> Vec<String> {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let summary = gene.summary.as_deref().unwrap_or("").to_ascii_lowercase();

    if gene.protein.is_some() || summary.contains("mitochond") || summary.contains("membrane") {
        out.push(format!("biomcp get gene {symbol} protein"));
    }
    if gene.hpa.is_some()
        || summary.contains("mitochond")
        || summary.contains("localiz")
        || summary.contains("tissue")
    {
        out.push(format!("biomcp get gene {symbol} hpa"));
    }
    if let Some(disease) = gene_trial_disease_label(gene) {
        out.push(format!(
            "biomcp search trial -c {} -s recruiting",
            force_quote_arg(&disease)
        ));
    }
    out.push(format!("biomcp search diagnostic --gene {symbol}"));
    out.push(format!("biomcp search pgx -g {symbol}"));
    out.push(format!("biomcp search variant -g {symbol}"));
    out.push(format!("biomcp search article -g {symbol}"));
    out.push(format!("biomcp search drug --target {symbol}"));
    out.push(format!("biomcp gene trials {symbol}"));
    dedupe_markdown_commands(out)
}

pub(super) fn related_variant(variant: &Variant) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let gene = variant.gene.trim();
    let significance = variant
        .significance
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let pathogenic_like = significance.contains("pathogenic");
    let vus_like = significance.contains("uncertain significance") || significance.contains("vus");

    if !gene.is_empty() {
        out.push(format!("biomcp get gene {gene}"));
    }

    if !pathogenic_like
        && vus_like
        && let Some(command) = variant_literature_follow_up(variant)
    {
        out.push(command);
    }

    if !gene.is_empty() {
        out.push(format!("biomcp search drug --target {gene}"));
    }

    if !variant.id.trim().is_empty() {
        let id = quote_arg(&variant.id);
        out.push(format!("biomcp variant trials {id}"));
        out.push(format!("biomcp variant articles {id}"));
        let has_oncokb_token = std::env::var("ONCOKB_TOKEN")
            .ok()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        if has_oncokb_token {
            out.push(format!("biomcp variant oncokb {id}"));
        }
    }
    dedupe_markdown_commands(out)
}

pub(super) fn related_variant_search_results(
    results: &[VariantSearchResult],
    gene_filter: Option<&str>,
    condition_filter: Option<&str>,
) -> Vec<String> {
    let mut out = Vec::new();

    if let Some(id) = results
        .first()
        .map(|result| quote_arg(&result.id))
        .filter(|id| !id.is_empty())
    {
        out.push(format!("biomcp get variant {id}"));
    }

    if let Some(gene) = gene_filter.map(str::trim).filter(|gene| !gene.is_empty()) {
        out.push(format!("biomcp get gene {gene}"));
    }

    if let Some(condition) = condition_filter
        .map(quote_arg)
        .filter(|value| !value.is_empty())
    {
        out.push(format!("biomcp search disease --query {condition}"));
    }

    dedupe_markdown_commands(out)
}

pub(super) fn related_article_search_results(
    results: &[ArticleSearchResult],
    filters: &ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    exact_entity_commands: &[String],
) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(pmid) = results
        .first()
        .map(|result| quote_arg(&result.pmid))
        .filter(|pmid| !pmid.is_empty())
    {
        out.push(format!("biomcp get article {pmid}"));
    }
    out.extend(
        exact_entity_commands
            .iter()
            .map(|command| command.trim().to_string())
            .filter(|command| !command.is_empty()),
    );
    out.extend(article_support::article_date_refinement_hint(
        results,
        filters,
        source_filter,
    ));
    dedupe_markdown_commands(out)
}

pub(super) fn markdown_related_article_search_results(
    results: &[ArticleSearchResult],
    filters: &ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    exact_entity_commands: &[String],
) -> Vec<String> {
    let mut out =
        related_article_search_results(results, filters, source_filter, exact_entity_commands);
    out.extend(article_support::article_keyword_cross_entity_markdown_hints(filters));
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_article(
    results: &[ArticleSearchResult],
    filters: &ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    exact_entity_commands: &[String],
) -> Vec<String> {
    let mut out =
        related_article_search_results(results, filters, source_filter, exact_entity_commands);
    if out.is_empty() {
        return out;
    }
    out.push("biomcp list article".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_trial(results: &[TrialSearchResult]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    if let Some(nct_id) = results
        .first()
        .map(|result| quote_arg(&result.nct_id))
        .filter(|nct_id| !nct_id.is_empty())
    {
        out.push(format!("biomcp get trial {nct_id}"));
    }
    out.push("biomcp list trial".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_variant(
    results: &[VariantSearchResult],
    gene_filter: Option<&str>,
    condition_filter: Option<&str>,
) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut out = related_variant_search_results(results, gene_filter, condition_filter);
    out.push("biomcp list variant".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_gene(results: &[GeneSearchResult]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    if let Some(symbol) = results
        .first()
        .map(|result| quote_arg(&result.symbol))
        .filter(|symbol| !symbol.is_empty())
    {
        out.push(format!("biomcp get gene {symbol}"));
    }
    out.push("biomcp list gene".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_disease(results: &[DiseaseSearchResult]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    if let Some(id) = results
        .first()
        .map(|result| quote_arg(&result.id))
        .filter(|id| !id.is_empty())
    {
        out.push(format!("biomcp get disease {id}"));
    }
    out.push("biomcp list disease".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_diagnostic(results: &[DiagnosticSearchResult]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    if let Some(accession) = results
        .first()
        .map(|result| quote_arg(&result.accession))
        .filter(|accession| !accession.is_empty())
    {
        out.push(format!("biomcp get diagnostic {accession}"));
    }
    out.push("biomcp list diagnostic".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn diagnostic_zero_result_recovery_commands() -> Vec<String> {
    vec!["biomcp list diagnostic".to_string()]
}

fn non_empty_owned(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn drug_search_next_commands(
    requested_name: Option<&str>,
    us_results: Option<&[DrugSearchResult]>,
    eu_results: Option<&[EmaDrugSearchResult]>,
    who_results: Option<&[WhoPrequalificationSearchResult]>,
) -> Vec<String> {
    let has_results = us_results.is_some_and(|results| !results.is_empty())
        || eu_results.is_some_and(|results| !results.is_empty())
        || who_results.is_some_and(|results| !results.is_empty());
    if !has_results {
        return Vec::new();
    }

    let who_only_vaccine_results = us_results.is_none_or(|results| results.is_empty())
        && eu_results.is_none_or(|results| results.is_empty())
        && who_results.is_some_and(|results| {
            !results.is_empty() && results.iter().all(|row| row.is_vaccine())
        });
    if who_only_vaccine_results {
        return vec!["biomcp list drug".to_string()];
    }

    let preferred = preferred_drug_name(
        us_results
            .into_iter()
            .flat_map(|results| results.iter().map(|result| result.name.as_str()))
            .chain(eu_results.into_iter().flat_map(|results| {
                results
                    .iter()
                    .flat_map(|result| [result.name.as_str(), result.active_substance.as_str()])
            }))
            .chain(
                who_results
                    .into_iter()
                    .flat_map(|results| results.iter().map(|result| result.inn.as_str())),
            ),
        requested_name,
    );
    let fallback = us_results
        .and_then(|results| results.first())
        .and_then(|result| non_empty_owned(&result.name))
        .or_else(|| {
            eu_results
                .and_then(|results| results.first())
                .and_then(|result| non_empty_owned(&result.active_substance))
        })
        .or_else(|| {
            eu_results
                .and_then(|results| results.first())
                .and_then(|result| non_empty_owned(&result.name))
        })
        .or_else(|| {
            who_results
                .and_then(|results| results.first())
                .and_then(|result| non_empty_owned(&result.inn))
        });

    let mut out = Vec::new();
    if let Some(name) = preferred.or(fallback) {
        out.push(format!("biomcp get drug {}", quote_arg(&name)));
    }
    out.push("biomcp list drug".to_string());
    dedupe_markdown_commands(out)
}

#[cfg(test)]
pub(super) fn search_next_commands_drug(
    results: &[DrugSearchResult],
    requested_name: Option<&str>,
) -> Vec<String> {
    drug_search_next_commands(requested_name, Some(results), None, None)
}

#[cfg(test)]
pub(super) fn search_next_commands_drug_eu(
    results: &[EmaDrugSearchResult],
    requested_name: Option<&str>,
) -> Vec<String> {
    drug_search_next_commands(requested_name, None, Some(results), None)
}

#[cfg(test)]
pub(super) fn search_next_commands_drug_who(
    results: &[WhoPrequalificationSearchResult],
    requested_name: Option<&str>,
) -> Vec<String> {
    drug_search_next_commands(requested_name, None, None, Some(results))
}

pub(super) fn search_next_commands_drug_regions(
    requested_name: Option<&str>,
    us_results: Option<&[DrugSearchResult]>,
    eu_results: Option<&[EmaDrugSearchResult]>,
    who_results: Option<&[WhoPrequalificationSearchResult]>,
) -> Vec<String> {
    drug_search_next_commands(requested_name, us_results, eu_results, who_results)
}

pub(super) fn search_next_commands_pgx(
    results: &[PgxSearchResult],
    gene_filter: Option<&str>,
    drug_filter: Option<&str>,
) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let top_query = gene_filter
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            drug_filter
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| {
            results.first().and_then(|result| {
                let gene = result.genesymbol.trim();
                if !gene.is_empty() {
                    return Some(gene.to_string());
                }
                let drug = result.drugname.trim();
                (!drug.is_empty()).then(|| drug.to_string())
            })
        });

    let mut out = Vec::new();
    if let Some(query) = top_query {
        out.push(format!("biomcp get pgx {}", quote_arg(&query)));
    }
    out.push("biomcp list pgx".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_pathway(results: &[PathwaySearchResult]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    if let Some(id) = results
        .first()
        .map(|result| quote_arg(&result.id))
        .filter(|id| !id.is_empty())
    {
        out.push(format!("biomcp get pathway {id}"));
    }
    out.push("biomcp list pathway".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_faers(results: &[AdverseEventSearchResult]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    if let Some(report_id) = results
        .first()
        .map(|result| quote_arg(&result.report_id))
        .filter(|report_id| !report_id.is_empty())
    {
        out.push(format!("biomcp get adverse-event {report_id}"));
    }
    out.push("biomcp list adverse-event".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_device_events(
    results: &[DeviceEventSearchResult],
) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    if let Some(report_id) = results
        .first()
        .map(|result| quote_arg(&result.report_id))
        .filter(|report_id| !report_id.is_empty())
    {
        out.push(format!("biomcp get adverse-event {report_id}"));
    }
    out.push("biomcp list adverse-event".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn search_next_commands_recalls(results: &[RecallSearchResult]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    vec!["biomcp list adverse-event".to_string()]
}

pub(super) fn search_next_commands_gwas(results: &[VariantGwasAssociation]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    if let Some(rsid) = results
        .first()
        .map(|result| quote_arg(&result.rsid))
        .filter(|rsid| !rsid.is_empty())
    {
        out.push(format!("biomcp get variant {rsid}"));
    }
    out.push("biomcp list gwas".to_string());
    dedupe_markdown_commands(out)
}

pub(super) fn related_phenotype_search_results(results: &[PhenotypeSearchResult]) -> Vec<String> {
    let Some(label) = results.first().and_then(|row| {
        let name = row.disease_name.trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
        let id = row.disease_id.trim();
        if id.is_empty() {
            None
        } else {
            Some(id.to_string())
        }
    }) else {
        return Vec::new();
    };

    dedupe_markdown_commands(vec![format!(
        "biomcp get disease {} genes phenotypes",
        force_quote_arg(&label)
    )])
}

#[derive(Clone, Copy)]
pub(super) enum ArticleAnnotationBucket {
    Gene,
    Disease,
    Chemical,
    Mutation,
}

pub(super) fn article_annotation_command(
    bucket: ArticleAnnotationBucket,
    text: &str,
) -> Option<String> {
    article_support::article_annotation_command(bucket, text)
}

pub(super) fn trial_results_search_command(trial: &Trial) -> Option<String> {
    article_support::trial_results_search_command(trial)
}

pub(super) fn article_related_id(paper: &ArticleRelatedPaper) -> String {
    article_support::article_related_id(paper)
}

pub(super) fn article_related_label(paper: &ArticleRelatedPaper) -> String {
    article_support::article_related_label(paper)
}

pub(super) fn related_article(article: &Article) -> Vec<String> {
    article_support::related_article(article)
}

pub(super) fn related_trial(trial: &Trial) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    if is_completed_or_terminated_trial_status(&trial.status)
        && let Some(command) = trial_results_search_command(trial)
    {
        out.push(command);
    }

    if let Some(condition) = trial.conditions.first().map(String::as_str) {
        let cond = quote_arg(condition);
        if !cond.is_empty() {
            out.push(format!("biomcp search disease --query {cond}"));
            out.push(format!("biomcp search article -d {cond}"));
            out.push(format!("biomcp search trial -c {cond}"));
        }
    }

    if let Some(detail) = trial.intervention_details.iter().find(|detail| {
        detail
            .other_names
            .iter()
            .any(|name| !name.trim().is_empty())
    }) {
        if let Some(alias) = detail
            .other_names
            .iter()
            .find(|name| !name.trim().is_empty())
        {
            let alias = force_quote_arg(alias);
            if !alias.is_empty() {
                out.push(format!("biomcp search drug -q {alias}"));
            }
        }
    } else if let Some(intervention) = trial.interventions.first().map(String::as_str) {
        let name = quote_arg(intervention);
        if !name.is_empty() {
            out.push(format!("biomcp get drug {name}"));
            out.push(format!("biomcp drug trials {name}"));
        }
    }

    dedupe_markdown_commands(out)
}

pub(super) fn related_disease(disease: &Disease) -> Vec<String> {
    let name = force_quote_arg(&disease_literature_query(disease));
    let mut out = Vec::new();
    if let Some(symbol) = disease_top_gene_symbol(disease) {
        out.push(format!("biomcp get gene {symbol} clingen constraint"));
    }
    if !name.is_empty() && !disease.phenotypes.is_empty() && disease.phenotypes.len() <= 3 {
        out.push(format!(
            "biomcp search article -d {name} --type review --limit 5"
        ));
    }
    if !name.is_empty() {
        out.push(format!("biomcp search trial -c {name}"));
        out.push(format!("biomcp search article -d {name}"));
        out.push(format!("biomcp search diagnostic --disease {name}"));
        out.push(format!("biomcp search drug --indication {name}"));
    }
    if is_oncology_disease(disease) {
        if let Some(study_id) = best_local_oncology_study_id(disease) {
            out.push(format!("biomcp study top-mutated --study {study_id}"));
        } else {
            out.push("biomcp study download --list".to_string());
        }
    }
    dedupe_markdown_commands(out)
}

pub(super) fn is_oncology_disease(disease: &Disease) -> bool {
    disease.civic.is_some()
        || disease.top_gene_scores.iter().any(|row| {
            row.summary
                .somatic_mutation_score
                .map(|score| score > 0.0)
                .unwrap_or(false)
        })
}

pub(super) fn normalize_match_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn preferred_drug_name<'a>(
    names: impl IntoIterator<Item = &'a str>,
    preferred: Option<&str>,
) -> Option<String> {
    let preferred = preferred.map(str::trim).filter(|value| !value.is_empty())?;
    let preferred = preferred.to_ascii_lowercase();
    names
        .into_iter()
        .map(str::trim)
        .filter_map(|name| {
            drug_parent_match_rank(name, &preferred).map(|rank| (rank, name.to_string()))
        })
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, name)| name)
}

pub(super) fn drug_parent_match_rank(name: &str, preferred_lower: &str) -> Option<u8> {
    let normalized = name.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if normalized == preferred_lower {
        return Some(0);
    }
    if normalized.starts_with(&format!("{preferred_lower} ")) {
        return Some(1);
    }
    if normalized.contains(preferred_lower) {
        if looks_like_metabolite_name(&normalized) {
            return Some(3);
        }
        return Some(2);
    }
    None
}

pub(super) fn looks_like_metabolite_name(value: &str) -> bool {
    value.contains("metabolite")
        || value.starts_with("desmethyl ")
        || value.starts_with("n-desmethyl ")
        || value.starts_with("hydroxy ")
        || value.starts_with("dealkyl ")
        || value.starts_with("oxo ")
        || value.starts_with("nor ")
        || value.starts_with("nor-")
}

pub(super) fn token_subset_match(left: &str, right: &str) -> bool {
    let left_tokens = left.split_whitespace().collect::<Vec<_>>();
    let right_tokens = right.split_whitespace().collect::<Vec<_>>();
    if left_tokens.is_empty() || right_tokens.is_empty() {
        return false;
    }

    left_tokens.iter().all(|token| right_tokens.contains(token))
        || right_tokens.iter().all(|token| left_tokens.contains(token))
}

pub(super) fn best_oncology_study_id(
    disease: &Disease,
    studies: &[crate::sources::cbioportal_study::StudyLookupRow],
) -> Option<String> {
    let candidate_labels = std::iter::once(disease.name.as_str())
        .chain(disease.synonyms.iter().map(String::as_str))
        .map(normalize_match_text)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    let mut best: Option<(u8, usize, &str)> = None;
    for study in studies.iter().filter(|study| study.has_mutations) {
        for term in &study.terms {
            let normalized_term = normalize_match_text(term);
            if normalized_term.is_empty() {
                continue;
            }

            let match_kind = if candidate_labels
                .iter()
                .any(|candidate| candidate == &normalized_term)
            {
                Some(0)
            } else if candidate_labels.iter().any(|candidate| {
                candidate.contains(&normalized_term) || normalized_term.contains(candidate)
            }) {
                Some(1)
            } else if candidate_labels
                .iter()
                .any(|candidate| token_subset_match(candidate, &normalized_term))
            {
                Some(2)
            } else {
                None
            };

            let Some(match_kind) = match_kind else {
                continue;
            };

            let candidate = (match_kind, normalized_term.len(), study.study_id.as_str());
            if best
                .as_ref()
                .map(|current| candidate < *current)
                .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
    }

    best.map(|(_, _, study_id)| study_id.to_string())
}

pub(super) fn best_local_oncology_study_id(disease: &Disease) -> Option<String> {
    let root = crate::sources::cbioportal_study::resolve_study_root();
    let rows = crate::sources::cbioportal_study::list_study_lookup_rows(&root).ok()?;
    best_oncology_study_id(disease, &rows)
}

pub(super) fn disease_literature_query(disease: &Disease) -> String {
    let name = disease.name.trim();
    if !name.is_empty() && !name.eq_ignore_ascii_case(disease.id.trim()) {
        return name.to_string();
    }

    disease
        .synonyms
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .find(|synonym| !synonym.is_empty())
        .unwrap_or(disease.id.trim())
        .to_string()
}

pub(super) fn related_pgx(pgx: &Pgx) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(gene) = pgx.gene.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        out.push(format!("biomcp search pgx -g {gene}"));
    }
    if let Some(drug) = pgx.drug.as_deref().map(quote_arg).filter(|v| !v.is_empty()) {
        out.push(format!("biomcp search pgx -d {drug}"));
    }
    out
}

pub(super) fn related_diagnostic(_diagnostic: &Diagnostic) -> Vec<String> {
    vec!["biomcp list diagnostic".to_string()]
}

pub(super) fn related_pathway(pathway: &Pathway) -> Vec<String> {
    let id = quote_arg(&pathway.id);
    if id.is_empty() {
        return Vec::new();
    }

    vec![format!("biomcp pathway drugs {id}")]
}

pub(super) fn related_protein(protein: &Protein, requested_sections: &[String]) -> Vec<String> {
    let accession = quote_arg(&protein.accession);
    let requested = requested_section_names(requested_sections);
    let requested_section = |name: &str| requested.iter().any(|value| value == name);
    let mut out = Vec::new();
    if !accession.is_empty() {
        if !requested_section("structures") {
            out.push(format!("biomcp get protein {accession} structures"));
        }
        if !requested_section("complexes") {
            out.push(format!("biomcp get protein {accession} complexes"));
        }
    }
    if let Some(symbol) = protein
        .gene_symbol
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push(format!("biomcp get gene {symbol}"));
    }
    out
}

pub(super) fn related_drug(drug: &Drug) -> Vec<String> {
    let name = quote_arg(&drug.name);
    if name.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();

    let sparse_regulatory = drug.label.is_none()
        && drug
            .approvals
            .as_ref()
            .map(|rows| rows.is_empty())
            .unwrap_or(true)
        && drug.ema_regulatory.is_none();
    let sparse_indications = drug.indications.is_empty();

    if sparse_regulatory || sparse_indications {
        out.push(format!(
            "biomcp search article --drug {name} --type review --limit 5"
        ));
    }

    out.push(format!("biomcp drug trials {name}"));
    out.push(format!("biomcp drug adverse-events {name}"));
    out.push(format!("biomcp search pgx -d {name}"));

    if let Some(target) = drug.targets.first().map(String::as_str) {
        let sym = target.trim();
        if !sym.is_empty() {
            out.push(format!("biomcp get gene {sym}"));
        }
    }

    dedupe_markdown_commands(out)
}

pub(super) fn related_drug_interactions(name: &str) -> Vec<String> {
    let name = quote_arg(name);
    if name.is_empty() {
        return Vec::new();
    }
    vec![
        format!("biomcp get drug {name} safety"),
        format!("biomcp search article --drug {name} --limit 5"),
    ]
}

pub(super) fn related_adverse_event(event: &AdverseEvent) -> Vec<String> {
    let drug = quote_arg(&event.drug);
    if drug.is_empty() {
        return Vec::new();
    }
    vec![
        format!("biomcp get drug {drug}"),
        format!("biomcp drug adverse-events {drug}"),
        format!("biomcp drug trials {drug}"),
    ]
}

pub(super) fn related_device_event(event: &DeviceEvent) -> Vec<String> {
    let device = quote_arg(&event.device);
    if device.is_empty() {
        return Vec::new();
    }
    vec![
        format!("biomcp search adverse-event --type device --device {device}"),
        "biomcp search adverse-event --type recall --classification \"Class I\"".to_string(),
    ]
}

#[cfg(test)]
mod tests;
