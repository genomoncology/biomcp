use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceProvider {
    label: &'static str,
}

impl SourceProvider {
    pub const ALPHAGENOME: Self = Self::new("AlphaGenome");
    pub const CANCER_HOTSPOTS: Self = Self::new("Cancer Hotspots");
    pub const CBIOPORTAL: Self = Self::new("cBioPortal");
    pub const CBIOPORTAL_DATAHUB: Self = Self::new("cBioPortal DataHub");
    pub const CHEMBL: Self = Self::new("ChEMBL");
    pub const CIVIC: Self = Self::new("CIViC");
    pub const CLINGEN: Self = Self::new("ClinGen");
    pub const CLINGEN_CAR: Self = Self::new("ClinGen Allele Registry");
    pub const CLINGEN_CSPEC: Self = Self::new("ClinGen CSpec");
    pub const CLINGEN_EREPO: Self = Self::new("ClinGen ERepo");
    pub const CLINGEN_LDH: Self = Self::new("ClinGen LDH");
    pub const CLINICAL_TRIALS: Self = Self::new("ClinicalTrials.gov");
    pub const COMPLEX_PORTAL: Self = Self::new("Complex Portal");
    pub const CPIC: Self = Self::new("CPIC");
    pub const CVX: Self = Self::new("CDC CVX/MVX");
    pub const DDINTER: Self = Self::new("DDInter");
    pub const DGIDB: Self = Self::new("DGIdb");
    pub const DISGENET: Self = Self::new("DisGeNET");
    pub const EMA: Self = Self::new("EMA");
    pub const ENRICHR: Self = Self::new("Enrichr");
    pub const EUROPE_PMC: Self = Self::new("Europe PMC");
    pub const FIGSHARE: Self = Self::new("Figshare");
    pub const GNOMAD: Self = Self::new("gnomAD");
    pub const GPROFILER: Self = Self::new("g:Profiler");
    pub const GTEX: Self = Self::new("GTEx");
    pub const GTR: Self = Self::new("NCBI Genetic Testing Registry");
    pub const GWAS: Self = Self::new("GWAS Catalog");
    pub const HPA: Self = Self::new("HPA");
    pub const HPO: Self = Self::new("HPO");
    pub const INTERPRO: Self = Self::new("InterPro");
    pub const KEGG: Self = Self::new("KEGG");
    pub const LITSENSE2: Self = Self::new("LitSense 2");
    pub const MEDLINEPLUS: Self = Self::new("MedlinePlus");
    pub const MONARCH: Self = Self::new("Monarch Initiative");
    pub const MUTALYZER: Self = Self::new("Mutalyzer");
    pub const MYCHEM: Self = Self::new("MyChem.info");
    pub const MYDISEASE: Self = Self::new("MyDisease.info");
    pub const MYGENE: Self = Self::new("MyGene.info");
    pub const MYVARIANT: Self = Self::new("MyVariant.info");
    pub const PUBMED: Self = Self::new("PubMed");
    pub const NCBI_EFETCH: Self = Self::new("NCBI EFetch");
    pub const NCBI_ID_CONVERTER: Self = Self::new("NCBI ID Converter");
    pub const NCI_CTS: Self = Self::new("NCI Clinical Trials Search");
    pub const NIH_REPORTER: Self = Self::new("NIH RePORTER");
    pub const OLS4: Self = Self::new("OLS4");
    pub const ONCOKB: Self = Self::new("OncoKB");
    pub const OPENFDA: Self = Self::new("OpenFDA");
    pub const OPEN_TARGETS: Self = Self::new("Open Targets");
    pub const PHARMGKB: Self = Self::new("PharmGKB");
    pub const PMC_OPEN_ACCESS: Self = Self::new("PMC Open Access");
    pub const PUBTATOR3: Self = Self::new("PubTator 3");
    pub const QUICKGO: Self = Self::new("QuickGO");
    pub const REACTOME: Self = Self::new("Reactome");
    pub const SEER: Self = Self::new("SEER Explorer");
    pub const SEMANTIC_SCHOLAR: Self = Self::new("Semantic Scholar");
    pub const STRING: Self = Self::new("STRING");
    pub const UMLS: Self = Self::new("UMLS");
    pub const UNIPROT: Self = Self::new("UniProt");
    pub const VAERS: Self = Self::new("VAERS");
    pub const VARIANT_VALIDATOR: Self = Self::new("VariantValidator");
    pub const WHO_IVD: Self = Self::new("WHO Prequalified IVD");
    pub const WHO_PREQUALIFICATION: Self = Self::new("WHO Prequalification");
    pub const WIKIPATHWAYS: Self = Self::new("WikiPathways");
    pub const UNKNOWN: Self = Self::new("BioMCP source");

    pub const ALL: &'static [Self] = &[
        Self::ALPHAGENOME,
        Self::CANCER_HOTSPOTS,
        Self::CBIOPORTAL,
        Self::CBIOPORTAL_DATAHUB,
        Self::CHEMBL,
        Self::CIVIC,
        Self::CLINGEN,
        Self::CLINGEN_CAR,
        Self::CLINGEN_CSPEC,
        Self::CLINGEN_EREPO,
        Self::CLINGEN_LDH,
        Self::CLINICAL_TRIALS,
        Self::COMPLEX_PORTAL,
        Self::CPIC,
        Self::CVX,
        Self::DDINTER,
        Self::DGIDB,
        Self::DISGENET,
        Self::EMA,
        Self::ENRICHR,
        Self::EUROPE_PMC,
        Self::FIGSHARE,
        Self::GNOMAD,
        Self::GPROFILER,
        Self::GTEX,
        Self::GTR,
        Self::GWAS,
        Self::HPA,
        Self::HPO,
        Self::INTERPRO,
        Self::KEGG,
        Self::LITSENSE2,
        Self::MEDLINEPLUS,
        Self::MONARCH,
        Self::MUTALYZER,
        Self::MYCHEM,
        Self::MYDISEASE,
        Self::MYGENE,
        Self::MYVARIANT,
        Self::PUBMED,
        Self::NCBI_EFETCH,
        Self::NCBI_ID_CONVERTER,
        Self::NCI_CTS,
        Self::NIH_REPORTER,
        Self::OLS4,
        Self::ONCOKB,
        Self::OPENFDA,
        Self::OPEN_TARGETS,
        Self::PHARMGKB,
        Self::PMC_OPEN_ACCESS,
        Self::PUBTATOR3,
        Self::QUICKGO,
        Self::REACTOME,
        Self::SEER,
        Self::SEMANTIC_SCHOLAR,
        Self::STRING,
        Self::UMLS,
        Self::UNIPROT,
        Self::VAERS,
        Self::VARIANT_VALIDATOR,
        Self::WHO_IVD,
        Self::WHO_PREQUALIFICATION,
        Self::WIKIPATHWAYS,
        Self::UNKNOWN,
    ];

    const fn new(label: &'static str) -> Self {
        Self { label }
    }

    pub const fn label(self) -> &'static str {
        self.label
    }

    fn from_legacy(name: &str) -> Option<Self> {
        Some(match name {
            "alphagenome" | "AlphaGenome" => Self::ALPHAGENOME,
            "cancerhotspots.org" | "Cancer Hotspots" => Self::CANCER_HOTSPOTS,
            "cbioportal" | "cBioPortal" => Self::CBIOPORTAL,
            "cbioportal-datahub" | "cbioportal-study" | "cBioPortal DataHub" => {
                Self::CBIOPORTAL_DATAHUB
            }
            "chembl" | "ChEMBL" => Self::CHEMBL,
            "civic" | "CIViC" => Self::CIVIC,
            "clingen" | "ClinGen" => Self::CLINGEN,
            "clingen_car" | "ClinGen Allele Registry" => Self::CLINGEN_CAR,
            "clingen_cspec" | "ClinGen CSpec" => Self::CLINGEN_CSPEC,
            "clingen_erepo" | "ClinGen ERepo" => Self::CLINGEN_EREPO,
            "clingen_ldh" | "ClinGen LDH" => Self::CLINGEN_LDH,
            "clinicaltrials.gov" | "ClinicalTrials.gov" => Self::CLINICAL_TRIALS,
            "complexportal" | "Complex Portal" => Self::COMPLEX_PORTAL,
            "cpic" | "CPIC" => Self::CPIC,
            "cdc-cvx" | "CDC CVX/MVX" => Self::CVX,
            "ddinter" | "DDInter" => Self::DDINTER,
            "dgidb" | "DGIdb" => Self::DGIDB,
            "disgenet" | "DisGeNET" => Self::DISGENET,
            "ema" | "EMA" => Self::EMA,
            "enrichr" | "Enrichr" => Self::ENRICHR,
            "europepmc" | "Europe PMC" => Self::EUROPE_PMC,
            "figshare" | "Figshare" => Self::FIGSHARE,
            "gnomAD" => Self::GNOMAD,
            "gprofiler" | "g:Profiler" => Self::GPROFILER,
            "gtex" | "GTEx" => Self::GTEX,
            "gtr" | "NCBI Genetic Testing Registry" => Self::GTR,
            "gwas" | "GWAS Catalog" => Self::GWAS,
            "hpa" | "HPA" => Self::HPA,
            "hpo" | "HPO" => Self::HPO,
            "interpro" | "InterPro" => Self::INTERPRO,
            "kegg" | "KEGG" => Self::KEGG,
            "litsense2" | "LitSense 2" => Self::LITSENSE2,
            "medlineplus" | "MedlinePlus" => Self::MEDLINEPLUS,
            "monarch" | "Monarch Initiative" => Self::MONARCH,
            "mutalyzer" | "Mutalyzer" => Self::MUTALYZER,
            "mychem.info" | "MyChem.info" => Self::MYCHEM,
            "mydisease.info" | "MyDisease.info" => Self::MYDISEASE,
            "mygene.info" | "MyGene.info" => Self::MYGENE,
            "myvariant.info" | "MyVariant.info" => Self::MYVARIANT,
            "pubmed-eutils" | "PubMed" => Self::PUBMED,
            "ncbi-efetch" | "NCBI EFetch" => Self::NCBI_EFETCH,
            "ncbi-idconv" | "NCBI ID Converter" => Self::NCBI_ID_CONVERTER,
            "nci_cts" | "NCI Clinical Trials Search" => Self::NCI_CTS,
            "nih_reporter" | "NIH RePORTER" => Self::NIH_REPORTER,
            "ols4" | "OLS4" => Self::OLS4,
            "oncokb" | "OncoKB" => Self::ONCOKB,
            "openfda" | "OpenFDA" => Self::OPENFDA,
            "opentargets" | "Open Targets" => Self::OPEN_TARGETS,
            "pharmgkb" | "PharmGKB" => Self::PHARMGKB,
            "pmc-oa" | "PMC Open Access" => Self::PMC_OPEN_ACCESS,
            "pubtator3" | "PubTator 3" => Self::PUBTATOR3,
            "quickgo" | "QuickGO" => Self::QUICKGO,
            "reactome" | "Reactome" => Self::REACTOME,
            "seer" | "SEER Explorer" => Self::SEER,
            "semantic_scholar" | "Semantic Scholar" => Self::SEMANTIC_SCHOLAR,
            "string" | "STRING" => Self::STRING,
            "umls" | "UMLS" => Self::UMLS,
            "uniprot" | "UniProt" => Self::UNIPROT,
            "vaers" | "VAERS" => Self::VAERS,
            "variantvalidator" | "VariantValidator" => Self::VARIANT_VALIDATOR,
            "who-ivd" | "WHO Prequalified IVD" => Self::WHO_IVD,
            "who-prequalification" | "WHO Prequalification" => Self::WHO_PREQUALIFICATION,
            "wikipathways" | "WikiPathways" => Self::WIKIPATHWAYS,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryAction {
    RetryRemoteSource,
    ReviewSourceConfiguration,
    NarrowRequest,
}

impl RecoveryAction {
    pub const fn message(self) -> &'static str {
        match self {
            Self::RetryRemoteSource => "Retry the remote source.",
            Self::ReviewSourceConfiguration => "Review source configuration and retry.",
            Self::NarrowRequest => "Narrow the request and retry.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceContext {
    provider: SourceProvider,
    recovery: RecoveryAction,
}

impl SourceContext {
    pub const fn new(provider: SourceProvider, recovery: RecoveryAction) -> Self {
        Self { provider, recovery }
    }

    pub const fn retry(provider: SourceProvider) -> Self {
        Self::new(provider, RecoveryAction::RetryRemoteSource)
    }

    pub const fn narrow(provider: SourceProvider) -> Self {
        Self::new(provider, RecoveryAction::NarrowRequest)
    }

    pub const fn provider(self) -> SourceProvider {
        self.provider
    }

    pub const fn recovery(self) -> RecoveryAction {
        self.recovery
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicErrorProjection {
    pub message: String,
    pub source: Option<&'static str>,
    pub recovery: Option<&'static str>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum BioMcpError {
    HttpClientInit(reqwest::Error),
    Http(reqwest::Error),
    HttpMiddleware(reqwest_middleware::Error),
    Api {
        api: String,
        message: String,
    },
    ApiJson {
        api: String,
        source: serde_json::Error,
    },
    BodyLimit {
        source_name: String,
        max_bytes: usize,
    },
    CtGovInterventionQueryRejected {
        reason: String,
    },
    NotFound {
        entity: String,
        id: String,
        suggestion: String,
    },
    InvalidArgument(String),
    CaptureUnavailable,
    CaptureCorrupt,
    BindingConflict,
    ApiKeyRequired {
        api: String,
        env_var: String,
        docs_url: String,
    },
    ApiKeyRejected {
        api: String,
        env_var: String,
        docs_url: String,
    },
    SourceUnavailable {
        source_name: String,
        reason: String,
        suggestion: String,
    },
    Template(minijinja::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
    WithSourceContext {
        context: SourceContext,
        source: Box<BioMcpError>,
    },
}

impl BioMcpError {
    pub fn with_source_context(self, context: SourceContext) -> Self {
        Self::WithSourceContext {
            context,
            source: Box::new(self),
        }
    }

    fn underlying(&self) -> &Self {
        match self {
            Self::WithSourceContext { source, .. } => source.underlying(),
            _ => self,
        }
    }

    fn message_for_source(&self, source: &'static str) -> String {
        match self.underlying() {
            Self::HttpClientInit(_) => "HTTP client initialization failed.".to_string(),
            Self::Http(_) | Self::HttpMiddleware(_) => {
                format!("HTTP request to {source} failed.")
            }
            Self::Api { .. } => format!("API request to {source} failed."),
            Self::ApiJson { .. } => format!("API response from {source} could not be decoded."),
            Self::BodyLimit { max_bytes, .. } => {
                format!("API error from {source}: Response body exceeded {max_bytes} bytes")
            }
            Self::SourceUnavailable { .. } => {
                format!("Source unavailable: {source} is not available.")
            }
            Self::CtGovInterventionQueryRejected { .. } => {
                format!("API request to {source} was rejected.")
            }
            Self::NotFound { .. } => format!("Requested item was not found in {source}."),
            Self::InvalidArgument(_) => format!("Invalid request for {source}."),
            Self::CaptureUnavailable | Self::CaptureCorrupt | Self::BindingConflict => {
                "Captured source material could not be used.".to_string()
            }
            Self::ApiKeyRequired { .. } => {
                format!("Source configuration for {source} is incomplete.")
            }
            Self::ApiKeyRejected { .. } => {
                format!("Source configuration for {source} was rejected.")
            }
            Self::Template(_) | Self::Json(_) | Self::Io(_) => {
                format!("Response from {source} could not be processed.")
            }
            Self::WithSourceContext { .. } => unreachable!("underlying error is never wrapped"),
        }
    }

    fn non_source_message(&self) -> String {
        match self {
            Self::HttpClientInit(_) => "HTTP client initialization failed.".to_string(),
            Self::Http(_) | Self::HttpMiddleware(_) => "HTTP request failed.".to_string(),
            Self::Api { api, .. } => format!("API request to {api} failed."),
            Self::ApiJson { api, .. } => format!("API response from {api} could not be decoded."),
            Self::BodyLimit {
                source_name,
                max_bytes,
            } => format!("API error from {source_name}: Response body exceeded {max_bytes} bytes"),
            Self::CtGovInterventionQueryRejected { .. } => {
                "ClinicalTrials.gov rejected the intervention query.".to_string()
            }
            Self::NotFound {
                entity,
                id,
                suggestion,
            } => format!("{entity} '{id}' not found.\n\n{suggestion}"),
            Self::InvalidArgument(message) => format!("Invalid argument: {message}"),
            Self::CaptureUnavailable => {
                "capture_unavailable: captured source material is unavailable".to_string()
            }
            Self::CaptureCorrupt => {
                "capture_corrupt: captured source material is corrupt".to_string()
            }
            Self::BindingConflict => {
                "binding_conflict: capture identity conflicts with the requested source".to_string()
            }
            Self::ApiKeyRequired {
                api,
                env_var,
                docs_url,
            } => format!(
                "API key required: {api} requires {env_var} environment variable.\n\nTo set:\n  export {env_var}=your-key\n\nMore info: {docs_url}"
            ),
            Self::ApiKeyRejected {
                api,
                env_var,
                docs_url,
            } => format!(
                "API key rejected: {api} rejected the configured {env_var} credential or the account lacks access.\n\nCheck the credential validity and account access.\n\nMore info: {docs_url}"
            ),
            Self::SourceUnavailable { source_name, .. } => format!(
                "Source unavailable: {source_name} is not available.\n\nCheck source setup and retry."
            ),
            Self::Template(_) => "Template rendering failed.".to_string(),
            Self::Json(_) => "JSON processing failed.".to_string(),
            Self::Io(_) => "I/O operation failed.".to_string(),
            Self::WithSourceContext { source, .. } => source.non_source_message(),
        }
    }

    pub fn public_projection(&self) -> PublicErrorProjection {
        let context = match self {
            Self::WithSourceContext { context, .. } => Some(*context),
            Self::Api { api, .. } | Self::ApiJson { api, .. } => Some(SourceContext::new(
                SourceProvider::from_legacy(api).unwrap_or(SourceProvider::UNKNOWN),
                if SourceProvider::from_legacy(api).is_some() {
                    RecoveryAction::RetryRemoteSource
                } else {
                    RecoveryAction::ReviewSourceConfiguration
                },
            )),
            Self::BodyLimit { source_name, .. } => Some(SourceContext::new(
                SourceProvider::from_legacy(source_name).unwrap_or(SourceProvider::UNKNOWN),
                if SourceProvider::from_legacy(source_name).is_some() {
                    RecoveryAction::NarrowRequest
                } else {
                    RecoveryAction::ReviewSourceConfiguration
                },
            )),
            Self::SourceUnavailable { source_name, .. } => Some(SourceContext::new(
                SourceProvider::from_legacy(source_name).unwrap_or(SourceProvider::UNKNOWN),
                RecoveryAction::ReviewSourceConfiguration,
            )),
            _ => None,
        };

        match context {
            Some(context) => PublicErrorProjection {
                message: if context.provider() == SourceProvider::SEMANTIC_SCHOLAR
                    && context.recovery() == RecoveryAction::ReviewSourceConfiguration
                    && matches!(self.underlying(), Self::Api { .. })
                {
                    "Rate limited by Semantic Scholar. Set S2_API_KEY for a dedicated rate limit."
                        .to_string()
                } else {
                    self.message_for_source(context.provider().label())
                },
                source: Some(context.provider().label()),
                recovery: Some(context.recovery().message()),
            },
            None => PublicErrorProjection {
                message: self.non_source_message(),
                source: None,
                recovery: None,
            },
        }
    }

    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::WithSourceContext { source, .. } => source.code(),
            Self::HttpClientInit(_) => "http_client_init",
            Self::Http(_) => "http",
            Self::HttpMiddleware(_) => "http_middleware",
            Self::Api { .. }
            | Self::BodyLimit { .. }
            | Self::CtGovInterventionQueryRejected { .. } => "api",
            Self::ApiJson { .. } => "api_json",
            Self::NotFound { .. } => "not_found",
            Self::InvalidArgument(_) => "invalid_argument",
            Self::CaptureUnavailable => "capture_unavailable",
            Self::CaptureCorrupt => "capture_corrupt",
            Self::BindingConflict => "binding_conflict",
            Self::ApiKeyRequired { .. } => "api_key_required",
            Self::ApiKeyRejected { .. } => "api_key_rejected",
            Self::SourceUnavailable { .. } => "source_unavailable",
            Self::Template(_) => "template",
            Self::Json(_) => "json",
            Self::Io(_) => "io",
        }
    }

    pub(crate) fn is_not_found(&self) -> bool {
        match self {
            Self::WithSourceContext { source, .. } => source.is_not_found(),
            Self::NotFound { .. } => true,
            _ => false,
        }
    }

    pub fn exit_code(&self) -> u8 {
        match self {
            Self::WithSourceContext { source, .. } => source.exit_code(),
            Self::InvalidArgument(_) => 2,
            _ => 1,
        }
    }
}

impl fmt::Display for BioMcpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(_)
            | Self::HttpMiddleware(_)
            | Self::Api { .. }
            | Self::ApiJson { .. }
            | Self::BodyLimit { .. }
            | Self::SourceUnavailable { .. }
            | Self::WithSourceContext { .. } => {
                let projection = self.public_projection();
                formatter.write_str(&projection.message)?;
                if let Some(recovery) = projection.recovery {
                    write!(formatter, " {recovery}")?;
                }
                Ok(())
            }
            Self::HttpClientInit(source) => {
                write!(formatter, "HTTP client initialization failed: {source}")
            }
            Self::CtGovInterventionQueryRejected { reason } => {
                write!(
                    formatter,
                    "ClinicalTrials.gov intervention query rejected: {reason}"
                )
            }
            Self::NotFound {
                entity,
                id,
                suggestion,
            } => write!(formatter, "{entity} '{id}' not found.\n\n{suggestion}"),
            Self::InvalidArgument(message) => write!(formatter, "Invalid argument: {message}"),
            Self::CaptureUnavailable => {
                formatter.write_str("capture_unavailable: captured source material is unavailable")
            }
            Self::CaptureCorrupt => {
                formatter.write_str("capture_corrupt: captured source material is corrupt")
            }
            Self::BindingConflict => formatter.write_str(
                "binding_conflict: capture identity conflicts with the requested source",
            ),
            Self::ApiKeyRequired {
                api,
                env_var,
                docs_url,
            } => write!(
                formatter,
                "API key required: {api} requires {env_var} environment variable.\n\nTo set:\n  export {env_var}=your-key\n\nMore info: {docs_url}"
            ),
            Self::ApiKeyRejected {
                api,
                env_var,
                docs_url,
            } => write!(
                formatter,
                "API key rejected: {api} rejected the configured {env_var} credential or the account lacks access.\n\nCheck the credential validity and account access.\n\nMore info: {docs_url}"
            ),
            Self::Template(source) => write!(formatter, "Template error: {source}"),
            Self::Json(source) => write!(formatter, "JSON error: {source}"),
            Self::Io(source) => write!(formatter, "IO error: {source}"),
        }
    }
}

impl std::error::Error for BioMcpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::HttpClientInit(source) | Self::Http(source) => Some(source),
            Self::HttpMiddleware(source) => Some(source),
            Self::ApiJson { source, .. } | Self::Json(source) => Some(source),
            Self::Template(source) => Some(source),
            Self::Io(source) => Some(source),
            Self::WithSourceContext { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for BioMcpError {
    fn from(source: reqwest::Error) -> Self {
        Self::Http(source)
    }
}

impl From<serde_json::Error> for BioMcpError {
    fn from(source: serde_json::Error) -> Self {
        Self::Json(source)
    }
}

impl From<minijinja::Error> for BioMcpError {
    fn from(source: minijinja::Error) -> Self {
        Self::Template(source)
    }
}

impl From<std::io::Error> for BioMcpError {
    fn from(source: std::io::Error) -> Self {
        Self::Io(source)
    }
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
    use super::{BioMcpError, RecoveryAction, SourceContext, SourceProvider};

    fn reqwest_error() -> reqwest::Error {
        reqwest::Client::new()
            .get("http://[::1")
            .build()
            .expect_err("invalid URL should fail")
    }

    #[test]
    fn source_policy_inventory_is_bounded_and_actionable() {
        for provider in SourceProvider::ALL {
            let label = provider.label();
            assert!(!label.is_empty());
            assert!(label.len() <= 80, "provider label exceeds bound: {label}");
            assert!(!label.contains(['\n', '\r']));
        }

        for action in [
            RecoveryAction::RetryRemoteSource,
            RecoveryAction::ReviewSourceConfiguration,
            RecoveryAction::NarrowRequest,
        ] {
            let message = action.message();
            assert!(!message.is_empty());
            assert!(message.len() <= 160);
            assert!(!message.contains(['\n', '\r']));
            assert!(message.to_ascii_lowercase().contains("retry"));
        }
    }

    #[test]
    fn source_wrapper_delegates_classifiers_for_every_wrappable_family() {
        let errors = [
            (BioMcpError::Http(reqwest_error()), "http", false, 1),
            (
                BioMcpError::HttpMiddleware(reqwest_middleware::Error::Reqwest(reqwest_error())),
                "http_middleware",
                false,
                1,
            ),
            (
                BioMcpError::Api {
                    api: "OLS4".into(),
                    message: "detail".into(),
                },
                "api",
                false,
                1,
            ),
            (
                BioMcpError::ApiJson {
                    api: "OLS4".into(),
                    source: serde_json::from_slice::<serde_json::Value>(b"{")
                        .expect_err("invalid JSON"),
                },
                "api_json",
                false,
                1,
            ),
            (
                BioMcpError::BodyLimit {
                    source_name: "OLS4".into(),
                    max_bytes: 42,
                },
                "api",
                false,
                1,
            ),
            (
                BioMcpError::SourceUnavailable {
                    source_name: "OLS4".into(),
                    reason: "detail".into(),
                    suggestion: "detail".into(),
                },
                "source_unavailable",
                false,
                1,
            ),
            (
                BioMcpError::NotFound {
                    entity: "gene".into(),
                    id: "x".into(),
                    suggestion: "retry".into(),
                },
                "not_found",
                true,
                1,
            ),
            (
                BioMcpError::InvalidArgument("bad".into()),
                "invalid_argument",
                false,
                2,
            ),
        ];

        for (error, code, not_found, exit_code) in errors {
            let wrapped = error.with_source_context(SourceContext::retry(SourceProvider::OLS4));
            assert_eq!(wrapped.code(), code);
            assert_eq!(wrapped.is_not_found(), not_found);
            assert_eq!(wrapped.exit_code(), exit_code);
        }
    }

    #[test]
    fn source_wrapper_never_projects_raw_non_source_fields() {
        let sentinel = "credential=fixture-secret /home/operator/private.json";
        let errors = [
            BioMcpError::NotFound {
                entity: sentinel.into(),
                id: sentinel.into(),
                suggestion: sentinel.into(),
            },
            BioMcpError::InvalidArgument(sentinel.into()),
            BioMcpError::CtGovInterventionQueryRejected {
                reason: sentinel.into(),
            },
            BioMcpError::ApiKeyRequired {
                api: sentinel.into(),
                env_var: sentinel.into(),
                docs_url: sentinel.into(),
            },
            BioMcpError::ApiKeyRejected {
                api: sentinel.into(),
                env_var: sentinel.into(),
                docs_url: sentinel.into(),
            },
            BioMcpError::Io(std::io::Error::other(sentinel)),
        ];

        for error in errors {
            let wrapped = error.with_source_context(SourceContext::retry(SourceProvider::OLS4));
            let projection = wrapped.public_projection();
            assert!(!projection.message.contains(sentinel), "{projection:?}");
            assert!(!wrapped.to_string().contains(sentinel));
        }
    }

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
