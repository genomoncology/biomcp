use std::sync::OnceLock;
use std::time::{Duration, Instant};

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
    pub total: usize,
    pub rows: Vec<HealthRow>,
}

impl HealthReport {
    pub fn all_healthy(&self) -> bool {
        self.healthy == self.total
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let show_affects = self.rows.iter().any(|row| row.affects.is_some());
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
            "\nStatus: {}/{} APIs healthy\n",
            self.healthy, self.total
        ));
        out
    }
}

fn affects_for_api(api: &str) -> Option<&'static str> {
    match api {
        "MyGene" => Some("get/search gene and gene helper commands"),
        "MyVariant" => Some("get/search variant and variant helper commands"),
        "ClinicalTrials" => Some("search/get trial and trial helper commands"),
        "Enrichr" => Some("gene/pathway enrichment sections"),
        "Europe PMC" => Some("article search coverage"),
        "PubTator3" => Some("article annotations and entity extraction"),
        "OpenFDA" => Some("adverse-event search"),
        "CPIC" | "PharmGKB" => Some("pgx recommendations and annotations"),
        "Monarch" => Some("disease genes, phenotypes, and models"),
        "GWAS Catalog" => Some("gwas search and variant gwas context"),
        _ => None,
    }
}

async fn check_one(client: reqwest::Client, api: &str, url: &str) -> HealthRow {
    let start = Instant::now();
    let resp = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await;

    match resp {
        Ok(resp) => {
            let status = resp.status();
            let elapsed = start.elapsed().as_millis();
            if status.is_success() {
                HealthRow {
                    api: api.to_string(),
                    status: "ok".into(),
                    latency: format!("{elapsed}ms"),
                    affects: None,
                }
            } else {
                HealthRow {
                    api: api.to_string(),
                    status: "error".into(),
                    latency: format!("{elapsed}ms (HTTP {})", status.as_u16()),
                    affects: affects_for_api(api).map(str::to_string),
                }
            }
        }
        Err(err) => {
            let reason = if err.is_timeout() {
                "timeout"
            } else if err.is_connect() {
                "connect"
            } else {
                "error"
            };
            HealthRow {
                api: api.to_string(),
                status: "error".into(),
                latency: reason.into(),
                affects: affects_for_api(api).map(str::to_string),
            }
        }
    }
}

fn health_http_client() -> Result<reqwest::Client, BioMcpError> {
    static HEALTH_HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

    if let Some(client) = HEALTH_HTTP_CLIENT.get() {
        return Ok(client.clone());
    }

    let client = reqwest::Client::builder()
        // Keep health checks snappy and deterministic for CLI/VV.
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

    let (
        mygene,
        myvariant,
        mychem,
        pubtator,
        ctgov,
        enrichr,
        europe_pmc,
        openfda,
        cpic,
        pharmgkb,
        monarch,
        gwas,
    ) = tokio::join!(
        check_one(
            client.clone(),
            "MyGene",
            "https://mygene.info/v3/query?q=BRAF&size=1"
        ),
        check_one(
            client.clone(),
            "MyVariant",
            "https://myvariant.info/v1/query?q=rs113488022&size=1"
        ),
        check_one(
            client.clone(),
            "MyChem",
            "https://mychem.info/v1/query?q=aspirin&size=1"
        ),
        check_one(
            client.clone(),
            "PubTator3",
            "https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=22663011"
        ),
        check_one(
            client.clone(),
            "ClinicalTrials",
            "https://clinicaltrials.gov/api/v2/studies?query.term=cancer&pageSize=1"
        ),
        check_one(
            client.clone(),
            "Enrichr",
            "https://maayanlab.cloud/Enrichr/datasetStatistics"
        ),
        check_one(
            client.clone(),
            "Europe PMC",
            "https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=BRAF&format=json&pageSize=1"
        ),
        check_one(
            client.clone(),
            "OpenFDA",
            "https://api.fda.gov/drug/event.json?limit=1"
        ),
        check_one(
            client.clone(),
            "CPIC",
            "https://api.cpicpgx.org/v1/pair_view?select=pairid&limit=1"
        ),
        check_one(
            client.clone(),
            "PharmGKB",
            "https://api.pharmgkb.org/v1/data/labelAnnotation?relatedChemicals.name=warfarin&view=min"
        ),
        check_one(
            client.clone(),
            "Monarch",
            "https://api-v3.monarchinitiative.org/v3/api/association?object=MONDO:0007739&subject_category=biolink:Gene&limit=1"
        ),
        check_one(
            client.clone(),
            "GWAS Catalog",
            "https://www.ebi.ac.uk/gwas/rest/api/singleNucleotidePolymorphisms/rs7903146"
        ),
    );

    let mut rows = vec![
        mygene, myvariant, mychem, pubtator, ctgov, enrichr, europe_pmc, openfda, cpic, pharmgkb,
        monarch, gwas,
    ];
    if !apis_only {
        rows.push(check_cache_dir().await);
    }
    let healthy = rows.iter().filter(|r| r.status == "ok").count();
    Ok(HealthReport {
        healthy,
        total: rows.len(),
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::{HealthReport, HealthRow};

    #[test]
    fn markdown_shows_affects_column_when_present() {
        let report = HealthReport {
            healthy: 1,
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
}
