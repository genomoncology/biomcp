use super::*;

fn sections(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn assets_is_standalone_json_only_route() {
    assert!(article_asset_route(&sections(&["assets"])));
    assert!(article_assets_request(&sections(&["assets"])).unwrap());
    let err = article_assets_request(&sections(&["assets", "fulltext"])).unwrap_err();
    assert!(err.to_string().contains("standalone JSON-only"));
}

#[test]
fn asset_requires_exactly_one_key_and_no_assets_section() {
    assert_eq!(
        article_asset_request(&sections(&["asset", "supplement.pdf"]))
            .unwrap()
            .as_deref(),
        Some("supplement.pdf")
    );
    for bad in [
        sections(&["asset"]),
        sections(&["asset", "supplement.pdf", "fulltext"]),
        sections(&["asset", "supplement.pdf", "assets"]),
        sections(&["asset", " "]),
    ] {
        let err = article_asset_request(&bad).unwrap_err();
        assert!(err.to_string().contains("asset"));
    }
}

#[test]
fn asset_pages_are_stable_and_emit_exact_continuations() {
    let values = vec![serde_json::json!({"key": 1}), serde_json::json!({"key": 2})];
    assert_eq!(page_values(&values, 1, 1), vec![values[1].clone()]);
    let page = pagination("PMC 1", "retrievable", 0, 1, 1, 2);
    assert_eq!(page["has_more"], true);
    assert_eq!(page["next_offset"], 1);
    assert_eq!(
        page["continuation_command"],
        "biomcp --json get article \"PMC 1\" --asset-view retrievable --asset-limit 1 --asset-offset 1 assets"
    );
}

#[test]
fn exact_duplicate_manifest_rows_are_removed_in_order() {
    let first = serde_json::json!({"key": 1});
    let second = serde_json::json!({"key": 2});
    let mut rows = vec![first.clone(), second.clone(), first];
    dedupe_values(&mut rows);
    assert_eq!(rows, vec![serde_json::json!({"key": 1}), second]);
}
