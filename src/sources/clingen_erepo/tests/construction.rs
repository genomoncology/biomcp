use super::super::*;
use crate::sources::HttpMethod;

#[test]
fn erepo_plans_use_exact_search_and_encoded_detail_segments() {
    let summary = ERepoClient::summary_plan("CA 015543");
    assert_eq!(summary.method, HttpMethod::Get);
    assert_eq!(summary.path, "evrepo/api/summary/classifications");
    assert_eq!(summary.query_value("columns"), Some("caId"));
    assert_eq!(summary.query_value("values"), Some("CA 015543"));
    assert_eq!(summary.query_value("matchTypes"), Some("exact"));
    assert_eq!(summary.query_value("pgSize"), Some("25"));
    assert_eq!(summary.query_value("pg"), Some("1"));

    let detail = ERepoClient::detail_plan("id/one", "1.0/rc");
    assert_eq!(detail.method, HttpMethod::Get);
    assert_eq!(
        detail.path,
        "evrepo/api/summary/classification/id%2Fone/doc/sepio/version/1.0%2Frc"
    );
}
