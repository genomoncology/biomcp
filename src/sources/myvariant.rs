use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::BioMcpError;
use crate::sources::is_valid_gene_symbol;
use crate::utils::serde::StringOrVec;

const MYVARIANT_BASE: &str = "https://myvariant.info/v1";
const MYVARIANT_API: &str = "myvariant.info";
const MYVARIANT_BASE_ENV: &str = "BIOMCP_MYVARIANT_BASE";

pub(crate) const MYVARIANT_FIELDS_GET: &str = concat!(
    "_id,cadd.phred,cadd.consequence,",
    "clinvar.rcv.clinical_significance,clinvar.rcv.review_status,clinvar.rcv.conditions,clinvar.variant_id,",
    "dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.hgvsc,",
    "dbnsfp.sift.pred,dbnsfp.sift.score,",
    "dbnsfp.polyphen2.hdiv.pred,",
    "dbnsfp.revel.score,dbnsfp.revel.rankscore,",
    "dbnsfp.alphamissense.score,dbnsfp.alphamissense.pred,dbnsfp.alphamissense.rankscore,",
    "dbnsfp.clinpred.score,dbnsfp.clinpred.pred,",
    "dbnsfp.metarnn.score,dbnsfp.metarnn.pred,",
    "dbnsfp.bayesdel_addaf.score,dbnsfp.bayesdel_addaf.pred,",
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
pub(crate) const MYVARIANT_FIELDS_SEARCH: &str = "_id,dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.revel.score,dbnsfp.gerp++.rs,clinvar.rcv.clinical_significance,clinvar.rcv.review_status,dbsnp.rsid,gnomad_exome.af.af,gnomad.exomes.af.af,gnomad.genomes.af.af,cadd.consequence";

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

pub struct VariantSearchParams {
    pub gene: Option<String>,
    pub hgvsp: Option<String>,
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
    "protein_altering_variant",
];

const POPULATION_VALUES: &[&str] = &["afr", "amr", "eas", "fin", "nfe", "sas", "asj", "oth"];

const IMPACT_VALUES: &[&str] = &["HIGH", "MODERATE", "LOW", "MODIFIER"];

fn normalize_filter_key(value: &str) -> String {
    let mut out = String::new();
    let mut prev_sep = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_sep = false;
            continue;
        }
        if matches!(ch, ' ' | ',' | '-' | '_') && !prev_sep {
            out.push('_');
            prev_sep = true;
        }
    }
    out.trim_matches('_').to_string()
}

fn invalid_filter_error(flag: &str, raw: &str, accepted: &[&str]) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Invalid {flag} value '{raw}'. Expected one of: {}",
        accepted.join(", ")
    ))
}

fn normalize_significance_filter(value: &str) -> Result<String, BioMcpError> {
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

fn normalize_consequence_filter(value: &str) -> Result<String, BioMcpError> {
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

fn normalize_population_filter(value: &str) -> Result<String, BioMcpError> {
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

fn normalize_impact_filter(value: &str) -> Result<String, BioMcpError> {
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

fn normalize_review_status_filter(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--review-status must not be empty".into(),
        ));
    }
    let lowered = raw.to_ascii_lowercase();
    let normalized = match lowered.as_str() {
        "0" | "0_star" | "0_stars" | "none" => "no_assertion_criteria_provided",
        "1" | "1_star" | "1_stars" => "criteria_provided_single_submitter",
        "2" | "2_star" | "2_stars" => "criteria_provided_multiple_submitters_no_conflicts",
        "3" | "3_star" | "3_stars" => "reviewed_by_expert_panel",
        "4" | "4_star" | "4_stars" => "practice_guideline",
        other => other,
    };
    Ok(normalized.to_string())
}

impl MyVariantClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MYVARIANT_BASE, MYVARIANT_BASE_ENV),
        })
    }

    #[cfg(test)]
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub(crate) fn escape_query_value(value: &str) -> String {
        crate::utils::query::escape_lucene_value(value)
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, MYVARIANT_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: MYVARIANT_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        crate::sources::ensure_json_content_type(MYVARIANT_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: MYVARIANT_API.to_string(),
            source,
        })
    }

    pub async fn query_with_fields(
        &self,
        q: &str,
        limit: usize,
        offset: usize,
        fields: &str,
    ) -> Result<MyVariantSearchResponse, BioMcpError> {
        let q = q.trim();
        if q.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search variant -g BRAF".into(),
            ));
        }
        crate::sources::validate_biothings_result_window("MyVariant search", limit, offset)?;

        let url = self.endpoint("query");
        let size = limit.to_string();
        let from = offset.to_string();
        self.get_json(self.client.get(&url).query(&[
            ("q", q),
            ("size", size.as_str()),
            ("from", from.as_str()),
            ("fields", fields),
        ]))
        .await
    }

    pub async fn search(
        &self,
        params: &VariantSearchParams,
    ) -> Result<MyVariantSearchResponse, BioMcpError> {
        crate::sources::validate_biothings_result_window(
            "MyVariant search",
            params.limit,
            params.offset,
        )?;

        let mut terms: Vec<String> = Vec::new();

        if let Some(gene) = params
            .gene
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
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
            if min < 0.0 {
                return Err(BioMcpError::InvalidArgument(format!(
                    "--min-cadd must be >= 0 (got {min})"
                )));
            }
            terms.push(format!("cadd.phred:[{min} TO *]"));
        }
        if let Some(consequence) = params
            .consequence
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            let normalized = normalize_consequence_filter(consequence)?;
            terms.push(format!(
                "cadd.consequence:{}",
                Self::escape_query_value(&normalized)
            ));
        }

        if let Some(review_status) = params
            .review_status
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            let normalized = normalize_review_status_filter(review_status)?;
            terms.push(format!(
                "clinvar.rcv.review_status:{}",
                Self::escape_query_value(&normalized)
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
            terms.push(format!("dbnsfp.gerp++_rs:[{gerp_min} TO *]"));
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

        if let Some(has) = params
            .has
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            terms.push(format!("_exists_:{}", Self::escape_query_value(has)));
        }

        if let Some(missing) = params
            .missing
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            terms.push(format!("_missing_:{}", Self::escape_query_value(missing)));
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
        let url = self.endpoint("query");
        let size = params.limit.to_string();
        let from = params.offset.to_string();
        self.get_json(self.client.get(&url).query(&[
            ("q", q.as_str()),
            ("size", size.as_str()),
            ("from", from.as_str()),
            ("fields", MYVARIANT_FIELDS_SEARCH),
        ]))
        .await
    }

    pub async fn get(&self, id: &str) -> Result<MyVariantHit, BioMcpError> {
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

        let url = self.endpoint(&format!("variant/{id}"));
        let value: serde_json::Value = self
            .get_json(
                self.client
                    .get(&url)
                    .query(&[("fields", MYVARIANT_FIELDS_GET)]),
            )
            .await?;

        let hit_value = match value {
            serde_json::Value::Object(_) => value,
            serde_json::Value::Array(mut arr) => {
                arr.drain(..).next().ok_or_else(|| BioMcpError::NotFound {
                    entity: "variant".into(),
                    id: id.to_string(),
                    suggestion: format!("Try searching: biomcp search variant -g \"{id}\""),
                })?
            }
            _ => {
                return Err(BioMcpError::Api {
                    api: MYVARIANT_API.to_string(),
                    message: "Unexpected response type".into(),
                });
            }
        };

        serde_json::from_value(hit_value).map_err(|source| BioMcpError::ApiJson {
            api: MYVARIANT_API.to_string(),
            source,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyVariantSearchResponse {
    #[allow(dead_code)]
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
    pub af_afr: Option<f64>,
    pub af_amr: Option<f64>,
    pub af_eas: Option<f64>,
    pub af_fin: Option<f64>,
    pub af_nfe: Option<f64>,
    pub af_oth: Option<f64>,
    pub af_sas: Option<f64>,
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
    pub bayesdel_addaf: Option<MyVariantPredScore>,
    pub phylop: Option<MyVariantConservationGroup>,
    pub phastcons: Option<MyVariantConservationGroup>,
    #[serde(rename = "gerp++")]
    pub gerp: Option<MyVariantGerp>,
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
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub rcv: Vec<MyVariantClinVarRcv>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyVariantClinVarRcv {
    pub clinical_significance: Option<String>,
    pub review_status: Option<String>,
    pub conditions: Option<serde_json::Value>,
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
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn query_sets_fields_and_size() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/query"))
            .and(query_param("q", "dbnsfp.genename:BRAF"))
            .and(query_param("size", "3"))
            .and(query_param("from", "0"))
            .and(query_param("fields", MYVARIANT_FIELDS_SEARCH))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 0,
                "hits": []
            })))
            .mount(&server)
            .await;

        let client = MyVariantClient::new_for_test(server.uri()).unwrap();
        let _ = client
            .query_with_fields("dbnsfp.genename:BRAF", 3, 0, MYVARIANT_FIELDS_SEARCH)
            .await
            .unwrap();
    }

    #[test]
    fn clinvar_rcv_deserializes_single_object() {
        let clinvar: MyVariantClinVar = serde_json::from_value(json!({
            "variant_id": 123,
            "rcv": {
                "clinical_significance": "Pathogenic",
                "review_status": "criteria provided",
                "conditions": "Lung carcinoma"
            }
        }))
        .expect("single-object RCV should deserialize");

        assert_eq!(clinvar.variant_id, Some(123));
        assert_eq!(clinvar.rcv.len(), 1);
        assert_eq!(
            clinvar.rcv[0].clinical_significance.as_deref(),
            Some("Pathogenic")
        );
    }

    #[test]
    fn clinvar_rcv_deserializes_array() {
        let clinvar: MyVariantClinVar = serde_json::from_value(json!({
            "variant_id": 456,
            "rcv": [
                { "clinical_significance": "Pathogenic" },
                { "clinical_significance": "Likely pathogenic" }
            ]
        }))
        .expect("array RCV should deserialize");

        assert_eq!(clinvar.variant_id, Some(456));
        assert_eq!(clinvar.rcv.len(), 2);
        assert_eq!(
            clinvar.rcv[0].clinical_significance.as_deref(),
            Some("Pathogenic")
        );
    }

    #[test]
    fn gnomad_nested_fields_deserialize() {
        let hit: MyVariantHit = serde_json::from_value(json!({
            "_id": "chr1:g.1A>T",
            "dbnsfp": {"genename": "TP53"},
            "gnomad": {
                "exomes": { "af": { "af": 0.001 } },
                "genomes": { "af": { "af": 0.002 } }
            }
        }))
        .expect("gnomad nested object should deserialize");

        assert_eq!(
            hit.gnomad
                .as_ref()
                .and_then(|g| g.exomes.as_ref())
                .and_then(|e| e.af.as_ref())
                .and_then(|a| a.af),
            Some(0.001)
        );
        assert_eq!(
            hit.gnomad
                .as_ref()
                .and_then(|g| g.genomes.as_ref())
                .and_then(|e| e.af.as_ref())
                .and_then(|a| a.af),
            Some(0.002)
        );
    }

    #[test]
    fn expanded_fields_deserialize() {
        let hit: MyVariantHit = serde_json::from_value(json!({
            "_id": "chr7:g.140453136A>T",
            "dbnsfp": {
                "genename": "BRAF",
                "revel": { "score": 0.931 },
                "alphamissense": { "score": [0.99], "pred": ["P"] },
                "gerp++": { "rs": 5.65 },
                "phylop": { "100way_vertebrate": { "rankscore": 0.94 } }
            },
            "exac": { "af": 0.00001 },
            "exac_nontcga": { "af": 0.00002 },
            "cosmic": { "cosmic_id": "COSM476", "mut_freq": 2.8, "tumor_site": "skin" },
            "cgi": [{ "drug": "vemurafenib", "association": "Responsive" }],
            "civic": {"molecularProfiles": [{"name": "BRAF V600E"}]}
        }))
        .expect("expanded fields should deserialize");

        assert_eq!(
            hit.dbnsfp
                .as_ref()
                .and_then(|d| d.revel.as_ref())
                .and_then(|r| r.score.as_ref())
                .and_then(FloatOrVec::first),
            Some(0.931)
        );
        assert_eq!(hit.exac.as_ref().and_then(|e| e.af), Some(0.00001));
        assert_eq!(hit.exac_nontcga.as_ref().and_then(|e| e.af), Some(0.00002));
        assert!(hit.cgi.is_some());
        assert!(hit.civic.is_some());
    }

    #[test]
    fn significance_filter_accepts_common_aliases() {
        assert_eq!(
            normalize_significance_filter("Likely Pathogenic").unwrap(),
            "likely_pathogenic"
        );
        assert_eq!(
            normalize_significance_filter("uncertain").unwrap(),
            "uncertain_significance"
        );
        assert_eq!(
            normalize_significance_filter("conflicting").unwrap(),
            "conflicting_interpretations_of_pathogenicity"
        );
    }

    #[test]
    fn significance_filter_rejects_unknown_value() {
        let err = normalize_significance_filter("bogus").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--significance"));
        assert!(msg.contains("Expected one of"));
    }

    #[test]
    fn consequence_filter_accepts_shorthand_and_aliases() {
        assert_eq!(
            normalize_consequence_filter("missense").unwrap(),
            "missense_variant"
        );
        assert_eq!(
            normalize_consequence_filter("synonymous").unwrap(),
            "synonymous_variant"
        );
        assert_eq!(
            normalize_consequence_filter("non-synonymous").unwrap(),
            "missense_variant"
        );
        assert_eq!(
            normalize_consequence_filter("splice donor").unwrap(),
            "splice_donor_variant"
        );
    }

    #[test]
    fn consequence_filter_rejects_unknown_value() {
        let err = normalize_consequence_filter("bogus").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--consequence"));
        assert!(msg.contains("Expected one of"));
    }

    #[tokio::test]
    async fn search_rejects_invalid_gene_symbol_characters() {
        let client = MyVariantClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let params = VariantSearchParams {
            gene: Some("BRAF:V600E".into()),
            hgvsp: None,
            significance: None,
            max_frequency: None,
            min_cadd: None,
            consequence: None,
            review_status: None,
            population: None,
            revel_min: None,
            gerp_min: None,
            tumor_site: None,
            condition: None,
            impact: None,
            lof: false,
            has: None,
            missing: None,
            therapy: None,
            limit: 3,
            offset: 0,
        };

        let err = client.search(&params).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("Gene symbol filter"));
    }

    #[tokio::test]
    async fn search_rejects_offset_at_biothings_window() {
        let client = MyVariantClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let params = VariantSearchParams {
            gene: Some("BRAF".into()),
            hgvsp: None,
            significance: None,
            max_frequency: None,
            min_cadd: None,
            consequence: None,
            review_status: None,
            population: None,
            revel_min: None,
            gerp_min: None,
            tumor_site: None,
            condition: None,
            impact: None,
            lof: false,
            has: None,
            missing: None,
            therapy: None,
            limit: 5,
            offset: 10_000,
        };

        let err = client.search(&params).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--offset must be less than 10000"));
    }

    #[tokio::test]
    async fn search_rejects_offset_limit_window_overflow() {
        let client = MyVariantClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let params = VariantSearchParams {
            gene: Some("BRAF".into()),
            hgvsp: None,
            significance: None,
            max_frequency: None,
            min_cadd: None,
            consequence: None,
            review_status: None,
            population: None,
            revel_min: None,
            gerp_min: None,
            tumor_site: None,
            condition: None,
            impact: None,
            lof: false,
            has: None,
            missing: None,
            therapy: None,
            limit: 25,
            offset: 9_980,
        };

        let err = client.search(&params).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(
            err.to_string()
                .contains("--offset + --limit must be <= 10000")
        );
    }
}
