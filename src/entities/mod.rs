//! Entity-level query and retrieval workflows used by the CLI.

pub(crate) mod adverse_event;
pub(crate) mod article;
pub(crate) mod author;
pub(crate) mod cell_line;
pub(crate) mod diagnostic;
pub(crate) mod discover;
pub(crate) mod disease;
pub(crate) mod drug;
pub(crate) mod gene;
pub(crate) mod pathway;
pub(crate) mod pgx;
pub(crate) mod pharmacodb;
pub(crate) mod protein;
pub(crate) mod section_outcome;
pub(crate) mod source_state_registry;
pub(crate) mod study;
pub(crate) mod trial;
pub(crate) mod variant;

/// A genomic coordinate together with the assembly and provider evidence that
/// make the coordinate meaningful.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenomicCoordinate {
    pub coordinate: String,
    pub genome_build: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct SearchPage<T> {
    pub results: Vec<T>,
    pub total: Option<usize>,
    pub next_page_token: Option<String>,
    pub upstream_total: Option<usize>,
    /// Degradation note for the whole page: set when some kept rows
    /// were produced without full verification (a source failure the
    /// renderers surface instead of dropping).
    pub partial_note: Option<String>,
}

impl<T> SearchPage<T> {
    pub(crate) fn offset(results: Vec<T>, total: Option<usize>) -> Self {
        Self {
            results,
            total,
            next_page_token: None,
            upstream_total: None,
            partial_note: None,
        }
    }

    pub(crate) fn cursor(
        results: Vec<T>,
        total: Option<usize>,
        next_page_token: Option<String>,
    ) -> Self {
        Self {
            results,
            total,
            next_page_token,
            upstream_total: None,
            partial_note: None,
        }
    }

    pub(crate) fn cursor_with_upstream(
        results: Vec<T>,
        total: Option<usize>,
        next_page_token: Option<String>,
        upstream_total: Option<usize>,
    ) -> Self {
        Self {
            results,
            total,
            next_page_token,
            upstream_total,
            partial_note: None,
        }
    }
}
