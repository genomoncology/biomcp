//! Tier 3 — response and archive parsing. Pure: feeds committed/synthetic bytes to
//! parsers. No network, no server.

use super::super::{
    MAX_ARCHIVE_ENTRY_BYTES, PmcOaArchivePackage, decode_archive_bytes, decode_text,
    extract_archive_entries, extract_first_nxml, parse_archive_manifest_xml, safe_archive_name,
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
fn parses_manifest_returns_none_without_tgz_link() {
    assert_eq!(
        parse_archive_manifest_xml("<records><record /></records>").unwrap(),
        None
    );
}

#[test]
fn malformed_or_unexpected_manifest_is_failure_not_absence() {
    assert!(parse_archive_manifest_xml("<records>").is_err());
    assert!(parse_archive_manifest_xml("<html><body>error</body></html>").is_err());
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
fn extract_archive_entries_rejects_unsafe_empty_and_oversized_members() {
    assert_eq!(
        safe_archive_name(Path::new("safe\\readme.txt")).as_deref(),
        Some("safe/readme.txt")
    );
    assert!(safe_archive_name(Path::new("../secret.csv")).is_none());
    assert!(safe_archive_name(Path::new("..\\secret.csv")).is_none());
    assert!(safe_archive_name(Path::new("/absolute.csv")).is_none());
    assert!(safe_archive_name(Path::new("C:\\absolute.csv")).is_none());

    let oversized = vec![b'x'; MAX_ARCHIVE_ENTRY_BYTES as usize + 1];
    let tgz = tgz_with_entries(&[
        ("article.nxml", &b"<article/>"[..]),
        ("safe/readme.txt", b"ok"),
        ("empty.bin", b""),
        ("huge.bin", oversized.as_slice()),
    ]);

    let entries = extract_archive_entries(&tgz).expect("archive should parse");
    let names = entries
        .iter()
        .map(|entry| entry.filename.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"article.nxml"));
    assert!(names.contains(&"safe/readme.txt"));
    assert!(!names.contains(&"empty.bin"));
    assert!(!names.contains(&"huge.bin"));
}

#[test]
fn decode_text_maps_http_error_status_with_excerpt() {
    let err = decode_text(StatusCode::INTERNAL_SERVER_ERROR, b"upstream failure").unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("pmc-oa"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

#[test]
fn decode_archive_bytes_preserves_success_bytes_and_maps_errors() {
    assert_eq!(
        decode_archive_bytes(StatusCode::OK, b"abc").unwrap(),
        b"abc".to_vec()
    );

    let err = decode_archive_bytes(StatusCode::BAD_GATEWAY, b"upstream failure").unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("502"), "got: {msg}");
}
