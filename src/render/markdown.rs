use std::collections::HashSet;
use std::sync::OnceLock;

use minijinja::{Environment, context};

use crate::entities::adverse_event::{
    AdverseEvent, AdverseEventCountBucket, AdverseEventSearchResult, AdverseEventSearchSummary,
    DeviceEvent, DeviceEventSearchResult, RecallSearchResult,
};
use crate::entities::article::{Article, ArticleAnnotations, ArticleSearchResult};
use crate::entities::disease::{Disease, DiseaseSearchResult, PhenotypeSearchResult};
use crate::entities::drug::{Drug, DrugSearchResult};
use crate::entities::gene::{Gene, GeneSearchResult};
use crate::entities::pathway::{Pathway, PathwaySearchResult};
use crate::entities::pgx::{Pgx, PgxSearchResult};
use crate::entities::protein::{Protein, ProteinSearchResult};
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

    let _ = ENV.set(env);
    Ok(ENV
        .get()
        .expect("ENV should be initialized by the time this is reached"))
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

fn gene_evidence_urls(gene: &Gene) -> Vec<(&'static str, String)> {
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
    urls
}

fn variant_evidence_urls(variant: &Variant) -> Vec<(&'static str, String)> {
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
    urls
}

fn article_evidence_urls(article: &Article) -> Vec<(&'static str, String)> {
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

fn trial_evidence_urls(trial: &Trial) -> Vec<(&'static str, String)> {
    if trial.nct_id.trim().is_empty() {
        return Vec::new();
    }
    vec![(
        "ClinicalTrials.gov",
        format!("https://clinicaltrials.gov/study/{}", trial.nct_id.trim()),
    )]
}

fn disease_evidence_urls(disease: &Disease) -> Vec<(&'static str, String)> {
    if disease.id.trim().is_empty() {
        return Vec::new();
    }
    vec![(
        "Monarch",
        format!("https://monarchinitiative.org/{}", disease.id.trim()),
    )]
}

fn drug_evidence_urls(drug: &Drug) -> Vec<(&'static str, String)> {
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
    urls
}

fn pathway_evidence_urls(pathway: &Pathway) -> Vec<(&'static str, String)> {
    if pathway.id.trim().is_empty() {
        return Vec::new();
    }
    vec![(
        "Reactome",
        format!("https://reactome.org/content/detail/{}", pathway.id.trim()),
    )]
}

fn protein_evidence_urls(protein: &Protein) -> Vec<(&'static str, String)> {
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

fn adverse_event_evidence_urls(event: &AdverseEvent) -> Vec<(&'static str, String)> {
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

fn device_event_evidence_urls(event: &DeviceEvent) -> Vec<(&'static str, String)> {
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

fn format_sections(sections: Vec<String>) -> String {
    sections.join(", ")
}

fn format_related(related: Vec<String>) -> String {
    related
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join(" | ")
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
    sections_for(requested, crate::entities::pathway::PATHWAY_SECTION_NAMES)
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

fn related_gene(gene: &Gene) -> Vec<String> {
    let symbol = gene.symbol.trim();
    if symbol.is_empty() {
        return Vec::new();
    }
    vec![
        format!("variants -g {symbol}"),
        format!("articles -g {symbol}"),
        format!("drugs --target {symbol}"),
        format!("trials {symbol}"),
    ]
}

fn related_variant(variant: &Variant) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if !variant.gene.trim().is_empty() {
        let gene = variant.gene.trim();
        out.push(format!("gene {gene}"));
        out.push(format!("drugs --target {gene}"));
    }
    if !variant.id.trim().is_empty() {
        out.push("trials".to_string());
        out.push("articles".to_string());
        let has_oncokb_token = std::env::var("ONCOKB_TOKEN")
            .ok()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        if has_oncokb_token {
            out.push(format!("oncokb {}", quote_arg(&variant.id)));
        }
    }
    out
}

fn related_article(article: &Article) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(ann) = article.annotations.as_ref() {
        for g in &ann.genes {
            let sym = g.text.trim();
            if sym.is_empty() {
                continue;
            }
            out.push(format!("gene {sym}"));
        }
        for d in &ann.diseases {
            let name = quote_arg(&d.text);
            if name.is_empty() {
                continue;
            }
            out.push(format!("disease {name}"));
        }
        for c in &ann.chemicals {
            let name = quote_arg(&c.text);
            if name.is_empty() {
                continue;
            }
            out.push(format!("drug {name}"));
        }
    }
    if let Some(pmid) = article
        .pmid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push(format!("article entities {pmid}"));
    }
    out
}

fn related_trial(trial: &Trial) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    if let Some(condition) = trial.conditions.first().map(String::as_str) {
        let cond = quote_arg(condition);
        if !cond.is_empty() {
            out.push(format!("disease {cond}"));
            out.push(format!("articles -d {cond}"));
            out.push(format!("trials -c {cond}"));
        }
    }

    if let Some(intervention) = trial.interventions.first().map(String::as_str) {
        let name = quote_arg(intervention);
        if !name.is_empty() {
            out.push(format!("drug {name}"));
            out.push(format!("drug trials {name}"));
        }
    }

    out
}

fn related_disease(disease: &Disease) -> Vec<String> {
    let name = quote_arg(&disease.name);
    if name.is_empty() {
        return Vec::new();
    }
    vec![
        format!("trials {name}"),
        format!("articles {name}"),
        format!("drugs {name}"),
    ]
}

fn related_pgx(pgx: &Pgx) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(gene) = pgx.gene.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        out.push(format!("pgx -g {gene}"));
    }
    if let Some(drug) = pgx.drug.as_deref().map(quote_arg).filter(|v| !v.is_empty()) {
        out.push(format!("pgx -d {drug}"));
    }
    out
}

fn related_pathway(pathway: &Pathway) -> Vec<String> {
    let id = quote_arg(&pathway.id);
    if id.is_empty() {
        return Vec::new();
    }

    vec![format!("drugs {id}")]
}

fn related_protein(protein: &Protein) -> Vec<String> {
    let accession = quote_arg(&protein.accession);
    let mut out = Vec::new();
    if !accession.is_empty() {
        out.push(format!("structures {accession}"));
    }
    if let Some(symbol) = protein
        .gene_symbol
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        out.push(format!("gene {symbol}"));
    }
    out
}

fn related_drug(drug: &Drug) -> Vec<String> {
    let name = quote_arg(&drug.name);
    if name.is_empty() {
        return Vec::new();
    }

    let mut out = vec![format!("trials {name}"), format!("adverse-events {name}")];

    if let Some(target) = drug.targets.first().map(String::as_str) {
        let sym = target.trim();
        if !sym.is_empty() {
            out.push(format!("gene {sym}"));
        }
    }

    out
}

fn related_adverse_event(event: &AdverseEvent) -> Vec<String> {
    let drug = quote_arg(&event.drug);
    if drug.is_empty() {
        return Vec::new();
    }
    vec![
        format!("drug {drug}"),
        format!("drug adverse-events {drug}"),
        format!("drug trials {drug}"),
    ]
}

fn related_device_event(event: &DeviceEvent) -> Vec<String> {
    let device = quote_arg(&event.device);
    if device.is_empty() {
        return Vec::new();
    }
    vec![format!("reports {device}"), "recalls class-i".to_string()]
}

pub fn gene_markdown(gene: &Gene, requested_sections: &[String]) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("gene.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let show_civic_section =
        include_all || requested.iter().any(|s| s.eq_ignore_ascii_case("civic"));
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
        show_civic_section => show_civic_section,
        sections => format_sections(sections_gene(gene, requested_sections)),
        related => format_related(related_gene(gene)),
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
        pubtator_fallback => article.pubtator_fallback,
        show_annotations_section => show_annotations_section,
        show_fulltext_section => show_fulltext_section,
        sections => format_sections(sections_article(article, requested_sections)),
        related => format_related(related_article(article)),
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
    let tmpl = env()?.get_template("article_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
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
    let disease_label = if disease.name.trim().is_empty() {
        disease.id.as_str()
    } else {
        disease.name.as_str()
    };

    let tmpl = env()?.get_template("disease.md.j2")?;
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
        top_genes => &disease.top_genes,
        treatment_landscape => &disease.treatment_landscape,
        recruiting_trial_count => &disease.recruiting_trial_count,
        pathways => &disease.pathways,
        phenotypes => &disease.phenotypes,
        variants => &disease.variants,
        models => &disease.models,
        prevalence => &disease.prevalence,
        prevalence_note => &disease.prevalence_note,
        civic => &disease.civic,
        show_genes_section => show_genes_section,
        show_pathways_section => show_pathways_section,
        show_phenotypes_section => show_phenotypes_section,
        show_variants_section => show_variants_section,
        show_models_section => show_models_section,
        show_prevalence_section => show_prevalence_section,
        show_civic_section => show_civic_section,
        xrefs => xrefs,
        sections => format_sections(sections_disease(disease, requested_sections)),
        related => format_related(related_disease(disease)),
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
        sections => format_sections(sections_pgx(pgx, requested_sections)),
        related => format_related(related_pgx(pgx)),
    })?;
    Ok(body)
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
        sections => format_sections(sections_trial(trial, requested_sections)),
        related => format_related(related_trial(trial)),
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
        gnomad_subpopulations => &variant.gnomad_subpopulations,
        population => &variant.population,
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
        sections => format_sections(sections_variant(variant, requested_sections)),
        related => format_related(related_variant(variant)),
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
        sections => format_sections(sections_drug(drug, requested_sections)),
        related => format_related(related_drug(drug)),
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
        sections => format_sections(sections_pathway(pathway, requested_sections)),
        related => format_related(related_pathway(pathway)),
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
    let show_structures_section = !section_only || include_all || has_requested("structures");
    let protein_label = if protein.name.trim().is_empty() {
        protein.accession.as_str()
    } else {
        protein.name.as_str()
    };
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
        show_domains_section => show_domains_section,
        show_interactions_section => show_interactions_section,
        show_structures_section => show_structures_section,
        sections => format_sections(sections_protein(protein, requested_sections)),
        related => format_related(related_protein(protein)),
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
        sections => format_sections(sections_adverse_event(event, requested_sections)),
        related => format_related(related_adverse_event(event)),
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
        related => format_related(related_device_event(event)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::gene::Gene;
    use crate::entities::variant::{TreatmentImplication, Variant, VariantOncoKbResult};

    #[test]
    fn quote_arg_wraps_whitespace_and_escapes_quotes() {
        assert_eq!(quote_arg("BRAF"), "BRAF");
        assert_eq!(quote_arg("BRAF V600E"), "\"BRAF V600E\"");
        assert_eq!(quote_arg("BRAF \"V600E\""), "\"BRAF \\\"V600E\\\"\"");
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
        };

        let markdown = gene_markdown(&gene, &[]).expect("rendered markdown");
        assert!(markdown.contains("BRAF"));
        assert!(markdown.contains("[NCBI Gene](https://www.ncbi.nlm.nih.gov/gene/673)"));
        assert!(markdown.contains("[UniProt](https://www.uniprot.org/uniprot/P15056)"));
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
}
