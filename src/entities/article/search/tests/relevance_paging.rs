use super::*;

use crate::test_support::TempDirGuard;

/// The recorded 25-row Europe PMC relevance reply: one full provider cursor
/// page for a BRAF melanoma query, receipted in testdata/sources.
const EUROPE_PMC_CURSOR_PAGE: &[u8] = include_bytes!(
    "../../../../../testdata/sources/europepmc/search_braf_melanoma_2012_page1_25rows.json"
);

#[serial_test::serial(source_env)]
#[tokio::test]
async fn europepmc_relevance_offset_page_lines_up_with_cursor_page() {
    let fixture = TestHttpFixture::spawn(|_| {
        TestHttpReply::Bytes(test_http_response(
            "200 OK",
            "application/json",
            EUROPE_PMC_CURSOR_PAGE,
        ))
    })
    .await;
    let cache = TempDirGuard::new("europepmc-relevance-paging");
    let mut env = TestEnv::new();
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    env.set("BIOMCP_CACHE_DIR", cache.path());

    let mut filters = empty_filters();
    filters.keyword = Some("BRAF melanoma".into());

    let whole = search_page(&filters, 10, 0, ArticleSourceFilter::EuropePmc)
        .await
        .expect("limit 10 relevance search answers");
    let head = search_page(&filters, 5, 0, ArticleSourceFilter::EuropePmc)
        .await
        .expect("limit 5 relevance search answers");
    let tail = search_page(&filters, 5, 5, ArticleSourceFilter::EuropePmc)
        .await
        .expect("limit 5 offset 5 relevance search answers");

    assert_eq!(whole.results.len(), 10, "limit 10 fills from one page");
    let paged: Vec<&str> = head
        .results
        .iter()
        .chain(tail.results.iter())
        .map(|row| row.pmid.as_str())
        .collect();
    let unpage: Vec<&str> = whole
        .results
        .iter()
        .map(|row| row.pmid.as_str())
        .collect();
    assert_eq!(
        paged, unpage,
        "offset page 1 must return the same articles in the same order as cursor page 1"
    );
}
