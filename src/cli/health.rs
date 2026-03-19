use std::sync::OnceLock;
use std::time::{Duration, Instant};

use futures::future::join_all;

use crate::error::BioMcpError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthRow {
    pub api: String,
    pub status: String,
    pub latency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affects: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthReport {
    pub healthy: usize,
    pub excluded: usize,
    pub total: usize,
    pub rows: Vec<HealthRow>,
}

impl HealthReport {
    pub fn all_healthy(&self) -> bool {
        self.healthy + self.excluded == self.total
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let show_affects = self.rows.iter().any(|row| row.affects.is_some());
        let errors = self.total.saturating_sub(self.healthy + self.excluded);

        out.push_str("# BioMCP Health Check\n\n");
        if show_affects {
            out.push_str("| API | Status | Latency | Affects |\n");
            out.push_str("|-----|--------|---------|---------|\n");
            for row in &self.rows {
                let affects = row.affects.as_deref().unwrap_or("-");
                out.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    row.api, row.status, row.latency, affects
                ));
            }
        } else {
            out.push_str("| API | Status | Latency |\n");
            out.push_str("|-----|--------|---------|\n");
            for row in &self.rows {
                out.push_str(&format!(
                    "| {} | {} | {} |\n",
                    row.api, row.status, row.latency
                ));
            }
        }

        out.push_str(&format!(
            "\nStatus: {} ok, {} error, {} excluded\n",
            self.healthy, errors, self.excluded
        ));
        out
    }
}

#[derive(Debug, Clone, Copy)]
struct SourceDescriptor {
    api: &'static str,
    affects: Option<&'static str>,
    probe: ProbeKind,
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
        api: "Europe PMC",
        affects: Some("article search coverage"),
        probe: ProbeKind::Get {
            url: "https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=BRAF&format=json&pageSize=1",
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
            url: "https://clinicaltrialsapi.cancer.gov/api/v2/trials?size=1&diseases=melanoma",
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
        api: "AlphaGenome",
        affects: Some("variant predict section"),
        probe: ProbeKind::AlphaGenomeConnect {
            env_var: "ALPHAGENOME_API_KEY",
        },
    },
    SourceDescriptor {
        api: "Semantic Scholar",
        affects: Some("article enrichment, citations, references, and recommendations"),
        probe: ProbeKind::AuthGet {
            url: "https://api.semanticscholar.org/graph/v1/paper/PMID:22663011?fields=paperId,title",
            env_var: "S2_API_KEY",
            header_name: "x-api-key",
            header_value_prefix: "",
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
        api: "g:Profiler",
        affects: Some("gene enrichment (biomcp enrich)"),
        probe: ProbeKind::PostJson {
            url: "https://biit.cs.ut.ee/gprofiler/api/gost/profile/",
            payload: r#"{"organism":"hsapiens","query":["BRAF"]}"#,
        },
    },
    SourceDescriptor {
        api: "OpenTargets",
        affects: Some("drug target and disease association sections"),
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
        api: "cBioPortal",
        affects: Some("cohort frequency section"),
        probe: ProbeKind::Get {
            url: "https://www.cbioportal.org/api/studies?projection=SUMMARY&pageSize=1",
        },
    },
];

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
) -> HealthRow {
    HealthRow {
        api: api.to_string(),
        status,
        latency,
        affects: affects.map(str::to_string),
    }
}

fn masked_key_hint(value: &str) -> String {
    let prefix: String = value.trim().chars().take(3).collect();
    format!("{prefix}***")
}

fn configured_key(env_var: &str) -> Option<String> {
    std::env::var(env_var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn decorated_status(base: &str, key_hint: Option<&str>) -> String {
    match key_hint {
        Some(key_hint) => format!("{base} (key: {key_hint})"),
        None => base.to_string(),
    }
}

fn excluded_row(api: &str, env_var: &str, affects: Option<&'static str>) -> HealthRow {
    health_row(
        api,
        format!("excluded (set {env_var})"),
        "n/a".into(),
        affects,
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
    key_hint: Option<String>,
) -> HealthRow {
    let start = Instant::now();
    let response = request.send().await;

    match response {
        Ok(response) => {
            let status = response.status();
            let elapsed = start.elapsed().as_millis();
            if status.is_success() {
                health_row(
                    api,
                    decorated_status("ok", key_hint.as_deref()),
                    format!("{elapsed}ms"),
                    None,
                )
            } else {
                health_row(
                    api,
                    decorated_status("error", key_hint.as_deref()),
                    format!("{elapsed}ms (HTTP {})", status.as_u16()),
                    affects,
                )
            }
        }
        Err(err) => health_row(
            api,
            decorated_status("error", key_hint.as_deref()),
            transport_error_latency(start, &err),
            affects,
        ),
    }
}

async fn check_get(
    client: reqwest::Client,
    api: &str,
    url: &str,
    affects: Option<&'static str>,
) -> HealthRow {
    send_request(api, affects, client.get(url), None).await
}

async fn check_post_json(
    client: reqwest::Client,
    api: &str,
    url: &str,
    payload: &str,
    affects: Option<&'static str>,
) -> HealthRow {
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
) -> HealthRow {
    let Some(key) = configured_key(env_var) else {
        return excluded_row(api, env_var, affects);
    };

    let key_hint = masked_key_hint(&key);
    let header_value = format!("{header_value_prefix}{key}");

    send_request(
        api,
        affects,
        client.get(url).header(header_name, header_value),
        Some(key_hint),
    )
    .await
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
) -> HealthRow {
    let Some(key) = configured_key(env_var) else {
        return excluded_row(api, env_var, affects);
    };

    let key_hint = masked_key_hint(&key);
    let header_value = format!("{header_value_prefix}{key}");

    send_request(
        api,
        affects,
        client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(header_name, header_value)
            .body(payload.to_string()),
        Some(key_hint),
    )
    .await
}

async fn check_alphagenome_connect(
    api: &str,
    env_var: &str,
    affects: Option<&'static str>,
) -> HealthRow {
    let Some(key) = configured_key(env_var) else {
        return excluded_row(api, env_var, affects);
    };

    let key_hint = masked_key_hint(&key);
    let start = Instant::now();

    match crate::sources::alphagenome::AlphaGenomeClient::new().await {
        Ok(_) => health_row(
            api,
            decorated_status("ok", Some(&key_hint)),
            format!("{}ms", start.elapsed().as_millis()),
            None,
        ),
        Err(err) => health_row(
            api,
            decorated_status("error", Some(&key_hint)),
            api_error_latency(start, &err),
            affects,
        ),
    }
}

async fn probe_source(client: reqwest::Client, source: &SourceDescriptor) -> HealthRow {
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
    }
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

async fn check_cache_dir() -> HealthRow {
    let start = Instant::now();
    let dir = crate::utils::download::biomcp_cache_dir();
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
        Ok(()) => HealthRow {
            api: format!("Cache dir ({})", dir.display()),
            status: "ok".into(),
            latency: format!("{}ms", start.elapsed().as_millis()),
            affects: None,
        },
        Err(err) => HealthRow {
            api: format!("Cache dir ({})", dir.display()),
            status: "error".into(),
            latency: format!("{:?}", err.kind()),
            affects: Some("local cache-backed lookups and downloads".into()),
        },
    }
}

/// Runs connectivity checks for configured upstream APIs and local cache directory.
///
/// # Errors
///
/// Returns an error when the shared HTTP client cannot be created.
pub async fn check(apis_only: bool) -> Result<HealthReport, BioMcpError> {
    let client = health_http_client()?;
    let mut rows = join_all(
        health_sources()
            .iter()
            .map(|source| probe_source(client.clone(), source)),
    )
    .await;

    if !apis_only {
        rows.push(check_cache_dir().await);
    }

    let healthy = rows
        .iter()
        .filter(|row| row.status.starts_with("ok"))
        .count();
    let excluded = rows
        .iter()
        .filter(|row| row.status.starts_with("excluded"))
        .count();

    Ok(HealthReport {
        healthy,
        excluded,
        total: rows.len(),
        rows,
    })
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::{Mutex, OnceLock};

    use super::{
        HealthReport, HealthRow, ProbeKind, affects_for_api, health_sources, masked_key_hint,
        probe_source,
    };

    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("health test runtime")
            .block_on(future)
    }

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        name: &'static str,
        previous: Option<String>,
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            // Safety: tests serialize environment mutation with `env_lock()`.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.name, value),
                    None => std::env::remove_var(self.name),
                }
            }
        }
    }

    fn set_env_var(name: &'static str, value: Option<&str>) -> EnvVarGuard {
        let previous = std::env::var(name).ok();
        // Safety: tests serialize environment mutation with `env_lock()`.
        unsafe {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
        EnvVarGuard { name, previous }
    }

    #[test]
    fn markdown_shows_affects_column_when_present() {
        let report = HealthReport {
            healthy: 1,
            excluded: 0,
            total: 2,
            rows: vec![
                HealthRow {
                    api: "MyGene".into(),
                    status: "ok".into(),
                    latency: "10ms".into(),
                    affects: None,
                },
                HealthRow {
                    api: "OpenFDA".into(),
                    status: "error".into(),
                    latency: "timeout".into(),
                    affects: Some("adverse-event search".into()),
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
            excluded: 0,
            total: 2,
            rows: vec![
                HealthRow {
                    api: "MyGene".into(),
                    status: "ok".into(),
                    latency: "10ms".into(),
                    affects: None,
                },
                HealthRow {
                    api: "MyVariant".into(),
                    status: "ok".into(),
                    latency: "11ms".into(),
                    affects: None,
                },
            ],
        };
        let md = report.to_markdown();
        assert!(md.contains("| API | Status | Latency |"));
        assert!(!md.contains("| API | Status | Latency | Affects |"));
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
                "Europe PMC",
                "PMC OA",
                "NCBI ID Converter",
                "ClinicalTrials.gov",
                "NCI CTS",
                "Enrichr",
                "OpenFDA",
                "OncoKB",
                "AlphaGenome",
                "Semantic Scholar",
                "CPIC",
                "PharmGKB",
                "Monarch",
                "HPO",
                "MyDisease",
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
                "g:Profiler",
                "OpenTargets",
                "ChEMBL",
                "HPA",
                "InterPro",
                "ComplexPortal",
                "cBioPortal",
            ]
        );
    }

    #[test]
    fn key_gated_source_is_excluded_when_env_missing() {
        let _lock = env_lock().lock().expect("env lock");
        let _env = set_env_var("ONCOKB_TOKEN", None);
        let source = health_sources()
            .iter()
            .find(|source| source.api == "OncoKB")
            .expect("oncokb health source");

        let row = block_on(probe_source(reqwest::Client::new(), source));

        assert_eq!(row.status, "excluded (set ONCOKB_TOKEN)");
        assert_eq!(row.latency, "n/a");
        assert_eq!(
            row.affects.as_deref(),
            Some("variant oncokb command and variant evidence section")
        );
    }

    #[test]
    fn key_gated_source_masks_present_key() {
        assert_eq!(masked_key_hint("OncokbTest"), "Onc***");
        assert_eq!(masked_key_hint("abc"), "abc***");
        assert_eq!(masked_key_hint("ab"), "ab***");
    }

    #[test]
    fn empty_key_is_treated_as_missing() {
        let _lock = env_lock().lock().expect("env lock");
        let _env = set_env_var("NCI_API_KEY", Some("   "));
        let source = health_sources()
            .iter()
            .find(|source| source.api == "NCI CTS")
            .expect("nci health source");

        let row = block_on(probe_source(reqwest::Client::new(), source));

        assert_eq!(row.status, "excluded (set NCI_API_KEY)");
        assert_eq!(row.latency, "n/a");
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
    fn all_healthy_ignores_excluded_rows() {
        let report = HealthReport {
            healthy: 1,
            excluded: 1,
            total: 2,
            rows: vec![
                HealthRow {
                    api: "MyGene".into(),
                    status: "ok".into(),
                    latency: "10ms".into(),
                    affects: None,
                },
                HealthRow {
                    api: "OncoKB".into(),
                    status: "excluded (set ONCOKB_TOKEN)".into(),
                    latency: "n/a".into(),
                    affects: Some("variant oncokb command and variant evidence section".into()),
                },
            ],
        };

        assert!(report.all_healthy());
    }

    #[test]
    fn markdown_summary_reports_ok_error_excluded_counts() {
        let report = HealthReport {
            healthy: 1,
            excluded: 1,
            total: 3,
            rows: vec![
                HealthRow {
                    api: "MyGene".into(),
                    status: "ok".into(),
                    latency: "10ms".into(),
                    affects: None,
                },
                HealthRow {
                    api: "OpenFDA".into(),
                    status: "error".into(),
                    latency: "timeout".into(),
                    affects: Some("adverse-event search".into()),
                },
                HealthRow {
                    api: "OncoKB".into(),
                    status: "excluded (set ONCOKB_TOKEN)".into(),
                    latency: "n/a".into(),
                    affects: Some("variant oncokb command and variant evidence section".into()),
                },
            ],
        };

        let md = report.to_markdown();
        assert!(md.contains("Status: 1 ok, 1 error, 1 excluded"));
    }

    #[test]
    fn markdown_shows_new_affects_mappings() {
        assert_eq!(affects_for_api("GTEx"), Some("gene expression section"));
        assert_eq!(affects_for_api("DGIdb"), Some("gene druggability section"));
        assert_eq!(affects_for_api("ClinGen"), Some("gene clingen section"));
        assert_eq!(affects_for_api("gnomAD"), Some("gene constraint section"));
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
