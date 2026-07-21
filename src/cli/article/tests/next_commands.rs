//! Article search next-command JSON tests.

use super::super::dispatch::{ArticleSearchJsonPage, article_search_json};
use crate::cli::PaginationMeta;

#[test]
fn article_search_json_keeps_exact_variant_follow_up_on_empty_page() {
    let mut filters = super::super::super::related_article_filters();
    filters.keyword = Some("MSH2 p.L341P".into());
    let next_commands = crate::render::markdown::search_next_commands_article(
        &[],
        &filters,
        crate::entities::article::ArticleSourceFilter::PubTator,
        &[],
    );
    let json = article_search_json(
        "keyword=\"MSH2 p.L341P\"",
        &filters,
        false,
        None,
        None,
        ArticleSearchJsonPage {
            results: Vec::new(),
            pagination: PaginationMeta::offset(0, 1, 0, Some(0)),
            next_commands,
            suggestions: Vec::new(),
            source_status: Vec::new(),
        },
    )
    .expect("empty article search JSON");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid article JSON");
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| commands.contains(&serde_json::Value::String(
                "biomcp variant articles \"MSH2 p.L341P\"".into()
            )))
    );
}
