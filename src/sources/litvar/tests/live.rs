//! Tier 4 — live upstream smoke. Ignored so normal gates stay pure and fast.

use crate::sources::litvar::LitVarClient;

#[tokio::test]
#[ignore = "live network"]
async fn live_autocomplete_and_publications_answer() {
    let deadline =
        crate::sources::VariantArticleDeadline::from_now(std::time::Duration::from_secs(60));
    let client = LitVarClient::new_with_deadline(&deadline)
        .await
        .expect("client");
    let entries = client
        .autocomplete("CDKN2A A57V")
        .await
        .expect("live LitVar2 autocomplete");
    let entry = entries
        .iter()
        .find(|entry| entry.id.as_deref() == Some("litvar@rs372266620##"))
        .expect("the recorded CDKN2A A57V entry");
    let publications = client
        .publications(entry.id.as_deref().unwrap_or_default())
        .await
        .expect("live LitVar2 publications");
    assert!(!publications.pmids.is_empty());
}
