//! Tier 3 — response parsing and local archive handling. Pure: feeds committed
//! fixture bytes or local tarballs into the decoder/installer. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use crate::test_support::TempDirGuard;
use flate2::Compression;
use flate2::write::GzEncoder;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tar::{Builder, Header};

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/cbioportal_download/",
            $name
        ))
    };
}

struct TempRoot {
    _guard: TempDirGuard,
    path: PathBuf,
}

impl TempRoot {
    fn new(name: &str) -> Self {
        let guard = TempDirGuard::new(&format!("study-download-{name}"));
        let path = guard.path().to_path_buf();
        Self {
            _guard: guard,
            path,
        }
    }
}

fn tar_gz_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut tar_buf = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_buf);
        for (path, contents) in entries {
            let mut header = Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, *path, *contents)
                .expect("append archive entry");
        }
        builder.finish().expect("finish archive");
    }

    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(&tar_buf).expect("write gz");
    gz.finish().expect("finish gz")
}

fn demo_study_archive() -> Vec<u8> {
    tar_gz_bytes(&[
        (
            "demo_study/meta_study.txt",
            b"cancer_study_identifier: demo_study\nname: Demo Study\n",
        ),
        (
            "demo_study/data_mutations.txt",
            b"Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\n",
        ),
    ])
}

fn append_declared_header(tar: &mut Vec<u8>, path: &str, size: u64, entry_type: tar::EntryType) {
    let mut header = Header::new_gnu();
    header.set_path(path).expect("set archive path");
    header.set_size(size);
    header.set_entry_type(entry_type);
    header.set_mode(0o644);
    header.set_cksum();
    tar.extend_from_slice(header.as_bytes());
}

fn gzip_incomplete_tar(mut tar: Vec<u8>) -> Vec<u8> {
    tar.extend_from_slice(&[0; 1024]);
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(&tar).expect("write gz");
    gz.finish().expect("finish gz")
}

fn tar_gz_with_declared_oversized_member() -> Vec<u8> {
    let metadata = b"cancer_study_identifier: demo_study\nname: Demo Study\n";
    let mut tar = Vec::new();
    append_declared_header(
        &mut tar,
        "demo_study/meta_study.txt",
        metadata.len() as u64,
        tar::EntryType::Regular,
    );
    tar.extend_from_slice(metadata);
    tar.resize(tar.len().next_multiple_of(512), 0);
    append_declared_header(
        &mut tar,
        "demo_study/oversized_payload.bin",
        1024 * 1024 * 1024 + 1,
        tar::EntryType::Regular,
    );
    gzip_incomplete_tar(tar)
}

fn tar_gz_with_declared_oversized_long_name() -> Vec<u8> {
    let mut tar = Vec::new();
    append_declared_header(
        &mut tar,
        "././@LongLink",
        1024 * 1024 * 1024 + 1,
        tar::EntryType::GNULongName,
    );
    gzip_incomplete_tar(tar)
}

#[test]
fn parses_study_list_fixture() {
    let content_type = HeaderValue::from_static("application/json");
    let study_ids = CBioPortalDownloadClient::decode_study_list_response(
        StatusCode::OK,
        Some(&content_type),
        fixture!("study_list.json"),
    )
    .unwrap();

    assert_eq!(
        study_ids,
        vec![
            "msk_impact_2017".to_string(),
            "brca_tcga_pan_can_atlas_2018".to_string()
        ]
    );
}

#[test]
fn study_list_decode_maps_http_and_content_type_errors() {
    let err = CBioPortalDownloadClient::decode_study_list_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failure"), "got: {msg}");

    let text_plain = HeaderValue::from_static("text/plain");
    assert!(
        CBioPortalDownloadClient::decode_study_list_response(
            StatusCode::OK,
            Some(&text_plain),
            b"not json",
        )
        .is_err()
    );

    let html = HeaderValue::from_static("text/html");
    let err = CBioPortalDownloadClient::decode_study_list_response(
        StatusCode::OK,
        Some(&html),
        b"<html><body>not json</body></html>",
    )
    .unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
}

#[test]
fn archive_status_maps_missing_archives_to_not_found_without_leaking_storage_body() {
    for status in [StatusCode::FORBIDDEN, StatusCode::NOT_FOUND] {
        let err = CBioPortalDownloadClient::decode_archive_status(
            "missing_study",
            status,
            b"<Error><Code>NoSuchKey</Code></Error>",
        )
        .unwrap_err();

        assert!(
            matches!(
                err,
                BioMcpError::NotFound {
                    ref entity,
                    ref id,
                    ..
                } if entity == "Study" && id == "missing_study"
            ),
            "expected NotFound, got {err:?}"
        );
        let msg = err.to_string();
        assert!(msg.contains("Study 'missing_study' not found."));
        assert!(msg.contains("biomcp study download --list"));
        assert!(!msg.contains("NoSuchKey"));
    }
}

#[test]
fn archive_status_maps_other_http_errors_with_excerpt() {
    let err = CBioPortalDownloadClient::decode_archive_status(
        "demo_study",
        StatusCode::INTERNAL_SERVER_ERROR,
        b"upstream failure",
    )
    .unwrap_err();
    let msg = err.to_string();

    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failure"), "got: {msg}");

    assert!(
        CBioPortalDownloadClient::decode_archive_status("demo_study", StatusCode::OK, b"").is_ok()
    );
}

#[test]
fn install_study_archive_extracts_a_valid_local_archive() {
    let root = TempRoot::new("install");
    let archive_path = root.path.join("demo_study.tar.gz");
    fs::write(&archive_path, demo_study_archive()).expect("write archive");

    let result = install_study_archive(&root.path, "demo_study", &archive_path).unwrap();

    assert!(result.downloaded);
    assert_eq!(result.study_id, "demo_study");
    assert_eq!(result.path, root.path.join("demo_study"));
    assert!(result.path.join("meta_study.txt").is_file());
    let studies =
        crate::sources::cbioportal_study::list_studies(&root.path).expect("local study list");
    assert_eq!(studies.len(), 1);
    assert_eq!(studies[0].study_id, "demo_study");
}

#[test]
fn install_study_archive_skips_existing_valid_target() {
    let root = TempRoot::new("existing");
    let study_dir = root.path.join("demo_study");
    fs::create_dir_all(&study_dir).expect("create study dir");
    fs::write(
        study_dir.join("meta_study.txt"),
        "cancer_study_identifier: demo_study\nname: Demo Study\n",
    )
    .expect("write meta");
    let archive_path = root.path.join("unused.tar.gz");
    fs::write(&archive_path, demo_study_archive()).expect("write archive");

    let result = install_study_archive(&root.path, "demo_study", &archive_path).unwrap();

    assert!(!result.downloaded);
    assert_eq!(result.path, study_dir);
}

#[test]
fn install_study_archive_rejects_declared_oversized_member_without_leaving_files() {
    let root = TempRoot::new("oversized-member");
    let archive_path = root.path.join("oversized-study.tar.gz");
    fs::write(&archive_path, tar_gz_with_declared_oversized_member()).expect("write archive");

    let err = install_study_archive(&root.path, "demo_study", &archive_path).unwrap_err();

    assert!(
        matches!(err, BioMcpError::SourceUnavailable { .. }),
        "oversized archive member should be sanitized as source-unavailable, got {err:?}"
    );
    assert!(err.to_string().contains("resource limit"));
    assert!(!err.to_string().contains("oversized_payload"));
    assert!(!root.path.join("demo_study").exists());
    let remaining = fs::read_dir(&root.path)
        .expect("read temp root")
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .expect("collect temp root entries");
    assert_eq!(remaining, vec![archive_path]);
}

#[test]
fn install_study_archive_rejects_oversized_path_metadata_before_buffering_it() {
    let root = TempRoot::new("oversized-metadata");
    let archive_path = root.path.join("oversized-metadata.tar.gz");
    fs::write(&archive_path, tar_gz_with_declared_oversized_long_name()).expect("write archive");

    let err = install_study_archive(&root.path, "demo_study", &archive_path).unwrap_err();

    assert!(
        matches!(err, BioMcpError::SourceUnavailable { .. }),
        "archive metadata limits should be source-unavailable, got {err:?}"
    );
    assert!(err.to_string().contains("resource limit"));
    assert!(!root.path.join("demo_study").exists());
}

#[test]
fn install_study_archive_rejects_entries_outside_expected_top_level_directory() {
    let root = TempRoot::new("traversal");
    let archive_path = root.path.join("bad_study.tar.gz");
    let archive = tar_gz_bytes(&[
        (
            "demo_study/meta_study.txt",
            b"cancer_study_identifier: demo_study\nname: Demo Study\n",
        ),
        ("other_study/evil.txt", b"bad"),
    ]);
    fs::write(&archive_path, archive).expect("write archive");

    let err = install_study_archive(&root.path, "demo_study", &archive_path).unwrap_err();

    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(!root.path.join("demo_study").exists());
    assert!(!root.path.join("evil.txt").exists());
    let remaining = fs::read_dir(&root.path)
        .expect("read temp root")
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .expect("collect temp root entries");
    assert_eq!(remaining, vec![archive_path]);
}

#[test]
fn list_decode_accepts_json_content_type_with_parameters() {
    let content_type = HeaderValue::from_static("application/json; charset=utf-8");

    let study_ids = CBioPortalDownloadClient::decode_study_list_response(
        StatusCode::OK,
        Some(&content_type),
        fixture!("study_list.json"),
    )
    .unwrap();

    assert_eq!(study_ids.len(), 2);
    assert!(study_ids.iter().any(|id| id == "msk_impact_2017"));
}
