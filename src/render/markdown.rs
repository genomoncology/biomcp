use std::collections::HashSet;
use std::sync::OnceLock;

use minijinja::{Environment, context};

use crate::cli::search_all::SearchAllResults;
use crate::entities::adverse_event::{
    AdverseEvent, AdverseEventCountBucket, AdverseEventSearchResult, AdverseEventSearchSummary,
    DeviceEvent, DeviceEventSearchResult, RecallSearchResult,
};
use crate::entities::article::{
    Article, ArticleAnnotations, ArticleGraphResult, ArticleRecommendationsResult,
    ArticleRelatedPaper, ArticleSearchResult, ArticleSource,
};
use crate::entities::disease::{
    Disease, DiseaseAssociationScoreSummary, DiseaseSearchResult, PhenotypeSearchResult,
};
use crate::entities::drug::{Drug, DrugSearchResult};
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
};
use crate::entities::trial::{Trial, TrialSearchResult};
use crate::entities::variant::{
    Variant, VariantGwasAssociation, VariantOncoKbResult, VariantPrediction, VariantSearchResult,
};
use crate::error::BioMcpError;

static ENV: OnceLock<Environment<'static>> = OnceLock::new();

#[derive(serde::Serialize)]
struct XrefRow {
    source: String,
    id: String,
}

#[derive(serde::Serialize)]
struct ArticleSearchSourceGroup {
    source_key: String,
    source_label: String,
    count: usize,
    results: Vec<ArticleSearchResult>,
}

#[derive(serde::Serialize)]
struct ProteinComplexSummaryRow {
    accession: String,
    name: String,
    component_count: usize,
    curation: String,
}

#[derive(serde::Serialize)]
struct ProteinComplexDetailRow {
    accession: String,
    component_count: usize,
    component_preview: String,
    remaining_count: usize,
    description: Option<String>,
}

#[derive(serde::Serialize)]
struct DiseaseGeneAssociationRenderRow {
    gene: String,
    relationship: Option<String>,
    source: Option<String>,
    opentargets: Option<String>,
}

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
    env.add_template("gene.md.j2", include_str!("../../templates/gene.md.j2"))?;
    env.add_template(
        "gene_search.md.j2",
        include_str!("../../templates/gene_search.md.j2"),
    )?;
    env.add_template(
        "article.md.j2",
        include_str!("../../templates/article.md.j2"),
    )?;
    env.add_template(
        "article_entities.md.j2",
        include_str!("../../templates/article_entities.md.j2"),
    )?;
    env.add_template(
        "article_search.md.j2",
        include_str!("../../templates/article_search.md.j2"),
    )?;
    env.add_template(
        "disease.md.j2",
        include_str!("../../templates/disease.md.j2"),
    )?;
    env.add_template(
        "disease_search.md.j2",
        include_str!("../../templates/disease_search.md.j2"),
    )?;
    env.add_template("pgx.md.j2", include_str!("../../templates/pgx.md.j2"))?;
    env.add_template(
        "pgx_search.md.j2",
        include_str!("../../templates/pgx_search.md.j2"),
    )?;
    env.add_template("trial.md.j2", include_str!("../../templates/trial.md.j2"))?;
    env.add_template(
        "trial_search.md.j2",
        include_str!("../../templates/trial_search.md.j2"),
    )?;
    env.add_template(
        "variant.md.j2",
        include_str!("../../templates/variant.md.j2"),
    )?;
    env.add_template(
        "variant_search.md.j2",
        include_str!("../../templates/variant_search.md.j2"),
    )?;
    env.add_template(
        "phenotype_search.md.j2",
        include_str!("../../templates/phenotype_search.md.j2"),
    )?;
    env.add_template(
        "gwas_search.md.j2",
        include_str!("../../templates/gwas_search.md.j2"),
    )?;
    env.add_template("drug.md.j2", include_str!("../../templates/drug.md.j2"))?;
    env.add_template(
        "drug_search.md.j2",
        include_str!("../../templates/drug_search.md.j2"),
    )?;
    env.add_template(
        "pathway.md.j2",
        include_str!("../../templates/pathway.md.j2"),
    )?;
    env.add_template(
        "pathway_search.md.j2",
        include_str!("../../templates/pathway_search.md.j2"),
    )?;
    env.add_template(
        "protein.md.j2",
        include_str!("../../templates/protein.md.j2"),
    )?;
    env.add_template(
        "protein_search.md.j2",
        include_str!("../../templates/protein_search.md.j2"),
    )?;
    env.add_template(
        "adverse_event.md.j2",
        include_str!("../../templates/adverse_event.md.j2"),
    )?;
    env.add_template(
        "adverse_event_search.md.j2",
        include_str!("../../templates/adverse_event_search.md.j2"),
    )?;
    env.add_template(
        "device_event.md.j2",
        include_str!("../../templates/device_event.md.j2"),
    )?;
    env.add_template(
        "device_event_search.md.j2",
        include_str!("../../templates/device_event_search.md.j2"),
    )?;
    env.add_template(
        "recall_search.md.j2",
        include_str!("../../templates/recall_search.md.j2"),
    )?;
    env.add_template(
        "search_all.md.j2",
        include_str!("../../templates/search_all.md.j2"),
    )?;

    let _ = ENV.set(env);
    Ok(ENV
        .get()
        .expect("ENV should be initialized by the time this is reached"))
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
            opentargets: row
                .opentargets_score
                .as_ref()
                .map(format_disease_association_score),
        })
        .collect()
}

fn append_evidence_urls(mut body: String, urls: Vec<(&str, String)>) -> String {
    let links = urls
        .into_iter()
        .filter_map(|(label, url)| {
            let label = label.trim();
            let url = url.trim();
            if label.is_empty() || url.is_empty() {
                return None;
            }
            Some(format!("[{label}]({url})"))
        })
        .collect::<Vec<_>>();
    if links.is_empty() {
        return body;
    }
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body.push('\n');
    body.push_str(&links.join(" | "));
    body.push('\n');
    body
}

pub(crate) fn gene_evidence_urls(gene: &Gene) -> Vec<(&'static str, String)> {
    let mut urls = Vec::new();
    if !gene.entrez_id.trim().is_empty() {
        urls.push((
            "NCBI Gene",
            format!(
                "https://www.ncbi.nlm.nih.gov/gene/{}",
                gene.entrez_id.trim()
            ),
        ));
    }
    if let Some(uniprot) = gene
        .uniprot_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push((
            "UniProt",
            format!("https://www.uniprot.org/uniprot/{uniprot}"),
        ));
    }
    if let Some(ensembl) = gene
        .ensembl_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push((
            "Ensembl",
            format!("https://www.ensembl.org/Homo_sapiens/Gene/Summary?g={ensembl}"),
        ));
    }
    if let Some(omim) = gene
        .omim_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push(("OMIM", format!("https://www.omim.org/entry/{omim}")));
    }
    urls
}

pub(crate) fn variant_evidence_urls(variant: &Variant) -> Vec<(&'static str, String)> {
    let mut urls = Vec::new();
    if let Some(clinvar_id) = variant
        .clinvar_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push((
            "ClinVar",
            format!("https://www.ncbi.nlm.nih.gov/clinvar/variation/{clinvar_id}/"),
        ));
    }
    if let Some(rsid) = variant
        .rsid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push(("dbSNP", format!("https://www.ncbi.nlm.nih.gov/snp/{rsid}")));
    }
    if let Some(cosmic_id) = variant
        .cosmic_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push((
            "COSMIC",
            format!("https://cancer.sanger.ac.uk/cosmic/mutation/overview?id={cosmic_id}"),
        ));
    }
    urls
}

pub(crate) fn article_evidence_urls(article: &Article) -> Vec<(&'static str, String)> {
    let mut urls = Vec::new();
    if let Some(pmid) = article
        .pmid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push(("PubMed", format!("https://pubmed.ncbi.nlm.nih.gov/{pmid}/")));
    }
    if let Some(pmcid) = article
        .pmcid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push((
            "PMC",
            format!("https://pmc.ncbi.nlm.nih.gov/articles/{pmcid}/"),
        ));
    }
    urls
}

pub(crate) fn trial_evidence_urls(trial: &Trial) -> Vec<(&'static str, String)> {
    if trial.nct_id.trim().is_empty() {
        return Vec::new();
    }
    vec![(
        "ClinicalTrials.gov",
        format!("https://clinicaltrials.gov/study/{}", trial.nct_id.trim()),
    )]
}

pub(crate) fn disease_evidence_urls(disease: &Disease) -> Vec<(&'static str, String)> {
    if disease.id.trim().is_empty() {
        return Vec::new();
    }
    vec![(
        "Monarch",
        format!("https://monarchinitiative.org/{}", disease.id.trim()),
    )]
}

pub(crate) fn drug_evidence_urls(drug: &Drug) -> Vec<(&'static str, String)> {
    let mut urls = Vec::new();
    if let Some(drugbank_id) = drug
        .drugbank_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push((
            "DrugBank",
            format!("https://go.drugbank.com/drugs/{drugbank_id}"),
        ));
    }
    if let Some(chembl_id) = drug
        .chembl_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        urls.push((
            "ChEMBL",
            format!("https://www.ebi.ac.uk/chembl/compound_report_card/{chembl_id}"),
        ));
    }
    urls
}

pub(crate) fn pathway_evidence_urls(pathway: &Pathway) -> Vec<(&'static str, String)> {
    if pathway.id.trim().is_empty() {
        return Vec::new();
    }
    if pathway.source.eq_ignore_ascii_case("KEGG") {
        return vec![(
            "KEGG",
            format!("https://www.kegg.jp/entry/{}", pathway.id.trim()),
        )];
    }
    vec![(
        "Reactome",
        format!("https://reactome.org/content/detail/{}", pathway.id.trim()),
    )]
}

pub(crate) fn protein_evidence_urls(protein: &Protein) -> Vec<(&'static str, String)> {
    if protein.accession.trim().is_empty() {
        return Vec::new();
    }
    vec![(
        "UniProt",
        format!(
            "https://www.uniprot.org/uniprot/{}",
            protein.accession.trim()
        ),
    )]
}

pub(crate) fn adverse_event_evidence_urls(event: &AdverseEvent) -> Vec<(&'static str, String)> {
    if event.report_id.trim().is_empty() {
        return Vec::new();
    }
    vec![(
        "OpenFDA",
        format!(
            "https://api.fda.gov/drug/event.json?search=safetyreportid:{}",
            event.report_id.trim()
        ),
    )]
}

pub(crate) fn device_event_evidence_urls(event: &DeviceEvent) -> Vec<(&'static str, String)> {
    if event.report_id.trim().is_empty() {
        return Vec::new();
    }
    vec![(
        "OpenFDA",
        format!(
            "https://api.fda.gov/device/event.json?search=mdr_report_key:{}",
            event.report_id.trim()
        ),
    )]
}

pub(crate) fn pgx_evidence_urls(pgx: &Pgx) -> Vec<(&'static str, String)> {
    let mut urls = Vec::new();
    if let Some(gene) = pgx.gene.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        urls.push((
            "CPIC",
            format!("https://cpicpgx.org/genes/{}/", gene.to_ascii_lowercase()),
        ));
        urls.push(("PharmGKB", format!("https://www.pharmgkb.org/gene/{gene}")));
    }
    if let Some(drug) = pgx.drug.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        urls.push((
            "PharmGKB",
            format!("https://www.pharmgkb.org/chemical/{drug}"),
        ));
    }
    urls
}

fn quote_arg(value: &str) -> String {
    let v = value.trim();
    if v.is_empty() {
        return String::new();
    }
    if v.chars().any(|c| c.is_whitespace()) {
        return format!("\"{}\"", v.replace('\"', "\\\""));
    }
    v.to_string()
}

fn has_all_section(requested: &[String]) -> bool {
    requested
        .iter()
        .any(|s| s.trim().eq_ignore_ascii_case("all"))
}

fn is_section_only_requested(requested: &[String]) -> bool {
    !has_all_section(requested) && requested.iter().any(|s| !s.trim().is_empty())
}

fn requested_section_names(requested: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for section in requested {
        let section = section.trim();
        if section.is_empty() || section.eq_ignore_ascii_case("all") {
            continue;
        }
        let normalized = section.to_ascii_lowercase();
        if out.iter().any(|v| v == &normalized) {
            continue;
        }
        out.push(normalized);
    }
    out
}

fn section_header(entity_label: &str, requested: &[String]) -> String {
    let names = requested_section_names(requested);
    if names.is_empty() {
        entity_label.to_string()
    } else {
        format!("{entity_label} - {}", names.join(", "))
    }
}

fn format_sections_block(entity: &str, id: &str, sections: Vec<String>) -> String {
    if sections.is_empty() {
        return String::new();
    }
    let id_q = quote_arg(id);
    if id_q.is_empty() {
        return String::new();
    }
    let top3 = sections
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    format!("More:  biomcp get {entity} {id_q} {top3}\nAll:   biomcp get {entity} {id_q} all")
}

fn format_related_block(commands: Vec<String>) -> String {
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
        out.push_str(&format!("\n  {cmd}"));
    }
    out
}

fn markdown_cell(value: &str) -> String {
    let value = value.replace(['\n', '\r'], " ").replace('|', "\\|");
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.is_empty() {
        "-".to_string()
    } else {
        value
    }
}

fn article_related_id(paper: &ArticleRelatedPaper) -> String {
    paper
        .pmid
        .as_deref()
        .or(paper.doi.as_deref())
        .or(paper.arxiv_id.as_deref())
        .or(paper.paper_id.as_deref())
        .map(markdown_cell)
        .unwrap_or_else(|| "-".to_string())
}

fn article_related_label(paper: &ArticleRelatedPaper) -> String {
    paper
        .pmid
        .as_deref()
        .map(|pmid| format!("PMID {pmid}"))
        .or_else(|| paper.doi.as_deref().map(|doi| format!("DOI {doi}")))
        .or_else(|| {
            paper
                .arxiv_id
                .as_deref()
                .map(|arxiv| format!("arXiv {arxiv}"))
        })
        .or_else(|| {
            paper
                .paper_id
                .as_deref()
                .map(|paper_id| format!("paper {paper_id}"))
        })
        .unwrap_or_else(|| markdown_cell(&paper.title))
}

fn sections_for(requested: &[String], available: &[&str]) -> Vec<String> {
    if has_all_section(requested) {
        return Vec::new();
    }

    let requested_set: HashSet<String> = requested
        .iter()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    available
        .iter()
        .copied()
        .filter(|s| *s != "all")
        .filter(|s| !requested_set.contains(&s.to_ascii_lowercase()))
        .map(|section| section.to_string())
        .collect()
}

fn sections_gene(gene: &Gene, requested: &[String]) -> Vec<String> {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        return Vec::new();
    }

    sections_for(requested, crate::entities::gene::GENE_SECTION_NAMES)
}

fn sections_variant(variant: &Variant, requested: &[String]) -> Vec<String> {
    let id = quote_arg(&variant.id);
    if id.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::variant::VARIANT_SECTION_NAMES)
}

fn sections_article(article: &Article, requested: &[String]) -> Vec<String> {
    let key = article
        .pmid
        .as_deref()
        .or(article.pmcid.as_deref())
        .or(article.doi.as_deref())
        .unwrap_or("");
    let key = quote_arg(key);
    if key.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::article::ARTICLE_SECTION_NAMES)
}

fn sections_trial(trial: &Trial, requested: &[String]) -> Vec<String> {
    let nct_id = trial.nct_id.trim();
    if nct_id.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::trial::TRIAL_SECTION_NAMES)
}

fn sections_drug(drug: &Drug, requested: &[String]) -> Vec<String> {
    let name = quote_arg(&drug.name);
    if name.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::drug::DRUG_SECTION_NAMES)
}

fn sections_disease(disease: &Disease, requested: &[String]) -> Vec<String> {
    let key = quote_arg(&disease.id);
    if key.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::disease::DISEASE_SECTION_NAMES)
}

fn sections_pgx(pgx: &Pgx, requested: &[String]) -> Vec<String> {
    if pgx.query.trim().is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::pgx::PGX_SECTION_NAMES)
}

fn sections_pathway(pathway: &Pathway, requested: &[String]) -> Vec<String> {
    let id = quote_arg(&pathway.id);
    if id.is_empty() {
        return Vec::new();
    }
    sections_for(
        requested,
        crate::entities::pathway::supported_pathway_sections_for_source(&pathway.source),
    )
}

fn sections_protein(protein: &Protein, requested: &[String]) -> Vec<String> {
    let accession = quote_arg(&protein.accession);
    if accession.is_empty() {
        return Vec::new();
    }
    sections_for(requested, crate::entities::protein::PROTEIN_SECTION_NAMES)
}

fn sections_adverse_event(event: &AdverseEvent, requested: &[String]) -> Vec<String> {
    let report_id = quote_arg(&event.report_id);
    if report_id.is_empty() {
        return Vec::new();
    }
    sections_for(
        requested,
        crate::entities::adverse_event::ADVERSE_EVENT_SECTION_NAMES,
    )
}

pub(crate) fn related_gene(gene: &Gene) -> Vec<String> {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        return Vec::new();
    }
    vec![
        format!("biomcp search variant -g {symbol}"),
        format!("biomcp search article -g {symbol}"),
        format!("biomcp search drug --target {symbol}"),
        format!("biomcp gene trials {symbol}"),
    ]
}

pub(crate) fn related_variant(variant: &Variant) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if !variant.gene.trim().is_empty() {
        let gene = variant.gene.trim();
        out.push(format!("biomcp get gene {gene}"));
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
    out
}

pub(crate) fn related_article(article: &Article) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(ann) = article.annotations.as_ref() {
        for g in &ann.genes {
            let sym = g.text.trim();
            if sym.is_empty() {
                continue;
            }
            out.push(format!("biomcp get gene {sym}"));
        }
        for d in &ann.diseases {
            let name = quote_arg(&d.text);
            if name.is_empty() {
                continue;
            }
            out.push(format!("biomcp search disease --query {name}"));
        }
        for c in &ann.chemicals {
            let name = quote_arg(&c.text);
            if name.is_empty() {
                continue;
            }
            out.push(format!("biomcp get drug {name}"));
        }
    }
    if let Some(pmid) = article
        .pmid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push(format!("biomcp article entities {pmid}"));
        out.push(format!("biomcp article citations {pmid} --limit 3"));
        out.push(format!("biomcp article references {pmid} --limit 3"));
        out.push(format!("biomcp article recommendations {pmid} --limit 3"));
    }
    out
}

pub(crate) fn related_trial(trial: &Trial) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    if let Some(condition) = trial.conditions.first().map(String::as_str) {
        let cond = quote_arg(condition);
        if !cond.is_empty() {
            out.push(format!("biomcp search disease --query {cond}"));
            out.push(format!("biomcp search article -d {cond}"));
            out.push(format!("biomcp search trial -c {cond}"));
        }
    }

    if let Some(intervention) = trial.interventions.first().map(String::as_str) {
        let name = quote_arg(intervention);
        if !name.is_empty() {
            out.push(format!("biomcp get drug {name}"));
            out.push(format!("biomcp drug trials {name}"));
        }
    }

    out
}

pub(crate) fn related_disease(disease: &Disease) -> Vec<String> {
    let name = quote_arg(&disease.name);
    if name.is_empty() {
        return Vec::new();
    }
    vec![
        format!("biomcp search trial -c {name}"),
        format!("biomcp search article -d {name}"),
        format!("biomcp search drug {name}"),
    ]
}

pub(crate) fn related_pgx(pgx: &Pgx) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(gene) = pgx.gene.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        out.push(format!("biomcp search pgx -g {gene}"));
    }
    if let Some(drug) = pgx.drug.as_deref().map(quote_arg).filter(|v| !v.is_empty()) {
        out.push(format!("biomcp search pgx -d {drug}"));
    }
    out
}

pub(crate) fn related_pathway(pathway: &Pathway) -> Vec<String> {
    let id = quote_arg(&pathway.id);
    if id.is_empty() {
        return Vec::new();
    }

    vec![format!("biomcp pathway drugs {id}")]
}

pub(crate) fn related_protein(protein: &Protein, requested_sections: &[String]) -> Vec<String> {
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

fn format_protein_complex_component(component: &ProteinComplexComponent) -> String {
    let accession = component.accession.trim();
    let name = component.name.trim();
    let label = if name.is_empty() { accession } else { name };
    let stoichiometry = component
        .stoichiometry
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match stoichiometry {
        Some(stoichiometry) => format!("{label} ({stoichiometry})"),
        None => label.to_string(),
    }
}

fn protein_complex_summary_rows(complexes: &[ProteinComplex]) -> Vec<ProteinComplexSummaryRow> {
    complexes
        .iter()
        .map(|complex| ProteinComplexSummaryRow {
            accession: markdown_cell(&complex.accession),
            name: markdown_cell(&complex.name),
            component_count: complex.components.len(),
            curation: match &complex.curation {
                ProteinComplexCuration::Curated => "curated".to_string(),
                ProteinComplexCuration::Predicted => "predicted".to_string(),
            },
        })
        .collect()
}

fn protein_complex_detail_rows(complexes: &[ProteinComplex]) -> Vec<ProteinComplexDetailRow> {
    complexes
        .iter()
        .map(|complex| {
            let component_count = complex.components.len();
            let preview_components = complex
                .components
                .iter()
                .take(5)
                .map(format_protein_complex_component)
                .map(|component| markdown_cell(&component))
                .collect::<Vec<_>>();
            ProteinComplexDetailRow {
                accession: markdown_cell(&complex.accession),
                component_count,
                component_preview: if preview_components.is_empty() {
                    "none listed".to_string()
                } else {
                    preview_components.join(", ")
                },
                remaining_count: component_count.saturating_sub(preview_components.len()),
                description: complex
                    .description
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(markdown_cell),
            }
        })
        .collect()
}

pub(crate) fn related_drug(drug: &Drug) -> Vec<String> {
    let name = quote_arg(&drug.name);
    if name.is_empty() {
        return Vec::new();
    }

    let mut out = vec![
        format!("biomcp drug trials {name}"),
        format!("biomcp drug adverse-events {name}"),
    ];

    if let Some(target) = drug.targets.first().map(String::as_str) {
        let sym = target.trim();
        if !sym.is_empty() {
            out.push(format!("biomcp get gene {sym}"));
        }
    }

    out
}

pub(crate) fn related_adverse_event(event: &AdverseEvent) -> Vec<String> {
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

pub(crate) fn related_device_event(event: &DeviceEvent) -> Vec<String> {
    let device = quote_arg(&event.device);
    if device.is_empty() {
        return Vec::new();
    }
    vec![
        format!("biomcp search adverse-event --type device --device {device}"),
        "biomcp search adverse-event --type recall --classification \"Class I\"".to_string(),
    ]
}

pub fn gene_markdown(gene: &Gene, requested_sections: &[String]) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("gene.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_civic_section = include_all || has_requested("civic");
    let show_expression_section = include_all || has_requested("expression");
    let show_hpa_section = include_all || has_requested("hpa");
    let show_druggability_section =
        include_all || has_requested("druggability") || has_requested("drugs");
    let show_clingen_section = include_all || has_requested("clingen");
    let show_constraint_section = include_all || has_requested("constraint");
    let show_disgenet_section = include_all || has_requested("disgenet");
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&gene.symbol, requested_sections),
        symbol => &gene.symbol,
        name => &gene.name,
        entrez_id => &gene.entrez_id,
        ensembl_id => &gene.ensembl_id,
        location => &gene.location,
        genomic_coordinates => &gene.genomic_coordinates,
        omim_id => &gene.omim_id,
        uniprot_id => &gene.uniprot_id,
        summary => &gene.summary,
        gene_type => &gene.gene_type,
        aliases => &gene.aliases,
        clinical_diseases => &gene.clinical_diseases,
        clinical_drugs => &gene.clinical_drugs,
        pathways => &gene.pathways,
        ontology => &gene.ontology,
        diseases => &gene.diseases,
        protein => &gene.protein,
        go_terms => &gene.go,
        interactions => &gene.interactions,
        civic => &gene.civic,
        expression => &gene.expression,
        hpa => &gene.hpa,
        druggability => &gene.druggability,
        clingen => &gene.clingen,
        constraint => &gene.constraint,
        disgenet => &gene.disgenet,
        show_civic_section => show_civic_section,
        show_expression_section => show_expression_section,
        show_hpa_section => show_hpa_section,
        show_druggability_section => show_druggability_section,
        show_clingen_section => show_clingen_section,
        show_constraint_section => show_constraint_section,
        show_disgenet_section => show_disgenet_section,
        sections_block => format_sections_block("gene", &gene.symbol, sections_gene(gene, requested_sections)),
        related_block => format_related_block(related_gene(gene)),
    })?;
    Ok(append_evidence_urls(body, gene_evidence_urls(gene)))
}

#[allow(dead_code)]
pub fn gene_search_markdown(
    query: &str,
    results: &[GeneSearchResult],
) -> Result<String, BioMcpError> {
    gene_search_markdown_with_footer(query, results, "")
}

pub fn gene_search_markdown_with_footer(
    query: &str,
    results: &[GeneSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("gene_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn article_markdown(
    article: &Article,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("article.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_annotations_section = include_all || has_requested("annotations");
    let show_fulltext_section = include_all || has_requested("fulltext");
    let show_semantic_scholar_section = !section_only || include_all || has_requested("tldr");
    let article_label = if article.title.trim().is_empty() {
        "Article"
    } else {
        article.title.trim()
    };
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(article_label, requested_sections),
        pmid => &article.pmid,
        pmcid => &article.pmcid,
        doi => &article.doi,
        title => &article.title,
        authors => &article.authors,
        journal => &article.journal,
        date => &article.date,
        citation_count => &article.citation_count,
        publication_type => &article.publication_type,
        open_access => &article.open_access,
        abstract_text => &article.abstract_text,
        full_text_path => &article.full_text_path,
        full_text_note => &article.full_text_note,
        annotations => &article.annotations,
        semantic_scholar => &article.semantic_scholar,
        pubtator_fallback => article.pubtator_fallback,
        show_annotations_section => show_annotations_section,
        show_fulltext_section => show_fulltext_section,
        show_semantic_scholar_section => show_semantic_scholar_section,
        sections_block => format_sections_block("article", article.pmid.as_deref().or(article.pmcid.as_deref()).or(article.doi.as_deref()).unwrap_or(""), sections_article(article, requested_sections)),
        related_block => format_related_block(related_article(article)),
    })?;
    Ok(append_evidence_urls(body, article_evidence_urls(article)))
}

pub fn article_entities_markdown(
    pmid: &str,
    annotations: Option<&ArticleAnnotations>,
    limit: Option<usize>,
) -> Result<String, BioMcpError> {
    #[derive(serde::Serialize)]
    struct EntityRow {
        text: String,
        count: u32,
        command: String,
    }

    fn row(text: &str, count: u32, command: String) -> EntityRow {
        EntityRow {
            text: text.to_string(),
            count,
            command,
        }
    }

    let (mut genes, mut diseases, mut chemicals, mut mutations) = if let Some(ann) = annotations {
        (
            ann.genes
                .iter()
                .filter_map(|g| {
                    let text = g.text.trim();
                    if text.is_empty() {
                        return None;
                    }
                    Some(row(text, g.count, format!("biomcp get gene {text}")))
                })
                .collect::<Vec<_>>(),
            ann.diseases
                .iter()
                .filter_map(|d| {
                    let text = d.text.trim();
                    let quoted = quote_arg(text);
                    if quoted.is_empty() {
                        return None;
                    }
                    Some(row(
                        text,
                        d.count,
                        format!("biomcp search disease --query {quoted}"),
                    ))
                })
                .collect::<Vec<_>>(),
            ann.chemicals
                .iter()
                .filter_map(|c| {
                    let text = c.text.trim();
                    let quoted = quote_arg(text);
                    if quoted.is_empty() {
                        return None;
                    }
                    Some(row(text, c.count, format!("biomcp get drug {quoted}")))
                })
                .collect::<Vec<_>>(),
            ann.mutations
                .iter()
                .filter_map(|m| {
                    let text = m.text.trim();
                    let quoted = quote_arg(text);
                    if quoted.is_empty() {
                        return None;
                    }
                    Some(row(text, m.count, format!("biomcp get variant {quoted}")))
                })
                .collect::<Vec<_>>(),
        )
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };

    if let Some(limit) = limit {
        genes.truncate(limit);
        diseases.truncate(limit);
        chemicals.truncate(limit);
        mutations.truncate(limit);
    }

    let tmpl = env()?.get_template("article_entities.md.j2")?;
    Ok(tmpl.render(context! {
        pmid => pmid,
        genes => genes,
        diseases => diseases,
        chemicals => chemicals,
        mutations => mutations,
    })?)
}

pub fn article_graph_markdown(
    kind: &str,
    result: &ArticleGraphResult,
) -> Result<String, BioMcpError> {
    let mut out = format!(
        "# {} for {}\n\n",
        markdown_cell(kind),
        markdown_cell(&article_related_label(&result.article))
    );
    out.push_str("| PMID | Title | Intents | Influential | Context |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    if result.edges.is_empty() {
        out.push_str("| - | - | - | - | No related papers returned |\n");
        return Ok(out);
    }
    for edge in &result.edges {
        let intents = if edge.intents.is_empty() {
            "-".to_string()
        } else {
            markdown_cell(&edge.intents.join(", "))
        };
        let context = edge
            .contexts
            .first()
            .map(|value| markdown_cell(value))
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            article_related_id(&edge.paper),
            markdown_cell(&edge.paper.title),
            intents,
            if edge.is_influential { "yes" } else { "no" },
            context,
        ));
    }
    Ok(out)
}

pub fn article_recommendations_markdown(
    result: &ArticleRecommendationsResult,
) -> Result<String, BioMcpError> {
    let positives = if result.positive_seeds.is_empty() {
        "article".to_string()
    } else {
        result
            .positive_seeds
            .iter()
            .map(article_related_label)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut out = format!("# Recommendations for {}\n\n", markdown_cell(&positives));
    if !result.negative_seeds.is_empty() {
        let negatives = result
            .negative_seeds
            .iter()
            .map(article_related_label)
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "Negative seeds: {}\n\n",
            markdown_cell(&negatives)
        ));
    }
    out.push_str("| PMID | Title | Journal | Year |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    if result.recommendations.is_empty() {
        out.push_str("| - | No recommendations returned | - | - |\n");
        return Ok(out);
    }
    for paper in &result.recommendations {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            article_related_id(paper),
            markdown_cell(&paper.title),
            paper
                .journal
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            paper
                .year
                .map(|year| year.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ));
    }
    Ok(out)
}

pub fn article_search_markdown(
    query: &str,
    results: &[ArticleSearchResult],
) -> Result<String, BioMcpError> {
    article_search_markdown_with_footer(query, results, "")
}

pub fn article_search_markdown_with_footer(
    query: &str,
    results: &[ArticleSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let groups = [ArticleSource::PubTator, ArticleSource::EuropePmc]
        .into_iter()
        .filter_map(|source| {
            let rows = results
                .iter()
                .filter(|row| row.source == source)
                .cloned()
                .collect::<Vec<_>>();
            if rows.is_empty() {
                None
            } else {
                Some(ArticleSearchSourceGroup {
                    source_key: source.as_str().to_string(),
                    source_label: source.display_name().to_string(),
                    count: rows.len(),
                    results: rows,
                })
            }
        })
        .collect::<Vec<_>>();

    let tmpl = env()?.get_template("article_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        groups => groups,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn disease_markdown(
    disease: &Disease,
    requested_sections: &[String],
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
    let show_civic_section = include_all || has_requested("civic");
    let show_disgenet_section = include_all || has_requested("disgenet");
    let disease_label = if disease.name.trim().is_empty() {
        disease.id.as_str()
    } else {
        disease.name.as_str()
    };

    let tmpl = env()?.get_template("disease.md.j2")?;
    let top_gene_score_labels = disease_top_gene_score_labels(disease);
    let gene_association_rows = disease_gene_association_rows(disease);
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(disease_label, requested_sections),
        id => &disease.id,
        name => &disease.name,
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
        phenotypes => &disease.phenotypes,
        variants => &disease.variants,
        models => &disease.models,
        prevalence => &disease.prevalence,
        prevalence_note => &disease.prevalence_note,
        civic => &disease.civic,
        disgenet => &disease.disgenet,
        show_genes_section => show_genes_section,
        show_pathways_section => show_pathways_section,
        show_phenotypes_section => show_phenotypes_section,
        show_variants_section => show_variants_section,
        show_models_section => show_models_section,
        show_prevalence_section => show_prevalence_section,
        show_civic_section => show_civic_section,
        show_disgenet_section => show_disgenet_section,
        xrefs => xrefs,
        sections_block => format_sections_block("disease", &disease.id, sections_disease(disease, requested_sections)),
        related_block => format_related_block(related_disease(disease)),
    })?;
    Ok(append_evidence_urls(body, disease_evidence_urls(disease)))
}

#[allow(dead_code)]
pub fn disease_search_markdown(
    query: &str,
    results: &[DiseaseSearchResult],
) -> Result<String, BioMcpError> {
    disease_search_markdown_with_footer(query, results, "")
}

pub fn disease_search_markdown_with_footer(
    query: &str,
    results: &[DiseaseSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("disease_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn pgx_markdown(pgx: &Pgx, requested_sections: &[String]) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("pgx.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_recommendations_section = include_all || has_requested("recommendations");
    let show_frequencies_section = include_all || has_requested("frequencies");
    let show_guidelines_section = include_all || has_requested("guidelines");
    let show_annotations_section = include_all || has_requested("annotations");
    let label = pgx
        .gene
        .as_deref()
        .or(pgx.drug.as_deref())
        .unwrap_or(pgx.query.as_str());

    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(label, requested_sections),
        query => &pgx.query,
        gene => &pgx.gene,
        drug => &pgx.drug,
        interactions => &pgx.interactions,
        recommendations => &pgx.recommendations,
        frequencies => &pgx.frequencies,
        guidelines => &pgx.guidelines,
        annotations => &pgx.annotations,
        annotations_note => &pgx.annotations_note,
        show_recommendations_section => show_recommendations_section,
        show_frequencies_section => show_frequencies_section,
        show_guidelines_section => show_guidelines_section,
        show_annotations_section => show_annotations_section,
        sections_block => format_sections_block("pgx", &pgx.query, sections_pgx(pgx, requested_sections)),
        related_block => format_related_block(related_pgx(pgx)),
    })?;
    Ok(append_evidence_urls(body, pgx_evidence_urls(pgx)))
}

#[allow(dead_code)]
pub fn pgx_search_markdown(
    query: &str,
    results: &[PgxSearchResult],
) -> Result<String, BioMcpError> {
    pgx_search_markdown_with_footer(query, results, "")
}

pub fn pgx_search_markdown_with_footer(
    query: &str,
    results: &[PgxSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("pgx_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn trial_markdown(trial: &Trial, requested_sections: &[String]) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("trial.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let show_eligibility_section = include_all
        || requested
            .iter()
            .any(|s| s.eq_ignore_ascii_case("eligibility"));
    let show_locations_section = include_all
        || requested
            .iter()
            .any(|s| s.eq_ignore_ascii_case("locations"));
    let show_outcomes_section =
        include_all || requested.iter().any(|s| s.eq_ignore_ascii_case("outcomes"));
    let show_arms_section = include_all || requested.iter().any(|s| s.eq_ignore_ascii_case("arms"));
    let show_references_section = include_all
        || requested
            .iter()
            .any(|s| s.eq_ignore_ascii_case("references"));
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&trial.nct_id, requested_sections),
        nct_id => &trial.nct_id,
        title => &trial.title,
        status => &trial.status,
        phase => &trial.phase,
        study_type => &trial.study_type,
        age_range => &trial.age_range,
        conditions => &trial.conditions,
        interventions => &trial.interventions,
        sponsor => &trial.sponsor,
        enrollment => &trial.enrollment,
        summary => &trial.summary,
        start_date => &trial.start_date,
        completion_date => &trial.completion_date,
        eligibility_text => &trial.eligibility_text,
        locations => &trial.locations,
        outcomes => &trial.outcomes,
        arms => &trial.arms,
        references => &trial.references,
        show_eligibility_section => show_eligibility_section,
        show_locations_section => show_locations_section,
        show_outcomes_section => show_outcomes_section,
        show_arms_section => show_arms_section,
        show_references_section => show_references_section,
        sections_block => format_sections_block("trial", &trial.nct_id, sections_trial(trial, requested_sections)),
        related_block => format_related_block(related_trial(trial)),
    })?;
    Ok(append_evidence_urls(body, trial_evidence_urls(trial)))
}

pub fn trial_search_markdown(
    query: &str,
    results: &[TrialSearchResult],
    total: Option<u32>,
) -> Result<String, BioMcpError> {
    trial_search_markdown_with_footer(query, results, total, "")
}

pub fn trial_search_markdown_with_footer(
    query: &str,
    results: &[TrialSearchResult],
    total: Option<u32>,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("trial_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        total => total,
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn variant_markdown(
    variant: &Variant,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("variant.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_prediction_section = !section_only || include_all || has_requested("predict");
    let show_predictions_section = include_all || has_requested("predictions");
    let show_clinvar_section = !section_only || include_all || has_requested("clinvar");
    let show_population_section = !section_only || include_all || has_requested("population");
    let show_conservation_section = include_all || has_requested("conservation");
    let show_cosmic_section = include_all || has_requested("cosmic");
    let show_cgi_section = include_all || has_requested("cgi");
    let show_civic_section = include_all || has_requested("civic");
    let show_cbioportal_section = include_all || has_requested("cbioportal");
    let show_gwas_section = include_all || has_requested("gwas");
    let variant_label = if !variant.gene.trim().is_empty() && variant.hgvs_p.is_some() {
        format!(
            "{} {}",
            variant.gene.trim(),
            variant.hgvs_p.as_deref().unwrap_or_default().trim()
        )
    } else if !variant.gene.trim().is_empty() {
        variant.gene.trim().to_string()
    } else {
        variant.id.trim().to_string()
    };
    let prediction = variant.prediction.as_ref();
    let (expr_i, splice_i, chrom_i) = prediction
        .map(prediction_interpretations)
        .unwrap_or((None, None, None));
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&variant_label, requested_sections),
        id => &variant.id,
        gene => &variant.gene,
        hgvs_p => &variant.hgvs_p,
        hgvs_c => &variant.hgvs_c,
        consequence => &variant.consequence,
        rsid => &variant.rsid,
        cosmic_id => &variant.cosmic_id,
        significance => &variant.significance,
        clinvar_id => &variant.clinvar_id,
        clinvar_review_status => &variant.clinvar_review_status,
        clinvar_review_stars => &variant.clinvar_review_stars,
        conditions => &variant.conditions,
        clinvar_conditions => &variant.clinvar_conditions,
        clinvar_condition_reports => &variant.clinvar_condition_reports,
        gnomad_af => &variant.gnomad_af,
        population_breakdown => &variant.population_breakdown,
        cadd_score => &variant.cadd_score,
        sift_pred => &variant.sift_pred,
        polyphen_pred => &variant.polyphen_pred,
        conservation => &variant.conservation,
        expanded_predictions => &variant.expanded_predictions,
        cosmic_context => &variant.cosmic_context,
        cgi_associations => &variant.cgi_associations,
        civic => &variant.civic,
        cancer_frequencies => &variant.cancer_frequencies,
        cancer_frequency_source => &variant.cancer_frequency_source,
        gwas => &variant.gwas,
        prediction => prediction,
        expression_interpretation => expr_i,
        splice_interpretation => splice_i,
        chromatin_interpretation => chrom_i,
        show_prediction_section => show_prediction_section,
        show_predictions_section => show_predictions_section,
        show_clinvar_section => show_clinvar_section,
        show_population_section => show_population_section,
        show_conservation_section => show_conservation_section,
        show_cosmic_section => show_cosmic_section,
        show_cgi_section => show_cgi_section,
        show_civic_section => show_civic_section,
        show_cbioportal_section => show_cbioportal_section,
        show_gwas_section => show_gwas_section,
        sections_block => format_sections_block("variant", &variant.id, sections_variant(variant, requested_sections)),
        related_block => format_related_block(related_variant(variant)),
    })?;
    Ok(append_evidence_urls(body, variant_evidence_urls(variant)))
}

fn prediction_interpretations(
    pred: &VariantPrediction,
) -> (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
) {
    let expr = pred.expression_lfc.map(|v| {
        if v > 0.2 {
            "Increased expression"
        } else if v < -0.2 {
            "Decreased expression"
        } else {
            "Minimal change"
        }
    });

    let splice = pred.splice_score.map(|v| {
        if v.abs() > 0.5 {
            "Higher splice impact"
        } else {
            "Low splice impact"
        }
    });

    let chrom = pred.chromatin_score.map(|v| {
        if v.abs() > 0.5 {
            "Altered accessibility"
        } else {
            "Low chromatin impact"
        }
    });

    (expr, splice, chrom)
}

#[allow(dead_code)]
pub fn variant_search_markdown(
    query: &str,
    results: &[VariantSearchResult],
) -> Result<String, BioMcpError> {
    variant_search_markdown_with_footer(query, results, "")
}

pub fn variant_search_markdown_with_footer(
    query: &str,
    results: &[VariantSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("variant_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

#[allow(dead_code)]
pub fn phenotype_search_markdown(
    query: &str,
    results: &[PhenotypeSearchResult],
) -> Result<String, BioMcpError> {
    phenotype_search_markdown_with_footer(query, results, "")
}

pub fn phenotype_search_markdown_with_footer(
    query: &str,
    results: &[PhenotypeSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("phenotype_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

#[allow(dead_code)]
pub fn gwas_search_markdown(
    query: &str,
    results: &[VariantGwasAssociation],
) -> Result<String, BioMcpError> {
    gwas_search_markdown_with_footer(query, results, "")
}

pub fn gwas_search_markdown_with_footer(
    query: &str,
    results: &[VariantGwasAssociation],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("gwas_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn variant_oncokb_markdown(result: &VariantOncoKbResult) -> String {
    let mut out = String::new();
    out.push_str("# OncoKB\n\n");
    out.push_str(&format!("Gene: {}\n", result.gene.trim()));
    out.push_str(&format!("Alteration: {}\n", result.alteration.trim()));
    if let Some(level) = result
        .level
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push_str(&format!("Level: {level}\n"));
    }
    if let Some(oncogenic) = result
        .oncogenic
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push_str(&format!("Oncogenic: {oncogenic}\n"));
    }
    if let Some(effect) = result
        .effect
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push_str(&format!("Effect: {effect}\n"));
    }
    out.push('\n');

    if result.therapies.is_empty() {
        out.push_str("No therapy implications returned by OncoKB.\n");
    } else {
        out.push_str("## Therapies\n\n");
        out.push_str("| Drug | Level | Cancer Type | Note |\n");
        out.push_str("|------|-------|-------------|------|\n");
        for row in &result.therapies {
            let drugs = if row.drugs.is_empty() {
                "unspecified".to_string()
            } else {
                row.drugs.join(" + ")
            };
            let cancer = row.cancer_type.as_deref().unwrap_or("-");
            let note = row.note.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "| {drugs} | {} | {cancer} | {note} |\n",
                row.level
            ));
        }
    }

    if !result.gene.trim().is_empty() && !result.alteration.trim().is_empty() {
        out.push_str(&format!(
            "\n[OncoKB](https://www.oncokb.org/gene/{}/{})\n",
            result.gene.trim(),
            result.alteration.trim()
        ));
    }

    out
}

pub fn drug_markdown(drug: &Drug, requested_sections: &[String]) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("drug.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_label_section = !section_only || include_all || has_requested("label");
    let show_shortage_section = !section_only || include_all || has_requested("shortage");
    let show_targets_section = !section_only || include_all || has_requested("targets");
    let show_indications_section = !section_only || include_all || has_requested("indications");
    let show_interactions_section = include_all || has_requested("interactions");
    let show_civic_section = include_all || has_requested("civic");
    let show_approvals_section = include_all || has_requested("approvals");
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&drug.name, requested_sections),
        name => &drug.name,
        drugbank_id => &drug.drugbank_id,
        chembl_id => &drug.chembl_id,
        unii => &drug.unii,
        drug_type => &drug.drug_type,
        mechanism => &drug.mechanism,
        mechanisms => &drug.mechanisms,
        approval_date => &drug.approval_date,
        brand_names => &drug.brand_names,
        route => &drug.route,
        top_adverse_events => &drug.top_adverse_events,
        targets => &drug.targets,
        indications => &drug.indications,
        interactions => &drug.interactions,
        interaction_text => &drug.interaction_text,
        pharm_classes => &drug.pharm_classes,
        label => &drug.label,
        shortage => &drug.shortage,
        approvals => &drug.approvals,
        civic => &drug.civic,
        show_label_section => show_label_section,
        show_shortage_section => show_shortage_section,
        show_targets_section => show_targets_section,
        show_indications_section => show_indications_section,
        show_interactions_section => show_interactions_section,
        show_civic_section => show_civic_section,
        show_approvals_section => show_approvals_section,
        sections_block => format_sections_block("drug", &drug.name, sections_drug(drug, requested_sections)),
        related_block => format_related_block(related_drug(drug)),
    })?;
    Ok(append_evidence_urls(body, drug_evidence_urls(drug)))
}

pub fn drug_search_markdown(
    query: &str,
    results: &[DrugSearchResult],
) -> Result<String, BioMcpError> {
    drug_search_markdown_with_footer(query, results, None, "")
}

pub fn drug_search_markdown_with_footer(
    query: &str,
    results: &[DrugSearchResult],
    total_count: Option<usize>,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("drug_search.md.j2")?;
    let count = total_count.unwrap_or(results.len());
    let body = tmpl.render(context! {
        query => query,
        count => count,
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn pathway_markdown(
    pathway: &Pathway,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("pathway.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_genes_section = !section_only || include_all || has_requested("genes");
    let show_events_section = !section_only || include_all || has_requested("events");
    let show_enrichment_section = !section_only || include_all || has_requested("enrichment");
    let pathway_label = if pathway.name.trim().is_empty() {
        pathway.id.as_str()
    } else {
        pathway.name.as_str()
    };
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(pathway_label, requested_sections),
        source => &pathway.source,
        id => &pathway.id,
        name => &pathway.name,
        species => &pathway.species,
        summary => &pathway.summary,
        genes => &pathway.genes,
        events => &pathway.events,
        enrichment => &pathway.enrichment,
        show_genes_section => show_genes_section,
        show_events_section => show_events_section,
        show_enrichment_section => show_enrichment_section,
        sections_block => format_sections_block("pathway", &pathway.id, sections_pathway(pathway, requested_sections)),
        related_block => format_related_block(related_pathway(pathway)),
    })?;
    Ok(append_evidence_urls(body, pathway_evidence_urls(pathway)))
}

#[allow(dead_code)]
pub fn pathway_search_markdown(
    query: &str,
    results: &[PathwaySearchResult],
    total: Option<usize>,
) -> Result<String, BioMcpError> {
    pathway_search_markdown_with_footer(query, results, total, "")
}

pub fn pathway_search_markdown_with_footer(
    query: &str,
    results: &[PathwaySearchResult],
    total: Option<usize>,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("pathway_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        total => total,
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn protein_markdown(
    protein: &Protein,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("protein.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_domains_section = !section_only || include_all || has_requested("domains");
    let show_interactions_section = !section_only || include_all || has_requested("interactions");
    let show_complexes_section = !section_only || include_all || has_requested("complexes");
    let show_structures_section = !section_only || include_all || has_requested("structures");
    let protein_label = if protein.name.trim().is_empty() {
        protein.accession.as_str()
    } else {
        protein.name.as_str()
    };
    let complex_summaries = protein_complex_summary_rows(&protein.complexes);
    let complex_details = protein_complex_detail_rows(&protein.complexes);
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(protein_label, requested_sections),
        accession => &protein.accession,
        entry_id => &protein.entry_id,
        name => &protein.name,
        gene_symbol => &protein.gene_symbol,
        organism => &protein.organism,
        length => &protein.length,
        function => &protein.function,
        structures => &protein.structures,
        structure_count => &protein.structure_count,
        domains => &protein.domains,
        interactions => &protein.interactions,
        complexes => complex_summaries,
        complex_details => complex_details,
        show_domains_section => show_domains_section,
        show_interactions_section => show_interactions_section,
        show_complexes_section => show_complexes_section,
        show_structures_section => show_structures_section,
        sections_block => format_sections_block("protein", &protein.accession, sections_protein(protein, requested_sections)),
        related_block => format_related_block(related_protein(protein, requested_sections)),
    })?;
    Ok(append_evidence_urls(body, protein_evidence_urls(protein)))
}

#[allow(dead_code)]
pub fn protein_search_markdown(
    query: &str,
    results: &[ProteinSearchResult],
) -> Result<String, BioMcpError> {
    protein_search_markdown_with_footer(query, results, "")
}

pub fn protein_search_markdown_with_footer(
    query: &str,
    results: &[ProteinSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("protein_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn adverse_event_markdown(
    event: &AdverseEvent,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("adverse_event.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let parsed = crate::entities::adverse_event::parse_sections(requested_sections)?;
    let show_reactions_section = !section_only || parsed.include_reactions;
    let show_outcomes_section = !section_only || parsed.include_outcomes;
    let show_concomitant_section = !section_only || parsed.include_concomitant;
    let show_guidance_section = !section_only || parsed.include_guidance;
    let drug = quote_arg(&event.drug);
    let indication = event
        .indication
        .as_deref()
        .map(quote_arg)
        .unwrap_or_default();
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header("Adverse Event", requested_sections),
        report_id => &event.report_id,
        drug => &event.drug,
        reactions => &event.reactions,
        outcomes => &event.outcomes,
        patient => &event.patient,
        concomitant_medications => &event.concomitant_medications,
        reporter_type => &event.reporter_type,
        reporter_country => &event.reporter_country,
        indication => &event.indication,
        guidance_indication => indication,
        guidance_drug => drug,
        show_reactions_section => show_reactions_section,
        show_outcomes_section => show_outcomes_section,
        show_concomitant_section => show_concomitant_section,
        show_guidance_section => show_guidance_section,
        serious => &event.serious,
        date => &event.date,
        sections_block => format_sections_block("adverse-event", &event.report_id, sections_adverse_event(event, requested_sections)),
        related_block => format_related_block(related_adverse_event(event)),
    })?;
    Ok(append_evidence_urls(
        body,
        adverse_event_evidence_urls(event),
    ))
}

pub fn adverse_event_search_markdown(
    query: &str,
    results: &[AdverseEventSearchResult],
    summary: &AdverseEventSearchSummary,
) -> Result<String, BioMcpError> {
    adverse_event_search_markdown_with_footer(query, results, summary, "")
}

pub fn adverse_event_search_markdown_with_footer(
    query: &str,
    results: &[AdverseEventSearchResult],
    summary: &AdverseEventSearchSummary,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("adverse_event_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        summary => summary,
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn adverse_event_count_markdown(
    query: &str,
    count_field: &str,
    buckets: &[AdverseEventCountBucket],
) -> Result<String, BioMcpError> {
    let mut out = String::new();
    out.push_str("# Adverse Event Counts\n");
    out.push_str(&format!("\nQuery: {query}\n"));
    out.push_str(&format!("Count field: {count_field}\n\n"));
    out.push_str("| Value | Count |\n");
    out.push_str("|---|---|\n");
    if buckets.is_empty() {
        out.push_str("| - | 0 |\n");
    } else {
        for bucket in buckets {
            out.push_str(&format!("| {} | {} |\n", bucket.value, bucket.count));
        }
    }
    Ok(out)
}

pub fn device_event_markdown(event: &DeviceEvent) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("device_event.md.j2")?;
    let body = tmpl.render(context! {
        report_id => &event.report_id,
        report_number => &event.report_number,
        device => &event.device,
        manufacturer => &event.manufacturer,
        event_type => &event.event_type,
        date => &event.date,
        description => &event.description,
        related_block => format_related_block(related_device_event(event)),
    })?;
    Ok(append_evidence_urls(
        body,
        device_event_evidence_urls(event),
    ))
}

#[allow(dead_code)]
pub fn device_event_search_markdown(
    query: &str,
    results: &[DeviceEventSearchResult],
) -> Result<String, BioMcpError> {
    device_event_search_markdown_with_footer(query, results, "")
}

pub fn device_event_search_markdown_with_footer(
    query: &str,
    results: &[DeviceEventSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("device_event_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

#[allow(dead_code)]
pub fn recall_search_markdown(
    query: &str,
    results: &[RecallSearchResult],
) -> Result<String, BioMcpError> {
    recall_search_markdown_with_footer(query, results, "")
}

pub fn recall_search_markdown_with_footer(
    query: &str,
    results: &[RecallSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("recall_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}

pub fn study_list_markdown(studies: &[StudyInfo]) -> String {
    let mut out = String::new();
    out.push_str("# Study Datasets\n\n");
    if studies.is_empty() {
        out.push_str("No local studies found.\n");
        return out;
    }

    out.push_str("| Study ID | Name | Cancer Type | Samples | Available Data |\n");
    out.push_str("|---|---|---|---|---|\n");
    for study in studies {
        let cancer_type = study.cancer_type.as_deref().unwrap_or("-");
        let sample_count = study
            .sample_count
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string());
        let available = if study.available_data.is_empty() {
            "-".to_string()
        } else {
            study.available_data.join(", ")
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            study.study_id, study.name, cancer_type, sample_count, available
        ));
    }
    out
}

fn format_optional_stat(value: Option<f64>, decimals: usize) -> String {
    value
        .map(|value| format!("{value:.prec$}", prec = decimals))
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_p_value(value: Option<f64>) -> String {
    value
        .map(|value| {
            if value == 0.0 {
                "0".to_string()
            } else if value < 0.001 {
                format!("{value:.2e}")
            } else if value < 0.01 {
                format!("{value:.4}")
            } else {
                format!("{value:.3}")
            }
        })
        .unwrap_or_else(|| "not available".to_string())
}

pub fn study_download_catalog_markdown(result: &StudyDownloadCatalog) -> String {
    let mut out = String::new();
    out.push_str("# Downloadable cBioPortal Studies\n\n");
    if result.study_ids.is_empty() {
        out.push_str("No remote study IDs found.\n");
        return out;
    }

    out.push_str("| Study ID |\n");
    out.push_str("|---|\n");
    for study_id in &result.study_ids {
        out.push_str(&format!("| {study_id} |\n"));
    }
    out
}

pub fn study_download_markdown(result: &StudyDownloadResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Study Download: {}\n\n", result.study_id));
    out.push_str("| Metric | Value |\n");
    out.push_str("|---|---|\n");
    out.push_str(&format!("| Study ID | {} |\n", result.study_id));
    out.push_str(&format!("| Path | {} |\n", result.path));
    out.push_str(&format!(
        "| Downloaded | {} |\n",
        if result.downloaded {
            "yes"
        } else {
            "already present"
        }
    ));
    out
}

pub fn study_query_markdown(result: &StudyQueryResult) -> String {
    match result {
        StudyQueryResult::MutationFrequency(result) => {
            let mut out = String::new();
            out.push_str(&format!(
                "# Study Mutation Frequency: {} ({})\n\n",
                result.gene, result.study_id
            ));
            out.push_str("| Metric | Value |\n");
            out.push_str("|---|---|\n");
            out.push_str(&format!(
                "| Mutation records | {} |\n",
                result.mutation_count
            ));
            out.push_str(&format!("| Unique samples | {} |\n", result.unique_samples));
            out.push_str(&format!("| Total samples | {} |\n", result.total_samples));
            out.push_str(&format!("| Frequency | {:.6} |\n", result.frequency));
            out.push_str("\n## Top Variant Classes\n\n");
            out.push_str("| Class | Count |\n");
            out.push_str("|---|---|\n");
            if result.top_variant_classes.is_empty() {
                out.push_str("| - | 0 |\n");
            } else {
                for (class_name, count) in &result.top_variant_classes {
                    out.push_str(&format!("| {} | {} |\n", class_name, count));
                }
            }
            out.push_str("\n## Top Protein Changes\n\n");
            out.push_str("| Change | Count |\n");
            out.push_str("|---|---|\n");
            if result.top_protein_changes.is_empty() {
                out.push_str("| - | 0 |\n");
            } else {
                for (change, count) in &result.top_protein_changes {
                    out.push_str(&format!("| {} | {} |\n", change, count));
                }
            }
            out
        }
        StudyQueryResult::CnaDistribution(result) => {
            let mut out = String::new();
            out.push_str(&format!(
                "# Study CNA Distribution: {} ({})\n\n",
                result.gene, result.study_id
            ));
            out.push_str("| Bucket | Count |\n");
            out.push_str("|---|---|\n");
            out.push_str(&format!(
                "| Deep deletion (-2) | {} |\n",
                result.deep_deletion
            ));
            out.push_str(&format!(
                "| Shallow deletion (-1) | {} |\n",
                result.shallow_deletion
            ));
            out.push_str(&format!("| Diploid (0) | {} |\n", result.diploid));
            out.push_str(&format!("| Gain (1) | {} |\n", result.gain));
            out.push_str(&format!(
                "| Amplification (2) | {} |\n",
                result.amplification
            ));
            out.push_str(&format!("| Total samples | {} |\n", result.total_samples));
            out
        }
        StudyQueryResult::ExpressionDistribution(result) => {
            let mut out = String::new();
            out.push_str(&format!(
                "# Study Expression Distribution: {} ({})\n\n",
                result.gene, result.study_id
            ));
            out.push_str("| Metric | Value |\n");
            out.push_str("|---|---|\n");
            out.push_str(&format!("| File | {} |\n", result.file));
            out.push_str(&format!("| Sample count | {} |\n", result.sample_count));
            out.push_str(&format!("| Mean | {:.6} |\n", result.mean));
            out.push_str(&format!("| Median | {:.6} |\n", result.median));
            out.push_str(&format!("| Min | {:.6} |\n", result.min));
            out.push_str(&format!("| Max | {:.6} |\n", result.max));
            out.push_str(&format!("| Q1 | {:.6} |\n", result.q1));
            out.push_str(&format!("| Q3 | {:.6} |\n", result.q3));
            out
        }
    }
}

pub fn study_filter_markdown(result: &StudyFilterResult) -> String {
    const SAMPLE_DISPLAY_LIMIT: usize = 50;

    let mut out = String::new();
    out.push_str(&format!("# Study Filter: {}\n\n", result.study_id));
    out.push_str("## Criteria\n\n");
    out.push_str("| Filter | Matching Samples |\n");
    out.push_str("|---|---|\n");
    if result.criteria.is_empty() {
        out.push_str("| - | 0 |\n");
    } else {
        for criterion in &result.criteria {
            out.push_str(&format!(
                "| {} | {} |\n",
                criterion.description, criterion.matched_count
            ));
        }
    }

    out.push_str("\n## Result\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|---|---|\n");
    let total = result
        .total_study_samples
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    out.push_str(&format!("| Study Total Samples | {total} |\n"));
    out.push_str(&format!("| Intersection | {} |\n", result.matched_count));

    out.push_str("\n## Matched Samples\n\n");
    if result.matched_sample_ids.is_empty() {
        out.push_str("None\n");
        return out;
    }

    for sample_id in result.matched_sample_ids.iter().take(SAMPLE_DISPLAY_LIMIT) {
        out.push_str(sample_id);
        out.push('\n');
    }
    let remaining = result
        .matched_sample_ids
        .len()
        .saturating_sub(SAMPLE_DISPLAY_LIMIT);
    if remaining > 0 {
        out.push_str(&format!(
            "... and {remaining} more (use --json for full list)\n"
        ));
    }
    out
}

pub fn study_cohort_markdown(result: &StudyCohortResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Study Cohort: {} ({})\n\n",
        result.gene, result.study_id
    ));
    let stratification = match result.stratification.as_str() {
        "mutation" => "mutation status",
        other => other,
    };
    out.push_str(&format!("Stratification: {stratification}\n\n"));
    out.push_str("| Group | Samples | Patients |\n");
    out.push_str("|---|---|---|\n");
    out.push_str(&format!(
        "| {}-mutant | {} | {} |\n",
        result.gene, result.mutant_samples, result.mutant_patients
    ));
    out.push_str(&format!(
        "| {}-wildtype | {} | {} |\n",
        result.gene, result.wildtype_samples, result.wildtype_patients
    ));
    out.push_str(&format!(
        "| Total | {} | {} |\n",
        result.total_samples, result.total_patients
    ));
    out
}

pub fn study_survival_markdown(result: &StudySurvivalResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Study Survival: {} ({})\n\n",
        result.gene, result.study_id
    ));
    out.push_str(&format!(
        "Endpoint: {} ({})\n\n",
        result.endpoint.label(),
        result.endpoint.code()
    ));
    out.push_str("| Group | N | Events | Censored | Event Rate | KM Median | 1yr | 3yr | 5yr |\n");
    out.push_str("|---|---|---|---|---|---|---|---|---|\n");
    for group in &result.groups {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {:.6} | {} | {} | {} | {} |\n",
            group.group_name,
            group.n_patients,
            group.n_events,
            group.n_censored,
            group.event_rate,
            format_optional_stat(group.km_median_months, 1),
            format_optional_stat(group.survival_1yr, 3),
            format_optional_stat(group.survival_3yr, 3),
            format_optional_stat(group.survival_5yr, 3)
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "Log-rank p-value: {}\n",
        format_optional_p_value(result.log_rank_p)
    ));
    out
}

pub fn study_compare_expression_markdown(result: &StudyExpressionComparisonResult) -> String {
    let mut out = String::new();
    out.push_str("# Study Group Comparison: Expression\n\n");
    out.push_str(&format!(
        "Stratify gene: {} | Target gene: {} | Study: {}\n\n",
        result.stratify_gene, result.target_gene, result.study_id
    ));
    out.push_str("| Group | N | Mean | Median | Q1 | Q3 | Min | Max |\n");
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    for group in &result.groups {
        out.push_str(&format!(
            "| {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |\n",
            group.group_name,
            group.sample_count,
            group.mean,
            group.median,
            group.q1,
            group.q3,
            group.min,
            group.max
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "Mann-Whitney U: {}\n",
        format_optional_stat(result.mann_whitney_u, 3)
    ));
    out.push_str(&format!(
        "Mann-Whitney p-value: {}\n",
        format_optional_p_value(result.mann_whitney_p)
    ));
    out
}

pub fn study_compare_mutations_markdown(result: &StudyMutationComparisonResult) -> String {
    let mut out = String::new();
    out.push_str("# Study Group Comparison: Mutation Rate\n\n");
    out.push_str(&format!(
        "Stratify gene: {} | Target gene: {} | Study: {}\n\n",
        result.stratify_gene, result.target_gene, result.study_id
    ));
    out.push_str("| Group | N | Mutated | Mutation Rate |\n");
    out.push_str("|---|---|---|---|\n");
    for group in &result.groups {
        out.push_str(&format!(
            "| {} | {} | {} | {:.6} |\n",
            group.group_name, group.sample_count, group.mutated_count, group.mutation_rate
        ));
    }
    out
}

pub fn study_co_occurrence_markdown(result: &StudyCoOccurrenceResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Study Co-occurrence: {}\n\n", result.study_id));
    out.push_str(&format!("Genes: {}\n\n", result.genes.join(", ")));
    out.push_str(&format!("Total samples: {}\n\n", result.total_samples));
    out.push_str(&format!(
        "Sample universe: {}\n\n",
        match result.sample_universe_basis {
            StudySampleUniverseBasis::ClinicalSampleFile => "clinical sample file",
            StudySampleUniverseBasis::MutationObserved => {
                "mutation-observed samples only (clinical sample file unavailable)"
            }
        }
    ));
    out.push_str(
        "| Gene A | Gene B | Both | A only | B only | Neither | Log Odds Ratio | p-value |\n",
    );
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    if result.pairs.is_empty() {
        out.push_str("| - | - | 0 | 0 | 0 | 0 | - | - |\n");
        return out;
    }
    for pair in &result.pairs {
        let lor = pair
            .log_odds_ratio
            .map(|v| format!("{v:.6}"))
            .unwrap_or_else(|| "-".to_string());
        let p_value = pair
            .p_value
            .map(|v| format!("{v:.3e}"))
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            pair.gene_a,
            pair.gene_b,
            pair.both_mutated,
            pair.a_only,
            pair.b_only,
            pair.neither,
            lor,
            p_value
        ));
    }
    out
}

pub fn search_all_markdown(
    results: &SearchAllResults,
    counts_only: bool,
) -> Result<String, BioMcpError> {
    #[derive(serde::Serialize)]
    struct SearchAllSectionView {
        entity: String,
        label: String,
        heading_count: usize,
        error: Option<String>,
        note: Option<String>,
        links: Vec<crate::cli::search_all::SearchAllLink>,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    }

    let tmpl = env()?.get_template("search_all.md.j2")?;
    let sections = results
        .sections
        .iter()
        .map(|section| {
            let rows = section.markdown_rows();
            let heading_count = if counts_only {
                section.total.unwrap_or(section.count)
            } else {
                rows.len()
            };
            SearchAllSectionView {
                entity: section.entity.clone(),
                label: section.label.clone(),
                heading_count,
                error: section.error.clone(),
                note: section.note.clone(),
                links: section.links.clone(),
                columns: section
                    .markdown_columns()
                    .iter()
                    .map(|column| (*column).to_string())
                    .collect(),
                rows,
            }
        })
        .collect::<Vec<_>>();

    Ok(tmpl.render(context! {
        query => &results.query,
        sections => sections,
        counts_only => counts_only,
        searches_dispatched => results.searches_dispatched,
        searches_with_results => results.searches_with_results,
        wall_time_ms => results.wall_time_ms,
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::adverse_event::DeviceEvent;
    use crate::entities::article::{
        AnnotationCount, Article, ArticleAnnotations, ArticleSearchResult, ArticleSource,
    };
    use crate::entities::drug::Drug;
    use crate::entities::gene::Gene;
    use crate::entities::pathway::Pathway;
    use crate::entities::pgx::Pgx;
    use crate::entities::study::{
        CnaDistributionResult as StudyCnaDistributionResult,
        CoOccurrencePair as StudyCoOccurrencePair, CoOccurrenceResult as StudyCoOccurrenceResult,
        CohortResult as StudyCohortResult,
        ExpressionComparisonResult as StudyExpressionComparisonResult,
        ExpressionDistributionResult as StudyExpressionDistributionResult,
        ExpressionGroupStats as StudyExpressionGroupStats,
        MutationComparisonResult as StudyMutationComparisonResult,
        MutationFrequencyResult as StudyMutationFrequencyResult,
        MutationGroupStats as StudyMutationGroupStats,
        SampleUniverseBasis as StudySampleUniverseBasis, StudyDownloadCatalog, StudyDownloadResult,
        StudyInfo, StudyQueryResult, SurvivalEndpoint as StudySurvivalEndpoint,
        SurvivalGroupResult as StudySurvivalGroupResult, SurvivalResult as StudySurvivalResult,
    };
    use crate::entities::variant::{TreatmentImplication, Variant, VariantOncoKbResult};

    #[test]
    fn quote_arg_wraps_whitespace_and_escapes_quotes() {
        assert_eq!(quote_arg("BRAF"), "BRAF");
        assert_eq!(quote_arg("BRAF V600E"), "\"BRAF V600E\"");
        assert_eq!(quote_arg("BRAF \"V600E\""), "\"BRAF \\\"V600E\\\"\"");
    }

    #[test]
    fn sections_pathway_for_kegg_excludes_unsupported_sections() {
        let pathway = Pathway {
            source: "KEGG".to_string(),
            id: "hsa05200".to_string(),
            name: "Pathways in cancer".to_string(),
            species: None,
            summary: None,
            genes: Vec::new(),
            events: Vec::new(),
            enrichment: Vec::new(),
        };

        let sections = sections_pathway(&pathway, &[]);
        assert_eq!(sections, vec!["genes".to_string()]);
    }

    #[test]
    fn sections_pathway_for_reactome_keeps_full_supported_set() {
        let pathway = Pathway {
            source: "Reactome".to_string(),
            id: "R-HSA-5673001".to_string(),
            name: "RAF/MAP kinase cascade".to_string(),
            species: None,
            summary: None,
            genes: Vec::new(),
            events: Vec::new(),
            enrichment: Vec::new(),
        };

        let sections = sections_pathway(&pathway, &[]);
        assert_eq!(
            sections,
            vec![
                "genes".to_string(),
                "events".to_string(),
                "enrichment".to_string()
            ]
        );
    }

    #[test]
    fn gene_markdown_includes_evidence_links() {
        let gene = Gene {
            symbol: "BRAF".to_string(),
            name: "B-Raf proto-oncogene".to_string(),
            entrez_id: "673".to_string(),
            ensembl_id: Some("ENSG00000157764".to_string()),
            location: Some("7q34".to_string()),
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: Some("P15056".to_string()),
            summary: Some("Kinase involved in MAPK signaling.".to_string()),
            gene_type: Some("protein-coding".to_string()),
            aliases: vec!["BRAF1".to_string()],
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
        };

        let markdown = gene_markdown(&gene, &[]).expect("rendered markdown");
        assert!(markdown.contains("BRAF"));
        assert!(markdown.contains("[NCBI Gene](https://www.ncbi.nlm.nih.gov/gene/673)"));
        assert!(markdown.contains("[UniProt](https://www.uniprot.org/uniprot/P15056)"));
    }

    #[test]
    fn gene_markdown_section_only_shows_new_gene_enrichment_sections() {
        let gene = Gene {
            symbol: "BRAF".to_string(),
            name: "B-Raf proto-oncogene".to_string(),
            entrez_id: "673".to_string(),
            ensembl_id: Some("ENSG00000157764".to_string()),
            location: Some("7q34".to_string()),
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: Some("P15056".to_string()),
            summary: Some("Kinase involved in MAPK signaling.".to_string()),
            gene_type: Some("protein-coding".to_string()),
            aliases: vec!["BRAF1".to_string()],
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
        };

        let markdown = gene_markdown(
            &gene,
            &[
                "expression".to_string(),
                "hpa".to_string(),
                "druggability".to_string(),
                "clingen".to_string(),
            ],
        )
        .expect("rendered markdown");

        assert!(markdown.contains("# BRAF - expression, hpa, druggability, clingen"));
        assert!(markdown.contains("## Expression (GTEx)"));
        assert!(markdown.contains("## Human Protein Atlas"));
        assert!(markdown.contains("## Druggability"));
        assert!(markdown.contains("## ClinGen"));
        assert!(markdown.contains("No GTEx expression records returned"));
        assert!(markdown.contains("No Human Protein Atlas records returned"));
        assert!(markdown.contains("No DGIdb interactions returned"));
        assert!(markdown.contains("No ClinGen records returned"));
    }

    #[test]
    fn gene_markdown_renders_combined_dgidb_and_opentargets_druggability() {
        let gene = Gene {
            symbol: "EGFR".to_string(),
            name: "epidermal growth factor receptor".to_string(),
            entrez_id: "1956".to_string(),
            ensembl_id: Some("ENSG00000146648".to_string()),
            location: Some("7p11.2".to_string()),
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: Some("P00533".to_string()),
            summary: None,
            gene_type: Some("protein-coding".to_string()),
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: Some(crate::sources::dgidb::GeneDruggability {
                categories: vec!["Kinase".to_string()],
                interactions: Vec::new(),
                tractability: vec![
                    crate::sources::dgidb::GeneTractabilityModality {
                        modality: "small molecule".to_string(),
                        tractable: true,
                        evidence_labels: vec!["Approved Drug".to_string()],
                    },
                    crate::sources::dgidb::GeneTractabilityModality {
                        modality: "antibody".to_string(),
                        tractable: true,
                        evidence_labels: vec!["Clinical Precedence".to_string()],
                    },
                ],
                safety_liabilities: vec![crate::sources::dgidb::GeneSafetyLiability {
                    event: "Skin rash".to_string(),
                    datasource: Some("ForceGenetics".to_string()),
                    effect_direction: Some("activation".to_string()),
                    biosample: Some("Skin".to_string()),
                }],
            }),
            clingen: None,
            constraint: None,
            disgenet: None,
        };

        let markdown =
            gene_markdown(&gene, &["druggability".to_string()]).expect("rendered markdown");

        assert!(markdown.contains("## Druggability"));
        assert!(!markdown.contains("## Druggability (DGIdb)"));
        assert!(markdown.contains("OpenTargets tractability"));
        assert!(markdown.contains("| Modality | Tractable | Evidence |"));
        assert!(markdown.contains("| small molecule | yes | Approved Drug |"));
        assert!(markdown.contains("| antibody | yes | Clinical Precedence |"));
        assert!(markdown.contains("OpenTargets safety liabilities"));
        assert!(markdown.contains("Skin rash"));
        assert!(markdown.contains("No DGIdb interactions returned for this gene query."));
    }

    #[test]
    fn gene_markdown_renders_dgidb_interaction_table_alongside_opentargets_data() {
        let gene = Gene {
            symbol: "BRAF".to_string(),
            name: "B-Raf proto-oncogene".to_string(),
            entrez_id: "673".to_string(),
            ensembl_id: None,
            location: None,
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: None,
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: Some(crate::sources::dgidb::GeneDruggability {
                categories: vec!["Kinase".to_string()],
                interactions: vec![crate::sources::dgidb::DrugInteraction {
                    drug: "Dabrafenib".to_string(),
                    interaction_types: vec!["inhibitor".to_string()],
                    score: Some(1.2),
                    approved: Some(true),
                    source_count: 2,
                }],
                tractability: vec![crate::sources::dgidb::GeneTractabilityModality {
                    modality: "small molecule".to_string(),
                    tractable: true,
                    evidence_labels: vec!["Approved Drug".to_string()],
                }],
                safety_liabilities: Vec::new(),
            }),
            clingen: None,
            constraint: None,
            disgenet: None,
        };

        let markdown =
            gene_markdown(&gene, &["druggability".to_string()]).expect("rendered markdown");

        assert!(markdown.contains("## Druggability"));
        assert!(markdown.contains("OpenTargets tractability"));
        assert!(markdown.contains("| small molecule | yes | Approved Drug |"));
        // DGIdb interaction table renders (not replaced by empty-state message)
        assert!(markdown.contains("| Drug | Interaction Types | Score | Approved | Sources |"));
        assert!(markdown.contains("| Dabrafenib | inhibitor | 1.200 | yes | 2 |"));
        assert!(!markdown.contains("No DGIdb interactions returned for this gene query."));
    }

    #[test]
    fn disease_markdown_renders_opentargets_scores_in_summary_and_genes_table() {
        let disease = Disease {
            id: "MONDO:0005105".to_string(),
            name: "melanoma".to_string(),
            definition: None,
            synonyms: Vec::new(),
            parents: Vec::new(),
            associated_genes: vec!["BRAF".to_string(), "NRAS".to_string()],
            gene_associations: vec![
                crate::entities::disease::DiseaseGeneAssociation {
                    gene: "BRAF".to_string(),
                    relationship: Some("causal".to_string()),
                    source: Some("Monarch".to_string()),
                    opentargets_score: Some(
                        crate::entities::disease::DiseaseAssociationScoreSummary {
                            overall_score: 0.912,
                            gwas_score: Some(0.321),
                            rare_variant_score: Some(0.654),
                            somatic_mutation_score: Some(0.876),
                        },
                    ),
                },
                crate::entities::disease::DiseaseGeneAssociation {
                    gene: "NRAS".to_string(),
                    relationship: Some("associated".to_string()),
                    source: Some("CIViC".to_string()),
                    opentargets_score: None,
                },
            ],
            top_genes: vec!["BRAF".to_string(), "NRAS".to_string()],
            top_gene_scores: vec![
                crate::entities::disease::DiseaseTargetScore {
                    symbol: "BRAF".to_string(),
                    summary: crate::entities::disease::DiseaseAssociationScoreSummary {
                        overall_score: 0.912,
                        gwas_score: Some(0.321),
                        rare_variant_score: Some(0.654),
                        somatic_mutation_score: Some(0.876),
                    },
                },
                crate::entities::disease::DiseaseTargetScore {
                    symbol: "NRAS".to_string(),
                    summary: crate::entities::disease::DiseaseAssociationScoreSummary {
                        overall_score: 0.701,
                        gwas_score: None,
                        rare_variant_score: None,
                        somatic_mutation_score: Some(0.443),
                    },
                },
            ],
            treatment_landscape: Vec::new(),
            recruiting_trial_count: None,
            pathways: Vec::new(),
            phenotypes: Vec::new(),
            variants: Vec::new(),
            models: Vec::new(),
            prevalence: Vec::new(),
            prevalence_note: None,
            civic: None,
            disgenet: None,
            xrefs: std::collections::HashMap::new(),
        };

        let summary = disease_markdown(&disease, &[]).expect("rendered markdown");
        assert!(summary.contains("Genes: BRAF (OT 0.912), NRAS (OT 0.701)"));

        let genes = disease_markdown(&disease, &["genes".to_string()]).expect("rendered markdown");
        assert!(genes.contains("| Gene | Relationship | Source | OpenTargets |"));
        assert!(genes.contains("overall 0.912; GWAS 0.321; rare 0.654; somatic 0.876"));
        assert!(genes.contains("| NRAS | associated | CIViC | - |"));
    }

    #[test]
    fn gene_markdown_renders_hpa_section_details() {
        let gene = Gene {
            symbol: "BRAF".to_string(),
            name: "B-Raf proto-oncogene".to_string(),
            entrez_id: "673".to_string(),
            ensembl_id: Some("ENSG00000157764".to_string()),
            location: Some("7q34".to_string()),
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: Some("P15056".to_string()),
            summary: Some("Kinase involved in MAPK signaling.".to_string()),
            gene_type: Some("protein-coding".to_string()),
            aliases: vec!["BRAF1".to_string()],
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: Some(crate::sources::hpa::GeneHpa {
                tissues: vec![
                    crate::sources::hpa::HpaTissueExpression {
                        tissue: "Liver".to_string(),
                        level: "High".to_string(),
                    },
                    crate::sources::hpa::HpaTissueExpression {
                        tissue: "Kidney".to_string(),
                        level: "Medium".to_string(),
                    },
                ],
                subcellular_main_location: vec!["cytosol".to_string(), "vesicles".to_string()],
                subcellular_additional_location: vec!["plasma membrane".to_string()],
                reliability: Some("Supported".to_string()),
                protein_summary: Some("Ubiquitous cytoplasmic expression.".to_string()),
                rna_summary: Some("Low tissue specificity; Detected in all".to_string()),
            }),
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
        };

        let markdown = gene_markdown(&gene, &["hpa".to_string()]).expect("rendered markdown");

        assert!(markdown.contains("# BRAF - hpa"));
        assert!(markdown.contains("## Human Protein Atlas"));
        assert!(markdown.contains("Reliability: Supported"));
        assert!(markdown.contains("Protein summary: Ubiquitous cytoplasmic expression."));
        assert!(markdown.contains("RNA summary: Low tissue specificity; Detected in all"));
        assert!(markdown.contains("Subcellular main locations: cytosol, vesicles"));
        assert!(markdown.contains("Subcellular additional locations: plasma membrane"));
        assert!(markdown.contains("| Tissue | Level |"));
        assert!(markdown.contains("| Liver | High |"));
        assert!(markdown.contains("| Kidney | Medium |"));
    }

    #[test]
    fn gene_markdown_section_only_shows_constraint_section() {
        let gene = Gene {
            symbol: "TP53".to_string(),
            name: "tumor protein p53".to_string(),
            entrez_id: "7157".to_string(),
            ensembl_id: Some("ENSG00000141510".to_string()),
            location: Some("17p13.1".to_string()),
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: Some("P04637".to_string()),
            summary: Some("Tumor suppressor.".to_string()),
            gene_type: Some("protein-coding".to_string()),
            aliases: vec!["P53".to_string()],
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: Some(crate::entities::gene::GeneConstraint {
                pli: None,
                loeuf: None,
                mis_z: None,
                syn_z: None,
                transcript: Some("ENST00000269305".to_string()),
                source: "gnomAD".to_string(),
                source_version: "v4".to_string(),
                reference_genome: "GRCh38".to_string(),
            }),
            disgenet: None,
        };

        let markdown =
            gene_markdown(&gene, &["constraint".to_string()]).expect("rendered markdown");

        assert!(markdown.contains("# TP53 - constraint"));
        assert!(markdown.contains("## Constraint (gnomAD)"));
        assert!(markdown.contains("Source: gnomAD"));
        assert!(markdown.contains("Version: v4"));
        assert!(markdown.contains("Reference genome: GRCh38"));
        assert!(markdown.contains("Transcript: ENST00000269305"));
        assert!(markdown.contains("No gnomAD constraint metrics returned for this gene query."));
    }

    #[test]
    fn gene_markdown_section_only_shows_disgenet_section() {
        let gene = Gene {
            symbol: "TP53".to_string(),
            name: "tumor protein p53".to_string(),
            entrez_id: "7157".to_string(),
            ensembl_id: None,
            location: None,
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: None,
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: Some(crate::entities::gene::GeneDisgenet {
                associations: vec![crate::entities::gene::GeneDisgenetAssociation {
                    disease_name: "Breast Carcinoma".to_string(),
                    disease_cui: "C0678222".to_string(),
                    score: 0.91,
                    publication_count: Some(1234),
                    clinical_trial_count: Some(4),
                    evidence_index: Some(0.72),
                    evidence_level: Some("Definitive".to_string()),
                }],
            }),
        };

        let markdown = gene_markdown(&gene, &["disgenet".to_string()]).expect("rendered markdown");

        assert!(markdown.contains("# TP53 - disgenet"));
        assert!(markdown.contains("## DisGeNET"));
        assert!(markdown.contains("| Disease | UMLS CUI | Score | PMIDs | Trials | EL | EI |"));
        assert!(
            markdown.contains(
                "| Breast Carcinoma | C0678222 | 0.910 | 1234 | 4 | Definitive | 0.720 |"
            )
        );
    }

    #[test]
    fn gene_markdown_pathways_show_source_labels() {
        let gene = Gene {
            symbol: "BRAF".to_string(),
            name: "B-Raf proto-oncogene".to_string(),
            entrez_id: "673".to_string(),
            ensembl_id: None,
            location: Some("7q34".to_string()),
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: None,
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: Some(vec![
                crate::entities::gene::GenePathway {
                    source: "KEGG".to_string(),
                    id: "hsa05200".to_string(),
                    name: "Pathways in cancer".to_string(),
                },
                crate::entities::gene::GenePathway {
                    source: "Reactome".to_string(),
                    id: "R-HSA-5673001".to_string(),
                    name: "RAF/MAP kinase cascade".to_string(),
                },
            ]),
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
        };

        let markdown = gene_markdown(&gene, &[]).expect("rendered markdown");
        assert!(markdown.contains("| Source | ID | Name |"));
        assert!(markdown.contains("| KEGG | hsa05200 | Pathways in cancer |"));
        assert!(markdown.contains("| Reactome | R-HSA-5673001 | RAF/MAP kinase cascade |"));
        assert!(!markdown.contains("Showing pathway rows from Reactome search results."));
    }

    #[test]
    fn pathway_markdown_uses_source_and_source_specific_evidence_url() {
        let pathway = Pathway {
            source: "KEGG".to_string(),
            id: "hsa05200".to_string(),
            name: "Pathways in cancer".to_string(),
            species: Some("Homo sapiens".to_string()),
            summary: Some("Cancer pathway overview.".to_string()),
            genes: vec!["BRAF".to_string(), "EGFR".to_string()],
            events: Vec::new(),
            enrichment: Vec::new(),
        };

        let markdown = pathway_markdown(&pathway, &[]).expect("rendered markdown");
        assert!(markdown.contains("Source: KEGG"));
        assert!(markdown.contains("[KEGG](https://www.kegg.jp/entry/hsa05200)"));
    }

    #[test]
    fn pathway_markdown_hides_genes_section_when_genes_are_empty() {
        let pathway = Pathway {
            source: "KEGG".to_string(),
            id: "hsa05200".to_string(),
            name: "Pathways in cancer".to_string(),
            species: Some("Homo sapiens".to_string()),
            summary: Some("Cancer pathway overview.".to_string()),
            genes: Vec::new(),
            events: Vec::new(),
            enrichment: Vec::new(),
        };

        let markdown = pathway_markdown(&pathway, &[]).expect("rendered markdown");
        assert!(!markdown.contains("## Genes"));
        assert!(!markdown.contains("BRAF"));
    }

    #[test]
    fn pathway_search_markdown_shows_source_column() {
        let results = vec![
            PathwaySearchResult {
                source: "Reactome".to_string(),
                id: "R-HSA-5673001".to_string(),
                name: "RAF/MAP kinase cascade".to_string(),
            },
            PathwaySearchResult {
                source: "KEGG".to_string(),
                id: "hsa04010".to_string(),
                name: "MAPK signaling pathway".to_string(),
            },
        ];

        let markdown =
            pathway_search_markdown("MAPK", &results, Some(results.len())).expect("markdown");
        assert!(markdown.contains("| Source | ID | Name |"));
        assert!(markdown.contains("| Reactome | R-HSA-5673001 | RAF/MAP kinase cascade |"));
        assert!(markdown.contains("| KEGG | hsa04010 | MAPK signaling pathway |"));
    }

    #[test]
    fn markdown_render_variant_entity() {
        let variant: Variant = serde_json::from_value(serde_json::json!({
            "id": "chr7:g.55259515T>G",
            "gene": "EGFR",
            "hgvs_p": "p.L858R",
            "significance": "Pathogenic"
        }))
        .expect("variant should deserialize");

        let markdown = variant_markdown(&variant, &[]).expect("rendered markdown");
        assert!(markdown.contains("EGFR"));
        assert!(markdown.contains("p.L858R"));
    }

    #[test]
    fn variant_oncokb_markdown_shows_truncation_note() {
        let result = VariantOncoKbResult {
            gene: "EGFR".to_string(),
            alteration: "L858R".to_string(),
            oncogenic: Some("Oncogenic".to_string()),
            level: Some("Level 1".to_string()),
            effect: Some("Gain-of-function".to_string()),
            therapies: vec![
                TreatmentImplication {
                    level: "Level 1".to_string(),
                    drugs: vec!["osimertinib".to_string()],
                    cancer_type: Some("Lung adenocarcinoma".to_string()),
                    note: None,
                },
                TreatmentImplication {
                    level: "Level 2".to_string(),
                    drugs: vec!["afatinib".to_string()],
                    cancer_type: Some("Lung adenocarcinoma".to_string()),
                    note: Some("(and 2 more)".to_string()),
                },
            ],
        };

        let markdown = variant_oncokb_markdown(&result);
        assert!(markdown.contains("| Drug | Level | Cancer Type | Note |"));
        assert!(markdown.contains("(and 2 more)"));
    }

    #[test]
    fn pagination_footer_offset_suppresses_more_when_complete_single_result() {
        let footer = pagination_footer(PaginationFooterMode::Offset, 0, 10, 1, Some(1), None);
        assert!(footer.contains("Showing 1 of 1 results."));
        assert!(!footer.contains("Use --offset"));
    }

    #[test]
    fn pagination_footer_offset_keeps_more_when_additional_rows_exist() {
        let footer = pagination_footer(PaginationFooterMode::Offset, 0, 2, 2, Some(10), None);
        assert!(footer.contains("Showing 1-2 of 10 results."));
        assert!(footer.contains("Use --offset 2 for more."));
    }

    #[test]
    fn pagination_footer_offset_suppresses_more_on_last_page() {
        let footer = pagination_footer(PaginationFooterMode::Offset, 8, 2, 2, Some(10), None);
        assert!(footer.contains("Showing 9-10 of 10 results."));
        assert!(!footer.contains("Use --offset"));
    }

    #[test]
    fn pagination_footer_cursor_prefers_offset_guidance_without_placeholder() {
        let footer = pagination_footer(
            PaginationFooterMode::Cursor,
            0,
            1,
            1,
            Some(20),
            Some("abc123"),
        );
        assert!(footer.contains("Use --offset 1 for more."));
        assert!(footer.contains("--next-page is also supported"));
        assert!(!footer.contains("<TOKEN>"));
    }

    #[test]
    fn related_article_uses_article_entities_helper_command() {
        let article = Article {
            pmid: Some("22663011".to_string()),
            pmcid: None,
            doi: None,
            title: "Example".to_string(),
            authors: Vec::new(),
            journal: None,
            date: None,
            citation_count: None,
            publication_type: None,
            open_access: None,
            abstract_text: None,
            full_text_path: None,
            full_text_note: None,
            annotations: Some(ArticleAnnotations {
                genes: vec![AnnotationCount {
                    text: "BRAF".to_string(),
                    count: 1,
                }],
                diseases: Vec::new(),
                chemicals: Vec::new(),
                mutations: Vec::new(),
            }),
            semantic_scholar: None,
            pubtator_fallback: false,
        };

        let related = related_article(&article);
        assert!(related.contains(&"biomcp article entities 22663011".to_string()));
        assert!(related.contains(&"biomcp article citations 22663011 --limit 3".to_string()));
        assert!(related.contains(&"biomcp article references 22663011 --limit 3".to_string()));
        assert!(related.contains(&"biomcp article recommendations 22663011 --limit 3".to_string()));
        assert!(!related.iter().any(|cmd| cmd.contains("biomcp get article")));
    }

    #[test]
    fn article_markdown_renders_semantic_scholar_section() {
        let article = Article {
            pmid: Some("22663011".to_string()),
            pmcid: None,
            doi: Some("10.1000/example".to_string()),
            title: "Example".to_string(),
            authors: Vec::new(),
            journal: Some("Example Journal".to_string()),
            date: Some("2024-01-01".to_string()),
            citation_count: Some(12),
            publication_type: None,
            open_access: Some(true),
            abstract_text: None,
            full_text_path: None,
            full_text_note: None,
            annotations: None,
            semantic_scholar: Some(crate::entities::article::ArticleSemanticScholar {
                paper_id: Some("paper-1".to_string()),
                tldr: Some("A concise summary.".to_string()),
                citation_count: Some(20),
                influential_citation_count: Some(4),
                reference_count: Some(10),
                is_open_access: Some(true),
                open_access_pdf: Some(crate::entities::article::ArticleSemanticScholarPdf {
                    url: "https://example.org/paper.pdf".to_string(),
                    status: Some("GREEN".to_string()),
                    license: Some("CC-BY".to_string()),
                }),
            }),
            pubtator_fallback: false,
        };

        let markdown =
            article_markdown(&article, &["tldr".to_string()]).expect("markdown should render");
        assert!(markdown.contains("## Semantic Scholar"));
        assert!(markdown.contains("TLDR: A concise summary."));
        assert!(markdown.contains("Influential citations: 4"));
        assert!(markdown.contains("Open-access PDF: https://example.org/paper.pdf"));
    }

    #[test]
    fn article_graph_markdown_renders_expected_table_headers() {
        let result = crate::entities::article::ArticleGraphResult {
            article: crate::entities::article::ArticleRelatedPaper {
                paper_id: Some("paper-1".to_string()),
                pmid: Some("22663011".to_string()),
                doi: None,
                arxiv_id: None,
                title: "Seed".to_string(),
                journal: None,
                year: Some(2012),
            },
            edges: vec![crate::entities::article::ArticleGraphEdge {
                paper: crate::entities::article::ArticleRelatedPaper {
                    paper_id: Some("paper-2".to_string()),
                    pmid: Some("24200969".to_string()),
                    doi: None,
                    arxiv_id: None,
                    title: "Related paper".to_string(),
                    journal: Some("Nature".to_string()),
                    year: Some(2014),
                },
                intents: vec!["Background".to_string()],
                contexts: vec!["Important supporting context".to_string()],
                is_influential: true,
            }],
        };

        let markdown = article_graph_markdown("Citations", &result).expect("graph markdown");
        assert!(markdown.contains("# Citations for PMID 22663011"));
        assert!(markdown.contains("| PMID | Title | Intents | Influential | Context |"));
        assert!(markdown.contains(
            "| 24200969 | Related paper | Background | yes | Important supporting context |"
        ));
    }

    #[test]
    fn related_pgx_uses_search_flags() {
        let pgx = Pgx {
            query: "CYP2D6".to_string(),
            gene: Some("CYP2D6".to_string()),
            drug: Some("warfarin sodium".to_string()),
            interactions: Vec::new(),
            recommendations: Vec::new(),
            frequencies: Vec::new(),
            guidelines: Vec::new(),
            annotations: Vec::new(),
            annotations_note: None,
        };

        let related = related_pgx(&pgx);
        assert!(related.contains(&"biomcp search pgx -g CYP2D6".to_string()));
        assert!(related.contains(&"biomcp search pgx -d \"warfarin sodium\"".to_string()));
    }

    #[test]
    fn gene_evidence_urls_include_ensembl_and_omim() {
        let gene = Gene {
            symbol: "BRAF".to_string(),
            name: "B-Raf proto-oncogene".to_string(),
            entrez_id: "673".to_string(),
            ensembl_id: Some("ENSG00000157764".to_string()),
            location: Some("7q34".to_string()),
            genomic_coordinates: None,
            omim_id: Some("164757".to_string()),
            uniprot_id: Some("P15056".to_string()),
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
        };

        let urls = gene_evidence_urls(&gene);
        assert!(urls.contains(&(
            "Ensembl",
            "https://www.ensembl.org/Homo_sapiens/Gene/Summary?g=ENSG00000157764".to_string()
        )));
        assert!(urls.contains(&("OMIM", "https://www.omim.org/entry/164757".to_string())));
    }

    #[test]
    fn disease_markdown_section_only_shows_disgenet_section() {
        let disease = Disease {
            id: "MONDO:0007254".to_string(),
            name: "breast cancer".to_string(),
            definition: None,
            synonyms: Vec::new(),
            parents: Vec::new(),
            associated_genes: Vec::new(),
            gene_associations: Vec::new(),
            top_genes: Vec::new(),
            top_gene_scores: Vec::new(),
            treatment_landscape: Vec::new(),
            recruiting_trial_count: None,
            pathways: Vec::new(),
            phenotypes: Vec::new(),
            variants: Vec::new(),
            models: Vec::new(),
            prevalence: Vec::new(),
            prevalence_note: None,
            civic: None,
            disgenet: Some(crate::entities::disease::DiseaseDisgenet {
                associations: vec![crate::entities::disease::DiseaseDisgenetAssociation {
                    symbol: "TP53".to_string(),
                    entrez_id: Some(7157),
                    score: 0.91,
                    publication_count: Some(1234),
                    clinical_trial_count: Some(4),
                    evidence_index: Some(0.72),
                    evidence_level: Some("Definitive".to_string()),
                }],
            }),
            xrefs: std::collections::HashMap::new(),
        };

        let markdown =
            disease_markdown(&disease, &["disgenet".to_string()]).expect("rendered markdown");

        assert!(markdown.contains("# breast cancer - disgenet"));
        assert!(markdown.contains("## DisGeNET"));
        assert!(markdown.contains("| Gene | Entrez ID | Score | PMIDs | Trials | EL | EI |"));
        assert!(markdown.contains("| TP53 | 7157 | 0.910 | 1234 | 4 | Definitive | 0.720 |"));
    }

    #[test]
    fn variant_evidence_urls_include_dbsnp_and_cosmic() {
        let variant: Variant = serde_json::from_value(serde_json::json!({
            "id": "chr7:g.140453136A>T",
            "gene": "BRAF",
            "rsid": "rs113488022",
            "cosmic_id": "COSM476",
            "clinvar_id": "13961"
        }))
        .expect("variant should deserialize");

        let urls = variant_evidence_urls(&variant);
        assert!(urls.contains(&(
            "dbSNP",
            "https://www.ncbi.nlm.nih.gov/snp/rs113488022".to_string()
        )));
        assert!(urls.contains(&(
            "COSMIC",
            "https://cancer.sanger.ac.uk/cosmic/mutation/overview?id=COSM476".to_string()
        )));
    }

    #[test]
    fn drug_evidence_urls_include_chembl() {
        let drug = Drug {
            name: "osimertinib".to_string(),
            drugbank_id: Some("DB09330".to_string()),
            chembl_id: Some("CHEMBL3353410".to_string()),
            unii: None,
            drug_type: None,
            mechanism: None,
            mechanisms: Vec::new(),
            approval_date: None,
            brand_names: Vec::new(),
            route: None,
            targets: Vec::new(),
            indications: Vec::new(),
            interactions: Vec::new(),
            interaction_text: None,
            pharm_classes: Vec::new(),
            top_adverse_events: Vec::new(),
            label: None,
            shortage: None,
            approvals: None,
            civic: None,
        };

        let urls = drug_evidence_urls(&drug);
        assert!(urls.contains(&(
            "ChEMBL",
            "https://www.ebi.ac.uk/chembl/compound_report_card/CHEMBL3353410".to_string()
        )));
    }

    #[test]
    fn drug_markdown_uses_label_interaction_text_before_public_unavailable_fallback() {
        let drug = Drug {
            name: "warfarin".to_string(),
            drugbank_id: Some("DB00682".to_string()),
            chembl_id: None,
            unii: None,
            drug_type: None,
            mechanism: None,
            mechanisms: Vec::new(),
            approval_date: None,
            brand_names: Vec::new(),
            route: None,
            targets: Vec::new(),
            indications: Vec::new(),
            interactions: Vec::new(),
            interaction_text: Some(
                "DRUG INTERACTIONS\n\nWarfarin interacts with aspirin.".to_string(),
            ),
            pharm_classes: Vec::new(),
            top_adverse_events: Vec::new(),
            label: None,
            shortage: None,
            approvals: None,
            civic: None,
        };

        let markdown = drug_markdown(&drug, &["interactions".to_string()]).expect("markdown");
        assert!(markdown.contains("## Interactions"));
        assert!(markdown.contains("DRUG INTERACTIONS"));
        assert!(!markdown.contains("No known drug-drug interactions found."));
    }

    #[test]
    fn drug_markdown_uses_truthful_public_unavailable_interactions_message() {
        let drug = Drug {
            name: "pembrolizumab".to_string(),
            drugbank_id: Some("DB09037".to_string()),
            chembl_id: None,
            unii: None,
            drug_type: None,
            mechanism: None,
            mechanisms: Vec::new(),
            approval_date: None,
            brand_names: Vec::new(),
            route: None,
            targets: Vec::new(),
            indications: Vec::new(),
            interactions: Vec::new(),
            interaction_text: None,
            pharm_classes: Vec::new(),
            top_adverse_events: Vec::new(),
            label: None,
            shortage: None,
            approvals: None,
            civic: None,
        };

        let markdown = drug_markdown(&drug, &["interactions".to_string()]).expect("markdown");
        assert!(markdown.contains("Interaction details not available from public sources."));
        assert!(!markdown.contains("No known drug-drug interactions found."));
    }

    #[test]
    fn pgx_markdown_includes_evidence_links() {
        let pgx = Pgx {
            query: "CYP2D6".to_string(),
            gene: Some("CYP2D6".to_string()),
            drug: Some("warfarin".to_string()),
            interactions: Vec::new(),
            recommendations: Vec::new(),
            frequencies: Vec::new(),
            guidelines: Vec::new(),
            annotations: Vec::new(),
            annotations_note: None,
        };

        let markdown = pgx_markdown(&pgx, &[]).expect("rendered markdown");
        assert!(markdown.contains("[CPIC](https://cpicpgx.org/genes/cyp2d6/)"));
        assert!(markdown.contains("[PharmGKB](https://www.pharmgkb.org/gene/CYP2D6)"));
        assert!(markdown.contains("[PharmGKB](https://www.pharmgkb.org/chemical/warfarin)"));
    }

    #[test]
    fn related_device_event_uses_supported_search_subcommands() {
        let event = DeviceEvent {
            report_id: "MDR-123".to_string(),
            report_number: None,
            device: "Infusion Pump".to_string(),
            manufacturer: None,
            event_type: None,
            date: None,
            description: None,
        };

        let related = related_device_event(&event);
        assert!(related.contains(
            &"biomcp search adverse-event --type device --device \"Infusion Pump\"".to_string()
        ));
        assert!(related.contains(
            &"biomcp search adverse-event --type recall --classification \"Class I\"".to_string()
        ));
    }

    #[test]
    fn related_protein_includes_complexes_follow_up() {
        let protein = Protein {
            accession: "P15056".to_string(),
            entry_id: Some("BRAF_HUMAN".to_string()),
            name: "Serine/threonine-protein kinase B-raf".to_string(),
            gene_symbol: Some("BRAF".to_string()),
            organism: Some("Homo sapiens".to_string()),
            length: Some(766),
            function: None,
            structures: vec!["6V34".to_string()],
            structure_count: Some(1),
            domains: Vec::new(),
            interactions: Vec::new(),
            complexes: Vec::new(),
        };

        let related = related_protein(&protein, &[]);
        assert!(related.contains(&"biomcp get protein P15056 structures".to_string()));
        assert!(related.contains(&"biomcp get protein P15056 complexes".to_string()));
        assert!(related.contains(&"biomcp get gene BRAF".to_string()));
    }

    #[test]
    fn related_protein_excludes_requested_sections() {
        let protein = Protein {
            accession: "P15056".to_string(),
            entry_id: Some("BRAF_HUMAN".to_string()),
            name: "Serine/threonine-protein kinase B-raf".to_string(),
            gene_symbol: Some("BRAF".to_string()),
            organism: Some("Homo sapiens".to_string()),
            length: Some(766),
            function: None,
            structures: vec!["6V34".to_string()],
            structure_count: Some(1),
            domains: Vec::new(),
            interactions: Vec::new(),
            complexes: Vec::new(),
        };

        let related = related_protein(
            &protein,
            &["complexes".to_string(), "structures".to_string()],
        );
        assert!(!related.contains(&"biomcp get protein P15056 structures".to_string()));
        assert!(!related.contains(&"biomcp get protein P15056 complexes".to_string()));
        assert!(related.contains(&"biomcp get gene BRAF".to_string()));
    }

    #[test]
    fn protein_markdown_renders_complexes_summary_and_detail_bullets() {
        let protein = Protein {
            accession: "P15056".to_string(),
            entry_id: Some("BRAF_HUMAN".to_string()),
            name: "Serine/threonine-protein kinase B-raf".to_string(),
            gene_symbol: Some("BRAF".to_string()),
            organism: Some("Homo sapiens".to_string()),
            length: Some(766),
            function: None,
            structures: Vec::new(),
            structure_count: None,
            domains: Vec::new(),
            interactions: Vec::new(),
            complexes: vec![
                ProteinComplex {
                    accession: "CPX-1234".to_string(),
                    name: "BRAF signaling complex with an intentionally long display label that should truncate in the summary table".to_string(),
                    description: Some("Signals through MAPK.".to_string()),
                    curation: ProteinComplexCuration::Curated,
                    components: vec![
                        ProteinComplexComponent {
                            accession: "P15056".to_string(),
                            name: "BRAF".to_string(),
                            stoichiometry: Some("1".to_string()),
                        },
                        ProteinComplexComponent {
                            accession: "Q02750".to_string(),
                            name: "MAP2K1".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "P10398".to_string(),
                            name: "RAF1".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "P07900".to_string(),
                            name: "HSP90AA1".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "Q16543".to_string(),
                            name: "CDC37".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "Q9Y243".to_string(),
                            name: "AKT3".to_string(),
                            stoichiometry: None,
                        },
                        ProteinComplexComponent {
                            accession: "P31749".to_string(),
                            name: "AKT1".to_string(),
                            stoichiometry: None,
                        },
                    ],
                },
                ProteinComplex {
                    accession: "CPX-5678".to_string(),
                    name: "Complex with no listed members".to_string(),
                    description: None,
                    curation: ProteinComplexCuration::Predicted,
                    components: Vec::new(),
                },
            ],
        };

        let markdown = protein_markdown(&protein, &["complexes".to_string()]).expect("markdown");
        assert!(markdown.contains("## Complexes"));
        assert!(markdown.contains("| ID | Name | Members | Curation |"));
        assert!(!markdown.contains("| ID | Name | Components | Curation |"));
        assert!(
            markdown.contains(
                "| CPX-1234 | BRAF signaling complex with an intentionally long di… | 7 | curated |"
            ),
            "expected truncated complex summary row, got:\n{markdown}"
        );
        assert!(markdown.contains(
            "- `CPX-1234` members (7): BRAF (1), MAP2K1, RAF1, HSP90AA1, CDC37, +2 more\n  Description: Signals through MAPK."
        ));
        assert!(markdown.contains("- `CPX-5678` members (0): none listed"));
        assert!(!markdown.contains("- `CPX-1234`: Signals through MAPK."));
        assert!(!markdown.contains("AKT3"));
        assert!(!markdown.contains("AKT1"));
        assert!(!markdown.contains("See also: biomcp get protein P15056 complexes"));
    }

    #[test]
    fn article_search_markdown_groups_results_by_source() {
        let rows = vec![
            ArticleSearchResult {
                pmid: "1".into(),
                title: "Entity-ranked".into(),
                journal: Some("Journal A".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(10),
                source: ArticleSource::PubTator,
                score: Some(99.1),
                is_retracted: Some(false),
            },
            ArticleSearchResult {
                pmid: "2".into(),
                title: "Field-ranked".into(),
                journal: Some("Journal B".into()),
                date: Some("2025-01-02".into()),
                citation_count: Some(12),
                source: ArticleSource::EuropePmc,
                score: None,
                is_retracted: Some(false),
            },
        ];

        let markdown = article_search_markdown("gene=BRAF", &rows).expect("markdown should render");
        assert!(markdown.contains("## PubTator3"));
        assert!(markdown.contains("## Europe PMC"));
        assert!(markdown.contains("| PMID | Title | Journal | Date | Score |"));
        assert!(markdown.contains("| PMID | Title | Journal | Date | Cit. |"));
    }

    #[test]
    fn search_all_markdown_renders_section_note() {
        let results = crate::cli::search_all::SearchAllResults {
            query: "gene=EGFR disease=non-small cell lung cancer".to_string(),
            sections: vec![crate::cli::search_all::SearchAllSection {
                entity: "variant".to_string(),
                label: "Variants".to_string(),
                count: 1,
                total: Some(1),
                error: None,
                note: Some(
                    "No disease-filtered variants found; showing top gene variants.".to_string(),
                ),
                results: vec![serde_json::json!({
                    "id": "rs121434568",
                    "gene": "EGFR",
                    "hgvs_p": "L858R",
                    "significance": "Pathogenic",
                })],
                links: Vec::new(),
            }],
            searches_dispatched: 1,
            searches_with_results: 1,
            wall_time_ms: 42,
        };

        let markdown = search_all_markdown(&results, false).expect("markdown should render");
        assert!(
            markdown.contains("> No disease-filtered variants found; showing top gene variants.")
        );
    }

    #[test]
    fn study_list_markdown_renders_study_table() {
        let markdown = study_list_markdown(&[StudyInfo {
            study_id: "msk_impact_2017".to_string(),
            name: "MSK-IMPACT".to_string(),
            cancer_type: Some("mixed".to_string()),
            citation: Some("Zehir et al.".to_string()),
            sample_count: Some(10945),
            available_data: vec!["mutations".to_string(), "cna".to_string()],
        }]);

        assert!(markdown.contains("# Study Datasets"));
        assert!(markdown.contains("| Study ID | Name | Cancer Type | Samples | Available Data |"));
        assert!(markdown.contains("msk_impact_2017"));
        assert!(markdown.contains("mutations, cna"));
    }

    #[test]
    fn study_query_markdown_renders_mutation_shape() {
        let markdown = study_query_markdown(&StudyQueryResult::MutationFrequency(
            StudyMutationFrequencyResult {
                study_id: "msk_impact_2017".to_string(),
                gene: "TP53".to_string(),
                mutation_count: 10,
                unique_samples: 9,
                total_samples: 100,
                frequency: 0.09,
                top_variant_classes: vec![("Missense_Mutation".to_string(), 8)],
                top_protein_changes: vec![("p.R175H".to_string(), 3)],
            },
        ));

        assert!(markdown.contains("# Study Mutation Frequency: TP53 (msk_impact_2017)"));
        assert!(markdown.contains("| Mutation records | 10 |"));
        assert!(markdown.contains("## Top Variant Classes"));
        assert!(markdown.contains("## Top Protein Changes"));
    }

    #[test]
    fn study_query_markdown_renders_cna_and_expression_shapes() {
        let cna = study_query_markdown(&StudyQueryResult::CnaDistribution(
            StudyCnaDistributionResult {
                study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
                gene: "ERBB2".to_string(),
                total_samples: 20,
                deep_deletion: 1,
                shallow_deletion: 2,
                diploid: 10,
                gain: 4,
                amplification: 3,
            },
        ));
        assert!(cna.contains("# Study CNA Distribution: ERBB2 (brca_tcga_pan_can_atlas_2018)"));
        assert!(cna.contains("| Amplification (2) | 3 |"));

        let expression = study_query_markdown(&StudyQueryResult::ExpressionDistribution(
            StudyExpressionDistributionResult {
                study_id: "paad_qcmg_uq_2016".to_string(),
                gene: "KRAS".to_string(),
                file: "data_mrna_seq_v2_rsem_zscores_ref_all_samples.txt".to_string(),
                sample_count: 50,
                mean: 0.2,
                median: 0.1,
                min: -2.0,
                max: 2.5,
                q1: -0.4,
                q3: 0.5,
            },
        ));
        assert!(expression.contains("# Study Expression Distribution: KRAS (paad_qcmg_uq_2016)"));
        assert!(expression.contains("| Sample count | 50 |"));
    }

    #[test]
    fn study_filter_markdown_renders_tables_and_samples() {
        let markdown = study_filter_markdown(&StudyFilterResult {
            study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
            criteria: vec![
                crate::entities::study::FilterCriterionSummary {
                    description: "mutated TP53".to_string(),
                    matched_count: 3,
                },
                crate::entities::study::FilterCriterionSummary {
                    description: "amplified ERBB2".to_string(),
                    matched_count: 2,
                },
            ],
            total_study_samples: Some(4),
            matched_count: 2,
            matched_sample_ids: vec!["S2".to_string(), "S3".to_string()],
        });

        assert!(markdown.contains("# Study Filter: brca_tcga_pan_can_atlas_2018"));
        assert!(markdown.contains("## Criteria"));
        assert!(markdown.contains("| Filter | Matching Samples |"));
        assert!(markdown.contains("| mutated TP53 | 3 |"));
        assert!(markdown.contains("## Result"));
        assert!(markdown.contains("| Study Total Samples | 4 |"));
        assert!(markdown.contains("| Intersection | 2 |"));
        assert!(markdown.contains("## Matched Samples"));
        assert!(markdown.contains("S2"));
        assert!(markdown.contains("S3"));
    }

    #[test]
    fn study_filter_markdown_renders_empty_results_and_unknown_totals() {
        let markdown = study_filter_markdown(&StudyFilterResult {
            study_id: "demo_study".to_string(),
            criteria: vec![crate::entities::study::FilterCriterionSummary {
                description: "expression > 1.5 for MYC".to_string(),
                matched_count: 0,
            }],
            total_study_samples: None,
            matched_count: 0,
            matched_sample_ids: Vec::new(),
        });

        assert!(markdown.contains("| Study Total Samples | - |"));
        assert!(markdown.contains("| Intersection | 0 |"));
        assert!(markdown.contains("## Matched Samples"));
        assert!(markdown.contains("\nNone\n"));
    }

    #[test]
    fn study_filter_markdown_truncates_long_sample_lists() {
        let markdown = study_filter_markdown(&StudyFilterResult {
            study_id: "long_study".to_string(),
            criteria: vec![crate::entities::study::FilterCriterionSummary {
                description: "mutated TP53".to_string(),
                matched_count: 55,
            }],
            total_study_samples: Some(100),
            matched_count: 55,
            matched_sample_ids: (1..=55).map(|idx| format!("S{idx}")).collect(),
        });

        assert!(markdown.contains("S1"));
        assert!(markdown.contains("S50"));
        assert!(!markdown.contains("S51\n"));
        assert!(markdown.contains("... and 5 more (use --json for full list)"));
    }

    #[test]
    fn study_co_occurrence_markdown_renders_pair_table() {
        let markdown = study_co_occurrence_markdown(&StudyCoOccurrenceResult {
            study_id: "msk_impact_2017".to_string(),
            genes: vec!["TP53".to_string(), "KRAS".to_string()],
            total_samples: 100,
            sample_universe_basis: StudySampleUniverseBasis::ClinicalSampleFile,
            pairs: vec![StudyCoOccurrencePair {
                gene_a: "TP53".to_string(),
                gene_b: "KRAS".to_string(),
                both_mutated: 10,
                a_only: 20,
                b_only: 15,
                neither: 55,
                log_odds_ratio: Some(0.1234),
                p_value: Some(6.0e-22),
            }],
        });

        assert!(markdown.contains("# Study Co-occurrence: msk_impact_2017"));
        assert!(markdown.contains("Sample universe: clinical sample file"));
        assert!(markdown.contains(
            "| Gene A | Gene B | Both | A only | B only | Neither | Log Odds Ratio | p-value |"
        ));
        assert!(markdown.contains("| TP53 | KRAS | 10 | 20 | 15 | 55 | 0.123400 | 6.000e-22 |"));
    }

    #[test]
    fn study_co_occurrence_markdown_marks_mutation_observed_fallback() {
        let markdown = study_co_occurrence_markdown(&StudyCoOccurrenceResult {
            study_id: "fallback_study".to_string(),
            genes: vec!["TP53".to_string(), "KRAS".to_string()],
            total_samples: 3,
            sample_universe_basis: StudySampleUniverseBasis::MutationObserved,
            pairs: vec![],
        });

        assert!(markdown.contains(
            "Sample universe: mutation-observed samples only (clinical sample file unavailable)"
        ));
    }

    #[test]
    fn study_cohort_markdown_renders_group_counts() {
        let markdown = study_cohort_markdown(&StudyCohortResult {
            study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
            gene: "TP53".to_string(),
            stratification: "mutation".to_string(),
            mutant_samples: 348,
            wildtype_samples: 736,
            mutant_patients: 348,
            wildtype_patients: 736,
            total_samples: 1084,
            total_patients: 1084,
        });

        assert!(markdown.contains("# Study Cohort: TP53 (brca_tcga_pan_can_atlas_2018)"));
        assert!(markdown.contains("Stratification: mutation status"));
        assert!(markdown.contains("| Group | Samples | Patients |"));
        assert!(markdown.contains("| TP53-mutant | 348 | 348 |"));
        assert!(markdown.contains("| Total | 1084 | 1084 |"));
    }

    #[test]
    fn study_survival_markdown_renders_group_table() {
        let markdown = study_survival_markdown(&StudySurvivalResult {
            study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
            gene: "TP53".to_string(),
            endpoint: StudySurvivalEndpoint::Os,
            groups: vec![
                StudySurvivalGroupResult {
                    group_name: "TP53-mutant".to_string(),
                    n_patients: 340,
                    n_events: 48,
                    n_censored: 292,
                    km_median_months: Some(85.2),
                    survival_1yr: Some(0.91),
                    survival_3yr: Some(0.72),
                    survival_5yr: None,
                    event_rate: 0.141176,
                    km_curve_points: Vec::new(),
                },
                StudySurvivalGroupResult {
                    group_name: "TP53-wildtype".to_string(),
                    n_patients: 720,
                    n_events: 64,
                    n_censored: 656,
                    km_median_months: None,
                    survival_1yr: Some(0.97),
                    survival_3yr: Some(0.88),
                    survival_5yr: Some(0.74),
                    event_rate: 0.088889,
                    km_curve_points: Vec::new(),
                },
            ],
            log_rank_p: Some(0.0042),
        });

        assert!(markdown.contains("# Study Survival: TP53 (brca_tcga_pan_can_atlas_2018)"));
        assert!(markdown.contains("Endpoint: Overall Survival (OS)"));
        assert!(markdown.contains(
            "| Group | N | Events | Censored | Event Rate | KM Median | 1yr | 3yr | 5yr |"
        ));
        assert!(
            markdown
                .contains("| TP53-mutant | 340 | 48 | 292 | 0.141176 | 85.2 | 0.910 | 0.720 | - |")
        );
        assert!(markdown.contains("Log-rank p-value: 0.004"));
    }

    #[test]
    fn study_download_markdown_renders_result_table() {
        let markdown = study_download_markdown(&StudyDownloadResult {
            study_id: "msk_impact_2017".to_string(),
            path: "/tmp/studies/msk_impact_2017".to_string(),
            downloaded: true,
        });

        assert!(markdown.contains("# Study Download: msk_impact_2017"));
        assert!(markdown.contains("| Study ID | msk_impact_2017 |"));
        assert!(markdown.contains("| Downloaded | yes |"));
    }

    #[test]
    fn study_download_catalog_markdown_renders_remote_ids() {
        let markdown = study_download_catalog_markdown(&StudyDownloadCatalog {
            study_ids: vec![
                "msk_impact_2017".to_string(),
                "brca_tcga_pan_can_atlas_2018".to_string(),
            ],
        });

        assert!(markdown.contains("# Downloadable cBioPortal Studies"));
        assert!(markdown.contains("| Study ID |"));
        assert!(markdown.contains("| msk_impact_2017 |"));
        assert!(markdown.contains("| brca_tcga_pan_can_atlas_2018 |"));
    }

    #[test]
    fn study_compare_expression_markdown_renders_distribution_table() {
        let markdown = study_compare_expression_markdown(&StudyExpressionComparisonResult {
            study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
            stratify_gene: "TP53".to_string(),
            target_gene: "ERBB2".to_string(),
            groups: vec![
                StudyExpressionGroupStats {
                    group_name: "TP53-mutant".to_string(),
                    sample_count: 345,
                    mean: 0.234,
                    median: 0.112,
                    min: -2.1,
                    max: 4.5,
                    q1: -0.45,
                    q3: 0.78,
                },
                StudyExpressionGroupStats {
                    group_name: "TP53-wildtype".to_string(),
                    sample_count: 730,
                    mean: -0.089,
                    median: -0.156,
                    min: -3.2,
                    max: 5.1,
                    q1: -0.67,
                    q3: 0.34,
                },
            ],
            mann_whitney_u: Some(9821.0),
            mann_whitney_p: Some(0.003),
        });

        assert!(markdown.contains("# Study Group Comparison: Expression"));
        assert!(markdown.contains(
            "Stratify gene: TP53 | Target gene: ERBB2 | Study: brca_tcga_pan_can_atlas_2018"
        ));
        assert!(markdown.contains("| Group | N | Mean | Median | Q1 | Q3 | Min | Max |"));
        assert!(markdown.contains("Mann-Whitney U: 9821.000"));
        assert!(markdown.contains("Mann-Whitney p-value: 0.003"));
        assert!(markdown.contains(
            "| TP53-wildtype | 730 | -0.089 | -0.156 | -0.670 | 0.340 | -3.200 | 5.100 |"
        ));
    }

    #[test]
    fn study_compare_mutations_markdown_renders_rate_table() {
        let markdown = study_compare_mutations_markdown(&StudyMutationComparisonResult {
            study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
            stratify_gene: "TP53".to_string(),
            target_gene: "PIK3CA".to_string(),
            groups: vec![
                StudyMutationGroupStats {
                    group_name: "TP53-mutant".to_string(),
                    sample_count: 348,
                    mutated_count: 120,
                    mutation_rate: 0.344828,
                },
                StudyMutationGroupStats {
                    group_name: "TP53-wildtype".to_string(),
                    sample_count: 736,
                    mutated_count: 220,
                    mutation_rate: 0.298913,
                },
            ],
        });

        assert!(markdown.contains("# Study Group Comparison: Mutation Rate"));
        assert!(markdown.contains(
            "Stratify gene: TP53 | Target gene: PIK3CA | Study: brca_tcga_pan_can_atlas_2018"
        ));
        assert!(markdown.contains("| Group | N | Mutated | Mutation Rate |"));
        assert!(markdown.contains("| TP53-mutant | 348 | 120 | 0.344828 |"));
    }
}
