use super::*;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::cell::Cell;
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use tar::{Builder, Header};

fn build_targz(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut tar_buf = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_buf);
        for (path, contents) in entries {
            let mut header = Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder
                .append_data(&mut header, *path, *contents)
                .expect("test archive entry should append");
        }
        builder.finish().expect("test archive should finish");
    }

    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(&tar_buf)
        .expect("test archive should gzip successfully");
    gz.finish().expect("test archive should finalize")
}

fn serve_body(body: Vec<u8>, chunked: bool) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 1024];
        let _ = std::io::Read::read(&mut stream, &mut request);
        if chunked {
            write!(stream, "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n", body.len()).unwrap();
            stream.write_all(&body).unwrap();
            stream.write_all(b"\r\n0\r\n\r\n").unwrap();
        } else {
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(&body).unwrap();
        }
    });
    (format!("http://{address}/asset"), handle)
}

#[tokio::test]
async fn archive_transport_accepts_exact_limit_for_declared_and_chunked_bodies() {
    for chunked in [false, true] {
        let (url, server) = serve_body(vec![b'x'; 4096], chunked);
        let bytes = download_asset_with_limit(&url, 4096).await.unwrap();
        assert_eq!(bytes.len(), 4096);
        server.join().unwrap();
    }
}

#[tokio::test]
async fn archive_transport_rejects_limit_plus_one_for_declared_and_chunked_bodies() {
    for chunked in [false, true] {
        let (url, server) = serve_body(vec![b'x'; 4097], chunked);
        let error = download_asset_with_limit(&url, 4096).await.unwrap_err();
        assert!(matches!(
            error,
            BioMcpError::BodyLimit {
                max_bytes: 4096,
                ..
            } | BioMcpError::HttpMiddleware(_)
        ));
        server.join().unwrap();
    }
}

#[tokio::test]
async fn archive_larger_than_shared_default_reaches_verification_and_extraction() {
    let mut binary = vec![0_u8; crate::sources::DEFAULT_MAX_BODY_BYTES + 1];
    let mut state = 1_u32;
    for byte in &mut binary {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *byte = (state >> 24) as u8;
    }
    let archive = build_targz(&[("release/bin/biomcp", &binary)]);
    assert!(archive.len() > crate::sources::DEFAULT_MAX_BODY_BYTES);
    for chunked in [false, true] {
        let (url, server) = serve_body(archive.clone(), chunked);
        let downloaded = download_archive(&url).await.unwrap();
        verify_archive_against_checksum(&sha256_hex(&archive), &downloaded).unwrap();
        assert_eq!(
            extract_binary_from_targz(&downloaded, "biomcp").unwrap(),
            binary
        );
        server.join().unwrap();
    }
}

#[tokio::test]
async fn release_metadata_endpoint_is_injectable() {
    let body = br#"{"tag_name":"v9.9.9","assets":[]}"#.to_vec();
    let (url, server) = serve_body(body, false);
    let release = fetch_latest_release_from(&url).await.unwrap();
    assert_eq!(release.tag_name, "v9.9.9");
    server.join().unwrap();
}

#[test]
fn production_archive_ceiling_is_exactly_256_mib() {
    assert_eq!(MAX_RELEASE_ARCHIVE_BYTES, 256 * 1024 * 1024);
}

#[cfg(unix)]
fn seed_owned_script(root: &Path, version: &str) -> PathBuf {
    use crate::cli::install::{
        INSTALLER_IDENTITY, InstallReceipt, RECEIPT_SCHEMA_VERSION, ReceiptState, receipt_path,
        sha256_file, write_receipt_atomic,
    };
    use std::os::unix::fs::PermissionsExt;
    let executable = root.join("biomcp");
    std::fs::write(&executable, format!("#!/bin/sh\necho 'biomcp {version}'\n")).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    let executable = std::fs::canonicalize(executable).unwrap();
    write_receipt_atomic(
        &receipt_path(&executable).unwrap(),
        &InstallReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            installer: INSTALLER_IDENTITY.into(),
            state: ReceiptState::Installed,
            executable_path: executable.clone(),
            version: version.into(),
            sha256: sha256_file(&executable).unwrap(),
            transaction_nonce: None,
            old_version: None,
            old_sha256: None,
            new_version: None,
            new_sha256: None,
        },
    )
    .unwrap();
    executable
}

#[cfg(unix)]
#[test]
fn owned_update_smokes_and_atomically_replaces_with_agreeing_receipt() {
    use crate::test_support::TempDirGuard;
    let root = TempDirGuard::new("update-owned");
    let executable = seed_owned_script(root.path(), "1.0.0");
    let predictable = root.path().join(".biomcp.new");
    std::os::unix::fs::symlink(root.path().join("do-not-touch"), &predictable).unwrap();
    let new_bytes = b"#!/bin/sh\necho 'biomcp 2.0.0'\n";
    replace_owned_binary_at(&executable, new_bytes, "v2.0.0").unwrap();
    assert_eq!(std::fs::read(&executable).unwrap(), new_bytes);
    let owned = crate::cli::install::validate_owned(&executable).unwrap();
    assert_eq!(owned.receipt.version, "v2.0.0");
    assert_eq!(owned.receipt.sha256, sha256_hex(new_bytes));
    assert!(predictable.is_symlink());
}

#[cfg(unix)]
#[test]
fn failed_staged_version_smoke_preserves_binary_and_receipt() {
    use crate::test_support::TempDirGuard;
    let root = TempDirGuard::new("update-smoke-fail");
    let executable = seed_owned_script(root.path(), "1.0.0");
    let before_binary = std::fs::read(&executable).unwrap();
    let receipt = crate::cli::install::receipt_path(&executable).unwrap();
    let before_receipt = std::fs::read(&receipt).unwrap();
    let error = replace_owned_binary_at(&executable, b"#!/bin/sh\necho 'biomcp wrong'\n", "2.0.0")
        .unwrap_err();
    assert!(error.to_string().contains("requested version"));
    assert_eq!(std::fs::read(&executable).unwrap(), before_binary);
    assert_eq!(std::fs::read(&receipt).unwrap(), before_receipt);
    assert!(!std::fs::read_dir(root.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".biomcp-stage-")
    }));
}

#[test]
fn extract_binary_from_targz_returns_matching_binary_bytes() {
    let expected = b"#!/bin/sh\necho biomcp\n";
    let archive = build_targz(&[
        ("release/README.txt", b"notes"),
        ("release/bin/biomcp", expected.as_slice()),
    ]);

    let extracted = extract_binary_from_targz(&archive, "biomcp").expect("binary should extract");

    assert_eq!(extracted, expected);
}

#[test]
fn extract_binary_from_targz_rejects_empty_binary() {
    let archive = build_targz(&[("release/bin/biomcp", b"")]);

    let err = extract_binary_from_targz(&archive, "biomcp")
        .expect_err("empty binary entry should be rejected");

    assert!(matches!(
        err,
        BioMcpError::Api { api, message }
            if api == "update" && message == "Downloaded archive contained an empty binary"
    ));
}

#[test]
fn extract_binary_from_targz_reports_missing_binary_as_not_found() {
    let archive = build_targz(&[("release/bin/other-binary", b"echo other\n")]);

    let err = extract_binary_from_targz(&archive, "biomcp")
        .expect_err("missing binary should be reported as not found");

    assert!(matches!(
        err,
        BioMcpError::NotFound {
            entity,
            id,
            suggestion,
        } if entity == "release asset"
            && id == "biomcp"
            && suggestion == "Release archive did not contain expected biomcp binary"
    ));
}

// ---- 331 fail-closed checksum policy assertions ----

#[test]
fn enforce_checksum_policy_missing_sidecar_without_override_fails_closed() {
    let err = enforce_checksum_policy(ChecksumStatus::MissingSidecar, "biomcp-linux-x86_64.tar.gz")
        .expect_err("missing sidecar without override must fail closed");
    let message = match err {
        BioMcpError::Api { message, .. } => message,
        other => panic!("expected BioMcpError::Api, got {other:?}"),
    };
    assert!(
        message.to_lowercase().contains("checksum"),
        "error message must name the checksum: {message}"
    );
    assert!(message.contains("verified standalone installer"));
    assert!(
        message.contains("biomcp-linux-x86_64.tar.gz"),
        "error message must name the asset: {message}"
    );
}

#[test]
fn enforce_checksum_policy_verified_succeeds() {
    enforce_checksum_policy(ChecksumStatus::Verified, "biomcp-linux-x86_64.tar.gz")
        .expect("verified archive must succeed");
}

#[test]
fn install_binary_after_checksum_policy_missing_sidecar_without_override_does_not_replace() {
    let replace_called = Cell::new(false);

    let err = install_binary_after_checksum_policy(
        ChecksumStatus::MissingSidecar,
        "biomcp-linux-x86_64.tar.gz",
        b"new-binary",
        |bytes| {
            replace_called.set(true);
            assert_eq!(bytes, b"new-binary");
            Ok(())
        },
    )
    .expect_err("missing sidecar without override must fail before replacement");

    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(
        !replace_called.get(),
        "missing checksum sidecar must not reach binary replacement"
    );
}

#[test]
fn verify_archive_against_checksum_accepts_matching_sha256() {
    let payload = b"archive bytes payload";
    let expected = sha256_hex(payload);
    verify_archive_against_checksum(&expected, payload).expect("matching sha256 must verify");
}

#[test]
fn verify_archive_against_checksum_rejects_mismatch() {
    let payload = b"archive bytes payload";
    let wrong = "0".repeat(64);
    let err = verify_archive_against_checksum(&wrong, payload)
        .expect_err("mismatched sha256 must fail closed");
    let message = match err {
        BioMcpError::Api { message, .. } => message,
        other => panic!("expected BioMcpError::Api, got {other:?}"),
    };
    assert!(
        message.to_lowercase().contains("mismatch"),
        "error message must name mismatch: {message}"
    );
}

#[test]
fn verify_archive_against_checksum_rejects_invalid_format() {
    let err = verify_archive_against_checksum("not-a-hex-token", b"payload")
        .expect_err("malformed sidecar must fail closed");
    let message = match err {
        BioMcpError::Api { message, .. } => message,
        other => panic!("expected BioMcpError::Api, got {other:?}"),
    };
    assert!(
        message.contains("Invalid checksum file format"),
        "error message must call out invalid format: {message}"
    );
}
