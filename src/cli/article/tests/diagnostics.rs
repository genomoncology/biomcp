//! Article search `--full` diagnostics JSON tests.

use super::super::ArticleSearchDetail;
use super::super::dispatch::{ArticleSearchJsonPage, article_search_json_with_detail};
use crate::cli::PaginationMeta;

#[test]
fn article_search_diagnostics_render_only_in_full_detail() {
    let filters = super::super::super::related_article_filters();
    let page = || ArticleSearchJsonPage {
        results: Vec::new(),
        pagination: PaginationMeta::offset(0, 1, 0, Some(0)),
        next_commands: Vec::new(),
        suggestions: Vec::new(),
        source_status: Vec::new(),
        diagnostics: crate::entities::article::ArticleSearchDiagnostics {
            deadline_ms: 60_000,
            source_timings: vec![crate::entities::article::ArticleSearchTiming {
                source: Some(crate::entities::article::ArticleSource::EuropePmc),
                stage: "search",
                elapsed_ms: 12_001,
            }],
        },
    };

    let compact = article_search_json_with_detail(
        "BRAF melanoma",
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        ArticleSearchDetail::Compact,
        false,
        None,
        None,
        page(),
    )
    .expect("compact JSON");
    assert!(
        !compact.contains("diagnostics"),
        "compact output omits diagnostics: {compact}"
    );

    let full = article_search_json_with_detail(
        "BRAF melanoma",
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        ArticleSearchDetail::Full,
        false,
        None,
        None,
        page(),
    )
    .expect("full JSON");
    let value: serde_json::Value = serde_json::from_str(&full).expect("full value");
    assert_eq!(value["diagnostics"]["deadline_ms"], 60_000);
    assert_eq!(
        value["diagnostics"]["source_timings"][0]["source"],
        "europepmc"
    );
    assert_eq!(value["diagnostics"]["source_timings"][0]["stage"], "search");
    assert_eq!(
        value["diagnostics"]["source_timings"][0]["elapsed_ms"],
        12_001
    );
}
