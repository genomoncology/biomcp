use std::time::Duration;

use anyhow::Context;

use crate::error::BioMcpError;

const OLS4_TIMEOUT: Duration = Duration::from_millis(1000);
const UMLS_TIMEOUT: Duration = Duration::from_millis(2500);
const MEDLINEPLUS_TIMEOUT: Duration = Duration::from_millis(800);

#[derive(Debug, Clone)]
pub struct DiscoverArgs {
    pub query: String,
}

pub async fn run(args: DiscoverArgs, json: bool) -> anyhow::Result<String> {
    let query = args.query.trim();
    if query.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Free-text query is required. Example: biomcp discover BRCA1".into(),
        )
        .into());
    }

    let ols_client = crate::sources::ols4::OlsClient::new()?;
    let umls_client = crate::sources::umls::UmlsClient::new()?;
    let medline_client = crate::sources::medlineplus::MedlinePlusClient::new()?;

    let query_owned = query.to_string();
    let ols_future = async {
        match tokio::time::timeout(OLS4_TIMEOUT, ols_client.search(&query_owned)).await {
            Ok(result) => result,
            Err(_) => Err(BioMcpError::Api {
                api: "ols4".to_string(),
                message: format!(
                    "Timed out after {}ms while resolving discover query",
                    OLS4_TIMEOUT.as_millis()
                ),
            }),
        }
    };

    let query_owned = query.to_string();
    let umls_future = async move {
        let Some(client) = umls_client else {
            return (
                Vec::new(),
                Some("UMLS enrichment unavailable (set UMLS_API_KEY)".to_string()),
            );
        };

        match tokio::time::timeout(UMLS_TIMEOUT, client.search(&query_owned)).await {
            Ok(Ok(rows)) => (rows, None),
            Ok(Err(err)) => (
                Vec::new(),
                Some(format!("UMLS enrichment unavailable ({err})")),
            ),
            Err(_) => (
                Vec::new(),
                Some("UMLS enrichment unavailable (timed out)".to_string()),
            ),
        }
    };

    let query_owned = query.to_string();
    let medline_future = async move {
        match tokio::time::timeout(MEDLINEPLUS_TIMEOUT, medline_client.search(&query_owned)).await {
            Ok(Ok(rows)) => rows,
            Ok(Err(_)) | Err(_) => Vec::new(),
        }
    };

    let (ols_docs, (umls_rows, umls_note), medline_topics) =
        tokio::join!(ols_future, umls_future, medline_future);

    let ols_docs = ols_docs.context("discover requires OLS4")?;
    let mut notes = Vec::new();
    if let Some(note) = umls_note
        && !note.trim().is_empty()
    {
        notes.push(note);
    }

    let result = crate::entities::discover::build_result(
        query,
        &ols_docs,
        &umls_rows,
        &medline_topics,
        notes,
    );

    if json {
        Ok(crate::render::json::to_discover_json(&result)?)
    } else {
        Ok(crate::render::markdown::render_discover(&result)?)
    }
}
