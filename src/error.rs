#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum BioMcpError {
    #[error("HTTP client initialization failed: {0}")]
    HttpClientInit(reqwest::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP middleware error: {0}")]
    HttpMiddleware(#[from] reqwest_middleware::Error),

    #[error("API error from {api}: {message}")]
    Api { api: String, message: String },

    #[error("API JSON error from {api}: {source}")]
    ApiJson {
        api: String,
        #[source]
        source: serde_json::Error,
    },

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
    fn source_unavailable_display_includes_reason() {
        let err = BioMcpError::SourceUnavailable {
            source_name: "nci".to_string(),
            reason: "Service is under maintenance".to_string(),
            suggestion: "Try --source ctgov".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("Source unavailable: nci"));
        assert!(msg.contains("Service is under maintenance"));
        assert!(msg.contains("Try --source ctgov"));
    }

    #[test]
    fn api_error_display_includes_api_name() {
        let err = BioMcpError::Api {
            api: "opentargets".to_string(),
            message: "HTTP 500".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("opentargets"));
        assert!(msg.contains("HTTP 500"));
    }
}
