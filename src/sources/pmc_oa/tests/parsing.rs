//! Tier 3 — response and archive parsing. Pure: feeds committed/synthetic bytes to
//! parsers. No network, no server.

use super::super::{
    MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_ENTRY_BYTES, MAX_ARCHIVE_METADATA_BYTES, MAX_TGZ_BYTES,
    PmcOaArchivePackage, decode_archive_bytes, decode_text, extract_archive_entries,
    extract_first_nxml, parse_archive_manifest_xml, safe_archive_name,
};
use crate::error::BioMcpError;
use flate2::Compression;
use flate2::write::GzEncoder;
use reqwest::StatusCode;
use std::io::Write;
use std::path::Path;
use tar::{Builder, Header};

fn tgz_with_entries(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut tar_buf = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_buf);
        for (name, body) in entries {
            let mut header = Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, *name, *body)
                .expect("archive entry should append");
        }
        builder.finish().expect("tar should finish");
    }

    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(&tar_buf).expect("gzip should write tar");
    gz.finish().expect("gzip should finish")
}

fn tgz_with_numbered_entries(count: usize) -> Vec<u8> {
    let mut tar_buf = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_buf);
        for index in 0..count {
            let mut header = Header::new_gnu();
            header.set_size(1);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, format!("entry-{index}.txt"), &b"x"[..])
                .expect("archive entry should append");
        }
        builder.finish().expect("tar should finish");
    }

    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(&tar_buf).expect("gzip should write tar");
    gz.finish().expect("gzip should finish")
}

fn tgz_with_repeated_entries(count: usize, size: usize) -> Vec<u8> {
    let body = vec![b'x'; size];
    let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
    {
        let mut builder = Builder::new(&mut gz);
        for index in 0..count {
            let mut header = Header::new_gnu();
            header.set_size(size as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, format!("entry-{index}.bin"), body.as_slice())
                .expect("archive entry should append");
        }
        builder.finish().expect("tar should finish");
    }
    gz.finish().expect("gzip should finish")
}

fn tgz_with_long_name(name_size: usize) -> Vec<u8> {
    let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
    {
        let mut builder = Builder::new(&mut gz);
        let name = format!("{}.txt", "a".repeat(name_size));
        let mut header = Header::new_gnu();
        header.set_size(1);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, name, &b"x"[..])
            .expect("archive entry should append");
        builder.finish().expect("tar should finish");
    }
    gz.finish().expect("gzip should finish")
}

#[test]
fn parses_manifest_and_rewrites_ftp_to_https() {
    let manifest = parse_archive_manifest_xml(
        r#"<records><record license="CC BY" retracted="no"><link format="tgz" href="ftp://ftp.ncbi.nlm.nih.gov/pub/pmc/file.tar.gz"/></record></records>"#,
    )
    .unwrap()
    .expect("manifest");

    assert_eq!(
        manifest.tgz_url,
        "https://ftp.ncbi.nlm.nih.gov/pub/pmc/file.tar.gz"
    );
    assert_eq!(manifest.package_url, manifest.tgz_url);
    assert_eq!(manifest.license.as_deref(), Some("CC BY"));
    assert_eq!(manifest.retracted, Some(false));
}

#[test]
fn parses_manifest_attributes_independent_of_order_and_quote_style() {
    let manifest = parse_archive_manifest_xml(
        "<records><record retracted='yes' license='CC0'><link href='https://example.test/archive.tgz' format='tgz'/></record></records>",
    )
    .unwrap()
    .expect("manifest");

    assert_eq!(manifest.tgz_url, "https://example.test/archive.tgz");
    assert_eq!(manifest.license.as_deref(), Some("CC0"));
    assert_eq!(manifest.retracted, Some(true));
}

#[test]
fn parses_manifest_returns_none_without_tgz_link() {
    assert_eq!(
        parse_archive_manifest_xml("<records><record /></records>").unwrap(),
        None
    );
}

#[test]
fn documented_not_open_access_response_is_healthy_absence() {
    let xml = r#"<OA><responseDate>2026-07-14 16:01:49</responseDate><request>https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi?id=PMC145899</request><error code="idIsNotOpenAccess">identifier 'PMC145899' is not Open Access</error></OA>"#;
    assert_eq!(parse_archive_manifest_xml(xml).unwrap(), None);
}

#[test]
fn malformed_or_unexpected_manifest_is_failure_not_absence() {
    assert!(parse_archive_manifest_xml("<records>").is_err());
    assert!(parse_archive_manifest_xml("<html><body>error</body></html>").is_err());
    assert!(parse_archive_manifest_xml("<OA><error code=\"unknown\">error</error></OA>").is_err());
    assert!(
        parse_archive_manifest_xml("<html><error code=\"idIsNotOpenAccess\">error</error></html>")
            .is_err()
    );
}

#[test]
fn extract_first_nxml_reads_xml_entry() {
    let tgz = tgz_with_entries(&[("sample.nxml", b"<article><body>ok</body></article>")]);

    let xml = extract_first_nxml(&tgz).unwrap().unwrap();
    assert!(xml.contains("<article>"));
}

#[test]
fn archive_package_enumerates_non_xml_and_preserves_binary_bytes() {
    let image_bytes = b"\x89PNG\r\n\x1a\n\0\xfffixture";
    let tgz = tgz_with_entries(&[
        ("article.nxml", b"<article><body>ok</body></article>"),
        ("figures/panel.png", image_bytes),
        ("supplement/traces.csv", b"time,value\n0,1\n"),
    ]);
    let manifest = parse_archive_manifest_xml(
        r#"<records><record license="CC BY" retracted="no"><link format="tgz" href="https://example.test/archive.tgz"/></record></records>"#,
    )
    .unwrap()
    .expect("manifest");
    let package = PmcOaArchivePackage {
        manifest,
        entries: extract_archive_entries(&tgz).expect("archive should parse"),
    };

    assert_eq!(package.manifest.license.as_deref(), Some("CC BY"));
    assert_eq!(package.manifest.retracted, Some(false));
    let image = package
        .entries
        .iter()
        .find(|entry| entry.filename == "figures/panel.png")
        .expect("image entry should be listed");
    assert!(!image.is_xml);
    assert_eq!(image.bytes, image_bytes);
    assert!(
        package
            .entries
            .iter()
            .any(|entry| entry.filename == "article.nxml" && entry.is_xml)
    );
}

#[test]
fn extract_archive_entries_skips_unsafe_and_empty_members_but_rejects_oversized_members() {
    assert_eq!(
        safe_archive_name(Path::new("safe\\readme.txt")).as_deref(),
        Some("safe/readme.txt")
    );
    assert!(safe_archive_name(Path::new("../secret.csv")).is_none());
    assert!(safe_archive_name(Path::new("..\\secret.csv")).is_none());
    assert!(safe_archive_name(Path::new("/absolute.csv")).is_none());
    assert!(safe_archive_name(Path::new("C:\\absolute.csv")).is_none());

    let tgz = tgz_with_entries(&[
        ("article.nxml", &b"<article/>"[..]),
        ("safe/readme.txt", b"ok"),
        ("empty.bin", b""),
    ]);

    let entries = extract_archive_entries(&tgz).expect("in-bound archive should parse");
    let names = entries
        .iter()
        .map(|entry| entry.filename.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"article.nxml"));
    assert!(names.contains(&"safe/readme.txt"));
    assert!(!names.contains(&"empty.bin"));

    let oversized = vec![b'x'; MAX_ARCHIVE_ENTRY_BYTES as usize + 1];
    let oversized_tgz = tgz_with_entries(&[("huge.bin", oversized.as_slice())]);
    let err = extract_archive_entries(&oversized_tgz)
        .expect_err("archive member resource cap should reject the package");
    assert_eq!(err.code(), "source_unavailable");
    assert!(format!("{err:?}").contains("resource limit"));
}

#[test]
fn extract_archive_entries_accepts_exact_member_count_limit() {
    let tgz = tgz_with_numbered_entries(MAX_ARCHIVE_ENTRIES as usize);

    let entries = extract_archive_entries(&tgz).expect("exact member count should pass");

    assert_eq!(entries.len(), MAX_ARCHIVE_ENTRIES as usize);
}

#[test]
fn extract_archive_entries_rejects_too_many_members() {
    let tgz = tgz_with_numbered_entries(MAX_ARCHIVE_ENTRIES as usize + 1);

    let err = extract_archive_entries(&tgz).expect_err("archive-wide member cap should reject");

    assert!(
        matches!(err, BioMcpError::SourceUnavailable { .. }),
        "archive resource limits should be source-unavailable, got {err:?}"
    );
    assert!(format!("{err:?}").contains("resource limit"));
}

#[test]
fn extract_archive_entries_rejects_aggregate_expansion() {
    let aggregate = tgz_with_repeated_entries(
        (super::super::MAX_ARCHIVE_EXPANDED_BYTES / MAX_ARCHIVE_ENTRY_BYTES + 1) as usize,
        MAX_ARCHIVE_ENTRY_BYTES as usize,
    );
    let err = extract_archive_entries(&aggregate).expect_err("aggregate cap should reject");
    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
}

#[test]
fn extract_archive_entries_rejects_single_metadata_record_over_limit() {
    let metadata = tgz_with_long_name(MAX_ARCHIVE_METADATA_BYTES as usize + 1);
    let err = extract_archive_entries(&metadata).expect_err("metadata cap should reject");
    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
}

#[test]
fn direct_buffered_archive_limit_is_sanitized() {
    let oversized = vec![0; MAX_TGZ_BYTES + 1];
    let err = extract_archive_entries(&oversized).expect_err("compressed cap should reject");
    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
    assert!(!err.to_string().contains(&MAX_TGZ_BYTES.to_string()));
}

#[test]
fn decode_text_maps_http_error_status_with_excerpt() {
    let err = decode_text(StatusCode::INTERNAL_SERVER_ERROR, b"upstream failure").unwrap_err();
    let msg = format!("{err:?}");
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("pmc-oa"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

#[test]
fn text_and_fulltext_archive_decoders_reject_invalid_utf8() {
    let manifest_err = decode_text(StatusCode::OK, b"<records>\xff</records>")
        .expect_err("invalid manifest bytes must remain a source failure");
    assert!(matches!(manifest_err, BioMcpError::Api { .. }));

    let archive = tgz_with_entries(&[("article.nxml", b"<article>\xff</article>")]);
    let article_err = extract_first_nxml(&archive)
        .expect_err("invalid article XML bytes must remain a source failure");
    assert!(matches!(article_err, BioMcpError::Api { .. }));
}

#[test]
fn decode_archive_bytes_preserves_success_bytes_and_maps_errors() {
    assert_eq!(
        decode_archive_bytes(StatusCode::OK, b"abc").unwrap(),
        b"abc".to_vec()
    );

    let err = decode_archive_bytes(StatusCode::BAD_GATEWAY, b"upstream failure").unwrap_err();
    let msg = format!("{err:?}");
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("502"), "got: {msg}");
}
