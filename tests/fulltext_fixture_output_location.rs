use std::path::Path;
use std::process::Command;

use biomcp_mcp_contract_client::provision_article_fulltext_fixture;

#[test]
fn fulltext_fixture_output_stays_out_of_source_inventory_and_cleans_up() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = provision_article_fulltext_fixture(repo_root).expect("start fulltext fixture");
    let workspace = fixture.cache_dir.parent().unwrap().to_path_buf();
    let cache_lock = fixture.cache_dir.join("blobs/fixture-only");
    std::fs::create_dir_all(cache_lock.parent().unwrap()).unwrap();
    std::fs::write(&cache_lock, b"fixture cache lock").unwrap();
    assert!(fixture.request_log.is_file());

    let inventory = Command::new("git")
        .args(["ls-files", "-z", "--others", "--exclude-standard"])
        .current_dir(repo_root)
        .output()
        .expect("read untracked source inventory");
    assert!(inventory.status.success());
    let inventory = String::from_utf8(inventory.stdout).unwrap();
    let relative_workspace = workspace.strip_prefix(repo_root).unwrap();
    let visible: Vec<_> = inventory
        .split('\0')
        .filter(|name| Path::new(name).starts_with(relative_workspace))
        .collect();
    assert!(
        visible.is_empty(),
        "live fulltext fixture output entered the source inventory: {visible:?}"
    );
    assert!(workspace.starts_with(repo_root.join(".cache/routine-test-tmp")));
    assert_eq!(
        fixture.cache_dir.file_name().unwrap(),
        "cache path naïve 🧬"
    );

    drop(fixture);
    assert!(
        !workspace.exists(),
        "fixture cleanup left its workspace behind"
    );
}
