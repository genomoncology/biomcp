//! Diagnostic entity models and workflows exposed through the stable diagnostic facade.

mod get;
mod search;

pub use self::get::get;
#[allow(unused_imports)]
pub use self::search::{search, search_page, search_query_summary};

use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::sources::gtr::{GtrIndex, GtrRecord};
use crate::sources::who_ivd::WhoIvdRecord;

pub(crate) const DIAGNOSTIC_SOURCE_GTR: &str = "gtr";
pub(crate) const DIAGNOSTIC_SOURCE_WHO_IVD: &str = "who-ivd";
const DIAGNOSTIC_SECTION_GENES: &str = "genes";
const DIAGNOSTIC_SECTION_CONDITIONS: &str = "conditions";
const DIAGNOSTIC_SECTION_METHODS: &str = "methods";
pub(crate) const DIAGNOSTIC_SECTION_REGULATORY: &str = "regulatory";
const DIAGNOSTIC_SECTION_ALL: &str = "all";

pub const DIAGNOSTIC_SECTION_NAMES: &[&str] = &[
    DIAGNOSTIC_SECTION_GENES,
    DIAGNOSTIC_SECTION_CONDITIONS,
    DIAGNOSTIC_SECTION_METHODS,
    DIAGNOSTIC_SECTION_REGULATORY,
    DIAGNOSTIC_SECTION_ALL,
];

const GTR_DIAGNOSTIC_SECTION_NAMES: &[&str] = &[
    DIAGNOSTIC_SECTION_GENES,
    DIAGNOSTIC_SECTION_CONDITIONS,
    DIAGNOSTIC_SECTION_METHODS,
    DIAGNOSTIC_SECTION_REGULATORY,
    DIAGNOSTIC_SECTION_ALL,
];
const WHO_IVD_DIAGNOSTIC_SECTION_NAMES: &[&str] = &[
    DIAGNOSTIC_SECTION_CONDITIONS,
    DIAGNOSTIC_SECTION_REGULATORY,
    DIAGNOSTIC_SECTION_ALL,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRegulatoryRecord {
    pub submission_type: String,
    pub number: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applicant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_committee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplement_count: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagnosticSourceFilter {
    Gtr,
    WhoIvd,
    #[default]
    All,
}

impl DiagnosticSourceFilter {
    #[allow(dead_code)]
    pub fn from_flag(value: &str) -> Result<Self, crate::error::BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "all" => Ok(Self::All),
            DIAGNOSTIC_SOURCE_GTR => Ok(Self::Gtr),
            DIAGNOSTIC_SOURCE_WHO_IVD => Ok(Self::WhoIvd),
            other => Err(crate::error::BioMcpError::InvalidArgument(format!(
                "--source must be one of: {DIAGNOSTIC_SOURCE_GTR}, {DIAGNOSTIC_SOURCE_WHO_IVD}, all (got {other})"
            ))),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Gtr => DIAGNOSTIC_SOURCE_GTR,
            Self::WhoIvd => DIAGNOSTIC_SOURCE_WHO_IVD,
            Self::All => "all",
        }
    }

    pub(crate) fn includes_gtr(self) -> bool {
        matches!(self, Self::Gtr | Self::All)
    }

    pub(crate) fn includes_who_ivd(self) -> bool {
        matches!(self, Self::WhoIvd | Self::All)
    }

    pub(crate) fn query_summary(self) -> Option<String> {
        (!matches!(self, Self::All)).then(|| format!("source={}", self.as_str()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub source: String,
    pub source_id: String,
    pub accession: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulatory_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prequalification_year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub laboratory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub institution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clia_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_licenses: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_status: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub method_categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulatory: Option<Vec<DiagnosticRegulatoryRecord>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSearchResult {
    pub source: String,
    pub accession: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer_or_lab: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DiagnosticSearchFilters {
    pub source: DiagnosticSourceFilter,
    pub gene: Option<String>,
    pub disease: Option<String>,
    pub test_type: Option<String>,
    pub manufacturer: Option<String>,
}

fn gtr_accession_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^GTR\d+\.\d+$").expect("valid GTR accession regex"))
}

pub(crate) fn looks_like_gtr_accession(value: &str) -> bool {
    gtr_accession_re().is_match(value.trim())
}

pub(crate) fn diagnostic_source_label(source: &str) -> &'static str {
    if source
        .trim()
        .eq_ignore_ascii_case(DIAGNOSTIC_SOURCE_WHO_IVD)
    {
        "WHO Prequalified IVD"
    } else {
        "NCBI Genetic Testing Registry"
    }
}

pub(crate) fn supported_diagnostic_sections_for_source(source: &str) -> &'static [&'static str] {
    if source
        .trim()
        .eq_ignore_ascii_case(DIAGNOSTIC_SOURCE_WHO_IVD)
    {
        WHO_IVD_DIAGNOSTIC_SECTION_NAMES
    } else {
        GTR_DIAGNOSTIC_SECTION_NAMES
    }
}

pub(crate) fn preferred_diagnostic_name(record: &GtrRecord) -> String {
    optional_text(&record.lab_test_name)
        .or_else(|| optional_text(&record.manufacturer_test_name))
        .unwrap_or_else(|| record.accession.clone())
}

pub(crate) fn manufacturer_or_lab_label(record: &GtrRecord) -> Option<String> {
    optional_text(&record.manufacturer_test_name)
        .or_else(|| optional_text(&record.name_of_laboratory))
        .or_else(|| optional_text(&record.lab_test_name))
}

pub(crate) fn optional_text(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

pub(crate) fn search_result(record: &GtrRecord, index: &GtrIndex) -> DiagnosticSearchResult {
    DiagnosticSearchResult {
        source: DIAGNOSTIC_SOURCE_GTR.to_string(),
        accession: record.accession.clone(),
        name: preferred_diagnostic_name(record),
        test_type: optional_text(&record.test_type),
        manufacturer_or_lab: manufacturer_or_lab_label(record),
        genes: index.merged_genes(&record.accession),
        conditions: index.conditions(&record.accession),
    }
}

pub(crate) fn who_ivd_search_result(record: &WhoIvdRecord) -> DiagnosticSearchResult {
    DiagnosticSearchResult {
        source: DIAGNOSTIC_SOURCE_WHO_IVD.to_string(),
        accession: record.product_code.clone(),
        name: optional_text(&record.product_name).unwrap_or_else(|| record.product_code.clone()),
        test_type: optional_text(&record.assay_format),
        manufacturer_or_lab: optional_text(&record.manufacturer_name),
        genes: Vec::new(),
        conditions: optional_text(&record.target_marker).into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    use crate::sources::gtr::{GTR_CONDITION_GENE_FILE, GTR_TEST_VERSION_FILE, resolve_gtr_root};
    use crate::sources::who_ivd::WHO_IVD_CSV_FILE;
    use crate::test_support::{TempDirGuard, env_lock, set_env_var};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn install_gtr_fixture_root(
        label: &str,
    ) -> (
        tokio::sync::MutexGuard<'static, ()>,
        TempDirGuard,
        crate::test_support::EnvVarGuard,
    ) {
        let lock = env_lock().lock().await;
        let root = TempDirGuard::new(label);
        write_gtr_fixture(root.path());
        let env = set_env_var(
            "BIOMCP_GTR_DIR",
            Some(root.path().to_str().expect("utf-8 path")),
        );
        (lock, root, env)
    }

    async fn install_all_fixture_roots(
        label: &str,
    ) -> (
        tokio::sync::MutexGuard<'static, ()>,
        TempDirGuard,
        crate::test_support::EnvVarGuard,
        TempDirGuard,
        crate::test_support::EnvVarGuard,
    ) {
        let lock = env_lock().lock().await;
        let gtr_root = TempDirGuard::new(&format!("{label}-gtr"));
        write_gtr_fixture(gtr_root.path());
        let gtr_env = set_env_var(
            "BIOMCP_GTR_DIR",
            Some(gtr_root.path().to_str().expect("utf-8 path")),
        );
        let who_root = TempDirGuard::new(&format!("{label}-who-ivd"));
        write_who_ivd_fixture(who_root.path());
        let who_env = set_env_var(
            "BIOMCP_WHO_IVD_DIR",
            Some(who_root.path().to_str().expect("utf-8 path")),
        );
        (lock, gtr_root, gtr_env, who_root, who_env)
    }

    fn write_gtr_fixture(root: &Path) {
        std::fs::write(
            root.join(GTR_TEST_VERSION_FILE),
            include_bytes!("../../../spec/fixtures/gtr/test_version.gz"),
        )
        .expect("write test_version.gz");
        std::fs::write(
            root.join(GTR_CONDITION_GENE_FILE),
            include_str!("../../../spec/fixtures/gtr/test_condition_gene.txt"),
        )
        .expect("write test_condition_gene.txt");
    }

    fn write_who_ivd_fixture(root: &Path) {
        std::fs::write(
            root.join(WHO_IVD_CSV_FILE),
            include_str!("../../../spec/fixtures/who-ivd/who_ivd.csv"),
        )
        .expect("write who_ivd.csv");
    }

    #[tokio::test]
    async fn search_page_requires_at_least_one_filter() {
        let err = search_page(&DiagnosticSearchFilters::default(), 10, 0)
            .await
            .expect_err("missing filters should fail");

        assert_eq!(
            err.to_string(),
            "Invalid argument: diagnostic search requires at least one of --gene, --disease, --type, or --manufacturer"
        );
    }

    #[tokio::test]
    async fn search_page_applies_conjunctive_filters_and_stable_ordering() {
        let (_lock, _root, _env) = install_gtr_fixture_root("diagnostic-search").await;

        let page = search_page(
            &DiagnosticSearchFilters {
                source: DiagnosticSourceFilter::All,
                gene: Some("EGFR".to_string()),
                disease: Some("melanoma".to_string()),
                test_type: Some("molecular".to_string()),
                manufacturer: Some("Precision".to_string()),
            },
            10,
            0,
        )
        .await
        .expect("search page");

        assert_eq!(page.total, Some(1));
        assert_eq!(page.results.len(), 1);
        assert_eq!(page.results[0].source, "gtr");
        assert_eq!(page.results[0].accession, "GTR000000002.1");
        assert_eq!(page.results[0].name, "EGFR Melanoma Molecular Assay");
        assert_eq!(page.results[0].genes, vec!["EGFR".to_string()]);
        assert_eq!(
            page.results[0].conditions,
            vec!["Cutaneous melanoma".to_string()]
        );
        assert_eq!(
            resolve_gtr_root()
                .file_name()
                .and_then(|value| value.to_str()),
            Some(
                _root
                    .path()
                    .file_name()
                    .and_then(|value| value.to_str())
                    .expect("fixture dir")
            )
        );
    }

    #[tokio::test]
    async fn search_page_rejects_short_disease_filter() {
        let (_lock, _gtr_root, _gtr_env, _who_root, _who_env) =
            install_all_fixture_roots("diagnostic-search-short-disease").await;

        let err = search_page(
            &DiagnosticSearchFilters {
                source: DiagnosticSourceFilter::All,
                disease: Some("ma".to_string()),
                ..DiagnosticSearchFilters::default()
            },
            10,
            0,
        )
        .await
        .expect_err("short disease filter should fail");

        assert_eq!(
            err.to_string(),
            "Invalid argument: --disease must contain at least 3 alphanumeric characters for diagnostic disease matching"
        );
    }

    #[tokio::test]
    async fn search_page_disease_filter_requires_word_boundary() {
        let (_lock, _root, _env) = install_gtr_fixture_root("diagnostic-search-boundary").await;

        let partial = search_page(
            &DiagnosticSearchFilters {
                source: DiagnosticSourceFilter::Gtr,
                disease: Some("oma".to_string()),
                ..DiagnosticSearchFilters::default()
            },
            10,
            0,
        )
        .await
        .expect("partial disease search");

        assert_eq!(partial.total, Some(0));
        assert!(partial.results.is_empty());

        let full_word = search_page(
            &DiagnosticSearchFilters {
                source: DiagnosticSourceFilter::Gtr,
                disease: Some("melanoma".to_string()),
                ..DiagnosticSearchFilters::default()
            },
            10,
            0,
        )
        .await
        .expect("word-boundary disease search");

        assert_eq!(full_word.total, Some(1));
        assert_eq!(full_word.results[0].accession, "GTR000000002.1");
    }

    #[tokio::test]
    async fn search_page_rejects_explicit_who_gene_filter() {
        let (_lock, _gtr_root, _gtr_env, _who_root, _who_env) =
            install_all_fixture_roots("diagnostic-search-who-gene").await;

        let err = search_page(
            &DiagnosticSearchFilters {
                source: DiagnosticSourceFilter::WhoIvd,
                gene: Some("BRCA1".to_string()),
                ..DiagnosticSearchFilters::default()
            },
            10,
            0,
        )
        .await
        .expect_err("WHO gene filter should fail");

        assert_eq!(
            err.to_string(),
            "Invalid argument: WHO IVD does not support --gene; use --source gtr or omit --source for gene-first diagnostic searches"
        );
    }

    #[tokio::test]
    async fn search_page_returns_who_rows_for_disease_filter() {
        let (_lock, _gtr_root, _gtr_env, _who_root, _who_env) =
            install_all_fixture_roots("diagnostic-search-who-disease").await;

        let page = search_page(
            &DiagnosticSearchFilters {
                source: DiagnosticSourceFilter::WhoIvd,
                disease: Some("HIV".to_string()),
                ..DiagnosticSearchFilters::default()
            },
            10,
            0,
        )
        .await
        .expect("WHO IVD search");

        assert_eq!(page.total, Some(1));
        assert_eq!(page.results.len(), 1);
        assert_eq!(page.results[0].source, "who-ivd");
        assert_eq!(page.results[0].accession, "ITPW02232- TC40");
        assert_eq!(page.results[0].name, "ONE STEP Anti-HIV (1&2) Test");
        assert_eq!(page.results[0].conditions, vec!["HIV".to_string()]);
    }

    #[tokio::test]
    async fn search_page_all_source_uses_unknown_total_when_both_sources_match() {
        let (_lock, _gtr_root, _gtr_env, _who_root, _who_env) =
            install_all_fixture_roots("diagnostic-search-all").await;

        let page = search_page(
            &DiagnosticSearchFilters {
                source: DiagnosticSourceFilter::All,
                disease: Some("tuberculosis".to_string()),
                ..DiagnosticSearchFilters::default()
            },
            10,
            0,
        )
        .await
        .expect("merged search");

        assert_eq!(page.total, None);
        assert_eq!(page.results.len(), 10);
        assert_eq!(page.results[0].source, "who-ivd");
        assert!(page.results.iter().any(|row| row.source == "gtr"));
    }

    #[tokio::test]
    async fn get_keeps_summary_by_default_and_requested_sections_as_options() {
        let (_lock, _root, _env) = install_gtr_fixture_root("diagnostic-get").await;

        let summary = get("GTR000000001.1", &[]).await.expect("summary get");
        assert_eq!(summary.source, "gtr");
        assert_eq!(summary.source_id, "GTR000000001.1");
        assert_eq!(summary.accession, "GTR000000001.1");
        assert_eq!(summary.name, "BRCA1 Hereditary Cancer Panel");
        assert!(summary.target_marker.is_none());
        assert!(summary.regulatory_version.is_none());
        assert!(summary.prequalification_year.is_none());
        assert!(summary.genes.is_none());
        assert!(summary.conditions.is_none());
        assert!(summary.methods.is_none());
        assert!(summary.regulatory.is_none());
        assert_eq!(
            summary.method_categories,
            vec!["Molecular genetics".to_string()]
        );

        let expanded = get(
            "GTR000000001.1",
            &[
                "genes".to_string(),
                "conditions".to_string(),
                "methods".to_string(),
            ],
        )
        .await
        .expect("expanded get");
        assert_eq!(
            expanded.genes,
            Some(vec!["BRCA1".to_string(), "BARD1".to_string()])
        );
        assert_eq!(
            expanded.conditions,
            Some(vec![
                "Hereditary breast ovarian cancer syndrome".to_string(),
                "Breast cancer".to_string()
            ])
        );
        assert_eq!(
            expanded.methods,
            Some(vec![
                "Sequence analysis".to_string(),
                "Deletion/duplication analysis".to_string()
            ])
        );
        assert!(expanded.regulatory.is_none());
    }

    #[tokio::test]
    async fn get_diagnostic_genes_returns_full_deduped_broad_panel_list() {
        let (_lock, _root, _env) = install_gtr_fixture_root("diagnostic-get-broad-panel").await;

        let expanded = get(
            "GTR000000003.1",
            &["genes".to_string(), "conditions".to_string()],
        )
        .await
        .expect("expanded broad-panel get");

        assert_eq!(
            expanded.genes,
            Some(vec![
                "BRAF".to_string(),
                "BRCA1".to_string(),
                "BRCA2".to_string(),
                "ATM".to_string(),
                "PALB2".to_string(),
                "CHEK2".to_string(),
                "NBN".to_string(),
                "CDH1".to_string(),
                "STK11".to_string(),
            ])
        );
        assert_eq!(
            expanded.conditions,
            Some(vec![
                "Breast cancer".to_string(),
                "Ovarian cancer".to_string(),
                "Hereditary breast ovarian cancer syndrome".to_string(),
                "Pancreatic cancer".to_string(),
                "Lynch syndrome".to_string(),
                "Colon cancer".to_string(),
            ])
        );
    }

    #[tokio::test]
    async fn get_who_ivd_keeps_summary_and_resolves_supported_sections() {
        let (_lock, _gtr_root, _gtr_env, _who_root, _who_env) =
            install_all_fixture_roots("diagnostic-get-who").await;

        let summary = get("ITPW02232- TC40", &[]).await.expect("summary get");
        assert_eq!(summary.source, "who-ivd");
        assert_eq!(summary.source_id, "ITPW02232- TC40");
        assert_eq!(summary.accession, "ITPW02232- TC40");
        assert_eq!(summary.name, "ONE STEP Anti-HIV (1&2) Test");
        assert_eq!(
            summary.test_type.as_deref(),
            Some("Immunochromatographic (lateral flow)")
        );
        assert_eq!(
            summary.manufacturer.as_deref(),
            Some("InTec Products, Inc.")
        );
        assert_eq!(summary.target_marker.as_deref(), Some("HIV"));
        assert_eq!(summary.regulatory_version.as_deref(), Some("Rest-of-World"));
        assert_eq!(summary.prequalification_year.as_deref(), Some("2019"));
        assert!(summary.genes.is_none());
        assert!(summary.conditions.is_none());
        assert!(summary.methods.is_none());
        assert!(summary.regulatory.is_none());
        assert!(summary.method_categories.is_empty());

        let conditions = get("ITPW02232- TC40", &["conditions".to_string()])
            .await
            .expect("conditions get");
        assert_eq!(conditions.conditions, Some(vec!["HIV".to_string()]));

        let expanded = get("ITPW02232- TC40", &["all".to_string()])
            .await
            .expect("all get");
        assert_eq!(expanded.conditions, Some(vec!["HIV".to_string()]));
        assert!(expanded.genes.is_none());
        assert!(expanded.methods.is_none());
        assert!(expanded.regulatory.is_none());
    }

    #[tokio::test]
    async fn get_who_ivd_regulatory_is_valid_and_returns_empty_vec_on_no_match() {
        let (_lock, _gtr_root, _gtr_env, _who_root, _who_env) =
            install_all_fixture_roots("diagnostic-get-who-regulatory").await;
        let server = MockServer::start().await;
        let _openfda_env = set_env_var("BIOMCP_OPENFDA_BASE", Some(&server.uri()));

        Mock::given(method("GET"))
            .and(path("/device/510k.json"))
            .and(query_param("limit", "25"))
            .and(query_param(
                "search",
                r#"device_name:"ONE STEP Anti\-HIV \(1\&2\) Test""#,
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "meta": {"results": {"skip": 0, "limit": 25, "total": 0}},
                "results": []
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/device/pma.json"))
            .and(query_param("limit", "25"))
            .and(query_param(
                "search",
                r#"trade_name:"ONE STEP Anti\-HIV \(1\&2\) Test""#,
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "meta": {"results": {"skip": 0, "limit": 25, "total": 0}},
                "results": []
            })))
            .mount(&server)
            .await;

        let diagnostic = get("ITPW02232- TC40", &["regulatory".to_string()])
            .await
            .expect("WHO regulatory get");

        assert_eq!(diagnostic.source, "who-ivd");
        assert!(
            diagnostic.regulatory.as_ref().is_some_and(Vec::is_empty),
            "requested WHO regulatory overlay should serialize as an empty section on no match"
        );
    }

    #[tokio::test]
    async fn get_who_ivd_rejects_unsupported_sections_with_recovery_hint() {
        let (_lock, _gtr_root, _gtr_env, _who_root, _who_env) =
            install_all_fixture_roots("diagnostic-get-who-sections").await;

        let genes = get("ITPW02232- TC40", &["genes".to_string()])
            .await
            .expect_err("WHO genes should fail");
        assert_eq!(
            genes.to_string(),
            "Invalid argument: Section `genes` is not available for WHO Prequalified IVD diagnostic records. Try `biomcp get diagnostic \"ITPW02232- TC40\" conditions`"
        );

        let methods = get("ITPW02232- TC40", &["methods".to_string()])
            .await
            .expect_err("WHO methods should fail");
        assert_eq!(
            methods.to_string(),
            "Invalid argument: Section `methods` is not available for WHO Prequalified IVD diagnostic records. Try `biomcp get diagnostic \"ITPW02232- TC40\" conditions`"
        );
    }

    #[tokio::test]
    async fn get_regulatory_uses_alias_queries_and_dedupes_pma_supplements() {
        let (_lock, _root, _gtr_env) = install_gtr_fixture_root("diagnostic-get-regulatory").await;
        // Isolate the shared OpenFDA HTTP cache for this mock-backed regulatory overlay test.
        let server = MockServer::start().await;
        let _openfda_env = set_env_var("BIOMCP_OPENFDA_BASE", Some(&server.uri()));

        Mock::given(method("GET"))
            .and(path("/device/510k.json"))
            .and(query_param("limit", "25"))
            .and(query_param(
                "search",
                "device_name:\"BRCA1 Hereditary Cancer Panel\" OR device_name:\"OncoPanel BRCA1\"",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "meta": {"results": {"skip": 0, "limit": 25, "total": 0}},
                "results": []
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/device/pma.json"))
            .and(query_param("limit", "25"))
            .and(query_param(
                "search",
                "trade_name:\"BRCA1 Hereditary Cancer Panel\" OR trade_name:\"OncoPanel BRCA1\"",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "meta": {"results": {"skip": 0, "limit": 25, "total": 2}},
                "results": [
                    {
                        "pma_number": "P000019",
                        "trade_name": "OncoPanel BRCA1",
                        "generic_name": "Hereditary cancer panel",
                        "applicant": "GenomOncology Lab",
                        "decision_date": "2024-01-15",
                        "decision_description": "supplement approved",
                        "product_code": "PQP",
                        "supplement_number": "S001"
                    },
                    {
                        "pma_number": "P000019",
                        "trade_name": "OncoPanel BRCA1",
                        "generic_name": "Hereditary cancer panel",
                        "applicant": "GenomOncology Lab",
                        "decision_date": "2024-09-10",
                        "decision_description": "panel expanded",
                        "product_code": "PQP",
                        "supplement_number": "S002"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let diagnostic = get("GTR000000001.1", &["regulatory".to_string()])
            .await
            .expect("regulatory get");
        let regulatory = diagnostic
            .regulatory
            .expect("regulatory records should be present when requested");

        assert_eq!(regulatory.len(), 1);
        assert_eq!(regulatory[0].submission_type, "PMA");
        assert_eq!(regulatory[0].number, "P000019");
        assert_eq!(regulatory[0].display_name, "OncoPanel BRCA1");
        assert_eq!(
            regulatory[0].decision_description.as_deref(),
            Some("panel expanded")
        );
        assert_eq!(regulatory[0].supplement_count, Some(2));
    }

    #[test]
    fn search_query_summary_uses_documented_filter_order() {
        let summary = search_query_summary(&DiagnosticSearchFilters {
            source: DiagnosticSourceFilter::Gtr,
            gene: Some("BRCA1".to_string()),
            disease: Some("melanoma".to_string()),
            test_type: Some("molecular".to_string()),
            manufacturer: Some("Tempus".to_string()),
        });

        assert_eq!(
            summary,
            "gene=BRCA1, disease=melanoma, type=molecular, manufacturer=Tempus, source=gtr"
        );
    }

    #[test]
    fn diagnostic_source_filter_from_flag_accepts_expected_values() {
        assert_eq!(
            DiagnosticSourceFilter::from_flag("gtr").expect("gtr"),
            DiagnosticSourceFilter::Gtr
        );
        assert_eq!(
            DiagnosticSourceFilter::from_flag("who-ivd").expect("who-ivd"),
            DiagnosticSourceFilter::WhoIvd
        );
        assert_eq!(
            DiagnosticSourceFilter::from_flag("").expect("default"),
            DiagnosticSourceFilter::All
        );
    }
}
