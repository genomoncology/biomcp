#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum BioMcpError {
    #[error("HTTP client initialization failed: {0}")]
    HttpClientInit(reqwest::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP middleware error: {0}")]
    HttpMiddleware(reqwest_middleware::Error),

    #[error("API error from {api}: {message}")]
    Api { api: String, message: String },

    #[error("API JSON error from {api}: {source}")]
    ApiJson {
        api: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("API error from {source_name}: Response body exceeded {max_bytes} bytes")]
    BodyLimit {
        source_name: String,
        max_bytes: usize,
    },

    #[error("ClinicalTrials.gov intervention query rejected: {reason}")]
    CtGovInterventionQueryRejected { reason: String },

    #[error("{entity} '{id}' not found.\n\n{suggestion}")]
    NotFound {
        entity: String,
        id: String,
        suggestion: String,
    },

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error(
        "API key required: {api} requires {env_var} environment variable.\n\nTo set:\n  export {env_var}=your-key\n\nMore info: {docs_url}"
    )]
    ApiKeyRequired {
        api: String,
        env_var: String,
        docs_url: String,
    },

    #[error(
        "API key rejected: {api} rejected the configured {env_var} credential or the account lacks access.\n\nCheck the credential validity and account access.\n\nMore info: {docs_url}"
    )]
    ApiKeyRejected {
        api: String,
        env_var: String,
        docs_url: String,
    },

    #[error("Source unavailable: {source_name} is not available. {reason}\n\nTry: {suggestion}")]
    SourceUnavailable {
        source_name: String,
        reason: String,
        suggestion: String,
    },

    #[error("Template error: {0}")]
    Template(#[from] minijinja::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<reqwest_middleware::Error> for BioMcpError {
    fn from(err: reqwest_middleware::Error) -> Self {
        if let reqwest_middleware::Error::Middleware(source) = &err {
            if let Some(limit) = source
                .downcast_ref::<crate::sources::ResponseBodyLimitError>()
                .or_else(|| {
                    source.chain().find_map(|cause| {
                        cause.downcast_ref::<crate::sources::ResponseBodyLimitError>()
                    })
                })
            {
                return Self::BodyLimit {
                    source_name: limit.source_name.to_string(),
                    max_bytes: limit.max_bytes,
                };
            }

            // http-cache wraps the inner BoxError in anyhow and does not retain the
            // concrete type in its source chain. This private marker carries only the
            // same payload-free fields as ResponseBodyLimitError.
            if let Some(marker) = source
                .chain()
                .map(ToString::to_string)
                .find(|message| message.starts_with("biomcp-response-body-limit|"))
            {
                let mut parts = marker.split('|');
                if parts.next() == Some("biomcp-response-body-limit")
                    && let (Some(source_name), Some(max_bytes), None) =
                        (parts.next(), parts.next(), parts.next())
                    && let Ok(max_bytes) = max_bytes.parse()
                {
                    return Self::BodyLimit {
                        source_name: source_name.to_string(),
                        max_bytes,
                    };
                }
            }
        }
        Self::HttpMiddleware(err)
    }
}

#[cfg(test)]
mod tests {
    use super::BioMcpError;

    #[test]
    fn not_found_display_includes_suggestion() {
        let err = BioMcpError::NotFound {
            entity: "gene".to_string(),
            id: "BRAF".to_string(),
            suggestion: "Try searching: biomcp search gene -q BRAF".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("gene 'BRAF' not found"));
        assert!(msg.contains("Try searching"));
    }

    #[test]
    fn api_key_required_display_includes_env_var_and_docs() {
        let err = BioMcpError::ApiKeyRequired {
            api: "nci_cts".to_string(),
            env_var: "NCI_API_KEY".to_string(),
            docs_url: "https://clinicaltrialsapi.cancer.gov/".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("NCI_API_KEY"));
        assert!(msg.contains("https://clinicaltrialsapi.cancer.gov/"));
    }

    #[test]
    fn api_key_rejected_display_includes_recovery_guidance() {
        let err = BioMcpError::ApiKeyRejected {
            api: "disgenet".to_string(),
            env_var: "DISGENET_API_KEY".to_string(),
            docs_url: "https://www.disgenet.com/".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("DISGENET_API_KEY"));
        assert!(msg.contains("rejected"));
        assert!(msg.contains("access"));
        assert!(msg.contains("https://www.disgenet.com/"));
    }

    #[test]
    fn human_source_errors_share_safe_projection() {
        let sentinels = [
            "credential=fixture-secret",
            "https://signed.example/private?token=fixture-secret",
            "raw provider payload",
            "parser detail at byte 42",
            "/home/operator/private.json",
            "hostile-provider-label",
            "\u{1b}[31mterminal-red",
            "\u{202e}bidi-override",
        ];
        let errors = [
            (
                BioMcpError::SourceUnavailable {
                    source_name: "ClinicalTrials.gov".to_string(),
                    reason: format!("{}: {}", sentinels[0], sentinels[2]),
                    suggestion: format!("Read {} then retry {}", sentinels[4], sentinels[1]),
                },
                "ClinicalTrials.gov",
            ),
            (
                BioMcpError::Api {
                    api: "OLS4".to_string(),
                    message: format!("{} {}", sentinels[1], sentinels[3]),
                },
                "OLS4",
            ),
            (
                BioMcpError::Api {
                    api: format!(
                        "{} {} {} {}",
                        sentinels[5], sentinels[0], sentinels[6], sentinels[7]
                    ),
                    message: format!("{} {}", sentinels[2], sentinels[4]),
                },
                "BioMCP source",
            ),
        ];

        for (error, expected_source) in &errors {
            let diagnostic = crate::cli::sanitize_human_diagnostic(&error.to_string());
            assert!(
                diagnostic.contains(expected_source),
                "human error must name its normalized source: {diagnostic}"
            );
            assert!(
                diagnostic.to_ascii_lowercase().contains("retry"),
                "human error must include a recovery action: {diagnostic}"
            );
            for sentinel in sentinels {
                assert!(
                    !diagnostic.contains(sentinel),
                    "human error leaked {sentinel}: {diagnostic}"
                );
            }
        }
    }
}
