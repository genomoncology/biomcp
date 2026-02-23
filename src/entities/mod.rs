//! Entity-level query and retrieval workflows used by the CLI.

pub(crate) mod adverse_event;
pub(crate) mod article;
pub(crate) mod biomarker;
pub(crate) mod disease;
pub(crate) mod drug;
pub(crate) mod gene;
pub(crate) mod intervention;
pub(crate) mod organization;
pub(crate) mod pathway;
pub(crate) mod pgx;
pub(crate) mod protein;
pub(crate) mod trial;
pub(crate) mod variant;

#[derive(Debug, Clone)]
pub(crate) struct SearchPage<T> {
    pub results: Vec<T>,
    pub total: Option<usize>,
    pub next_page_token: Option<String>,
}

impl<T> SearchPage<T> {
    pub(crate) fn offset(results: Vec<T>, total: Option<usize>) -> Self {
        Self {
            results,
            total,
            next_page_token: None,
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
        }
    }
}
