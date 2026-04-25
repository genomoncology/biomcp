//! Evidence-link helpers and entity-specific evidence URL builders for markdown outputs.

use std::collections::HashSet;

use super::*;

pub(super) fn source_matches(source: Option<&str>, needle: &str) -> bool {
    source
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some_and(|value| value.to_ascii_lowercase().contains(needle))
}

pub(super) fn orphanet_disease_url(disease: &Disease) -> Option<String> {
    disease
        .xrefs
        .get("Orphanet")
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("https://www.orpha.net/en/disease/detail/{value}"))
}

pub(super) fn omim_disease_url(disease: &Disease) -> Option<String> {
    disease
        .xrefs
        .get("OMIM")
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("https://www.omim.org/entry/{value}"))
}

pub(super) fn mgi_model_url(model_id: Option<&str>) -> Option<String> {
    let model_id = model_id
        .map(str::trim)
        .filter(|value| value.starts_with("MGI:"))?;
    Some(format!(
        "https://www.informatics.jax.org/accession/{model_id}"
    ))
}

pub(super) fn disease_source_url(
    disease: &Disease,
    source: Option<&str>,
    model_id: Option<&str>,
) -> Option<String> {
    if source_matches(source, "orphanet") {
        return orphanet_disease_url(disease);
    }
    if source_matches(source, "omim") {
        return omim_disease_url(disease);
    }
    if source_matches(source, "mgi") {
        return mgi_model_url(model_id);
    }
    None
}

pub(super) fn openfda_count_query_url(
    query: &str,
    count_field: &str,
    limit: usize,
) -> Option<String> {
    let mut url = reqwest::Url::parse("https://api.fda.gov/drug/event.json").ok()?;
    url.query_pairs_mut()
        .append_pair("search", query)
        .append_pair("count", count_field)
        .append_pair("limit", &limit.to_string());
    Some(url.into())
}

pub(super) fn dailymed_setid_url(set_id: &str) -> Option<String> {
    let set_id = set_id.trim();
    if set_id.is_empty() {
        return None;
    }
    let mut url = reqwest::Url::parse("https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm").ok()?;
    url.query_pairs_mut().append_pair("setid", set_id);
    Some(url.into())
}

pub(super) fn dailymed_search_url(name: &str) -> Option<String> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let mut url = reqwest::Url::parse("https://dailymed.nlm.nih.gov/dailymed/search.cfm").ok()?;
    url.query_pairs_mut().append_pair("query", name);
    Some(url.into())
}

pub(super) fn append_evidence_urls<L>(mut body: String, urls: Vec<(L, String)>) -> String
where
    L: AsRef<str>,
{
    let links = urls
        .into_iter()
        .filter_map(|(label, url)| {
            let label = label.as_ref().trim();
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

pub(super) fn gene_evidence_urls(gene: &Gene) -> Vec<(&'static str, String)> {
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

pub(super) fn variant_evidence_urls(variant: &Variant) -> Vec<(&'static str, String)> {
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
    if (variant.gnomad_af.is_some() || variant.population_breakdown.is_some())
        && let Some(variant_id) = variant
            .rsid
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| gnomad_variant_slug(&variant.id))
    {
        urls.push((
            "gnomAD",
            format!("https://gnomad.broadinstitute.org/variant/{variant_id}"),
        ));
    }
    urls
}

pub(super) fn discover_evidence_urls(result: &DiscoverResult) -> Vec<(&'static str, String)> {
    let mut urls = Vec::new();
    if let Ok(mut url) = reqwest::Url::parse("https://www.ebi.ac.uk/ols4/api/search") {
        url.query_pairs_mut()
            .append_pair("q", result.query.trim())
            .append_pair("rows", "10")
            .append_pair("groupField", "iri");
        urls.push(("OLS4", url.into()));
    }
    if let Some(topic) = result.plain_language.as_ref() {
        urls.push(("MedlinePlus", topic.url.clone()));
    }
    urls
}

pub(super) fn article_evidence_urls(article: &Article) -> Vec<(&'static str, String)> {
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

pub(super) fn trial_evidence_urls(trial: &Trial) -> Vec<(&'static str, String)> {
    if trial.nct_id.trim().is_empty() {
        return Vec::new();
    }
    vec![(
        "ClinicalTrials.gov",
        format!("https://clinicaltrials.gov/study/{}", trial.nct_id.trim()),
    )]
}

pub(super) fn disease_evidence_urls(disease: &Disease) -> Vec<(String, String)> {
    let mut urls = Vec::new();
    if !disease.id.trim().is_empty() {
        urls.push((
            "Monarch".to_string(),
            format!("https://monarchinitiative.org/{}", disease.id.trim()),
        ));
    }
    if disease
        .gene_associations
        .iter()
        .any(|row| source_matches(row.source.as_deref(), "orphanet"))
        && let Some(url) = orphanet_disease_url(disease)
    {
        urls.push(("Orphanet".to_string(), url));
    }
    let has_omim_source = disease
        .gene_associations
        .iter()
        .any(|row| source_matches(row.source.as_deref(), "omim"))
        || disease
            .phenotypes
            .iter()
            .any(|row| source_matches(row.source.as_deref(), "omim"));
    if has_omim_source && let Some(url) = omim_disease_url(disease) {
        urls.push(("OMIM".to_string(), url));
    }
    if let Some(url) = disease
        .models
        .iter()
        .find(|row| source_matches(row.source.as_deref(), "mgi"))
        .and_then(|row| {
            mgi_model_url(row.model_id.as_deref()).or_else(|| mgi_model_url(Some(&row.model)))
        })
    {
        urls.push(("MGI".to_string(), url));
    }
    let mut seen_clinical_urls = HashSet::new();
    for row in &disease.clinical_features {
        let Some(url) = row
            .source_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        if !seen_clinical_urls.insert(url.to_string()) {
            continue;
        }
        let label = row
            .topic_title
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|title| format!("MedlinePlus: {title}"))
            .unwrap_or_else(|| "MedlinePlus".to_string());
        urls.push((label, url.to_string()));
    }
    urls
}

pub(super) fn drug_evidence_urls(drug: &Drug) -> Vec<(&'static str, String)> {
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
    if !drug.top_adverse_events.is_empty()
        && let Some(query) = drug
            .faers_query
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        && let Some(url) =
            openfda_count_query_url(query, "patient.reaction.reactionmeddrapt.exact", 50)
    {
        urls.push(("OpenFDA FAERS", url));
    }
    if drug.label.is_some()
        && let Some(url) = drug
            .label_set_id
            .as_deref()
            .and_then(dailymed_setid_url)
            .or_else(|| dailymed_search_url(&drug.name))
    {
        urls.push(("DailyMed", url));
    }
    urls
}

pub(super) fn drug_interaction_report_evidence_urls(
    report: &crate::entities::drug::DrugInteractionReport,
) -> Vec<(String, String)> {
    let mut urls = Vec::new();
    if let Some(drugbank_id) = report
        .drugbank_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        urls.push((
            "DrugBank".to_string(),
            format!("https://go.drugbank.com/drugs/{drugbank_id}"),
        ));
    }
    if let Some(chembl_id) = report
        .chembl_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        urls.push((
            "ChEMBL".to_string(),
            format!("https://www.ebi.ac.uk/chembl/compound_report_card/{chembl_id}"),
        ));
    }
    urls.push((
        "DDInter download bundle".to_string(),
        "https://ddinter.scbdd.com/download/".to_string(),
    ));
    urls.push((
        "DDInter terms".to_string(),
        "https://ddinter.scbdd.com/terms/".to_string(),
    ));
    urls
}

pub(super) fn pathway_evidence_urls(pathway: &Pathway) -> Vec<(&'static str, String)> {
    let id = pathway.id.trim();
    if id.is_empty() {
        return Vec::new();
    }
    if pathway.source.eq_ignore_ascii_case("KEGG") {
        return vec![("KEGG", format!("https://www.kegg.jp/entry/{id}"))];
    }
    if pathway.source.eq_ignore_ascii_case("WikiPathways") {
        return vec![(
            "WikiPathways",
            format!("https://www.wikipathways.org/pathways/{id}.html"),
        )];
    }
    vec![(
        "Reactome",
        format!("https://reactome.org/content/detail/{id}"),
    )]
}

pub(super) fn protein_evidence_urls(protein: &Protein) -> Vec<(&'static str, String)> {
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

pub(super) fn adverse_event_evidence_urls(event: &AdverseEvent) -> Vec<(&'static str, String)> {
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

pub(super) fn device_event_evidence_urls(event: &DeviceEvent) -> Vec<(&'static str, String)> {
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

pub(super) fn pgx_evidence_urls(pgx: &Pgx) -> Vec<(&'static str, String)> {
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

#[cfg(test)]
mod tests;
