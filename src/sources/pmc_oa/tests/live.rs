//! Tier 4 — live upstream smoke. Ignored so normal gates stay pure and fast.

use crate::sources::pmc_oa::PmcOaClient;
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "live network"]
async fn live_archive_manifest_lookup_retrieves_receipted_s3_xml_bytes() {
    let client = PmcOaClient::new().expect("client");
    let (xml, manifest) = client
        .get_full_text_xml_with_manifest("PMC9984800")
        .await
        .expect("live PMC OA package route should resolve")
        .expect("known open-access article should have XML");

    assert!(
        manifest
            .tgz_url
            .starts_with("https://pmc-oa-opendata.s3.amazonaws.com/")
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(xml.as_bytes())),
        "c1059f44a3a6a25826cbeeae88e8c30c903ecbc2544ac622c2a18acac027d303"
    );
}
