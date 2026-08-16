use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::entities::variant::VariantProteinAlias;
use crate::error::BioMcpError;

mod filter_conflicts;
mod filter_keys;
use crate::sources::{RequestPlan, is_valid_gene_symbol, request_from_plan};
use crate::utils::serde::StringOrVec;
use filter_conflicts::validate_missing_filter_conflicts;
use filter_keys::normalize_filter_key;

const MYVARIANT_BASE: &str = "https://myvariant.info/v1";
const MYVARIANT_API: &str = "myvariant.info";
const MYVARIANT_BASE_ENV: &str = "BIOMCP_MYVARIANT_BASE";

pub(crate) const MYVARIANT_FIELDS_GET: &str = concat!(
    "_id,cadd.phred,cadd.consequence,",
    "clinvar.gene.symbol,clinvar.rcv.clinical_significance,clinvar.rcv.review_status,clinvar.rcv.conditions,clinvar.rcv.preferred_name,clinvar.variant_id,",
    "snpeff.ann.feature_id,snpeff.ann.genename,snpeff.ann.hgvs_c,snpeff.ann.hgvs_p,",
    "dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.hgvsc,",
    "dbnsfp.sift.pred,dbnsfp.sift.score,",
    "dbnsfp.polyphen2.hdiv.pred,",
    "dbnsfp.revel.score,dbnsfp.revel.rankscore,",
    "dbnsfp.alphamissense.score,dbnsfp.alphamissense.pred,dbnsfp.alphamissense.rankscore,",
    "dbnsfp.clinpred.score,dbnsfp.clinpred.pred,",
    "dbnsfp.metarnn.score,dbnsfp.metarnn.pred,",
    "dbnsfp.bayesdel.add_af.score,dbnsfp.bayesdel.add_af.pred,",
    "dbnsfp.bayesdel.no_af.score,dbnsfp.bayesdel.no_af.pred,",
    "dbnsfp.phylop.100way_vertebrate.rankscore,dbnsfp.phylop.470way_mammalian.rankscore,",
    "dbnsfp.phastcons.100way_vertebrate.rankscore,dbnsfp.phastcons.470way_mammalian.rankscore,",
    "dbnsfp.gerp++.rs,",
    "dbsnp.rsid,",
    "gnomad_exome.af.af,gnomad_exome.af.af_afr,gnomad_exome.af.af_eas,gnomad_exome.af.af_nfe,gnomad_exome.af.af_sas,",
    "gnomad_exome.af.af_amr,gnomad_exome.af.af_asj,gnomad_exome.af.af_fin,",
    "gnomad_exome.af.af_afr_female,gnomad_exome.af.af_afr_male,",
    "gnomad_exome.af.af_amr_female,gnomad_exome.af.af_amr_male,",
    "gnomad_exome.af.af_eas_jpn,gnomad_exome.af.af_eas_kor,",
    "gnomad_exome.af.af_nfe_bgr,gnomad_exome.af.af_nfe_est,gnomad_exome.af.af_nfe_nwe,",
    "gnomad_exome.af.af_nfe_onf,gnomad_exome.af.af_nfe_seu,gnomad_exome.af.af_nfe_swe,",
    "gnomad_exome.af.af_oth,",
    "gnomad.exomes.af.af,gnomad.exomes.af.af_afr,gnomad.exomes.af.af_eas,gnomad.exomes.af.af_nfe,",
    "gnomad.exomes.af.af_sas,gnomad.exomes.af.af_amr,gnomad.exomes.af.af_asj,gnomad.exomes.af.af_fin,",
    "gnomad.genomes.af.af,gnomad.genomes.af.af_afr,gnomad.genomes.af.af_eas,gnomad.genomes.af.af_nfe,",
    "gnomad.genomes.af.af_sas,gnomad.genomes.af.af_amr,gnomad.genomes.af.af_asj,gnomad.genomes.af.af_fin,",
    "exac.af,exac_nontcga.af,",
    "cosmic.cosmic_id,cosmic.mut_freq,cosmic.tumor_site,cosmic.mut_nt,",
    "cgi,civic"
);
pub(crate) const MYVARIANT_FIELDS_SEARCH: &str = "_id,dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.hgvsc,dbnsfp.revel.score,dbnsfp.gerp*.rs,clinvar.gene.symbol,clinvar.rcv.clinical_significance,clinvar.rcv.review_status,clinvar.rcv.preferred_name,clinvar.variant_id,snpeff.ann.feature_id,snpeff.ann.genename,snpeff.ann.hgvs_c,snpeff.ann.hgvs_p,dbsnp.rsid,gnomad_exome.af.af,gnomad.exomes.af.af,gnomad.genomes.af.af,cadd.consequence";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

fn de_vec_or_single<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let value = Option::<OneOrMany<T>>::deserialize(deserializer)?;
    Ok(match value {
        Some(OneOrMany::One(v)) => vec![v],
        Some(OneOrMany::Many(v)) => v,
        None => Vec::new(),
    })
}

pub struct MyVariantClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

pub(crate) fn civic_pubmed_ids(hit: &MyVariantHit) -> Vec<String> {
    let Some(profiles) = hit
        .civic
        .as_ref()
        .and_then(|value| value.get("molecularProfiles"))
        .and_then(serde_json::Value::as_array)
    else {
        return Vec::new();
    };
    let mut pmids = std::collections::BTreeSet::new();
    for source in profiles
        .iter()
        .filter_map(|profile| profile.get("evidenceItems")?.as_array())
        .flatten()
        .filter_map(|item| item.get("source"))
    {
        let is_pubmed = source
            .get("sourceType")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|kind| kind.trim().eq_ignore_ascii_case("PUBMED"));
        if !is_pubmed {
            continue;
        }
        let citation = match source.get("citation") {
            Some(serde_json::Value::String(value)) => value.trim(),
            Some(serde_json::Value::Number(value)) => value.as_u64().map_or("", |_| "numeric"),
            _ => "",
        };
        let parsed = if citation == "numeric" {
            source.get("citation").and_then(serde_json::Value::as_u64)
        } else {
            let digits = citation
                .split_once(':')
                .filter(|(prefix, _)| prefix.trim().eq_ignore_ascii_case("PMID"))
                .map(|(_, digits)| digits.trim())
                .unwrap_or(citation);
            (!digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()))
                .then(|| digits.parse::<u64>().ok())
                .flatten()
        };
        if let Some(pmid) = parsed
            .filter(|pmid| *pmid > 0)
            .and_then(|pmid| u32::try_from(pmid).ok())
        {
            pmids.insert(pmid.to_string());
        }
    }
    pmids.into_iter().collect()
}

pub struct VariantSearchParams {
    pub gene: Option<String>,
    pub hgvsp: Option<String>,
    pub hgvsc: Option<String>,
    pub rsid: Option<String>,
    pub protein_alias: Option<VariantProteinAlias>,
    pub significance: Option<String>,
    pub max_frequency: Option<f64>,
    pub min_cadd: Option<f64>,
    pub consequence: Option<String>,
    pub review_status: Option<String>,
    pub population: Option<String>,
    pub revel_min: Option<f64>,
    pub gerp_min: Option<f64>,
    pub tumor_site: Option<String>,
    pub condition: Option<String>,
    pub impact: Option<String>,
    pub lof: bool,
    pub has: Option<String>,
    pub missing: Option<String>,
    pub therapy: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

const SIGNIFICANCE_VALUES: &[&str] = &[
    "pathogenic",
    "likely_pathogenic",
    "benign",
    "likely_benign",
    "uncertain_significance",
    "conflicting_interpretations_of_pathogenicity",
    "drug_response",
    "risk_factor",
    "association",
    "protective",
    "affects",
    "not_provided",
];

const CONSEQUENCE_VALUES: &[&str] = &[
    "missense_variant",
    "synonymous_variant",
    "frameshift_variant",
    "nonsense_variant",
    "stop_gained",
    "stop_lost",
    "start_lost",
    "splice_acceptor_variant",
    "splice_donor_variant",
    "inframe_insertion",
    "inframe_deletion",
    "intron_variant",
    "upstream_gene_variant",
    "downstream_gene_variant",
    "non_coding_transcript_variant",
];

const REVIEW_STATUS_VALUES: &[&str] = &[
    "0",
    "0_star",
    "0_stars",
    "none",
    "1",
    "1_star",
    "1_stars",
    "2",
    "2_star",
    "2_stars",
    "3",
    "3_star",
    "3_stars",
    "expert_panel",
    "4",
    "4_star",
    "4_stars",
    "criteria_provided",
];

const FIELD_VALUES: &[&str] = &[
    "cadd", "revel", "gerp", "clinvar", "gnomad", "dbsnp", "snpeff", "civic", "cosmic",
];

const POPULATION_VALUES: &[&str] = &["afr", "amr", "eas", "fin", "nfe", "sas", "asj", "oth"];

const IMPACT_VALUES: &[&str] = &["HIGH", "MODERATE", "LOW", "MODIFIER"];

fn invalid_filter_error(flag: &str, raw: &str, accepted: &[&str]) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Invalid {flag} value '{raw}'. Expected one of: {}",
        accepted.join(", ")
    ))
}

pub(crate) fn normalize_significance_filter(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--significance must not be empty".into(),
        ));
    }
    let key = normalize_filter_key(raw);
    let canonical = match key.as_str() {
        "pathogenic" => "pathogenic",
        "likely_pathogenic" | "likelypathogenic" => "likely_pathogenic",
        "benign" => "benign",
        "likely_benign" | "likelybenign" => "likely_benign",
        "uncertain_significance" | "uncertain" | "vus" => "uncertain_significance",
        "conflicting_interpretations_of_pathogenicity"
        | "conflicting_interpretation_of_pathogenicity"
        | "conflicting_pathogenicity"
        | "conflicting" => "conflicting_interpretations_of_pathogenicity",
        "drug_response" => "drug_response",
        "risk_factor" => "risk_factor",
        "association" => "association",
        "protective" => "protective",
        "affects" => "affects",
        "not_provided" => "not_provided",
        _ => {
            return Err(invalid_filter_error(
                "--significance",
                raw,
                SIGNIFICANCE_VALUES,
            ));
        }
    };
    Ok(canonical.to_string())
}

pub(crate) fn normalize_consequence_filter(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--consequence must not be empty".into(),
        ));
    }
    let key = normalize_filter_key(raw);
    let mut canonical = match key.as_str() {
        "nonsynonymous" | "non_synonymous" | "non_synonymous_variant" => {
            "missense_variant".to_string()
        }
        "nonsense_variant" => "stop_gained".to_string(),
        "splice_acceptor" => "splice_acceptor_variant".to_string(),
        "splice_donor" => "splice_donor_variant".to_string(),
        "noncoding" | "non_coding" => "non_coding_transcript_variant".to_string(),
        _ => key,
    };
    if !CONSEQUENCE_VALUES.contains(&canonical.as_str()) && !canonical.ends_with("_variant") {
        let expanded = format!("{canonical}_variant");
        if CONSEQUENCE_VALUES.contains(&expanded.as_str()) {
            canonical = expanded;
        }
    }
    if !CONSEQUENCE_VALUES.contains(&canonical.as_str()) {
        return Err(invalid_filter_error(
            "--consequence",
            raw,
            CONSEQUENCE_VALUES,
        ));
    }
    Ok(canonical)
}

pub(crate) fn normalize_population_filter(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--population must not be empty".into(),
        ));
    }
    let normalized = raw.to_ascii_lowercase();
    if !POPULATION_VALUES.contains(&normalized.as_str()) {
        return Err(invalid_filter_error("--population", raw, POPULATION_VALUES));
    }
    Ok(normalized)
}

pub(crate) fn normalize_impact_filter(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--impact must not be empty".into(),
        ));
    }
    let normalized = raw.to_ascii_uppercase();
    if !IMPACT_VALUES.contains(&normalized.as_str()) {
        return Err(invalid_filter_error("--impact", raw, IMPACT_VALUES));
    }
    Ok(normalized)
}

pub(crate) fn normalize_review_status_filter(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--review-status must not be empty".into(),
        ));
    }
    let key = normalize_filter_key(raw);
    let normalized = match key.as_str() {
        "0" | "0_star" | "0_stars" | "none" => "no assertion criteria provided",
        "1" | "1_star" | "1_stars" => "criteria provided, single submitter",
        "2" | "2_star" | "2_stars" => "criteria provided, multiple submitters, no conflicts",
        "3" | "3_star" | "3_stars" | "expert_panel" => "reviewed by expert panel",
        "4" | "4_star" | "4_stars" => "practice guideline",
        "criteria_provided" => "criteria provided",
        _ => {
            return Err(invalid_filter_error(
                "--review-status",
                raw,
                REVIEW_STATUS_VALUES,
            ));
        }
    };
    Ok(normalized.to_string())
}

fn field_presence_expression(flag: &str, value: &str) -> Result<&'static str, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(format!(
            "{flag} must not be empty"
        )));
    }
    match normalize_filter_key(raw).as_str() {
        "cadd" => Ok("_exists_:cadd"),
        "revel" => Ok("_exists_:dbnsfp.revel.score"),
        "gerp" => Ok("_exists_:dbnsfp.gerp++.rs"),
        "clinvar" => Ok("_exists_:clinvar"),
        "gnomad" => {
            Ok("(_exists_:gnomad_exome OR _exists_:gnomad.exomes OR _exists_:gnomad.genomes)")
        }
        "dbsnp" => Ok("_exists_:dbsnp.rsid"),
        "snpeff" => Ok("_exists_:snpeff.ann.effect"),
        "civic" => Ok("_exists_:civic"),
        "cosmic" => Ok("_exists_:cosmic"),
        _ => Err(invalid_filter_error(flag, raw, FIELD_VALUES)),
    }
}

impl MyVariantClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MYVARIANT_BASE, MYVARIANT_BASE_ENV),
        })
    }

    pub(crate) fn escape_query_value(value: &str) -> String {
        crate::utils::query::escape_lucene_value(value)
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::MYVARIANT,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::MYVARIANT),
        )
        .await?;
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
            status,
            content_type.as_ref(),
            &bytes,
            true,
        )
    }

    /// Build the outbound free-form `/query` request (pure — Tier-2 testable, never sent).
    pub(crate) fn query_plan(
        q: &str,
        limit: usize,
        offset: usize,
        fields: &str,
    ) -> Result<RequestPlan, BioMcpError> {
        let q = q.trim();
        if q.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search variant -g BRAF".into(),
            ));
        }
        crate::sources::validate_biothings_result_window("MyVariant search", limit, offset)?;

        Ok(RequestPlan::get("query")
            .query("q", q)
            .query("size", limit.to_string())
            .query("from", offset.to_string())
            .query("fields", fields))
    }

    pub async fn query_with_fields(
        &self,
        q: &str,
        limit: usize,
        offset: usize,
        fields: &str,
    ) -> Result<MyVariantSearchResponse, BioMcpError> {
        let plan = Self::query_plan(q, limit, offset, fields)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }

    /// Build the outbound filter-driven `/query` request (pure — Tier-2 testable).
    pub(crate) fn search_plan(params: &VariantSearchParams) -> Result<RequestPlan, BioMcpError> {
        crate::sources::validate_biothings_result_window(
            "MyVariant search",
            params.limit,
            params.offset,
        )?;
        validate_missing_filter_conflicts(params)?;

        let mut terms: Vec<String> = Vec::new();
        let gene = params
            .gene
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());

        if let Some(gene) = gene {
            if !is_valid_gene_symbol(gene) {
                return Err(BioMcpError::InvalidArgument(
                    "Gene symbol filter must contain only letters, numbers, '_' or '-'".into(),
                ));
            }
            terms.push(format!(
                "dbnsfp.genename:{}",
                Self::escape_query_value(gene)
            ));
        }

        if let Some(alias) = params.protein_alias.as_ref() {
            if gene.is_none() {
                return Err(BioMcpError::InvalidArgument(
                    "Residue alias search requires a gene symbol. Example: biomcp search variant -g PTPN22 620W".into(),
                ));
            }
            let trailing_alias = alias.label();
            let leading_alias = format!("{}{}*", alias.residue, alias.position);
            terms.push(format!(
                "(dbnsfp.hgvsp:*{trailing_alias} OR dbnsfp.hgvsp:*{leading_alias})"
            ));
        }

        if let Some(hgvsp) = params
            .hgvsp
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            let mut v = hgvsp.to_string();
            if !v.starts_with("p.") && !v.starts_with("P.") {
                v = format!("p.{v}");
            }
            terms.push(format!("dbnsfp.hgvsp:\"{}\"", Self::escape_query_value(&v)));
        }

        if let Some(hgvsc) = params
            .hgvsc
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            let value = if hgvsc.starts_with("c.") || hgvsc.starts_with("C.") {
                hgvsc.to_string()
            } else {
                format!("c.{hgvsc}")
            };
            terms.push(format!(
                "dbnsfp.hgvsc:\"{}\"",
                Self::escape_query_value(&value)
            ));
        }

        if let Some(rsid) = params
            .rsid
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            let normalized = rsid.to_ascii_lowercase();
            terms.push(format!(
                "dbsnp.rsid:\"{}\"",
                Self::escape_query_value(&normalized)
            ));
        }

        if let Some(sig) = params
            .significance
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            let sig = normalize_significance_filter(sig)?;
            terms.push(format!(
                "clinvar.rcv.clinical_significance:{}",
                Self::escape_query_value(&sig)
            ));
        }

        if let Some(max) = params.max_frequency {
            if !(0.0..=1.0).contains(&max) {
                return Err(BioMcpError::InvalidArgument(format!(
                    "--max-frequency must be between 0 and 1 (got {max})"
                )));
            }
            if let Some(population) = params
                .population
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            {
                let population = normalize_population_filter(population)?;
                terms.push(format!("gnomad_exome.af.af_{population}:[* TO {max}]"));
            } else {
                terms.push(format!("gnomad_exome.af.af:[* TO {max}]"));
            }
        }

        if let Some(min) = params.min_cadd {
            if !min.is_finite() {
                return Err(BioMcpError::InvalidArgument(
                    "--min-cadd must be finite".to_string(),
                ));
            }
            if min < 0.0 {
                return Err(BioMcpError::InvalidArgument(format!(
                    "--min-cadd must be >= 0 (got {min})"
                )));
            }
            terms.push(format!("cadd.phred:[{min} TO *]"));
        }
        if let Some(consequence) = params.consequence.as_deref() {
            let provider_term = normalize_consequence_filter(consequence)?;
            terms.push(format!("snpeff.ann.effect:*{provider_term}*"));
        }

        if let Some(review_status) = params.review_status.as_deref() {
            let provider_phrase = normalize_review_status_filter(review_status)?;
            terms.push(format!(
                "clinvar.rcv.review_status:\"{}\"",
                Self::escape_query_value(&provider_phrase)
            ));
        }

        if let Some(population) = params
            .population
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            let population = normalize_population_filter(population)?;
            terms.push(format!("gnomad_exome.af.af_{population}:*"));
        }

        if let Some(revel_min) = params.revel_min {
            if !(0.0..=1.0).contains(&revel_min) {
                return Err(BioMcpError::InvalidArgument(format!(
                    "--revel-min must be between 0 and 1 (got {revel_min})"
                )));
            }
            terms.push(format!("dbnsfp.revel.score:[{revel_min} TO *]"));
        }

        if let Some(gerp_min) = params.gerp_min {
            if !gerp_min.is_finite() {
                return Err(BioMcpError::InvalidArgument(
                    "--gerp-min must be finite".to_string(),
                ));
            }
            terms.push(format!("dbnsfp.gerp++.rs:[{gerp_min} TO *]"));
        }

        if let Some(tumor_site) = params
            .tumor_site
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            terms.push(format!(
                "cosmic.tumor_site:\"{}\"",
                Self::escape_query_value(tumor_site)
            ));
        }

        if let Some(condition) = params
            .condition
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            terms.push(format!(
                "clinvar.rcv.conditions.name:\"{}\"",
                Self::escape_query_value(condition)
            ));
        }

        if let Some(impact) = params
            .impact
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            let normalized = normalize_impact_filter(impact)?;
            terms.push(format!("snpeff.ann.putative_impact:{normalized}"));
        }

        if params.lof {
            terms.push("snpeff.lof.genename:*".to_string());
        }

        if let Some(has) = params.has.as_deref() {
            terms.push(field_presence_expression("--has", has)?.to_string());
        }

        if let Some(missing) = params.missing.as_deref() {
            let expression = field_presence_expression("--missing", missing)?;
            terms.push(format!("NOT {expression}"));
        }

        if let Some(therapy) = params
            .therapy
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            terms.push(format!(
                "civic.molecularProfiles.evidenceItems.therapies.name:\"{}\"",
                Self::escape_query_value(therapy)
            ));
        }

        if terms.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "At least one filter is required. Example: biomcp search variant -g BRAF".into(),
            ));
        }

        let q = terms.join(" AND ");
        Ok(RequestPlan::get("query")
            .query("q", q)
            .query("size", params.limit.to_string())
            .query("from", params.offset.to_string())
            .query("fields", MYVARIANT_FIELDS_SEARCH))
    }

    pub async fn search(
        &self,
        params: &VariantSearchParams,
    ) -> Result<MyVariantSearchResponse, BioMcpError> {
        let plan = Self::search_plan(params)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }

    /// Build the outbound single-variant lookup request (pure — Tier-2 testable).
    pub(crate) fn get_plan(
        id: &str,
        genome_build: Option<crate::entities::variant::GenomeBuild>,
    ) -> Result<RequestPlan, BioMcpError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Variant ID is required. Example: biomcp get variant rs113488022".into(),
            ));
        }
        if id.len() > 512 {
            return Err(BioMcpError::InvalidArgument(
                "Variant ID is too long.".into(),
            ));
        }

        let plan = RequestPlan::get(format!("variant/{id}")).query("fields", MYVARIANT_FIELDS_GET);
        Ok(match genome_build {
            Some(build) => plan.query("assembly", build.provider_value()),
            None => plan,
        })
    }

    /// Reduce a `/variant/{id}` response to every returned hit value (pure — Tier-3 testable).
    pub(crate) fn select_get_hit_values(
        value: serde_json::Value,
    ) -> Result<Vec<serde_json::Value>, BioMcpError> {
        match value {
            serde_json::Value::Object(_) => Ok(vec![value]),
            serde_json::Value::Array(arr) => Ok(arr),
            _ => Err(BioMcpError::Api {
                api: MYVARIANT_API.to_string(),
                message: "Unexpected response type".into(),
            }),
        }
    }

    /// Reduce a `/variant/{id}` response to a single hit value (pure — Tier-3 testable).
    ///
    /// This legacy API intentionally retains first-hit selection for unaffected callers.
    pub(crate) fn select_get_hit_value(
        value: serde_json::Value,
        id: &str,
    ) -> Result<serde_json::Value, BioMcpError> {
        Self::select_get_hit_values(value)?
            .into_iter()
            .next()
            .ok_or_else(|| BioMcpError::NotFound {
                entity: "variant".into(),
                id: id.to_string(),
                suggestion: format!("Try searching: biomcp search variant -g \"{id}\""),
            })
    }

    pub async fn get(
        &self,
        id: &str,
        genome_build: Option<crate::entities::variant::GenomeBuild>,
    ) -> Result<MyVariantHit, BioMcpError> {
        let id = id.trim();
        let plan = Self::get_plan(id, genome_build)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let value: serde_json::Value = self.get_json(req).await.map_err(|error| {
            if error.is_not_found() {
                BioMcpError::NotFound {
                    entity: "variant".into(),
                    id: id.to_string(),
                    suggestion: "Try searching: biomcp search variant".into(),
                }
            } else {
                error
            }
        })?;
        let hit_value = Self::select_get_hit_value(value, id)?;
        serde_json::from_value(hit_value).map_err(|source| BioMcpError::ApiJson {
            api: MYVARIANT_API.to_string(),
            source,
        })
    }

    pub(crate) async fn get_all(&self, id: &str) -> Result<Vec<MyVariantHit>, BioMcpError> {
        let id = id.trim();
        let plan = Self::get_plan(id, None)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let response = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::MYVARIANT,
            ))
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned();
        let bytes = crate::sources::read_limited_source_body(
            response,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::MYVARIANT),
        )
        .await?;
        let value: serde_json::Value = crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
            status,
            content_type.as_ref(),
            &bytes,
            true,
        )?;
        Self::select_get_hit_values(value)?
            .into_iter()
            .map(|value| {
                serde_json::from_value(value).map_err(|source| BioMcpError::ApiJson {
                    api: MYVARIANT_API.to_string(),
                    source,
                })
            })
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyVariantSearchResponse {
    pub total: Option<usize>,
    #[serde(default)]
    pub hits: Vec<MyVariantHit>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantHit {
    #[serde(rename = "_id")]
    pub id: String,

    pub cadd: Option<MyVariantCadd>,
    pub clinvar: Option<MyVariantClinVar>,
    pub dbnsfp: Option<MyVariantDbnsfp>,
    pub dbsnp: Option<MyVariantDbsnp>,
    pub gnomad_exome: Option<MyVariantGnomadExome>,
    pub gnomad: Option<MyVariantGnomad>,
    pub exac: Option<MyVariantExac>,
    pub exac_nontcga: Option<MyVariantExac>,
    pub cosmic: Option<MyVariantCosmic>,
    pub cgi: Option<serde_json::Value>,
    pub civic: Option<serde_json::Value>,
    pub snpeff: Option<MyVariantSnpeff>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantSnpeff {
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub ann: Vec<MyVariantSnpeffAnnotation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantSnpeffAnnotation {
    pub feature_id: Option<String>,
    pub genename: Option<String>,
    pub hgvs_c: Option<String>,
    pub hgvs_p: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantCadd {
    pub phred: Option<f64>,
    pub consequence: Option<StringOrVec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantDbsnp {
    pub rsid: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantCosmic {
    #[serde(default)]
    pub cosmic_id: StringOrVec,
    pub mut_freq: Option<f64>,
    #[serde(default)]
    pub tumor_site: StringOrVec,
    #[serde(default)]
    pub mut_nt: StringOrVec,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantGnomadExome {
    pub af: Option<MyVariantGnomadAf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantGnomad {
    pub exomes: Option<MyVariantGnomadExome>,
    pub genomes: Option<MyVariantGnomadExome>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantGnomadAf {
    pub af: Option<f64>,
    pub af_afr: Option<f64>,
    pub af_eas: Option<f64>,
    pub af_nfe: Option<f64>,
    pub af_sas: Option<f64>,
    pub af_amr: Option<f64>,
    pub af_asj: Option<f64>,
    pub af_fin: Option<f64>,
    pub af_afr_female: Option<f64>,
    pub af_afr_male: Option<f64>,
    pub af_amr_female: Option<f64>,
    pub af_amr_male: Option<f64>,
    pub af_eas_jpn: Option<f64>,
    pub af_eas_kor: Option<f64>,
    pub af_nfe_bgr: Option<f64>,
    pub af_nfe_est: Option<f64>,
    pub af_nfe_nwe: Option<f64>,
    pub af_nfe_onf: Option<f64>,
    pub af_nfe_seu: Option<f64>,
    pub af_nfe_swe: Option<f64>,
    pub af_oth: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantExac {
    pub af: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantDbnsfp {
    #[serde(default)]
    pub genename: StringOrVec,
    #[serde(default)]
    pub hgvsp: StringOrVec,
    #[serde(default)]
    pub hgvsc: StringOrVec,
    pub sift: Option<MyVariantSift>,
    pub polyphen2: Option<MyVariantPolyPhen2>,
    pub revel: Option<MyVariantScoreRank>,
    pub alphamissense: Option<MyVariantPredScore>,
    pub clinpred: Option<MyVariantPredScore>,
    pub metarnn: Option<MyVariantPredScore>,
    pub bayesdel: Option<MyVariantBayesDel>,
    pub phylop: Option<MyVariantConservationGroup>,
    pub phastcons: Option<MyVariantConservationGroup>,
    #[serde(rename = "gerp++")]
    pub gerp: Option<MyVariantGerp>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantBayesDel {
    pub add_af: Option<MyVariantPredScore>,
    pub no_af: Option<MyVariantPredScore>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantSift {
    pub pred: Option<StringOrVec>,
    pub score: Option<FloatOrVec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantPolyPhen2 {
    pub hdiv: Option<MyVariantPolyPhen2Hdiv>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantPolyPhen2Hdiv {
    pub pred: Option<StringOrVec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantScoreRank {
    pub score: Option<FloatOrVec>,
    pub rankscore: Option<FloatOrVec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantPredScore {
    #[serde(alias = "am_pathogenicity")]
    pub score: Option<FloatOrVec>,
    #[serde(alias = "am_class")]
    pub pred: Option<StringOrVec>,
    pub rankscore: Option<FloatOrVec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantConservationGroup {
    #[serde(rename = "100way_vertebrate")]
    pub way_100_vertebrate: Option<MyVariantRankScore>,
    #[serde(rename = "470way_mammalian")]
    pub way_470_mammalian: Option<MyVariantRankScore>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantRankScore {
    pub rankscore: Option<FloatOrVec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantGerp {
    pub rs: Option<FloatOrVec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantClinVar {
    pub variant_id: Option<u64>,
    pub gene: Option<MyVariantClinVarGene>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub rcv: Vec<MyVariantClinVarRcv>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantClinVarGene {
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantClinVarRcv {
    pub clinical_significance: Option<String>,
    pub review_status: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub preferred_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FloatOrVec {
    Single(f64),
    Multiple(Vec<f64>),
}

impl FloatOrVec {
    pub fn first(&self) -> Option<f64> {
        match self {
            Self::Single(v) => Some(*v),
            Self::Multiple(v) => v.first().copied(),
        }
    }
}

#[cfg(test)]
mod tests;
