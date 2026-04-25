use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use bytesize::ByteSize;
use futures::stream::{self, StreamExt};

use crate::error::BioMcpError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthRow {
    pub api: String,
    pub status: String,
    pub latency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affects: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_configured: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthReport {
    pub healthy: usize,
    pub warning: usize,
    pub excluded: usize,
    pub total: usize,
    pub rows: Vec<HealthRow>,
}

impl HealthReport {
    pub fn all_healthy(&self) -> bool {
        self.healthy + self.warning + self.excluded == self.total
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let show_affects = self.rows.iter().any(|row| row.affects.is_some());
        let errors = self
            .total
            .saturating_sub(self.healthy + self.warning + self.excluded);

        out.push_str("# BioMCP Health Check\n\n");
        if show_affects {
            out.push_str("| API | Status | Latency | Affects |\n");
            out.push_str("|-----|--------|---------|---------|\n");
            for row in &self.rows {
                let affects = row.affects.as_deref().unwrap_or("-");
                let status = markdown_status(row);
                out.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    row.api, status, row.latency, affects
                ));
            }
        } else {
            out.push_str("| API | Status | Latency |\n");
            out.push_str("|-----|--------|---------|\n");
            for row in &self.rows {
                let status = markdown_status(row);
                out.push_str(&format!("| {} | {} | {} |\n", row.api, status, row.latency));
            }
        }

        out.push_str(&format!(
            "\nStatus: {} ok, {} error, {} excluded",
            self.healthy, errors, self.excluded
        ));
        if self.warning > 0 {
            out.push_str(&format!(", {} warning", self.warning));
        }
        out.push('\n');
        out
    }
}

fn markdown_status(row: &HealthRow) -> String {
    match (row.status.as_str(), row.key_configured) {
        ("ok", Some(true)) => "ok (key configured)".to_string(),
        ("error", Some(true)) => "error (key configured)".to_string(),
        ("error", Some(false)) => "error (key not configured)".to_string(),
        _ => row.status.clone(),
    }
}

#[derive(Debug, Clone, Copy)]
struct SourceDescriptor {
    api: &'static str,
    affects: Option<&'static str>,
    probe: ProbeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeClass {
    Healthy,
    Warning,
    Error,
    Excluded,
}

#[derive(Debug, Clone)]
struct ProbeOutcome {
    row: HealthRow,
    class: ProbeClass,
}

#[derive(Debug, Clone, Copy)]
enum ProbeKind {
    Get {
        url: &'static str,
    },
    PostJson {
        url: &'static str,
        payload: &'static str,
    },
    AuthGet {
        url: &'static str,
        env_var: &'static str,
        header_name: &'static str,
        header_value_prefix: &'static str,
    },
    OptionalAuthGet {
        url: &'static str,
        env_var: &'static str,
        header_name: &'static str,
        header_value_prefix: &'static str,
        unauthenticated_ok_status: &'static str,
        authenticated_ok_status: &'static str,
        unauthenticated_rate_limited_status: Option<&'static str>,
    },
    AuthQueryParam {
        url: &'static str,
        env_var: &'static str,
        param_name: &'static str,
    },
    #[allow(dead_code)]
    AuthPostJson {
        url: &'static str,
        payload: &'static str,
        env_var: &'static str,
        header_name: &'static str,
        header_value_prefix: &'static str,
    },
    AlphaGenomeConnect {
        env_var: &'static str,
    },
    VaersQuery,
}

const HEALTH_SOURCES: &[SourceDescriptor] = &[
    SourceDescriptor {
        api: "MyGene",
        affects: Some("get/search gene and gene helper commands"),
        probe: ProbeKind::Get {
            url: "https://mygene.info/v3/query?q=BRAF&size=1",
        },
    },
    SourceDescriptor {
        api: "MyVariant",
        affects: Some("get/search variant and variant helper commands"),
        probe: ProbeKind::Get {
            url: "https://myvariant.info/v1/query?q=rs113488022&size=1",
        },
    },
    SourceDescriptor {
        api: "MyChem",
        affects: Some("get/search drug and drug helper commands"),
        probe: ProbeKind::Get {
            url: "https://mychem.info/v1/query?q=aspirin&size=1",
        },
    },
    SourceDescriptor {
        api: "PubTator3",
        affects: Some("article annotations and entity extraction"),
        probe: ProbeKind::Get {
            url: "https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=22663011",
        },
    },
    SourceDescriptor {
        api: "PubMed",
        affects: Some("PubMed-backed article search foundation"),
        probe: ProbeKind::Get {
            url: "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&retmode=json&retmax=1&term=BRAF",
        },
    },
    SourceDescriptor {
        api: "Europe PMC",
        affects: Some("article search coverage"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=BRAF&format=json&pageSize=1",
        },
    },
    SourceDescriptor {
        api: "NCBI E-utilities",
        affects: Some("article fulltext fallback resolution"),
        probe: ProbeKind::Get {
            url: "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pmc&id=9984800&rettype=xml",
        },
    },
    SourceDescriptor {
        api: "LitSense2",
        affects: Some("keyword-gated semantic article search"),
        probe: ProbeKind::Get {
            url: "https://www.ncbi.nlm.nih.gov/research/litsense2-api/api/sentences/?query=test&rerank=true",
        },
    },
    SourceDescriptor {
        api: "PMC OA",
        affects: Some("article fulltext resolution"),
        probe: ProbeKind::Get {
            url: "https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi?id=PMC9984800",
        },
    },
    SourceDescriptor {
        api: "NCBI ID Converter",
        affects: Some("article fulltext resolution and identifier bridging"),
        probe: ProbeKind::Get {
            url: "https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles/?format=json&idtype=pmid&ids=22663011",
        },
    },
    SourceDescriptor {
        api: "ClinicalTrials.gov",
        affects: Some("search/get trial and trial helper commands"),
        probe: ProbeKind::Get {
            url: "https://clinicaltrials.gov/api/v2/studies?query.term=cancer&pageSize=1",
        },
    },
    SourceDescriptor {
        api: "NCI CTS",
        affects: Some("trial --source nci"),
        probe: ProbeKind::AuthGet {
            url: "https://clinicaltrialsapi.cancer.gov/api/v2/trials?size=1&keyword=melanoma",
            env_var: "NCI_API_KEY",
            header_name: "X-API-KEY",
            header_value_prefix: "",
        },
    },
    SourceDescriptor {
        api: "Enrichr",
        affects: Some("gene/pathway enrichment sections"),
        probe: ProbeKind::Get {
            url: "https://maayanlab.cloud/Enrichr/datasetStatistics",
        },
    },
    SourceDescriptor {
        api: "OpenFDA",
        affects: Some("adverse-event search"),
        probe: ProbeKind::Get {
            url: "https://api.fda.gov/drug/event.json?limit=1",
        },
    },
    SourceDescriptor {
        api: "CDC WONDER VAERS",
        affects: Some("vaccine adverse-event search for --source vaers|all"),
        probe: ProbeKind::VaersQuery,
    },
    SourceDescriptor {
        api: "OncoKB",
        affects: Some("variant oncokb command and variant evidence section"),
        probe: ProbeKind::AuthGet {
            url: "https://www.oncokb.org/api/v1/annotate/mutations/byProteinChange?hugoSymbol=BRAF&alteration=V600E",
            env_var: "ONCOKB_TOKEN",
            header_name: "Authorization",
            header_value_prefix: "Bearer ",
        },
    },
    SourceDescriptor {
        api: "DisGeNET",
        affects: Some("gene and disease disgenet sections"),
        probe: ProbeKind::AuthGet {
            url: "https://api.disgenet.com/api/v1/gda/summary?gene_ncbi_id=7157&page_number=0",
            env_var: "DISGENET_API_KEY",
            header_name: "Authorization",
            header_value_prefix: "",
        },
    },
    SourceDescriptor {
        api: "AlphaGenome",
        affects: Some("variant predict section"),
        probe: ProbeKind::AlphaGenomeConnect {
            env_var: "ALPHAGENOME_API_KEY",
        },
    },
    SourceDescriptor {
        api: "Semantic Scholar",
        affects: Some("Semantic Scholar features"),
        probe: ProbeKind::OptionalAuthGet {
            url: "https://api.semanticscholar.org/graph/v1/paper/search?query=BRAF&fields=paperId,title&limit=1",
            env_var: "S2_API_KEY",
            header_name: "x-api-key",
            header_value_prefix: "",
            unauthenticated_ok_status: "available (unauthenticated, shared rate limit)",
            authenticated_ok_status: "configured (authenticated)",
            unauthenticated_rate_limited_status: Some(
                "unavailable (set S2_API_KEY for reliable access)",
            ),
        },
    },
    SourceDescriptor {
        api: "CPIC",
        affects: Some("pgx recommendations and annotations"),
        probe: ProbeKind::Get {
            url: "https://api.cpicpgx.org/v1/pair_view?select=pairid&limit=1",
        },
    },
    SourceDescriptor {
        api: "PharmGKB",
        affects: Some("pgx recommendations and annotations"),
        probe: ProbeKind::Get {
            url: "https://api.pharmgkb.org/v1/data/labelAnnotation?relatedChemicals.name=warfarin&view=min",
        },
    },
    SourceDescriptor {
        api: "Monarch",
        affects: Some("disease genes, phenotypes, and models"),
        probe: ProbeKind::Get {
            url: "https://api-v3.monarchinitiative.org/v3/api/association?object=MONDO:0007739&subject_category=biolink:Gene&limit=1",
        },
    },
    SourceDescriptor {
        api: "HPO",
        affects: Some("phenotype search and disease ranking"),
        probe: ProbeKind::Get {
            url: "https://ontology.jax.org/api/hp/terms/HP:0001250",
        },
    },
    SourceDescriptor {
        api: "MyDisease",
        affects: Some("disease search and normalization"),
        probe: ProbeKind::Get {
            url: "https://mydisease.info/v1/query?q=melanoma&size=1&fields=disease_ontology.name,mondo.label",
        },
    },
    SourceDescriptor {
        api: "SEER Explorer",
        affects: Some("disease survival section"),
        probe: ProbeKind::Get {
            url: "https://seer.cancer.gov/statistics-network/explorer/source/content_writers/get_var_formats.php",
        },
    },
    SourceDescriptor {
        api: "NIH Reporter",
        affects: Some("gene and disease funding sections"),
        probe: ProbeKind::PostJson {
            url: "https://api.reporter.nih.gov/v2/projects/search",
            payload: r#"{"criteria":{"advanced_text_search":{"operator":"and","search_field":"projecttitle,abstracttext","search_text":"\"ERBB2\""},"fiscal_years":[2022,2023,2024,2025,2026]},"include_fields":["ProjectNum"],"offset":0,"limit":1,"sort_field":"award_amount","sort_order":"desc"}"#,
        },
    },
    SourceDescriptor {
        api: "CIViC",
        affects: Some("disease genes and variants sections"),
        probe: ProbeKind::PostJson {
            url: "https://civicdb.org/api/graphql",
            payload: r#"{"query":"query { evidenceItems(first: 1) { totalCount } }"}"#,
        },
    },
    SourceDescriptor {
        api: "GWAS Catalog",
        affects: Some("gwas search and variant gwas context"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/gwas/rest/api/singleNucleotidePolymorphisms/rs7903146",
        },
    },
    SourceDescriptor {
        api: "GTEx",
        affects: Some("gene expression section"),
        probe: ProbeKind::Get {
            url: "https://gtexportal.org/api/v2/",
        },
    },
    SourceDescriptor {
        api: "DGIdb",
        affects: Some("gene druggability section"),
        probe: ProbeKind::PostJson {
            url: "https://dgidb.org/api/graphql",
            payload: r#"{"query":"query { __typename }"}"#,
        },
    },
    SourceDescriptor {
        api: "ClinGen",
        affects: Some("gene clingen section"),
        probe: ProbeKind::Get {
            url: "https://search.clinicalgenome.org/api/genes/look/BRAF",
        },
    },
    SourceDescriptor {
        api: "gnomAD",
        affects: Some("gene constraint section"),
        probe: ProbeKind::PostJson {
            url: "https://gnomad.broadinstitute.org/api",
            payload: r#"{"query":"query { __typename }"}"#,
        },
    },
    SourceDescriptor {
        api: "UniProt",
        affects: Some("gene protein summary and protein detail sections"),
        probe: ProbeKind::Get {
            url: "https://rest.uniprot.org/uniprotkb/P15056.json",
        },
    },
    SourceDescriptor {
        api: "QuickGO",
        affects: Some("gene go terms and protein annotation sections"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/QuickGO/services/annotation/search?geneProductId=P15056&limit=5",
        },
    },
    SourceDescriptor {
        api: "STRING",
        affects: Some("gene interactions and protein interaction sections"),
        probe: ProbeKind::Get {
            url: "https://string-db.org/api/json/network?identifiers=BRAF&species=9606&limit=5",
        },
    },
    SourceDescriptor {
        api: "Reactome",
        affects: Some("pathway search and disease pathway sections"),
        probe: ProbeKind::Get {
            url: "https://reactome.org/ContentService/search/query?query=MAPK&species=Homo%20sapiens&pageSize=1",
        },
    },
    SourceDescriptor {
        api: "KEGG",
        affects: Some("pathway search and detail sections"),
        probe: ProbeKind::Get {
            url: "https://rest.kegg.jp/find/pathway/MAPK",
        },
    },
    SourceDescriptor {
        api: "WikiPathways",
        affects: Some("pathway search and WikiPathways detail/genes sections"),
        probe: ProbeKind::Get {
            url: "https://www.wikipathways.org/json/findPathwaysByText.json",
        },
    },
    SourceDescriptor {
        api: "g:Profiler",
        affects: Some("gene enrichment (biomcp enrich)"),
        probe: ProbeKind::PostJson {
            url: "https://biit.cs.ut.ee/gprofiler/api/gost/profile/",
            payload: r#"{"organism":"hsapiens","query":["BRAF"]}"#,
        },
    },
    SourceDescriptor {
        api: "OpenTargets",
        affects: Some("gene druggability, drug target, and disease association sections"),
        probe: ProbeKind::PostJson {
            url: "https://api.platform.opentargets.org/api/v4/graphql",
            payload: r#"{"query":"query { drug(chemblId: \"CHEMBL25\") { id name } }"}"#,
        },
    },
    SourceDescriptor {
        api: "ChEMBL",
        affects: Some("drug targets and indications sections"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/chembl/api/data/molecule/CHEMBL25.json",
        },
    },
    SourceDescriptor {
        api: "HPA",
        affects: Some("gene protein tissue expression and localization section"),
        probe: ProbeKind::Get {
            url: "https://www.proteinatlas.org/ENSG00000157764.xml",
        },
    },
    SourceDescriptor {
        api: "InterPro",
        affects: Some("protein domains section"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/interpro/api/entry/interpro/protein/uniprot/P15056/?page_size=5",
        },
    },
    SourceDescriptor {
        api: "ComplexPortal",
        affects: Some("protein complex membership section"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/intact/complex-ws/search/P15056?number=25&filters=species_f:(%22Homo%20sapiens%22)",
        },
    },
    SourceDescriptor {
        api: "OLS4",
        affects: Some("discover command concept resolution"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/ols4/api/search?q=BRCA1&rows=1&groupField=iri&ontology=hgnc",
        },
    },
    SourceDescriptor {
        api: "UMLS",
        affects: Some("discover command clinical crosswalk enrichment"),
        probe: ProbeKind::AuthQueryParam {
            url: "https://uts-ws.nlm.nih.gov/rest/search/current?string=BRCA1&pageSize=1",
            env_var: "UMLS_API_KEY",
            param_name: "apiKey",
        },
    },
    SourceDescriptor {
        api: "MedlinePlus",
        affects: Some("discover command plain-language disease and symptom context"),
        probe: ProbeKind::Get {
            url: "https://wsearch.nlm.nih.gov/ws/query?db=healthTopics&term=chest+pain&retmax=1",
        },
    },
    SourceDescriptor {
        api: "cBioPortal",
        affects: Some("cohort frequency section"),
        probe: ProbeKind::Get {
            url: "https://www.cbioportal.org/api/studies?projection=SUMMARY&pageSize=1",
        },
    },
];

const EMA_LOCAL_DATA_AFFECTS: &str = "default plain-name drug search plus search/get drug --region eu|all and EU regulatory/safety/shortage sections";
const CVX_LOCAL_DATA_AFFECTS: &str = "EMA vaccine identity bridge for plain-name drug search";
const WHO_LOCAL_DATA_AFFECTS: &str = "default plain-name drug search plus search/get drug --region who|all and WHO regulatory sections";
const DDINTER_LOCAL_DATA_AFFECTS: &str =
    "drug interactions helper plus get drug interactions section";
const GTR_LOCAL_DATA_AFFECTS: &str =
    "search/get diagnostic and local GTR-backed diagnostic routing";
const WHO_IVD_LOCAL_DATA_AFFECTS: &str =
    "search/get diagnostic and local WHO IVD-backed infectious-disease diagnostic routing";

fn health_sources() -> &'static [SourceDescriptor] {
    HEALTH_SOURCES
}

#[cfg_attr(not(test), allow(dead_code))]
fn affects_for_api(api: &str) -> Option<&'static str> {
    health_sources()
        .iter()
        .find(|source| source.api == api)
        .and_then(|source| source.affects)
}

fn health_row(
    api: &str,
    status: String,
    latency: String,
    affects: Option<&'static str>,
    key_configured: Option<bool>,
) -> HealthRow {
    HealthRow {
        api: api.to_string(),
        status,
        latency,
        affects: affects.map(str::to_string),
        key_configured,
    }
}

fn outcome(row: HealthRow, class: ProbeClass) -> ProbeOutcome {
    ProbeOutcome { row, class }
}

fn configured_key(env_var: &str) -> Option<String> {
    std::env::var(env_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn excluded_outcome(api: &str, env_var: &str, affects: Option<&'static str>) -> ProbeOutcome {
    outcome(
        health_row(
            api,
            format!("excluded (set {env_var})"),
            "n/a".into(),
            affects,
            Some(false),
        ),
        ProbeClass::Excluded,
    )
}

fn transport_error_latency(start: Instant, err: &reqwest::Error) -> String {
    let elapsed = start.elapsed().as_millis();
    if err.is_timeout() {
        format!("{elapsed}ms (timeout)")
    } else if err.is_connect() {
        format!("{elapsed}ms (connect)")
    } else {
        format!("{elapsed}ms (error)")
    }
}

fn api_error_latency(start: Instant, err: &BioMcpError) -> String {
    let elapsed = start.elapsed().as_millis();
    match err {
        BioMcpError::Api { message, .. } if message.contains("connect failed") => {
            format!("{elapsed}ms (connect)")
        }
        _ => format!("{elapsed}ms (error)"),
    }
}

async fn send_request(
    api: &str,
    affects: Option<&'static str>,
    request: reqwest::RequestBuilder,
    key_configured: Option<bool>,
) -> ProbeOutcome {
    let start = Instant::now();
    let response = request.send().await;

    match response {
        Ok(response) => {
            let status = response.status();
            let elapsed = start.elapsed().as_millis();
            if status.is_success() {
                outcome(
                    health_row(
                        api,
                        "ok".into(),
                        format!("{elapsed}ms"),
                        None,
                        key_configured,
                    ),
                    ProbeClass::Healthy,
                )
            } else {
                outcome(
                    health_row(
                        api,
                        "error".into(),
                        format!("{elapsed}ms (HTTP {})", status.as_u16()),
                        affects,
                        key_configured,
                    ),
                    ProbeClass::Error,
                )
            }
        }
        Err(err) => outcome(
            health_row(
                api,
                "error".into(),
                transport_error_latency(start, &err),
                affects,
                key_configured,
            ),
            ProbeClass::Error,
        ),
    }
}

async fn check_get(
    client: reqwest::Client,
    api: &str,
    url: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    send_request(api, affects, client.get(url), None).await
}

async fn check_post_json(
    client: reqwest::Client,
    api: &str,
    url: &str,
    payload: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    send_request(
        api,
        affects,
        client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(payload.to_string()),
        None,
    )
    .await
}

async fn check_auth_get(
    client: reqwest::Client,
    api: &str,
    url: &str,
    env_var: &str,
    header_name: &str,
    header_value_prefix: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let Some(key) = configured_key(env_var) else {
        return excluded_outcome(api, env_var, affects);
    };

    let header_value = format!("{header_value_prefix}{key}");

    send_request(
        api,
        affects,
        client.get(url).header(header_name, header_value),
        Some(true),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn check_optional_auth_get(
    client: reqwest::Client,
    api: &str,
    url: &str,
    env_var: &str,
    header_name: &str,
    header_value_prefix: &str,
    unauthenticated_ok_status: &str,
    authenticated_ok_status: &str,
    unauthenticated_rate_limited_status: Option<&str>,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let key = configured_key(env_var);
    let key_configured = Some(key.is_some());
    let request = match key {
        Some(key) => client
            .get(url)
            .header(header_name, format!("{header_value_prefix}{key}")),
        None => client.get(url),
    };
    let success_status = if key_configured == Some(true) {
        authenticated_ok_status
    } else {
        unauthenticated_ok_status
    };
    let start = Instant::now();
    let error_outcome = |latency: String| {
        outcome(
            health_row(api, "error".into(), latency, affects, key_configured),
            ProbeClass::Error,
        )
    };

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let elapsed = start.elapsed().as_millis();
            if status.is_success() {
                outcome(
                    health_row(
                        api,
                        success_status.to_string(),
                        format!("{elapsed}ms"),
                        None,
                        key_configured,
                    ),
                    ProbeClass::Healthy,
                )
            } else if key_configured == Some(false)
                && status == reqwest::StatusCode::TOO_MANY_REQUESTS
                && let Some(status_message) = unauthenticated_rate_limited_status
            {
                outcome(
                    health_row(
                        api,
                        status_message.to_string(),
                        format!("{elapsed}ms"),
                        None,
                        key_configured,
                    ),
                    ProbeClass::Healthy,
                )
            } else {
                error_outcome(format!("{elapsed}ms (HTTP {})", status.as_u16()))
            }
        }
        Err(err) => error_outcome(transport_error_latency(start, &err)),
    }
}

async fn check_auth_query_param(
    client: reqwest::Client,
    api: &str,
    url: &str,
    env_var: &str,
    param_name: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let Some(key) = configured_key(env_var) else {
        return excluded_outcome(api, env_var, affects);
    };

    let req = match reqwest::Url::parse(url) {
        Ok(mut parsed) => {
            parsed.query_pairs_mut().append_pair(param_name, &key);
            client.get(parsed)
        }
        Err(err) => {
            return outcome(
                health_row(
                    api,
                    "error".into(),
                    format!("invalid url: {err}"),
                    affects,
                    Some(true),
                ),
                ProbeClass::Error,
            );
        }
    };

    send_request(api, affects, req, Some(true)).await
}

#[allow(clippy::too_many_arguments)]
async fn check_auth_post_json(
    client: reqwest::Client,
    api: &str,
    url: &str,
    payload: &str,
    env_var: &str,
    header_name: &str,
    header_value_prefix: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let Some(key) = configured_key(env_var) else {
        return excluded_outcome(api, env_var, affects);
    };

    let header_value = format!("{header_value_prefix}{key}");

    send_request(
        api,
        affects,
        client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(header_name, header_value)
            .body(payload.to_string()),
        Some(true),
    )
    .await
}

async fn check_alphagenome_connect(
    api: &str,
    env_var: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let Some(_key) = configured_key(env_var) else {
        return excluded_outcome(api, env_var, affects);
    };

    let start = Instant::now();

    match crate::sources::alphagenome::AlphaGenomeClient::new().await {
        Ok(_) => outcome(
            health_row(
                api,
                "ok".into(),
                format!("{}ms", start.elapsed().as_millis()),
                None,
                Some(true),
            ),
            ProbeClass::Healthy,
        ),
        Err(err) => outcome(
            health_row(
                api,
                "error".into(),
                api_error_latency(start, &err),
                affects,
                Some(true),
            ),
            ProbeClass::Error,
        ),
    }
}

async fn check_vaers_query(api: &str, affects: Option<&'static str>) -> ProbeOutcome {
    let start = Instant::now();
    let client = match crate::sources::vaers::VaersClient::new() {
        Ok(client) => client,
        Err(err) => {
            return outcome(
                health_row(
                    api,
                    "error".into(),
                    api_error_latency(start, &err),
                    affects,
                    None,
                ),
                ProbeClass::Error,
            );
        }
    };

    match client.health_check().await {
        Ok(()) => outcome(
            health_row(
                api,
                "ok".into(),
                format!("{}ms", start.elapsed().as_millis()),
                None,
                None,
            ),
            ProbeClass::Healthy,
        ),
        Err(err) => outcome(
            health_row(
                api,
                "error".into(),
                api_error_latency(start, &err),
                affects,
                None,
            ),
            ProbeClass::Error,
        ),
    }
}

fn local_data_is_stale(root: &Path, files: &[&str], stale_after: Duration) -> bool {
    files.iter().any(|file| {
        root.join(file)
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| std::time::SystemTime::now().duration_since(modified).ok())
            .is_some_and(|age| age >= stale_after)
    })
}

fn local_data_outcome<F>(
    label: &str,
    root: &Path,
    env_configured: bool,
    required_files: &[&str],
    stale_after: Duration,
    affects: &'static str,
    missing_files: F,
) -> ProbeOutcome
where
    F: for<'a> Fn(&'a Path, &[&'a str]) -> Vec<&'a str>,
{
    let api = format!("{label} ({})", root.display());
    let missing = missing_files(root, required_files);

    if missing.is_empty() {
        let stale = local_data_is_stale(root, required_files, stale_after);
        let (status, class, row_affects) = match (env_configured, stale) {
            (true, false) => ("configured".to_string(), ProbeClass::Healthy, None),
            (true, true) => (
                "configured (stale)".to_string(),
                ProbeClass::Warning,
                Some(affects),
            ),
            (false, false) => (
                "available (default path)".to_string(),
                ProbeClass::Healthy,
                None,
            ),
            (false, true) => (
                "available (default path, stale)".to_string(),
                ProbeClass::Warning,
                Some(affects),
            ),
        };
        return outcome(
            health_row(&api, status, "n/a".into(), row_affects, None),
            class,
        );
    }

    if !env_configured && missing.len() == required_files.len() {
        return outcome(
            health_row(
                &api,
                "not configured".into(),
                "n/a".into(),
                Some(affects),
                None,
            ),
            ProbeClass::Excluded,
        );
    }

    outcome(
        health_row(
            &api,
            format!("error (missing: {})", missing.join(", ")),
            "n/a".into(),
            Some(affects),
            None,
        ),
        ProbeClass::Error,
    )
}

fn ema_local_data_outcome(root: &Path, env_configured: bool) -> ProbeOutcome {
    local_data_outcome(
        "EMA local data",
        root,
        env_configured,
        crate::sources::ema::EMA_REQUIRED_FILES,
        crate::sources::ema::EMA_STALE_AFTER,
        EMA_LOCAL_DATA_AFFECTS,
        crate::sources::ema::ema_missing_files,
    )
}

fn check_ema_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_EMA_DIR").is_some();
    let root = crate::sources::ema::resolve_ema_root();
    ema_local_data_outcome(&root, env_configured)
}

fn cvx_local_data_outcome(root: &Path, env_configured: bool) -> ProbeOutcome {
    local_data_outcome(
        "CDC CVX/MVX local data",
        root,
        env_configured,
        crate::sources::cvx::CVX_REQUIRED_FILES,
        crate::sources::cvx::CVX_STALE_AFTER,
        CVX_LOCAL_DATA_AFFECTS,
        crate::sources::cvx::cvx_missing_files,
    )
}

fn check_cvx_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_CVX_DIR").is_some();
    let root = crate::sources::cvx::resolve_cvx_root();
    cvx_local_data_outcome(&root, env_configured)
}

fn ddinter_local_data_outcome(root: &Path, env_configured: bool) -> ProbeOutcome {
    local_data_outcome(
        "DDInter local data",
        root,
        env_configured,
        crate::sources::ddinter::DDINTER_REQUIRED_FILES,
        crate::sources::ddinter::DDINTER_STALE_AFTER,
        DDINTER_LOCAL_DATA_AFFECTS,
        crate::sources::ddinter::ddinter_missing_files,
    )
}

fn check_ddinter_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_DDINTER_DIR").is_some();
    let root = crate::sources::ddinter::resolve_ddinter_root();
    ddinter_local_data_outcome(&root, env_configured)
}

fn who_local_data_outcome(root: &Path, env_configured: bool) -> ProbeOutcome {
    local_data_outcome(
        "WHO Prequalification local data",
        root,
        env_configured,
        crate::sources::who_pq::WHO_PQ_REQUIRED_FILES,
        crate::sources::who_pq::WHO_PQ_STALE_AFTER,
        WHO_LOCAL_DATA_AFFECTS,
        crate::sources::who_pq::who_pq_missing_files,
    )
}

fn check_who_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_WHO_DIR").is_some();
    let root = crate::sources::who_pq::resolve_who_pq_root();
    who_local_data_outcome(&root, env_configured)
}

fn gtr_local_data_outcome(root: &Path, env_configured: bool) -> ProbeOutcome {
    local_data_outcome(
        "GTR local data",
        root,
        env_configured,
        &crate::sources::gtr::GTR_REQUIRED_FILES,
        crate::sources::gtr::GTR_STALE_AFTER,
        GTR_LOCAL_DATA_AFFECTS,
        |root, required_files| {
            let required = required_files.to_vec();
            crate::sources::gtr::gtr_missing_files(root)
                .into_iter()
                .filter_map(|missing| {
                    required
                        .iter()
                        .copied()
                        .find(|expected| *expected == missing.as_str())
                })
                .collect()
        },
    )
}

fn check_gtr_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_GTR_DIR").is_some();
    let root = crate::sources::gtr::resolve_gtr_root();
    gtr_local_data_outcome(&root, env_configured)
}

fn who_ivd_local_data_outcome(root: &Path, env_configured: bool) -> ProbeOutcome {
    local_data_outcome(
        "WHO IVD local data",
        root,
        env_configured,
        crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES,
        crate::sources::who_ivd::WHO_IVD_STALE_AFTER,
        WHO_IVD_LOCAL_DATA_AFFECTS,
        crate::sources::who_ivd::who_ivd_missing_files,
    )
}

fn check_who_ivd_local_data() -> ProbeOutcome {
    let env_configured = configured_key("BIOMCP_WHO_IVD_DIR").is_some();
    let root = crate::sources::who_ivd::resolve_who_ivd_root();
    who_ivd_local_data_outcome(&root, env_configured)
}

async fn probe_source(client: reqwest::Client, source: &SourceDescriptor) -> ProbeOutcome {
    match source.probe {
        ProbeKind::Get { url } => check_get(client, source.api, url, source.affects).await,
        ProbeKind::PostJson { url, payload } => {
            check_post_json(client, source.api, url, payload, source.affects).await
        }
        ProbeKind::AuthGet {
            url,
            env_var,
            header_name,
            header_value_prefix,
        } => {
            check_auth_get(
                client,
                source.api,
                url,
                env_var,
                header_name,
                header_value_prefix,
                source.affects,
            )
            .await
        }
        ProbeKind::OptionalAuthGet {
            url,
            env_var,
            header_name,
            header_value_prefix,
            unauthenticated_ok_status,
            authenticated_ok_status,
            unauthenticated_rate_limited_status,
        } => {
            check_optional_auth_get(
                client,
                source.api,
                url,
                env_var,
                header_name,
                header_value_prefix,
                unauthenticated_ok_status,
                authenticated_ok_status,
                unauthenticated_rate_limited_status,
                source.affects,
            )
            .await
        }
        ProbeKind::AuthQueryParam {
            url,
            env_var,
            param_name,
        } => {
            check_auth_query_param(client, source.api, url, env_var, param_name, source.affects)
                .await
        }
        ProbeKind::AuthPostJson {
            url,
            payload,
            env_var,
            header_name,
            header_value_prefix,
        } => {
            check_auth_post_json(
                client,
                source.api,
                url,
                payload,
                env_var,
                header_name,
                header_value_prefix,
                source.affects,
            )
            .await
        }
        ProbeKind::AlphaGenomeConnect { env_var } => {
            check_alphagenome_connect(source.api, env_var, source.affects).await
        }
        ProbeKind::VaersQuery => check_vaers_query(source.api, source.affects).await,
    }
}

const HEALTH_API_PROBE_CONCURRENCY_LIMIT: usize = 16;
const HEALTH_API_PROBE_TIMEOUT: Duration = Duration::from_secs(12);

async fn run_buffered_in_order<T, O, F, Fut, I>(
    items: I,
    concurrency_limit: usize,
    runner: F,
) -> Vec<O>
where
    I: IntoIterator<Item = T>,
    F: FnMut(T) -> Fut,
    Fut: std::future::Future<Output = O>,
{
    assert!(
        concurrency_limit > 0,
        "concurrency_limit must be greater than zero"
    );
    stream::iter(items)
        .map(runner)
        .buffered(concurrency_limit)
        .collect()
        .await
}

fn timeout_key_configured(source: SourceDescriptor) -> Option<bool> {
    match source.probe {
        ProbeKind::AuthGet { .. }
        | ProbeKind::AuthQueryParam { .. }
        | ProbeKind::AuthPostJson { .. }
        | ProbeKind::AlphaGenomeConnect { .. } => Some(true),
        ProbeKind::OptionalAuthGet { env_var, .. } => Some(configured_key(env_var).is_some()),
        ProbeKind::Get { .. } | ProbeKind::PostJson { .. } | ProbeKind::VaersQuery => None,
    }
}

fn timed_out_probe_outcome(source: SourceDescriptor, timeout: Duration) -> ProbeOutcome {
    outcome(
        health_row(
            source.api,
            "error".into(),
            format!("{}ms (timeout)", timeout.as_millis()),
            source.affects,
            timeout_key_configured(source),
        ),
        ProbeClass::Error,
    )
}

async fn probe_source_with_timeout_for_test(
    client: reqwest::Client,
    source: SourceDescriptor,
    timeout: Duration,
) -> ProbeOutcome {
    match tokio::time::timeout(timeout, probe_source(client, &source)).await {
        Ok(outcome) => outcome,
        Err(_) => timed_out_probe_outcome(source, timeout),
    }
}

async fn probe_source_with_timeout(
    client: reqwest::Client,
    source: SourceDescriptor,
) -> ProbeOutcome {
    probe_source_with_timeout_for_test(client, source, HEALTH_API_PROBE_TIMEOUT).await
}

async fn run_api_probes(client: reqwest::Client) -> Vec<ProbeOutcome> {
    run_buffered_in_order(
        health_sources().iter().copied(),
        HEALTH_API_PROBE_CONCURRENCY_LIMIT,
        move |source| probe_source_with_timeout(client.clone(), source),
    )
    .await
}

fn health_http_client() -> Result<reqwest::Client, BioMcpError> {
    static HEALTH_HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

    if let Some(client) = HEALTH_HTTP_CLIENT.get() {
        return Ok(client.clone());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(BioMcpError::HttpClientInit)?;

    match HEALTH_HTTP_CLIENT.set(client.clone()) {
        Ok(()) => Ok(client),
        Err(_) => HEALTH_HTTP_CLIENT
            .get()
            .cloned()
            .ok_or_else(|| BioMcpError::Api {
                api: "health".into(),
                message: "Health HTTP client initialization race".into(),
            }),
    }
}

async fn check_cache_dir() -> ProbeOutcome {
    let dir = match crate::cache::resolve_cache_config() {
        Ok(config) => config.cache_root,
        Err(err) => {
            return outcome(
                HealthRow {
                    api: "Cache dir".into(),
                    status: "error".into(),
                    latency: err.to_string(),
                    affects: Some("local cache-backed lookups and downloads".into()),
                    key_configured: None,
                },
                ProbeClass::Error,
            );
        }
    };
    probe_cache_dir(&dir).await
}

async fn check_cache_limits() -> ProbeOutcome {
    let config = match crate::cache::resolve_cache_config() {
        Ok(config) => config,
        Err(err) => {
            return cache_limits_error_outcome(err.to_string());
        }
    };

    match tokio::task::spawn_blocking(move || {
        check_cache_limits_with(
            || Ok(config),
            crate::cache::snapshot_cache,
            crate::cache::inspect_filesystem_space,
        )
    })
    .await
    {
        Ok(outcome) => outcome,
        Err(err) => cache_limits_error_outcome(err.to_string()),
    }
}

async fn probe_cache_dir(dir: &Path) -> ProbeOutcome {
    let start = Instant::now();
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let probe = dir.join(format!(".biomcp-healthcheck-{suffix}.tmp"));

    let result = async {
        tokio::fs::create_dir_all(&dir).await?;
        tokio::fs::write(&probe, b"ok").await?;
        match tokio::fs::remove_file(&probe).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }
    .await;

    match result {
        Ok(()) => outcome(
            HealthRow {
                api: format!("Cache dir ({})", dir.display()),
                status: "ok".into(),
                latency: format!("{}ms", start.elapsed().as_millis()),
                affects: None,
                key_configured: None,
            },
            ProbeClass::Healthy,
        ),
        Err(err) => outcome(
            HealthRow {
                api: format!("Cache dir ({})", dir.display()),
                status: "error".into(),
                latency: format!("{:?}", err.kind()),
                affects: Some("local cache-backed lookups and downloads".into()),
                key_configured: None,
            },
            ProbeClass::Error,
        ),
    }
}

fn report_from_outcomes(outcomes: Vec<ProbeOutcome>) -> HealthReport {
    let healthy = outcomes
        .iter()
        .filter(|outcome| outcome.class == ProbeClass::Healthy)
        .count();
    let warning = outcomes
        .iter()
        .filter(|outcome| outcome.class == ProbeClass::Warning)
        .count();
    let excluded = outcomes
        .iter()
        .filter(|outcome| outcome.class == ProbeClass::Excluded)
        .count();
    let rows = outcomes
        .into_iter()
        .map(|outcome| outcome.row)
        .collect::<Vec<_>>();

    HealthReport {
        healthy,
        warning,
        excluded,
        total: rows.len(),
        rows,
    }
}

/// Runs connectivity checks for configured upstream APIs and local EMA/CVX/WHO/GTR/WHO IVD/cache readiness.
///
/// # Errors
///
/// Returns an error when the shared HTTP client cannot be created.
pub async fn check(apis_only: bool) -> Result<HealthReport, BioMcpError> {
    let client = health_http_client()?;
    let mut outcomes = run_api_probes(client).await;

    if !apis_only {
        outcomes.push(check_ema_local_data());
        outcomes.push(check_cvx_local_data());
        outcomes.push(check_ddinter_local_data());
        outcomes.push(check_who_local_data());
        outcomes.push(check_gtr_local_data());
        outcomes.push(check_who_ivd_local_data());
        outcomes.push(check_cache_dir().await);
        outcomes.push(check_cache_limits().await);
    }

    Ok(report_from_outcomes(outcomes))
}

fn check_cache_limits_with<R, S, I>(
    resolve_config: R,
    snapshotter: S,
    inspect_space: I,
) -> ProbeOutcome
where
    R: FnOnce() -> Result<crate::cache::ResolvedCacheConfig, BioMcpError>,
    S: FnOnce(&Path) -> Result<crate::cache::CacheSnapshot, crate::cache::CachePlannerError>,
    I: FnOnce(&Path) -> Result<crate::cache::FilesystemSpace, BioMcpError>,
{
    let config = match resolve_config() {
        Ok(config) => config,
        Err(err) => return cache_limits_error_outcome(err.to_string()),
    };
    let cache_path = config.cache_root.join("http");
    let snapshot = match snapshotter(&cache_path) {
        Ok(snapshot) => snapshot,
        Err(err) => return cache_limits_error_outcome(err.to_string()),
    };
    let space = match inspect_space(&config.cache_root) {
        Ok(space) => space,
        Err(err) => return cache_limits_error_outcome(err.to_string()),
    };
    let evaluation = crate::cache::evaluate_cache_limits(&snapshot, &config, space);

    if evaluation.over_max_size || evaluation.below_min_disk_free {
        return outcome(
            HealthRow {
                api: "Cache limits".into(),
                status: "warning".into(),
                latency: cache_limits_warning_message(&config, space, &evaluation),
                affects: None,
                key_configured: None,
            },
            ProbeClass::Warning,
        );
    }

    outcome(
        HealthRow {
            api: "Cache limits".into(),
            status: "ok".into(),
            latency: "within limits".into(),
            affects: None,
            key_configured: None,
        },
        ProbeClass::Healthy,
    )
}

fn cache_limits_warning_message(
    config: &crate::cache::ResolvedCacheConfig,
    space: crate::cache::FilesystemSpace,
    evaluation: &crate::cache::CacheLimitEvaluation,
) -> String {
    let mut clauses = Vec::new();
    if evaluation.over_max_size {
        clauses.push(format!(
            "referenced bytes {} exceed max_size {}",
            evaluation.usage.referenced_blob_bytes, config.max_size
        ));
    }
    if evaluation.below_min_disk_free {
        clauses.push(format!(
            "available disk {} is below min_disk_free {}",
            ByteSize(space.available_bytes),
            config.min_disk_free.display()
        ));
    }
    format!("{}; run biomcp cache clean", clauses.join("; "))
}

fn cache_limits_error_outcome(message: String) -> ProbeOutcome {
    outcome(
        HealthRow {
            api: "Cache limits".into(),
            status: "error".into(),
            latency: message,
            affects: None,
            key_configured: None,
        },
        ProbeClass::Error,
    )
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use ssri::Integrity;
    use tokio::sync::MutexGuard;
    use wiremock::matchers::{body_string_contains, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{
        CVX_LOCAL_DATA_AFFECTS, EMA_LOCAL_DATA_AFFECTS, GTR_LOCAL_DATA_AFFECTS,
        HEALTH_API_PROBE_CONCURRENCY_LIMIT, HealthReport, HealthRow, ProbeClass, ProbeKind,
        ProbeOutcome, SourceDescriptor, WHO_IVD_LOCAL_DATA_AFFECTS, WHO_LOCAL_DATA_AFFECTS,
        affects_for_api, check_cache_dir, check_cache_limits_with, cvx_local_data_outcome,
        ema_local_data_outcome, gtr_local_data_outcome, health_sources, probe_cache_dir,
        probe_source, probe_source_with_timeout_for_test, report_from_outcomes,
        run_buffered_in_order, who_ivd_local_data_outcome, who_local_data_outcome,
    };
    use crate::cache::{
        CacheBlob, CacheConfigOrigins, CacheEntry, CachePlannerError, CacheSnapshot, ConfigOrigin,
        DiskFreeThreshold, FilesystemSpace, ResolvedCacheConfig,
    };
    use crate::test_support::{TempDirGuard, set_env_var};

    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("health test runtime")
            .block_on(future)
    }

    fn env_lock() -> MutexGuard<'static, ()> {
        crate::test_support::env_lock().blocking_lock()
    }

    fn fixture_ema_root() -> TempDirGuard {
        let root = TempDirGuard::new("health-ema");
        write_ema_files(root.path(), crate::sources::ema::EMA_REQUIRED_FILES);
        root
    }

    fn write_ema_files(root: &Path, files: &[&str]) {
        for file in files {
            std::fs::write(root.join(file), b"{}").expect("write EMA fixture file");
        }
    }

    fn write_who_files(root: &Path, files: &[&str]) {
        for file in files {
            let bytes: &[u8] = match *file {
                crate::sources::who_pq::WHO_PQ_CSV_FILE => {
                    b"WHO Reference Number,INN, Dosage Form and Strength,Product Type,Therapeutic Area,Applicant,Dosage Form,Basis of Listing,Basis of alternative listing,Date of Prequalification\n"
                }
                crate::sources::who_pq::WHO_PQ_API_CSV_FILE => {
                    b"WHO Product ID,INN,Grade,Therapeutic area,Applicant organization,Date of prequalification,Confirmation of Prequalification Document Date\n"
                }
                crate::sources::who_pq::WHO_VACCINES_CSV_FILE => {
                    b"Date of Prequalification ,Vaccine Type,Commercial Name,Presentation,No. of doses,Manufacturer,Responsible NRA\n"
                }
                other => panic!("unexpected WHO fixture file: {other}"),
            };
            std::fs::write(root.join(file), bytes).expect("write WHO fixture file");
        }
    }

    fn write_cvx_files(root: &Path, files: &[&str]) {
        for file in files {
            let bytes: &[u8] = match *file {
                crate::sources::cvx::CVX_FILE => {
                    b"62|HPV, quadrivalent|human papilloma virus vaccine, quadrivalent||Active|False|2020/06/02\n"
                }
                crate::sources::cvx::TRADENAME_FILE => {
                    b"GARDASIL|HPV, quadrivalent|62|Merck and Co., Inc.|MSD|Active|Active|2010/05/28|\n"
                }
                crate::sources::cvx::MVX_FILE => {
                    b"MSD|Merck and Co., Inc.||Active|2012/10/18\n"
                }
                other => panic!("unexpected CVX fixture file: {other}"),
            };
            std::fs::write(root.join(file), bytes).expect("write CVX fixture file");
        }
    }

    fn write_gtr_files(root: &Path, files: &[&str]) {
        for file in files {
            match *file {
                crate::sources::gtr::GTR_TEST_VERSION_FILE => std::fs::write(
                    root.join(file),
                    include_bytes!("../../spec/fixtures/gtr/test_version.gz"),
                )
                .expect("write GTR gzip fixture"),
                crate::sources::gtr::GTR_CONDITION_GENE_FILE => std::fs::write(
                    root.join(file),
                    include_str!("../../spec/fixtures/gtr/test_condition_gene.txt"),
                )
                .expect("write GTR tsv fixture"),
                other => panic!("unexpected GTR fixture file: {other}"),
            }
        }
    }

    fn write_who_ivd_files(root: &Path, files: &[&str]) {
        for file in files {
            let bytes: &[u8] = match *file {
                crate::sources::who_ivd::WHO_IVD_CSV_FILE => {
                    b"Product name,Product Code,WHO Product ID,Assay Format,Regulatory Version,Manufacturer name,Pathogen/Disease/Marker,Year prequalification\n"
                }
                other => panic!("unexpected WHO IVD fixture file: {other}"),
            };
            std::fs::write(root.join(file), bytes).expect("write WHO IVD fixture file");
        }
    }

    fn set_stale_mtime_with_age(path: &Path, age: std::time::Duration) {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect("fixture file should open");
        file.set_modified(
            std::time::SystemTime::now()
                .checked_sub(age)
                .expect("stale time should be valid"),
        )
        .expect("mtime should update");
    }

    fn set_stale_mtime(path: &Path) {
        set_stale_mtime_with_age(path, std::time::Duration::from_secs(73 * 60 * 60));
    }

    fn set_stale_ema_mtimes(root: &Path) {
        for file_name in crate::sources::ema::EMA_REQUIRED_FILES {
            set_stale_mtime(&root.join(file_name));
        }
    }

    fn set_fresh_ema_mtimes(root: &Path) {
        for file_name in crate::sources::ema::EMA_REQUIRED_FILES {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .open(root.join(file_name))
                .expect("fixture file should open");
            file.set_modified(std::time::SystemTime::now())
                .expect("mtime should update");
        }
    }

    fn assert_cache_dir_affects(value: Option<&str>) {
        assert_eq!(value, Some("local cache-backed lookups and downloads"));
    }

    fn assert_millisecond_latency(value: &str) {
        let digits = value
            .strip_suffix("ms")
            .expect("latency should end with ms");
        assert!(
            !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()),
            "unexpected latency: {value}"
        );
    }

    fn update_max(target: &AtomicUsize, candidate: usize) {
        let mut observed = target.load(Ordering::SeqCst);
        while candidate > observed {
            match target.compare_exchange(observed, candidate, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(actual) => observed = actual,
            }
        }
    }

    fn semantic_scholar_source(url: &'static str) -> SourceDescriptor {
        let source = health_sources()
            .iter()
            .find(|source| source.api == "Semantic Scholar")
            .expect("semantic scholar health source");
        let ProbeKind::OptionalAuthGet {
            env_var,
            header_name,
            header_value_prefix,
            unauthenticated_ok_status,
            authenticated_ok_status,
            unauthenticated_rate_limited_status,
            ..
        } = source.probe
        else {
            panic!("semantic scholar should use optional auth get");
        };

        SourceDescriptor {
            api: source.api,
            affects: source.affects,
            probe: ProbeKind::OptionalAuthGet {
                url,
                env_var,
                header_name,
                header_value_prefix,
                unauthenticated_ok_status,
                authenticated_ok_status,
                unauthenticated_rate_limited_status,
            },
        }
    }

    fn test_integrity(bytes: &[u8]) -> Integrity {
        Integrity::from(bytes)
    }

    fn test_entry(key: &str, bytes: &[u8], time_ms: u128) -> CacheEntry {
        CacheEntry {
            key: key.to_string(),
            integrity: test_integrity(bytes),
            time_ms,
            size_bytes: bytes.len() as u64,
        }
    }

    fn test_blob(label: &str, bytes: &[u8], refcount: usize) -> CacheBlob {
        CacheBlob {
            integrity: test_integrity(bytes),
            path: PathBuf::from(format!("content-v2/mock/{label}.blob")),
            size_bytes: bytes.len() as u64,
            refcount,
        }
    }

    fn test_snapshot(
        cache_path: impl Into<PathBuf>,
        entries: Vec<CacheEntry>,
        blobs: Vec<CacheBlob>,
    ) -> CacheSnapshot {
        CacheSnapshot {
            cache_path: cache_path.into(),
            entries,
            blobs,
        }
    }

    fn test_config(
        cache_root: impl Into<PathBuf>,
        max_size: u64,
        min_disk_free: DiskFreeThreshold,
    ) -> ResolvedCacheConfig {
        ResolvedCacheConfig {
            cache_root: cache_root.into(),
            max_size,
            min_disk_free,
            max_age: Duration::from_secs(86_400),
            origins: CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            },
        }
    }

    #[test]
    fn markdown_shows_affects_column_when_present() {
        let report = HealthReport {
            healthy: 1,
            warning: 0,
            excluded: 0,
            total: 2,
            rows: vec![
                HealthRow {
                    api: "MyGene".into(),
                    status: "ok".into(),
                    latency: "10ms".into(),
                    affects: None,
                    key_configured: None,
                },
                HealthRow {
                    api: "OpenFDA".into(),
                    status: "error".into(),
                    latency: "timeout".into(),
                    affects: Some("adverse-event search".into()),
                    key_configured: None,
                },
            ],
        };
        let md = report.to_markdown();
        assert!(md.contains("| API | Status | Latency | Affects |"));
        assert!(md.contains("adverse-event search"));
    }

    #[test]
    fn markdown_omits_affects_column_when_all_healthy() {
        let report = HealthReport {
            healthy: 2,
            warning: 0,
            excluded: 0,
            total: 2,
            rows: vec![
                HealthRow {
                    api: "MyGene".into(),
                    status: "ok".into(),
                    latency: "10ms".into(),
                    affects: None,
                    key_configured: None,
                },
                HealthRow {
                    api: "MyVariant".into(),
                    status: "ok".into(),
                    latency: "11ms".into(),
                    affects: None,
                    key_configured: None,
                },
            ],
        };
        let md = report.to_markdown();
        assert!(md.contains("| API | Status | Latency |"));
        assert!(!md.contains("| API | Status | Latency | Affects |"));
    }

    #[test]
    fn markdown_decorates_keyed_success_rows_without_changing_status() {
        let report = HealthReport {
            healthy: 1,
            warning: 0,
            excluded: 0,
            total: 1,
            rows: vec![HealthRow {
                api: "OncoKB".into(),
                status: "ok".into(),
                latency: "10ms".into(),
                affects: None,
                key_configured: Some(true),
            }],
        };

        assert_eq!(report.rows[0].status, "ok");
        let md = report.to_markdown();
        assert!(md.contains("| OncoKB | ok (key configured) | 10ms |"));
    }

    #[test]
    fn markdown_decorates_keyed_error_rows_without_changing_status() {
        let report = HealthReport {
            healthy: 0,
            warning: 0,
            excluded: 0,
            total: 1,
            rows: vec![HealthRow {
                api: "OncoKB".into(),
                status: "error".into(),
                latency: "10ms (HTTP 401)".into(),
                affects: Some("variant oncokb command and variant evidence section".into()),
                key_configured: Some(true),
            }],
        };

        assert_eq!(report.rows[0].status, "error");
        let md = report.to_markdown();
        assert!(md.contains(
            "| OncoKB | error (key configured) | 10ms (HTTP 401) | variant oncokb command and variant evidence section |",
        ));
    }

    #[test]
    fn health_inventory_includes_all_expected_sources() {
        let names: Vec<_> = health_sources().iter().map(|source| source.api).collect();

        assert_eq!(
            names,
            vec![
                "MyGene",
                "MyVariant",
                "MyChem",
                "PubTator3",
                "PubMed",
                "Europe PMC",
                "NCBI E-utilities",
                "LitSense2",
                "PMC OA",
                "NCBI ID Converter",
                "ClinicalTrials.gov",
                "NCI CTS",
                "Enrichr",
                "OpenFDA",
                "CDC WONDER VAERS",
                "OncoKB",
                "DisGeNET",
                "AlphaGenome",
                "Semantic Scholar",
                "CPIC",
                "PharmGKB",
                "Monarch",
                "HPO",
                "MyDisease",
                "SEER Explorer",
                "NIH Reporter",
                "CIViC",
                "GWAS Catalog",
                "GTEx",
                "DGIdb",
                "ClinGen",
                "gnomAD",
                "UniProt",
                "QuickGO",
                "STRING",
                "Reactome",
                "KEGG",
                "WikiPathways",
                "g:Profiler",
                "OpenTargets",
                "ChEMBL",
                "HPA",
                "InterPro",
                "ComplexPortal",
                "OLS4",
                "UMLS",
                "MedlinePlus",
                "cBioPortal",
            ]
        );
    }

    #[test]
    fn probe_source_runs_vaers_query_against_fixture_server() {
        const REACTIONS_RESPONSE_FIXTURE: &str =
            include_str!("../../spec/fixtures/vaers/reactions-response.xml");

        let _env_lock = env_lock();
        block_on(async {
            let server = MockServer::start().await;
            let _vaers_env = set_env_var("BIOMCP_VAERS_BASE", Some(&server.uri()));
            let source = health_sources()
                .iter()
                .find(|source| source.api == "CDC WONDER VAERS")
                .expect("vaers health source");

            Mock::given(method("POST"))
                .and(path("/controller/datarequest/D8"))
                .and(body_string_contains("request_xml="))
                .and(body_string_contains("MMR"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("content-type", "text/html; charset=ISO-8859-1")
                        .set_body_raw(REACTIONS_RESPONSE_FIXTURE, "text/html; charset=ISO-8859-1"),
                )
                .mount(&server)
                .await;

            let outcome = probe_source(reqwest::Client::new(), source).await;
            assert_eq!(outcome.class, ProbeClass::Healthy);
            assert_eq!(outcome.row.api, "CDC WONDER VAERS");
            assert_eq!(outcome.row.status, "ok");
            assert_eq!(outcome.row.affects, None);
            assert_millisecond_latency(&outcome.row.latency);
        });
    }

    #[test]
    fn ema_local_data_not_configured_when_default_root_is_empty() {
        let root = TempDirGuard::new("health");

        let outcome = ema_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Excluded);
        assert_eq!(
            outcome.row.api,
            format!("EMA local data ({})", root.path().display())
        );
        assert_eq!(outcome.row.status, "not configured");
        assert_eq!(outcome.row.latency, "n/a");
        assert_eq!(outcome.row.affects.as_deref(), Some(EMA_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn ema_local_data_errors_when_default_root_is_partial() {
        let root = TempDirGuard::new("health");
        write_ema_files(root.path(), &[crate::sources::ema::EMA_REQUIRED_FILES[0]]);

        let outcome = ema_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(
            outcome.row.status,
            format!(
                "error (missing: {})",
                crate::sources::ema::EMA_REQUIRED_FILES[1..].join(", ")
            )
        );
        assert_eq!(outcome.row.affects.as_deref(), Some(EMA_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn ema_local_data_errors_when_env_root_is_missing_files() {
        let root = TempDirGuard::new("health");

        let outcome = ema_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(
            outcome.row.status,
            format!(
                "error (missing: {})",
                crate::sources::ema::EMA_REQUIRED_FILES.join(", ")
            )
        );
        assert_eq!(outcome.row.affects.as_deref(), Some(EMA_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn ema_local_data_reports_available_when_default_root_is_complete() {
        let fixture_root = fixture_ema_root();
        set_stale_ema_mtimes(fixture_root.path());
        set_fresh_ema_mtimes(fixture_root.path());

        let outcome = ema_local_data_outcome(fixture_root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(
            outcome.row.api,
            format!("EMA local data ({})", fixture_root.path().display())
        );
        assert_eq!(outcome.row.status, "available (default path)");
        assert_eq!(outcome.row.latency, "n/a");
        assert_eq!(outcome.row.affects, None);
    }

    #[test]
    fn ema_local_data_reports_configured_when_env_root_is_complete() {
        let fixture_root = fixture_ema_root();
        set_stale_ema_mtimes(fixture_root.path());
        set_fresh_ema_mtimes(fixture_root.path());

        let outcome = ema_local_data_outcome(fixture_root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(outcome.row.status, "configured");
        assert_eq!(outcome.row.affects, None);
    }

    #[test]
    fn ema_local_data_json_reports_healthy_row_without_affects() {
        let fixture_root = fixture_ema_root();
        set_stale_ema_mtimes(fixture_root.path());
        set_fresh_ema_mtimes(fixture_root.path());
        let report = report_from_outcomes(vec![ema_local_data_outcome(fixture_root.path(), false)]);

        let value = serde_json::to_value(&report).expect("serialize health report");
        let rows = value["rows"].as_array().expect("rows array");
        let row = rows.first().expect("EMA row");

        assert_eq!(
            row["api"],
            format!("EMA local data ({})", fixture_root.path().display())
        );
        assert_eq!(row["status"], "available (default path)");
        assert_eq!(row["latency"], "n/a");
        assert!(row.get("affects").is_none());
        assert!(row.get("key_configured").is_none());
    }

    #[test]
    fn ema_local_data_json_reports_error_row_with_affects() {
        let root = TempDirGuard::new("health");
        write_ema_files(root.path(), &[crate::sources::ema::EMA_REQUIRED_FILES[0]]);
        let report = report_from_outcomes(vec![ema_local_data_outcome(root.path(), false)]);

        let value = serde_json::to_value(&report).expect("serialize health report");
        let rows = value["rows"].as_array().expect("rows array");
        let row = rows.first().expect("EMA row");

        assert_eq!(
            row["status"],
            format!(
                "error (missing: {})",
                crate::sources::ema::EMA_REQUIRED_FILES[1..].join(", ")
            )
        );
        assert_eq!(row["affects"], EMA_LOCAL_DATA_AFFECTS);
        assert!(row.get("key_configured").is_none());
    }

    #[test]
    fn cvx_local_data_not_configured_when_default_root_is_empty() {
        let root = TempDirGuard::new("health");

        let outcome = cvx_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Excluded);
        assert_eq!(
            outcome.row.api,
            format!("CDC CVX/MVX local data ({})", root.path().display())
        );
        assert_eq!(outcome.row.status, "not configured");
        assert_eq!(outcome.row.latency, "n/a");
        assert_eq!(outcome.row.affects.as_deref(), Some(CVX_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn cvx_local_data_errors_when_default_root_is_partial() {
        let root = TempDirGuard::new("health");
        write_cvx_files(root.path(), &[crate::sources::cvx::CVX_FILE]);

        let outcome = cvx_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(
            outcome.row.status,
            format!(
                "error (missing: {})",
                crate::sources::cvx::CVX_REQUIRED_FILES[1..].join(", ")
            )
        );
        assert_eq!(outcome.row.affects.as_deref(), Some(CVX_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn cvx_local_data_reports_available_when_default_root_is_complete() {
        let root = TempDirGuard::new("health");
        write_cvx_files(root.path(), crate::sources::cvx::CVX_REQUIRED_FILES);

        let outcome = cvx_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(
            outcome.row.api,
            format!("CDC CVX/MVX local data ({})", root.path().display())
        );
        assert_eq!(outcome.row.status, "available (default path)");
        assert_eq!(outcome.row.affects, None);
    }

    #[test]
    fn cvx_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
        let root = TempDirGuard::new("health");
        write_cvx_files(root.path(), crate::sources::cvx::CVX_REQUIRED_FILES);
        set_stale_mtime_with_age(
            &root.path().join(crate::sources::cvx::MVX_FILE),
            crate::sources::cvx::CVX_STALE_AFTER + std::time::Duration::from_secs(60),
        );

        let outcome = cvx_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Warning);
        assert_eq!(outcome.row.status, "configured (stale)");
        assert_eq!(outcome.row.affects.as_deref(), Some(CVX_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn who_local_data_not_configured_when_default_root_is_empty() {
        let root = TempDirGuard::new("health");

        let outcome = who_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Excluded);
        assert_eq!(
            outcome.row.api,
            format!(
                "WHO Prequalification local data ({})",
                root.path().display()
            )
        );
        assert_eq!(outcome.row.status, "not configured");
        assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn who_local_data_errors_when_env_root_is_missing_file() {
        let root = TempDirGuard::new("health");

        let outcome = who_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(
            outcome.row.status,
            format!(
                "error (missing: {})",
                crate::sources::who_pq::WHO_PQ_REQUIRED_FILES.join(", ")
            )
        );
        assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn who_local_data_reports_available_when_default_root_is_complete() {
        let root = TempDirGuard::new("health");
        write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);

        let outcome = who_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(
            outcome.row.api,
            format!(
                "WHO Prequalification local data ({})",
                root.path().display()
            )
        );
        assert_eq!(outcome.row.status, "available (default path)");
        assert_eq!(outcome.row.affects, None);
    }

    #[test]
    fn who_local_data_reports_configured_when_env_root_is_complete() {
        let root = TempDirGuard::new("health");
        write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);

        let outcome = who_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(outcome.row.status, "configured");
        assert_eq!(outcome.row.affects, None);
    }

    #[test]
    fn who_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
        let root = TempDirGuard::new("health");
        write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);
        set_stale_mtime(
            &root
                .path()
                .join(crate::sources::who_pq::WHO_PQ_API_CSV_FILE),
        );

        let outcome = who_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Warning);
        assert_eq!(outcome.row.status, "configured (stale)");
        assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn who_local_data_reports_default_path_stale_when_complete_but_old() {
        let root = TempDirGuard::new("health");
        write_who_files(root.path(), crate::sources::who_pq::WHO_PQ_REQUIRED_FILES);
        set_stale_mtime(
            &root
                .path()
                .join(crate::sources::who_pq::WHO_PQ_API_CSV_FILE),
        );

        let outcome = who_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Warning);
        assert_eq!(outcome.row.status, "available (default path, stale)");
        assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn who_local_data_errors_when_only_api_file_is_missing() {
        let root = TempDirGuard::new("health");
        write_who_files(
            root.path(),
            &[
                crate::sources::who_pq::WHO_PQ_CSV_FILE,
                crate::sources::who_pq::WHO_VACCINES_CSV_FILE,
            ],
        );

        let outcome = who_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(outcome.row.status, "error (missing: who_api.csv)");
        assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn who_local_data_errors_when_only_vaccine_file_is_missing() {
        let root = TempDirGuard::new("health");
        write_who_files(
            root.path(),
            &[
                crate::sources::who_pq::WHO_PQ_CSV_FILE,
                crate::sources::who_pq::WHO_PQ_API_CSV_FILE,
            ],
        );

        let outcome = who_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(outcome.row.status, "error (missing: who_vaccines.csv)");
        assert_eq!(outcome.row.affects.as_deref(), Some(WHO_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn who_ivd_local_data_not_configured_when_default_root_is_empty() {
        let root = TempDirGuard::new("health");

        let outcome = who_ivd_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Excluded);
        assert_eq!(
            outcome.row.api,
            format!("WHO IVD local data ({})", root.path().display())
        );
        assert_eq!(outcome.row.status, "not configured");
        assert_eq!(
            outcome.row.affects.as_deref(),
            Some(WHO_IVD_LOCAL_DATA_AFFECTS)
        );
    }

    #[test]
    fn who_ivd_local_data_errors_when_env_root_is_missing_file() {
        let root = TempDirGuard::new("health");

        let outcome = who_ivd_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(
            outcome.row.status,
            format!(
                "error (missing: {})",
                crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES.join(", ")
            )
        );
        assert_eq!(
            outcome.row.affects.as_deref(),
            Some(WHO_IVD_LOCAL_DATA_AFFECTS)
        );
    }

    #[test]
    fn who_ivd_local_data_reports_available_when_default_root_is_complete() {
        let root = TempDirGuard::new("health");
        write_who_ivd_files(root.path(), crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES);

        let outcome = who_ivd_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(
            outcome.row.api,
            format!("WHO IVD local data ({})", root.path().display())
        );
        assert_eq!(outcome.row.status, "available (default path)");
        assert_eq!(outcome.row.affects, None);
    }

    #[test]
    fn who_ivd_local_data_reports_configured_when_env_root_is_complete() {
        let root = TempDirGuard::new("health");
        write_who_ivd_files(root.path(), crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES);

        let outcome = who_ivd_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(outcome.row.status, "configured");
        assert_eq!(outcome.row.affects, None);
    }

    #[test]
    fn who_ivd_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
        let root = TempDirGuard::new("health");
        write_who_ivd_files(root.path(), crate::sources::who_ivd::WHO_IVD_REQUIRED_FILES);
        set_stale_mtime(&root.path().join(crate::sources::who_ivd::WHO_IVD_CSV_FILE));

        let outcome = who_ivd_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Warning);
        assert_eq!(outcome.row.status, "configured (stale)");
        assert_eq!(
            outcome.row.affects.as_deref(),
            Some(WHO_IVD_LOCAL_DATA_AFFECTS)
        );
    }

    #[test]
    fn gtr_local_data_not_configured_when_default_root_is_empty() {
        let root = TempDirGuard::new("health");

        let outcome = gtr_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Excluded);
        assert_eq!(
            outcome.row.api,
            format!("GTR local data ({})", root.path().display())
        );
        assert_eq!(outcome.row.status, "not configured");
        assert_eq!(outcome.row.affects.as_deref(), Some(GTR_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn gtr_local_data_errors_when_default_root_is_partial() {
        let root = TempDirGuard::new("health");
        write_gtr_files(root.path(), &[crate::sources::gtr::GTR_TEST_VERSION_FILE]);

        let outcome = gtr_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(
            outcome.row.status,
            format!(
                "error (missing: {})",
                crate::sources::gtr::GTR_CONDITION_GENE_FILE
            )
        );
        assert_eq!(outcome.row.affects.as_deref(), Some(GTR_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn gtr_local_data_reports_available_when_default_root_is_complete() {
        let root = TempDirGuard::new("health");
        write_gtr_files(root.path(), &crate::sources::gtr::GTR_REQUIRED_FILES);

        let outcome = gtr_local_data_outcome(root.path(), false);

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(outcome.row.status, "available (default path)");
        assert_eq!(outcome.row.affects, None);
    }

    #[test]
    fn gtr_local_data_reports_configured_stale_when_env_root_is_complete_but_old() {
        let root = TempDirGuard::new("health");
        write_gtr_files(root.path(), &crate::sources::gtr::GTR_REQUIRED_FILES);
        set_stale_mtime_with_age(
            &root.path().join(crate::sources::gtr::GTR_TEST_VERSION_FILE),
            crate::sources::gtr::GTR_STALE_AFTER + std::time::Duration::from_secs(60),
        );

        let outcome = gtr_local_data_outcome(root.path(), true);

        assert_eq!(outcome.class, ProbeClass::Warning);
        assert_eq!(outcome.row.status, "configured (stale)");
        assert_eq!(outcome.row.affects.as_deref(), Some(GTR_LOCAL_DATA_AFFECTS));
    }

    #[test]
    fn key_gated_source_is_excluded_when_env_missing() {
        let _lock = env_lock();
        let _env = set_env_var("ONCOKB_TOKEN", None);
        let source = health_sources()
            .iter()
            .find(|source| source.api == "OncoKB")
            .expect("oncokb health source");

        let outcome = block_on(probe_source(reqwest::Client::new(), source));

        assert_eq!(outcome.class, ProbeClass::Excluded);
        assert_eq!(outcome.row.status, "excluded (set ONCOKB_TOKEN)");
        assert_eq!(outcome.row.latency, "n/a");
        assert_eq!(
            outcome.row.affects.as_deref(),
            Some("variant oncokb command and variant evidence section")
        );
        assert_eq!(outcome.row.key_configured, Some(false));
    }

    #[test]
    fn excluded_key_gated_row_serializes_key_configured_false() {
        let report = report_from_outcomes(vec![ProbeOutcome {
            row: HealthRow {
                api: "OncoKB".into(),
                status: "excluded (set ONCOKB_TOKEN)".into(),
                latency: "n/a".into(),
                affects: Some("variant oncokb command and variant evidence section".into()),
                key_configured: Some(false),
            },
            class: ProbeClass::Excluded,
        }]);

        let value = serde_json::to_value(&report).expect("serialize health report");
        let rows = value["rows"].as_array().expect("rows array");
        let row = rows.first().expect("oncokb row");

        assert_eq!(row["status"], "excluded (set ONCOKB_TOKEN)");
        assert_eq!(row["key_configured"], false);
    }

    #[test]
    fn public_row_omits_key_configured_in_json() {
        let report = report_from_outcomes(vec![ProbeOutcome {
            row: HealthRow {
                api: "MyGene".into(),
                status: "ok".into(),
                latency: "10ms".into(),
                affects: None,
                key_configured: None,
            },
            class: ProbeClass::Healthy,
        }]);

        let value = serde_json::to_value(&report).expect("serialize health report");
        let rows = value["rows"].as_array().expect("rows array");
        let row = rows.first().expect("mygene row");

        assert!(row.get("key_configured").is_none());
    }

    #[test]
    fn keyed_row_serializes_raw_status_with_key_configured_true() {
        let value = serde_json::to_value(HealthRow {
            api: "OncoKB".into(),
            status: "ok".into(),
            latency: "10ms".into(),
            affects: None,
            key_configured: Some(true),
        })
        .expect("serialize keyed row");

        assert_eq!(value["status"], "ok");
        assert_eq!(value["key_configured"], true);
    }

    #[test]
    fn empty_key_is_treated_as_missing() {
        let _lock = env_lock();
        let _env = set_env_var("NCI_API_KEY", Some("   "));
        let source = health_sources()
            .iter()
            .find(|source| source.api == "NCI CTS")
            .expect("nci health source");

        let outcome = block_on(probe_source(reqwest::Client::new(), source));

        assert_eq!(outcome.class, ProbeClass::Excluded);
        assert_eq!(outcome.row.status, "excluded (set NCI_API_KEY)");
        assert_eq!(outcome.row.latency, "n/a");
        assert_eq!(outcome.row.key_configured, Some(false));
    }

    #[test]
    fn nci_health_probe_uses_keyword_query() {
        let source = health_sources()
            .iter()
            .find(|source| source.api == "NCI CTS")
            .expect("nci health source");

        let ProbeKind::AuthGet { url, .. } = source.probe else {
            panic!("NCI CTS health source should use an authenticated GET probe");
        };

        assert!(url.contains("keyword=melanoma"));
        assert!(!url.contains("diseases=melanoma"));
    }

    #[test]
    fn alpha_genome_health_probe_connects_without_scoring() {
        let source = health_sources()
            .iter()
            .find(|source| source.api == "AlphaGenome")
            .expect("alphagenome health source");

        assert!(matches!(source.probe, ProbeKind::AlphaGenomeConnect { .. }));
    }

    #[test]
    fn all_healthy_includes_warning_and_excluded_rows() {
        let report = HealthReport {
            healthy: 1,
            warning: 1,
            excluded: 1,
            total: 3,
            rows: vec![
                HealthRow {
                    api: "MyGene".into(),
                    status: "ok".into(),
                    latency: "10ms".into(),
                    affects: None,
                    key_configured: None,
                },
                HealthRow {
                    api: "OncoKB".into(),
                    status: "excluded (set ONCOKB_TOKEN)".into(),
                    latency: "n/a".into(),
                    affects: Some("variant oncokb command and variant evidence section".into()),
                    key_configured: Some(false),
                },
                HealthRow {
                    api: "Cache limits".into(),
                    status: "warning".into(),
                    latency: "referenced bytes 12 exceed max_size 8; run biomcp cache clean".into(),
                    affects: None,
                    key_configured: None,
                },
            ],
        };

        assert!(report.all_healthy());
    }

    #[test]
    fn markdown_summary_reports_ok_error_excluded_and_warning_counts() {
        let report = HealthReport {
            healthy: 1,
            warning: 1,
            excluded: 1,
            total: 4,
            rows: vec![
                HealthRow {
                    api: "MyGene".into(),
                    status: "ok".into(),
                    latency: "10ms".into(),
                    affects: None,
                    key_configured: None,
                },
                HealthRow {
                    api: "OpenFDA".into(),
                    status: "error".into(),
                    latency: "timeout".into(),
                    affects: Some("adverse-event search".into()),
                    key_configured: None,
                },
                HealthRow {
                    api: "OncoKB".into(),
                    status: "excluded (set ONCOKB_TOKEN)".into(),
                    latency: "n/a".into(),
                    affects: Some("variant oncokb command and variant evidence section".into()),
                    key_configured: Some(false),
                },
                HealthRow {
                    api: "Cache limits".into(),
                    status: "warning".into(),
                    latency:
                        "available disk 10 B is below min_disk_free 20 B; run biomcp cache clean"
                            .into(),
                    affects: None,
                    key_configured: None,
                },
            ],
        };

        let md = report.to_markdown();
        assert!(md.contains("Status: 1 ok, 1 error, 1 excluded, 1 warning"));
    }

    #[test]
    fn report_counts_use_probe_class_not_status_prefixes() {
        let report = report_from_outcomes(vec![
            ProbeOutcome {
                row: HealthRow {
                    api: "Semantic Scholar".into(),
                    status: "available (unauthenticated, shared rate limit)".into(),
                    latency: "15ms".into(),
                    affects: None,
                    key_configured: Some(false),
                },
                class: ProbeClass::Healthy,
            },
            ProbeOutcome {
                row: HealthRow {
                    api: "OncoKB".into(),
                    status: "excluded (set ONCOKB_TOKEN)".into(),
                    latency: "n/a".into(),
                    affects: Some("variant oncokb command and variant evidence section".into()),
                    key_configured: Some(false),
                },
                class: ProbeClass::Excluded,
            },
            ProbeOutcome {
                row: HealthRow {
                    api: "Cache limits".into(),
                    status: "warning".into(),
                    latency: "referenced bytes 12 exceed max_size 8; run biomcp cache clean".into(),
                    affects: None,
                    key_configured: None,
                },
                class: ProbeClass::Warning,
            },
        ]);

        assert_eq!(report.healthy, 1);
        assert_eq!(report.warning, 1);
        assert_eq!(report.excluded, 1);
        assert_eq!(report.total, 3);
    }

    #[test]
    fn check_cache_limits_within_limits_returns_healthy_row() {
        let config = test_config("/tmp/cache", 1_024, DiskFreeThreshold::Percent(10));
        let snapshot = test_snapshot(
            "/tmp/cache/http",
            vec![test_entry("retained", b"live-bytes", 100)],
            vec![test_blob("retained", b"live-bytes", 1)],
        );

        let outcome = check_cache_limits_with(
            || Ok(config),
            |_| Ok(snapshot.clone()),
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 90,
                    total_bytes: 100,
                })
            },
        );

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(outcome.row.api, "Cache limits");
        assert_eq!(outcome.row.status, "ok");
        assert_eq!(outcome.row.latency, "within limits");
    }

    #[test]
    fn check_cache_limits_warns_when_referenced_bytes_exceed_max_size() {
        let config = test_config("/tmp/cache", 5, DiskFreeThreshold::Percent(10));
        let snapshot = test_snapshot(
            "/tmp/cache/http",
            vec![
                test_entry("old", b"abcde", 100),
                test_entry("new", b"fghij", 200),
            ],
            vec![test_blob("old", b"abcde", 1), test_blob("new", b"fghij", 1)],
        );

        let outcome = check_cache_limits_with(
            || Ok(config),
            |_| Ok(snapshot.clone()),
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 90,
                    total_bytes: 100,
                })
            },
        );

        assert_eq!(outcome.class, ProbeClass::Warning);
        assert_eq!(outcome.row.status, "warning");
        assert!(outcome.row.latency.contains("referenced bytes"));
        assert!(outcome.row.latency.contains("biomcp cache clean"));
    }

    #[test]
    fn check_cache_limits_warns_when_disk_floor_is_violated() {
        let config = test_config("/tmp/cache", 1_024, DiskFreeThreshold::Percent(20));
        let snapshot = test_snapshot(
            "/tmp/cache/http",
            vec![test_entry("retained", b"live-bytes", 100)],
            vec![test_blob("retained", b"live-bytes", 1)],
        );

        let outcome = check_cache_limits_with(
            || Ok(config),
            |_| Ok(snapshot.clone()),
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 10,
                    total_bytes: 100,
                })
            },
        );

        assert_eq!(outcome.class, ProbeClass::Warning);
        assert_eq!(outcome.row.status, "warning");
        assert!(outcome.row.latency.contains("available disk"));
        assert!(outcome.row.latency.contains("biomcp cache clean"));
    }

    #[test]
    fn check_cache_limits_reports_snapshot_errors_as_error_rows() {
        let config = test_config("/tmp/cache", 1_024, DiskFreeThreshold::Percent(10));

        let outcome = check_cache_limits_with(
            || Ok(config),
            |_| {
                Err(CachePlannerError::Io {
                    path: PathBuf::from("/tmp/cache/http"),
                    source: io::Error::other("boom"),
                })
            },
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 90,
                    total_bytes: 100,
                })
            },
        );

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(outcome.row.api, "Cache limits");
        assert_eq!(outcome.row.status, "error");
        assert!(outcome.row.latency.contains("boom"));
    }

    #[test]
    fn health_probes_respect_concurrency_limit_and_source_order() {
        let input: Vec<_> = (0..(HEALTH_API_PROBE_CONCURRENCY_LIMIT + 5)).collect();
        let in_flight = Arc::new(AtomicUsize::new(0));
        let max_in_flight = Arc::new(AtomicUsize::new(0));

        let output = block_on(run_buffered_in_order(
            input.clone(),
            HEALTH_API_PROBE_CONCURRENCY_LIMIT,
            {
                let in_flight = Arc::clone(&in_flight);
                let max_in_flight = Arc::clone(&max_in_flight);
                move |index| {
                    let in_flight = Arc::clone(&in_flight);
                    let max_in_flight = Arc::clone(&max_in_flight);
                    async move {
                        let current = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                        update_max(&max_in_flight, current);
                        tokio::time::sleep(Duration::from_millis(25)).await;
                        in_flight.fetch_sub(1, Ordering::SeqCst);
                        index
                    }
                }
            },
        ));

        assert_eq!(output, input);
        assert_eq!(
            max_in_flight.load(Ordering::SeqCst),
            HEALTH_API_PROBE_CONCURRENCY_LIMIT
        );
        assert_eq!(in_flight.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn timed_out_probe_returns_error_row_with_timeout_latency() {
        let _lock = env_lock();
        let server = block_on(MockServer::start());
        let slow_url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());

        block_on(async {
            Mock::given(method("GET"))
                .and(path("/health"))
                .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(100)))
                .expect(3)
                .mount(&server)
                .await;
        });

        let optional_source = SourceDescriptor {
            api: "Semantic Scholar",
            affects: Some("Semantic Scholar features"),
            probe: ProbeKind::OptionalAuthGet {
                url: slow_url,
                env_var: "S2_API_KEY",
                header_name: "x-api-key",
                header_value_prefix: "",
                unauthenticated_ok_status: "available (unauthenticated, shared rate limit)",
                authenticated_ok_status: "configured (authenticated)",
                unauthenticated_rate_limited_status: Some(
                    "unavailable (set S2_API_KEY for reliable access)",
                ),
            },
        };
        let _optional_env = set_env_var("S2_API_KEY", None);
        let optional_outcome = block_on(probe_source_with_timeout_for_test(
            reqwest::Client::new(),
            optional_source,
            Duration::from_millis(10),
        ));
        assert_eq!(optional_outcome.class, ProbeClass::Error);
        assert_eq!(optional_outcome.row.status, "error");
        assert_eq!(optional_outcome.row.latency, "10ms (timeout)");
        assert_eq!(
            optional_outcome.row.affects.as_deref(),
            Some("Semantic Scholar features")
        );
        assert_eq!(optional_outcome.row.key_configured, Some(false));

        let auth_source = SourceDescriptor {
            api: "OncoKB",
            affects: Some("variant oncokb command and variant evidence section"),
            probe: ProbeKind::AuthGet {
                url: slow_url,
                env_var: "ONCOKB_TOKEN",
                header_name: "Authorization",
                header_value_prefix: "Bearer ",
            },
        };
        let _auth_env = set_env_var("ONCOKB_TOKEN", Some("test-token"));
        let auth_outcome = block_on(probe_source_with_timeout_for_test(
            reqwest::Client::new(),
            auth_source,
            Duration::from_millis(10),
        ));
        assert_eq!(auth_outcome.class, ProbeClass::Error);
        assert_eq!(auth_outcome.row.status, "error");
        assert_eq!(auth_outcome.row.latency, "10ms (timeout)");
        assert_eq!(
            auth_outcome.row.affects.as_deref(),
            Some("variant oncokb command and variant evidence section")
        );
        assert_eq!(auth_outcome.row.key_configured, Some(true));

        let public_source = SourceDescriptor {
            api: "MyGene",
            affects: Some("gene search and gene get"),
            probe: ProbeKind::Get { url: slow_url },
        };
        let public_outcome = block_on(probe_source_with_timeout_for_test(
            reqwest::Client::new(),
            public_source,
            Duration::from_millis(10),
        ));
        assert_eq!(public_outcome.class, ProbeClass::Error);
        assert_eq!(public_outcome.row.status, "error");
        assert_eq!(public_outcome.row.latency, "10ms (timeout)");
        assert_eq!(
            public_outcome.row.affects.as_deref(),
            Some("gene search and gene get")
        );
        assert_eq!(public_outcome.row.key_configured, None);
    }

    #[test]
    fn optional_auth_get_reports_unauthed_semantic_scholar_as_healthy() {
        let _lock = env_lock();
        let _env = set_env_var("S2_API_KEY", None);
        let server = block_on(MockServer::start());
        let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
        let source = semantic_scholar_source(url);

        block_on(async {
            Mock::given(method("GET"))
                .and(path("/health"))
                .and(|request: &wiremock::Request| !request.headers.contains_key("x-api-key"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;
        });

        let outcome = block_on(probe_source(reqwest::Client::new(), &source));
        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(
            outcome.row.status,
            "available (unauthenticated, shared rate limit)"
        );
        assert_eq!(outcome.row.key_configured, Some(false));
    }

    #[test]
    fn optional_auth_get_reports_authed_semantic_scholar_as_configured() {
        let _lock = env_lock();
        let _env = set_env_var("S2_API_KEY", Some("test-key-abc"));
        let server = block_on(MockServer::start());
        let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
        let source = semantic_scholar_source(url);

        block_on(async {
            Mock::given(method("GET"))
                .and(path("/health"))
                .and(header("x-api-key", "test-key-abc"))
                .respond_with(ResponseTemplate::new(200))
                .expect(1)
                .mount(&server)
                .await;
        });

        let outcome = block_on(probe_source(reqwest::Client::new(), &source));
        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(outcome.row.status, "configured (authenticated)");
        assert_eq!(outcome.row.key_configured, Some(true));
    }

    #[test]
    fn optional_auth_get_reports_unauthenticated_429_as_unavailable() {
        let _lock = env_lock();
        let _env = set_env_var("S2_API_KEY", None);
        let server = block_on(MockServer::start());
        let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
        let source = semantic_scholar_source(url);

        block_on(async {
            Mock::given(method("GET"))
                .and(path("/health"))
                .and(|request: &wiremock::Request| !request.headers.contains_key("x-api-key"))
                .respond_with(ResponseTemplate::new(429))
                .expect(1)
                .mount(&server)
                .await;
        });

        let outcome = block_on(probe_source(reqwest::Client::new(), &source));
        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(
            outcome.row.status,
            "unavailable (set S2_API_KEY for reliable access)"
        );
        assert_millisecond_latency(&outcome.row.latency);
        assert!(!outcome.row.latency.contains("HTTP 429"));
        assert_eq!(outcome.row.affects, None);
        assert_eq!(outcome.row.key_configured, Some(false));

        let report = report_from_outcomes(vec![outcome.clone()]);
        assert_eq!(report.healthy, 1);
        assert_eq!(report.excluded, 0);
        assert_eq!(report.total, 1);
        assert!(report.all_healthy());

        let value = serde_json::to_value(&report).expect("serialize health report");
        let rows = value["rows"].as_array().expect("rows array");
        let row = rows.first().expect("semantic scholar row");
        assert!(row.get("affects").is_none());
        assert_eq!(row["key_configured"], false);

        let md = report_from_outcomes(vec![
            outcome.clone(),
            ProbeOutcome {
                row: HealthRow {
                    api: "OpenFDA".into(),
                    status: "error".into(),
                    latency: "timeout".into(),
                    affects: Some("adverse-event search".into()),
                    key_configured: None,
                },
                class: ProbeClass::Error,
            },
        ])
        .to_markdown();
        assert!(md.contains(&format!(
            "| Semantic Scholar | {} | {} | - |",
            outcome.row.status, outcome.row.latency
        )));
    }

    #[test]
    fn optional_auth_get_reports_unauthenticated_non_429_as_error() {
        let _lock = env_lock();
        let _env = set_env_var("S2_API_KEY", None);
        let server = block_on(MockServer::start());
        let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
        let source = semantic_scholar_source(url);

        block_on(async {
            Mock::given(method("GET"))
                .and(path("/health"))
                .and(|request: &wiremock::Request| !request.headers.contains_key("x-api-key"))
                .respond_with(ResponseTemplate::new(403))
                .expect(1)
                .mount(&server)
                .await;
        });

        let outcome = block_on(probe_source(reqwest::Client::new(), &source));
        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(outcome.row.status, "error");
        assert!(outcome.row.latency.contains("HTTP 403"));
        assert_eq!(
            outcome.row.affects.as_deref(),
            Some("Semantic Scholar features")
        );
        assert_eq!(outcome.row.key_configured, Some(false));
    }

    #[test]
    fn optional_auth_get_reports_authenticated_429_as_error() {
        let _lock = env_lock();
        let _env = set_env_var("S2_API_KEY", Some("test-key-abc"));
        let server = block_on(MockServer::start());
        let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
        let source = semantic_scholar_source(url);

        block_on(async {
            Mock::given(method("GET"))
                .and(path("/health"))
                .and(header("x-api-key", "test-key-abc"))
                .respond_with(ResponseTemplate::new(429))
                .expect(1)
                .mount(&server)
                .await;
        });

        let outcome = block_on(probe_source(reqwest::Client::new(), &source));
        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(outcome.row.status, "error");
        assert!(outcome.row.latency.contains("HTTP 429"));
        assert_eq!(
            outcome.row.affects.as_deref(),
            Some("Semantic Scholar features")
        );
        assert_eq!(outcome.row.key_configured, Some(true));
    }

    #[test]
    fn check_cache_dir_success_row_uses_resolved_path_and_ok_contract() {
        let _lock = env_lock();
        let root = TempDirGuard::new("health");
        let cache_home = root.path().join("cache-home");
        let config_home = root.path().join("config-home");
        let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
        let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
        let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", None);

        let outcome = block_on(check_cache_dir());

        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(
            outcome.row.api,
            format!("Cache dir ({})", cache_home.join("biomcp").display())
        );
        assert_eq!(outcome.row.status, "ok");
        assert_millisecond_latency(&outcome.row.latency);
        assert_eq!(outcome.row.affects, None);
        assert_eq!(outcome.row.key_configured, None);
    }

    #[test]
    fn probe_cache_dir_failure_preserves_error_contract() {
        let root = TempDirGuard::new("health");
        let blocking_path = root.path().join("not-a-dir");
        std::fs::write(&blocking_path, b"occupied").expect("blocking file should exist");

        let outcome = block_on(probe_cache_dir(&blocking_path));

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(
            outcome.row.api,
            format!("Cache dir ({})", blocking_path.display())
        );
        assert_eq!(outcome.row.status, "error");
        assert!(
            outcome.row.latency.contains("AlreadyExists")
                || outcome.row.latency.contains("NotADirectory")
                || outcome.row.latency.contains("PermissionDenied"),
            "unexpected latency: {}",
            outcome.row.latency
        );
        assert_cache_dir_affects(outcome.row.affects.as_deref());
        assert_eq!(outcome.row.key_configured, None);
    }

    #[test]
    fn check_cache_dir_config_error_matches_pinned_contract() {
        let _lock = env_lock();
        let root = TempDirGuard::new("health");
        let cache_home = root.path().join("cache-home");
        let config_home = root.path().join("config-home");
        let config_dir = config_home.join("biomcp");
        std::fs::create_dir_all(&config_dir).expect("config dir should exist");
        let config_path = config_dir.join("cache.toml");
        std::fs::write(&config_path, "[cache]\nmax_size = 0\n").expect("cache.toml should exist");
        let _cache_home = set_env_var("XDG_CACHE_HOME", Some(&cache_home.to_string_lossy()));
        let _config_home = set_env_var("XDG_CONFIG_HOME", Some(&config_home.to_string_lossy()));
        let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", None);

        let outcome = block_on(check_cache_dir());

        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(outcome.row.api, "Cache dir");
        assert_eq!(outcome.row.status, "error");
        assert_eq!(
            outcome.row.latency,
            format!(
                "Invalid argument: {}: [cache].max_size must be greater than 0",
                config_path.display()
            )
        );
        assert_cache_dir_affects(outcome.row.affects.as_deref());
        assert_eq!(outcome.row.key_configured, None);
    }

    #[test]
    fn markdown_shows_new_affects_mappings() {
        assert_eq!(affects_for_api("GTEx"), Some("gene expression section"));
        assert_eq!(affects_for_api("DGIdb"), Some("gene druggability section"));
        assert_eq!(
            affects_for_api("OpenTargets"),
            Some("gene druggability, drug target, and disease association sections")
        );
        assert_eq!(affects_for_api("ClinGen"), Some("gene clingen section"));
        assert_eq!(affects_for_api("gnomAD"), Some("gene constraint section"));
        assert_eq!(
            affects_for_api("NIH Reporter"),
            Some("gene and disease funding sections")
        );
        assert_eq!(
            affects_for_api("KEGG"),
            Some("pathway search and detail sections")
        );
        assert_eq!(
            affects_for_api("HPA"),
            Some("gene protein tissue expression and localization section")
        );
        assert_eq!(
            affects_for_api("ComplexPortal"),
            Some("protein complex membership section")
        );
        assert_eq!(
            affects_for_api("g:Profiler"),
            Some("gene enrichment (biomcp enrich)")
        );
    }
}
