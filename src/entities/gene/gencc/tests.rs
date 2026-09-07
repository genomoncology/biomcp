use std::sync::atomic::AtomicBool;

use super::*;
use crate::sources::gencc::model::GenCcDataset;

fn dataset() -> GenCcDataset {
    GenCcDataset::parse(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gencc/submissions-new-odc1.csv"
        )),
        &AtomicBool::new(false),
    )
    .unwrap()
}

fn data() -> GenCcData {
    GenCcData {
        dataset: Some(dataset()),
        status: GenCcStatus {
            freshness: GenCcFreshness::Fresh,
            result: GenCcResult::Data,
            operation: GenCcOperation::LocalQuery,
            checked_at: Some("2026-09-05T22:51:21Z".into()),
            retrieved_at: Some("2026-09-05T22:51:21Z".into()),
            attempted_at: Some("2026-09-05T22:51:21Z".into()),
            etag: Some("\"fixture\"".into()),
            last_modified: Some("Sun, 30 Aug 2026 06:00:29 GMT".into()),
            upstream_version: None,
            message: None,
        },
        lease: None,
    }
}

#[test]
fn canonical_symbol_and_hgnc_return_three_submission_rows() {
    let (section, outcome) = project("ODC1", Some("HGNC:8109"), data());
    assert_eq!(section.assertions.len(), 3);
    assert_eq!(section.total_matching_assertions, 3);
    assert!(!section.truncated);
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Data
    );
}

#[test]
fn missing_hgnc_uses_unique_symbol_identity() {
    let (section, _) = project("odc1", None, data());
    assert_eq!(section.assertions.len(), 3);
}

#[test]
fn one_sided_identity_match_is_inconclusive() {
    for (symbol, hgnc) in [("ODC1", "HGNC:42"), ("OTHER", "HGNC:8109")] {
        let (section, outcome) = project(symbol, Some(hgnc), data());
        assert_eq!(section.status.operation, GenCcOperation::IdentityMatch);
        assert_eq!(section.status.result, GenCcResult::Unknown);
        assert!(section.assertions.is_empty());
        assert_eq!(
            outcome.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Unavailable
        );
    }
}

#[test]
fn index_identity_conflict_overrides_every_queryable_lifecycle_status_without_losing_events() {
    for (freshness, operation) in [
        (GenCcFreshness::Fresh, GenCcOperation::InitialDownload),
        (GenCcFreshness::Fresh, GenCcOperation::LocalQuery),
        (GenCcFreshness::Fresh, GenCcOperation::ConditionalRefresh),
        (GenCcFreshness::Stale, GenCcOperation::ConditionalRefresh),
        (GenCcFreshness::Stale, GenCcOperation::RetrySuppressed),
        (GenCcFreshness::Stale, GenCcOperation::RefreshDeferred),
    ] {
        let mut candidate = data();
        candidate.status.freshness = freshness;
        candidate.status.operation = operation;
        let expected_events = (
            candidate.status.checked_at.clone(),
            candidate.status.retrieved_at.clone(),
            candidate.status.attempted_at.clone(),
        );
        let (section, outcome) = project("ODC1", Some("HGNC:42"), candidate);
        assert_eq!(section.status.operation, GenCcOperation::IdentityMatch);
        assert_eq!(section.status.freshness, GenCcFreshness::Unavailable);
        assert_eq!(section.status.result, GenCcResult::Unknown);
        assert_eq!(
            (
                section.status.checked_at,
                section.status.retrieved_at,
                section.status.attempted_at
            ),
            expected_events
        );
        assert!(section.assertions.is_empty());
        assert!(outcome.sources().is_empty());
    }
}

#[tokio::test]
#[serial_test::serial(gencc_env)]
async fn pre_index_identity_failure_precedes_store_creation() {
    let temp = tempfile::tempdir().unwrap();
    secure_anchor(temp.path());
    let root = temp.path().join("gencc");
    unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
    for (symbol, hgnc) in [
        ("", Err(())),
        ("ODC1", Ok(vec!["HGNC:1".into(), "HGNC:2".into()])),
    ] {
        let (section, _) = fetch_section(symbol, hgnc, std::time::Duration::from_millis(20)).await;
        assert_eq!(section.status.operation, GenCcOperation::IdentityMatch);
        assert!(!root.exists());
    }
    unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
}

#[test]
fn stale_positive_and_zero_outcomes_preserve_the_lifecycle_message() {
    let mut positive = data();
    positive.status.freshness = GenCcFreshness::Stale;
    positive.status.operation = GenCcOperation::RefreshDeferred;
    positive.status.message = Some(
        "GenCC refresh is still in progress; results come from the last validated dataset.".into(),
    );
    let (section, outcome) = project("ODC1", Some("HGNC:8109"), positive);
    assert_eq!(outcome.message(), section.status.message.as_deref());
    assert_eq!(outcome.sources(), &["GenCC"]);

    let mut zero = data();
    zero.status.freshness = GenCcFreshness::Stale;
    zero.status.message =
        Some("GenCC refresh failed; results come from the last validated dataset.".into());
    let (section, outcome) = project("NOTFOUND", None, zero);
    assert_eq!(section.status.result, GenCcResult::Empty);
    assert_eq!(outcome.message(), section.status.message.as_deref());
    assert!(outcome.sources().is_empty());
}

#[test]
fn assertion_cap_is_separate_from_the_total() {
    let mut reader = csv::Reader::from_reader(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gencc/submissions-new-odc1.csv"
        ))
        .as_slice(),
    );
    let header = reader.headers().unwrap().clone();
    let base = reader.records().next().unwrap().unwrap();
    let mut bytes = Vec::new();
    {
        let mut writer = csv::Writer::from_writer(&mut bytes);
        writer.write_record(&header).unwrap();
        for index in 1..=101 {
            let mut fields = base.iter().map(str::to_string).collect::<Vec<_>>();
            fields[0] = format!("SGC-{index}");
            writer.write_record(fields).unwrap();
        }
        writer.flush().unwrap();
    }
    let data = GenCcData {
        dataset: Some(GenCcDataset::parse(&bytes, &AtomicBool::new(false)).unwrap()),
        status: data().status,
        lease: None,
    };
    let (section, _) = project("ODC1", Some("HGNC:8109"), data);
    assert_eq!(section.assertions.len(), 100);
    assert_eq!(section.total_matching_assertions, 101);
    assert!(section.truncated);
}
use std::fs;
use std::process::{Command, Stdio};

use crate::sources::gencc::ENDPOINT;
use crate::sources::gencc::store::{
    PUBLICATION_CRASH_POINTS, PublishMetadata, Snapshot, Store, StoreError,
};
use sha2::{Digest, Sha256};

fn fixture() -> &'static [u8] {
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/gencc/submissions-new-odc1.csv"
    ))
}

#[cfg(unix)]
fn secure_anchor(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[cfg(not(unix))]
fn secure_anchor(_path: &std::path::Path) {}

fn publish(store: &Store, dataset: &GenCcDataset, now: &str, etag: &str) -> Snapshot {
    store
        .publish(
            dataset,
            PublishMetadata {
                now,
                etag,
                last_modified: "Sun, 06 Sep 2026 06:00:29 GMT",
                endpoint: ENDPOINT,
                body_sha256: &format!("{:x}", Sha256::digest(fixture())),
                row_count: dataset.row_count(),
            },
        )
        .unwrap()
}

#[test]
#[serial_test::serial(gencc_env)]
#[cfg(unix)]
fn descriptor_bootstrap_rejects_unsafe_or_substituted_anchors_and_reuses_inode() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    for attack in ["writable", "symlink"] {
        let temp = tempfile::tempdir().unwrap();
        secure_anchor(temp.path());
        let anchor = temp.path().join("anchor");
        fs::create_dir(&anchor).unwrap();
        fs::set_permissions(&anchor, fs::Permissions::from_mode(0o700)).unwrap();
        let selected_parent = if attack == "writable" {
            fs::set_permissions(&anchor, fs::Permissions::from_mode(0o770)).unwrap();
            anchor
        } else {
            let link = temp.path().join("link");
            std::os::unix::fs::symlink(&anchor, &link).unwrap();
            link
        };
        let root = selected_parent.join("gencc");
        unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
        assert!(Store::open().is_err(), "{attack}");
        assert!(!root.exists());
        unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
    }
    for attack in ["root-mode", "lock-link"] {
        let temp = tempfile::tempdir().unwrap();
        secure_anchor(temp.path());
        let root = temp.path().join("gencc");
        fs::create_dir(&root).unwrap();
        secure_anchor(&root);
        if attack == "root-mode" {
            fs::set_permissions(&root, fs::Permissions::from_mode(0o750)).unwrap();
        } else {
            std::os::unix::fs::symlink(temp.path().join("outside"), root.join(".store.lock"))
                .unwrap();
        }
        unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
        assert!(Store::open().is_err(), "{attack}");
        unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
    }
    let temp = tempfile::tempdir().unwrap();
    secure_anchor(temp.path());
    let root = temp.path().join("gencc");
    unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
    drop(Store::open().unwrap());
    let anchor = fs::read_dir(temp.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".biomcp-gencc-root-")
        })
        .unwrap();
    let identity = (
        fs::metadata(&anchor).unwrap().dev(),
        fs::metadata(&anchor).unwrap().ino(),
    );
    fs::remove_dir_all(&root).unwrap();
    drop(Store::open().unwrap());
    assert_eq!(
        (
            fs::metadata(&anchor).unwrap().dev(),
            fs::metadata(&anchor).unwrap().ino()
        ),
        identity
    );
    unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
}

#[test]
#[serial_test::serial(gencc_env)]
#[cfg(unix)]
fn publication_rejects_a_substituted_generations_component_without_writing_through_it() {
    let temp = tempfile::tempdir().unwrap();
    secure_anchor(temp.path());
    let root = temp.path().join("gencc");
    let outside = temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    secure_anchor(&outside);
    unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
    let store = Store::open().unwrap();
    fs::remove_dir(root.join("generations")).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("generations")).unwrap();
    let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
    let result = store.publish(
        &dataset,
        PublishMetadata {
            now: "2026-01-01T00:00:00Z",
            etag: "\"substitution\"",
            last_modified: "Sun, 06 Sep 2026 06:00:29 GMT",
            endpoint: ENDPOINT,
            body_sha256: &format!("{:x}", Sha256::digest(fixture())),
            row_count: dataset.row_count(),
        },
    );
    assert!(result.is_err());
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
}

#[test]
#[serial_test::serial(gencc_env)]
#[cfg(unix)]
fn default_and_override_subprocesses_reuse_the_external_anchor_inode_after_root_recreation() {
    use std::os::unix::fs::MetadataExt;
    for default_root in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        secure_anchor(temp.path());
        let root = if default_root {
            temp.path().join("biomcp/gencc")
        } else {
            temp.path().join("gencc")
        };
        let run = || {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--ignored",
                    "--exact",
                    "sources::gencc::tests::gencc_subprocess_client",
                ])
                .env("BIOMCP_GENCC_CHILD_OPEN", "1")
                .env("XDG_DATA_HOME", temp.path())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            if default_root {
                command.env_remove("BIOMCP_GENCC_DIR");
            } else {
                command.env("BIOMCP_GENCC_DIR", &root);
            }
            assert!(command.status().unwrap().success());
        };
        run();
        let anchor = fs::read_dir(temp.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".biomcp-gencc-root-")
            })
            .unwrap();
        let metadata = fs::metadata(&anchor).unwrap();
        let identity = (metadata.dev(), metadata.ino());
        fs::remove_dir_all(&root).unwrap();
        run();
        let metadata = fs::metadata(anchor).unwrap();
        assert_eq!((metadata.dev(), metadata.ino()), identity);
    }
}

#[test]
#[serial_test::serial(gencc_env)]
fn crash_boundaries_preserve_one_complete_namespace_generation() {
    for point in PUBLICATION_CRASH_POINTS {
        let temp = tempfile::tempdir().unwrap();
        secure_anchor(temp.path());
        let root = temp.path().join("gencc");
        let marker = temp.path().join("crashed");
        let previous = std::env::var_os("BIOMCP_GENCC_DIR");
        unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
        let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
        let old = publish(
            &Store::open().unwrap(),
            &dataset,
            "2026-01-01T00:00:00Z",
            "\"crash-old\"",
        )
        .state
        .active_generation
        .unwrap();
        match previous {
            Some(value) => unsafe { std::env::set_var("BIOMCP_GENCC_DIR", value) },
            None => unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") },
        }
        let status = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "sources::gencc::tests::gencc_subprocess_client",
            ])
            .env("BIOMCP_GENCC_DIR", &root)
            .env("BIOMCP_GENCC_CHILD_CRASH_PUBLISH", "1")
            .env("BIOMCP_GENCC_TEST_CRASH_AT", point)
            .env("BIOMCP_GENCC_TEST_CRASH_MARKER", &marker)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(!status.success(), "{point}");
        assert_eq!(fs::read_to_string(&marker).unwrap(), point);
        unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
        let store = Store::open().unwrap();
        let visible = store.load().unwrap().unwrap();
        let renamed = matches!(
            point,
            "after-state-rename" | "before-root-directory-fsync" | "after-root-directory-fsync"
        );
        assert_eq!(
            visible.state.active_generation.as_deref() != Some(&old),
            renamed,
            "{point}"
        );
        assert_eq!(visible.manifest.etag == "\"crash-new\"", renamed, "{point}");
        store.cleanup_abandoned();
        unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
    }
}

#[test]
#[serial_test::serial(gencc_env)]
fn first_publication_crash_recovery_never_requires_an_incomplete_generation() {
    for point in PUBLICATION_CRASH_POINTS.into_iter().filter(|point| {
        point.contains("temporary-generations")
            || point.contains("index-")
            || point.contains("lease-")
            || point.contains("manifest-")
            || point.contains("generation-directory")
            || point.contains("generation-rename")
            || point.contains("generations-directory")
            || point.contains("state-")
            || point.contains("root-directory")
    }) {
        let temp = tempfile::tempdir().unwrap();
        secure_anchor(temp.path());
        let root = temp.path().join("gencc");
        let marker = temp.path().join("crashed");
        let status = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "sources::gencc::tests::gencc_subprocess_client",
            ])
            .env("BIOMCP_GENCC_DIR", &root)
            .env("BIOMCP_GENCC_CHILD_CRASH_PUBLISH", "1")
            .env("BIOMCP_GENCC_TEST_CRASH_AT", point)
            .env("BIOMCP_GENCC_TEST_CRASH_MARKER", &marker)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(!status.success(), "{point}");
        unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
        let snapshot = Store::open().unwrap().load().unwrap();
        let finalized = matches!(
            point,
            "after-generation-rename"
                | "before-generations-directory-fsync"
                | "after-generations-directory-fsync"
                | "before-state-file-fsync"
                | "after-state-file-fsync"
                | "before-state-rename"
                | "after-state-rename"
                | "before-root-directory-fsync"
                | "after-root-directory-fsync"
        );
        assert_eq!(snapshot.is_some(), finalized, "{point}");
        if let Some(snapshot) = snapshot {
            assert_eq!(snapshot.manifest.etag, "\"crash-new\"");
            assert_eq!(snapshot.dataset.assertions().len(), 3);
        }
        unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
    }
}

#[test]
#[serial_test::serial(gencc_env)]
fn state_only_304_and_failure_crashes_select_one_whole_visible_record() {
    let points = [
        "before-state-file-fsync",
        "after-state-file-fsync",
        "before-state-rename",
        "after-state-rename",
        "before-root-directory-fsync",
        "after-root-directory-fsync",
    ];
    for kind in ["304", "failure"] {
        for point in points {
            let temp = tempfile::tempdir().unwrap();
            secure_anchor(temp.path());
            let root = temp.path().join("gencc");
            unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
            let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
            let old = publish(
                &Store::open().unwrap(),
                &dataset,
                "2026-01-01T00:00:00Z",
                "\"old\"",
            )
            .state
            .active_generation
            .unwrap();
            unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
            let marker = temp.path().join("crashed");
            let status = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "sources::gencc::tests::gencc_subprocess_client",
                ])
                .env("BIOMCP_GENCC_DIR", &root)
                .env("BIOMCP_GENCC_CHILD_CRASH_STATE", kind)
                .env("BIOMCP_GENCC_TEST_CRASH_AT", point)
                .env("BIOMCP_GENCC_TEST_CRASH_MARKER", &marker)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            assert!(!status.success(), "{kind} {point}");
            unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
            let state = Store::open().unwrap().load_state().unwrap();
            assert_eq!(state.active_generation.as_deref(), Some(old.as_str()));
            let renamed = matches!(
                point,
                "after-state-rename" | "before-root-directory-fsync" | "after-root-directory-fsync"
            );
            assert_eq!(
                state.last_attempt,
                Some(if renamed {
                    if kind == "304" {
                        crate::sources::gencc::store::Attempt::Success304
                    } else {
                        crate::sources::gencc::store::Attempt::Failure
                    }
                } else {
                    crate::sources::gencc::store::Attempt::Success200
                }),
                "{kind} {point}"
            );
            assert_eq!(
                state.attempted_at.as_deref(),
                Some(if renamed {
                    "2026-03-01T00:00:00Z"
                } else {
                    "2026-01-01T00:00:00Z"
                })
            );
            unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
        }
    }
}

#[test]
#[serial_test::serial(gencc_env)]
fn cleanup_faults_retain_unowned_or_unfinished_entries_for_a_later_pass() {
    let temp = tempfile::tempdir().unwrap();
    secure_anchor(temp.path());
    let root = temp.path().join("gencc");
    unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
    let store = Store::open().unwrap();
    for (point, relative, directory) in [
        ("before-abandoned-root-delete", ".raw-left.tmp", false),
        (
            "before-abandoned-generation-delete",
            "generations/.tmp-left",
            true,
        ),
    ] {
        let path = root.join(relative);
        if directory {
            fs::create_dir(&path).unwrap();
        } else {
            fs::write(&path, b"left").unwrap();
        }
        unsafe { std::env::set_var("BIOMCP_GENCC_TEST_FAIL_AT", point) };
        store.cleanup_abandoned();
        unsafe { std::env::remove_var("BIOMCP_GENCC_TEST_FAIL_AT") };
        assert!(path.exists(), "{point}");
        store.cleanup_abandoned();
        assert!(!path.exists(), "{point}");
    }
    unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
}

#[test]
#[serial_test::serial(gencc_env)]
fn bootstrap_parent_fsync_failures_fail_closed_before_any_store_is_returned() {
    for point in [
        "before-bootstrap-directory-parent-fsync",
        "after-bootstrap-directory-parent-fsync",
    ] {
        let temp = tempfile::tempdir().unwrap();
        secure_anchor(temp.path());
        let root = temp.path().join("gencc");
        unsafe {
            std::env::set_var("BIOMCP_GENCC_DIR", &root);
            std::env::set_var("BIOMCP_GENCC_TEST_FAIL_AT", point);
        }
        assert!(Store::open().is_err(), "{point}");
        unsafe { std::env::remove_var("BIOMCP_GENCC_TEST_FAIL_AT") };
        drop(Store::open().unwrap());
        assert!(root.join(".refresh.lock").is_file());
        assert!(root.join(".store.lock").is_file());
        unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
    }
}

#[test]
#[serial_test::serial(gencc_env)]
fn subprocess_lease_defers_old_generation_cleanup_until_reader_exits() {
    let temp = tempfile::tempdir().unwrap();
    secure_anchor(temp.path());
    let root = temp.path().join("gencc");
    let entered = temp.path().join("entered");
    let release = temp.path().join("release");
    unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
    let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
    let store = Store::open().unwrap();
    drop(publish(&store, &dataset, "2026-01-01T00:00:00Z", "\"g1\""));
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "sources::gencc::tests::gencc_subprocess_client",
        ])
        .env("BIOMCP_GENCC_DIR", &root)
        .env("BIOMCP_GENCC_CHILD_HOLD_LEASE", &entered)
        .env("BIOMCP_GENCC_CHILD_RELEASE", &release)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !entered.exists() {
        assert!(child.try_wait().unwrap().is_none());
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    drop(publish(&store, &dataset, "2026-01-02T00:00:00Z", "\"g2\""));
    drop(publish(&store, &dataset, "2026-01-03T00:00:00Z", "\"g3\""));
    assert_eq!(fs::read_dir(root.join("generations")).unwrap().count(), 3);
    fs::write(&release, b"release").unwrap();
    assert!(child.wait().unwrap().success());
    drop(publish(&store, &dataset, "2026-01-04T00:00:00Z", "\"g4\""));
    assert_eq!(fs::read_dir(root.join("generations")).unwrap().count(), 2);
    unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
}

#[test]
#[serial_test::serial(gencc_env)]
fn injected_state_rename_failures_report_the_visible_namespace() {
    let temp = tempfile::tempdir().unwrap();
    secure_anchor(temp.path());
    let root = temp.path().join("gencc");
    unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
    let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
    let store = Store::open().unwrap();
    let old = publish(&store, &dataset, "2026-01-01T00:00:00Z", "\"old\"")
        .state
        .active_generation
        .unwrap();
    for (point, renamed) in [("before-state-rename", false), ("after-state-rename", true)] {
        unsafe { std::env::set_var("BIOMCP_GENCC_TEST_FAIL_AT", point) };
        let result = store.publish(
            &dataset,
            PublishMetadata {
                now: "2026-02-01T00:00:00Z",
                etag: "\"new\"",
                last_modified: "Sun, 06 Sep 2026 06:00:29 GMT",
                endpoint: ENDPOINT,
                body_sha256: &format!("{:x}", Sha256::digest(fixture())),
                row_count: dataset.row_count(),
            },
        );
        unsafe { std::env::remove_var("BIOMCP_GENCC_TEST_FAIL_AT") };
        assert_eq!(matches!(result, Err(StoreError::PostRenameSync)), renamed);
        assert_eq!(
            store
                .load()
                .unwrap()
                .unwrap()
                .state
                .active_generation
                .as_deref()
                != Some(&old),
            renamed
        );
    }
    unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
}
