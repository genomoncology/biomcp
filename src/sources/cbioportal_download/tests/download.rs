//! Local transport contracts for bounded archive download behavior.

use super::super::*;
use crate::test_support::TempDirGuard;
use std::borrow::Cow;
use std::time::Duration;

#[test]
fn archive_download_accounting_accepts_exact_limit_and_rejects_overflow() {
    assert_eq!(
        account_download_bytes(
            MAX_ARCHIVE_DOWNLOAD_BYTES - 1,
            1,
            MAX_ARCHIVE_DOWNLOAD_BYTES,
        )
        .expect("exact limit"),
        MAX_ARCHIVE_DOWNLOAD_BYTES
    );
    assert!(
        account_download_bytes(MAX_ARCHIVE_DOWNLOAD_BYTES, 1, MAX_ARCHIVE_DOWNLOAD_BYTES).is_err()
    );
    assert!(account_download_bytes(usize::MAX, 1, usize::MAX).is_err());
}

#[tokio::test]
async fn archive_download_rejects_declared_oversize_before_creating_destination() {
    let mut server = crate::test_support::TestHttpServer::respond(|_| async {
        Ok(b"HTTP/1.1 200 OK\r\nContent-Length: 2147483649\r\nConnection: close\r\n\r\n".to_vec())
    })
    .expect("start download fixture");

    let root = TempDirGuard::new("study-download-declared-oversize");
    let destination = root.path().join("archive.tar.gz");
    let base = server.base.clone();
    let client = CBioPortalDownloadClient {
        client: crate::sources::ordinary_url_policy::test_middleware_client_for_base(
            &base,
            |builder| {
                builder
                    .connect_timeout(Duration::from_secs(1))
                    .timeout(Duration::from_secs(2))
            },
        )
        .expect("test client"),
        base: Cow::Owned(base),
        download_idle_timeout: Duration::from_secs(1),
        max_archive_download_bytes: MAX_ARCHIVE_DOWNLOAD_BYTES,
    };

    let err = client
        .download_study_archive_to_path("demo_study", &destination)
        .await
        .expect_err("declared oversized response should be rejected");

    assert_eq!(
        err.code(),
        "source_unavailable",
        "download resource limits should be source-unavailable, got {err:?}"
    );
    assert!(format!("{err:?}").contains("resource limit"));
    assert!(!destination.exists());
    server.finish().expect("test server should finish");
}

#[tokio::test]
async fn archive_download_rejects_chunked_max_plus_one_and_removes_partial_file() {
    let mut server = crate::test_support::TestHttpServer::respond(|_| async {
        Ok(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\nabcd\r\n0\r\n\r\n".to_vec())
    })
    .expect("start download fixture");

    let root = TempDirGuard::new("study-download-chunked-oversize");
    let base = server.base.clone();
    let client = CBioPortalDownloadClient {
        client: crate::sources::ordinary_url_policy::test_middleware_client_for_base(
            &base,
            |builder| {
                builder
                    .connect_timeout(Duration::from_secs(1))
                    .timeout(Duration::from_secs(2))
            },
        )
        .expect("test client"),
        base: Cow::Owned(base),
        download_idle_timeout: Duration::from_secs(1),
        max_archive_download_bytes: 3,
    };

    let err = client
        .download_study("demo_study", root.path())
        .await
        .expect_err("chunked max+1 response should be rejected");

    assert_eq!(err.code(), "source_unavailable");
    assert!(format!("{err:?}").contains("resource limit"));
    assert_eq!(
        std::fs::read_dir(root.path())
            .expect("read download root")
            .count(),
        0,
        "outer download flow should remove the partial archive"
    );
    server.finish().expect("test server should finish");
}
