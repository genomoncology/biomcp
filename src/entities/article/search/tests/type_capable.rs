use super::*;

fn failed_leg(
    source: ArticleSource,
    message: &str,
) -> FederatedSourceOutcome<SearchPage<ArticleSearchResult>> {
    FederatedSourceOutcome::Unavailable {
        error: Some(BioMcpError::InvalidArgument(message.to_string())),
        status: source_degraded_status(
            source,
            format!("{} search unavailable", source.display_name()),
        ),
    }
}

fn timed_out_leg(source: ArticleSource) -> FederatedSourceOutcome<SearchPage<ArticleSearchResult>> {
    FederatedSourceOutcome::Unavailable {
        error: None,
        status: timed_out_source_status(source),
    }
}

#[test]
fn both_available_sources_survive_in_source_order_without_degradation() {
    let collected = collect_type_capable_article_rows(
        FederatedSourceOutcome::Available(SearchPage::offset(
            vec![row("europe", ArticleSource::EuropePmc)],
            Some(1),
        )),
        FederatedSourceOutcome::Available(SearchPage::offset(
            vec![row("pubmed", ArticleSource::PubMed)],
            Some(1),
        )),
    )
    .expect("available capable sources should be collected");

    let pmids: Vec<&str> = collected.rows.iter().map(|row| row.pmid.as_str()).collect();
    assert_eq!(pmids, vec!["europe", "pubmed"]);
    assert_eq!(collected.total, None);
    assert!(collected.source_status.is_empty());
}

#[test]
fn europe_pmc_rows_and_total_survive_pubmed_failure() {
    let collected = collect_type_capable_article_rows(
        FederatedSourceOutcome::Available(SearchPage::offset(
            vec![row("europe", ArticleSource::EuropePmc)],
            Some(7),
        )),
        failed_leg(ArticleSource::PubMed, "pubmed failed"),
    )
    .expect("Europe PMC rows should survive PubMed failure");

    assert_eq!(collected.rows[0].source, ArticleSource::EuropePmc);
    assert_eq!(collected.total, Some(7));
    let status = collected.source_status.first().expect("PubMed status");
    assert_eq!(status.source, ArticleSource::PubMed);
    assert_eq!(status.status, Some(ArticleSourceAvailability::Degraded));
}

#[test]
fn pubmed_rows_and_total_survive_europe_pmc_failure() {
    let collected = collect_type_capable_article_rows(
        failed_leg(ArticleSource::EuropePmc, "europe failed"),
        FederatedSourceOutcome::Available(SearchPage::offset(
            vec![row("pubmed", ArticleSource::PubMed)],
            Some(9),
        )),
    )
    .expect("PubMed rows should survive Europe PMC failure");

    assert_eq!(collected.rows[0].source, ArticleSource::PubMed);
    assert_eq!(collected.total, Some(9));
    let status = collected.source_status.first().expect("Europe PMC status");
    assert_eq!(status.source, ArticleSource::EuropePmc);
    assert_eq!(status.status, Some(ArticleSourceAvailability::Degraded));
}

#[test]
fn dual_failure_preserves_europe_pmc_error() {
    let error = collect_type_capable_article_rows(
        failed_leg(ArticleSource::EuropePmc, "europe diagnostic"),
        failed_leg(ArticleSource::PubMed, "pubmed diagnostic"),
    )
    .err()
    .expect("both capable sources unavailable should fail");

    assert!(matches!(
        error,
        BioMcpError::InvalidArgument(message) if message == "europe diagnostic"
    ));
}

#[test]
fn pubmed_error_is_used_when_europe_pmc_timed_out() {
    let error = collect_type_capable_article_rows(
        timed_out_leg(ArticleSource::EuropePmc),
        failed_leg(ArticleSource::PubMed, "pubmed diagnostic"),
    )
    .err()
    .expect("both capable sources unavailable should fail");

    assert!(matches!(
        error,
        BioMcpError::InvalidArgument(message) if message == "pubmed diagnostic"
    ));
}

#[test]
fn dual_timeout_returns_deterministic_source_unavailable_error() {
    let error = collect_type_capable_article_rows(
        timed_out_leg(ArticleSource::EuropePmc),
        timed_out_leg(ArticleSource::PubMed),
    )
    .err()
    .expect("both capable sources timing out should fail");

    assert!(matches!(
        error,
        BioMcpError::SourceUnavailable { source_name, .. }
            if source_name == ArticleSource::EuropePmc.display_name()
    ));
}
