//! Local transport contracts for bounded archive download behavior.

use super::super::*;
use crate::error::BioMcpError;
use crate::test_support::TempDirGuard;
use std::borrow::Cow;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn archive_download_rejects_declared_oversize_before_creating_destination() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let address = listener.local_addr().expect("test listener address");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept request");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await.expect("read request");
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 2147483649\r\nConnection: close\r\n\r\n",
            )
            .await
            .expect("write declared oversized response");
    });

    let root = TempDirGuard::new("study-download-declared-oversize");
    let destination = root.path().join("archive.tar.gz");
    let client = CBioPortalDownloadClient {
        client: datahub_client(Duration::from_secs(1), Some(Duration::from_secs(2)))
            .expect("test client"),
        base: Cow::Owned(format!("http://{address}")),
        download_idle_timeout: Duration::from_secs(1),
    };

    let err = client
        .download_study_archive_to_path("demo_study", &destination)
        .await
        .expect_err("declared oversized response should be rejected");

    assert!(
        matches!(err, BioMcpError::SourceUnavailable { .. }),
        "download resource limits should be source-unavailable, got {err:?}"
    );
    assert!(err.to_string().contains("resource limit"));
    assert!(!destination.exists());
    server.await.expect("test server should finish");
}
