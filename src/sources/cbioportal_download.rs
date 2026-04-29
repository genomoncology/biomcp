use std::borrow::Cow;
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;
use tokio::io::AsyncWriteExt;

use crate::error::BioMcpError;

const DATAHUB_BASE: &str = "https://datahub.assets.cbioportal.org";
const DATAHUB_API: &str = "cbioportal-datahub";
const DATAHUB_BASE_ENV: &str = "BIOMCP_CBIOPORTAL_DATAHUB_BASE";
const DATAHUB_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DATAHUB_ARCHIVE_IDLE_TIMEOUT: Duration = Duration::from_secs(120);
#[cfg(test)]
const DATAHUB_ARCHIVE_IDLE_TIMEOUT_FOR_TEST: Duration = Duration::from_millis(100);

#[derive(Debug, Clone)]
pub struct StudyInstallResult {
    pub study_id: String,
    pub path: PathBuf,
    pub downloaded: bool,
}

pub struct CBioPortalDownloadClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    download_idle_timeout: Duration,
}

impl CBioPortalDownloadClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: datahub_client(DATAHUB_CONNECT_TIMEOUT, None)?,
            base: crate::sources::env_base(DATAHUB_BASE, DATAHUB_BASE_ENV),
            download_idle_timeout: DATAHUB_ARCHIVE_IDLE_TIMEOUT,
        })
    }

    #[cfg(test)]
    pub fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: datahub_client(DATAHUB_CONNECT_TIMEOUT, None)?,
            base: Cow::Owned(base),
            download_idle_timeout: DATAHUB_ARCHIVE_IDLE_TIMEOUT_FOR_TEST,
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let body = crate::sources::read_limited_body(resp, DATAHUB_API).await?;
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: DATAHUB_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(&body)),
            });
        }
        crate::sources::ensure_json_content_type(DATAHUB_API, content_type.as_ref(), &body)?;
        serde_json::from_slice(&body).map_err(|source| BioMcpError::ApiJson {
            api: DATAHUB_API.to_string(),
            source,
        })
    }

    async fn download_study_archive_to_path(
        &self,
        study_id: &str,
        dest: &Path,
    ) -> Result<(), BioMcpError> {
        let mut resp = crate::sources::apply_cache_mode(
            self.client
                .get(self.endpoint(&format!("{study_id}.tar.gz"))),
        )
        .send()
        .await?;
        let status = resp.status();
        if matches!(
            status,
            reqwest::StatusCode::FORBIDDEN | reqwest::StatusCode::NOT_FOUND
        ) {
            let _ = crate::sources::read_limited_body(resp, DATAHUB_API).await?;
            return Err(BioMcpError::NotFound {
                entity: "Study".to_string(),
                id: study_id.to_string(),
                suggestion: "Run `biomcp study download --list` to see available study IDs."
                    .to_string(),
            });
        }
        if !status.is_success() {
            let body = crate::sources::read_limited_body(resp, DATAHUB_API).await?;
            return Err(BioMcpError::Api {
                api: DATAHUB_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(&body)),
            });
        }
        let mut file = tokio::fs::File::create(dest).await?;
        loop {
            match tokio::time::timeout(self.download_idle_timeout, resp.chunk()).await {
                Ok(Ok(Some(chunk))) => file.write_all(&chunk).await?,
                Ok(Ok(None)) => break,
                Ok(Err(err)) => return Err(err.into()),
                Err(_) => {
                    return Err(BioMcpError::SourceUnavailable {
                        source_name: "cBioPortal DataHub".to_string(),
                        reason: format!(
                            "Archive download stalled because no bytes or progress arrived within {:?}.",
                            self.download_idle_timeout
                        ),
                        suggestion: "Retry the study download later.".to_string(),
                    });
                }
            }
        }
        file.flush().await?;
        Ok(())
    }

    pub async fn list_study_ids(&self) -> Result<Vec<String>, BioMcpError> {
        self.get_json(self.client.get(self.endpoint("study_list.json")))
            .await
    }

    pub async fn download_study(
        &self,
        study_id: &str,
        root: &Path,
    ) -> Result<StudyInstallResult, BioMcpError> {
        let study_id = validate_study_id(study_id)?;

        fs::create_dir_all(root)?;
        let target = root.join(study_id);
        if target.exists() {
            if is_valid_installed_study(root, study_id, &target)? {
                return Ok(StudyInstallResult {
                    study_id: study_id.to_string(),
                    path: target,
                    downloaded: false,
                });
            }
            return Err(BioMcpError::SourceUnavailable {
                source_name: DATAHUB_API.to_string(),
                reason: format!(
                    "Target directory already exists but is not a valid study: {}",
                    target.display()
                ),
                suggestion: "Remove the incomplete study directory and retry.".to_string(),
            });
        }

        let archive_path = unique_temp_path(root, &format!(".{study_id}.download"))?;
        let download_result = self
            .download_study_archive_to_path(study_id, &archive_path)
            .await;
        if let Err(err) = download_result {
            let _ = fs::remove_file(&archive_path);
            return Err(err);
        }

        let root = root.to_path_buf();
        let study_id = study_id.to_string();
        let archive_path_for_install = archive_path.clone();
        let install_result = tokio::task::spawn_blocking(move || {
            install_study_archive(&root, &study_id, &archive_path_for_install)
        })
        .await;
        let _ = fs::remove_file(&archive_path);
        install_result.map_err(|err| BioMcpError::Api {
            api: DATAHUB_API.to_string(),
            message: format!("Study install worker failed: {err}"),
        })?
    }
}

fn datahub_client(
    connect_timeout: Duration,
    total_timeout: Option<Duration>,
) -> Result<reqwest_middleware::ClientWithMiddleware, BioMcpError> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(connect_timeout)
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")));
    if let Some(timeout) = total_timeout {
        builder = builder.timeout(timeout);
    }
    let client = builder.build().map_err(BioMcpError::HttpClientInit)?;
    Ok(reqwest_middleware::ClientBuilder::new(client).build())
}

fn unique_temp_path(parent: &Path, prefix: &str) -> Result<PathBuf, BioMcpError> {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    for attempt in 0..32_u32 {
        let candidate = parent.join(format!(
            "{prefix}-{}-{}-{}",
            std::process::id(),
            seed,
            attempt
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(BioMcpError::Io(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "Unable to allocate temporary study-install path",
    )))
}

fn archive_relative_path(study_id: &str, path: &Path) -> Result<Option<PathBuf>, BioMcpError> {
    let mut components = path.components();
    match components.next() {
        Some(Component::Normal(component)) if component == study_id => {}
        _ => {
            return Err(BioMcpError::Api {
                api: DATAHUB_API.to_string(),
                message: format!(
                    "Archive entry is outside the expected top-level study directory: {}",
                    path.display()
                ),
            });
        }
    }

    let mut relative = PathBuf::new();
    for component in components {
        match component {
            Component::Normal(segment) => relative.push(segment),
            Component::CurDir => {}
            _ => {
                return Err(BioMcpError::Api {
                    api: DATAHUB_API.to_string(),
                    message: format!("Unsafe archive entry path: {}", path.display()),
                });
            }
        }
    }

    if relative.as_os_str().is_empty() {
        Ok(None)
    } else {
        Ok(Some(relative))
    }
}

fn validate_study_id(study_id: &str) -> Result<&str, BioMcpError> {
    let study_id = study_id.trim();
    if study_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Study ID is required.".to_string(),
        ));
    }
    let mut components = Path::new(study_id).components();
    let is_single_segment = matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    );
    if !is_single_segment
        || study_id.contains('\\')
        || study_id
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid study ID '{study_id}'. Expected a single identifier such as 'msk_impact_2017'."
        )));
    }
    Ok(study_id)
}

fn extract_archive_into(
    root: &Path,
    study_id: &str,
    archive_path: &Path,
) -> Result<PathBuf, BioMcpError> {
    let staging_root = unique_temp_path(root, &format!(".{study_id}.extract"))?;
    fs::create_dir_all(&staging_root)?;
    let staging_dir = staging_root.join(study_id);
    fs::create_dir_all(&staging_dir)?;

    let extract_result = (|| -> Result<(), BioMcpError> {
        let file = File::open(archive_path)?;
        let gz = GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);
        for entry in archive.entries()? {
            let mut entry = entry?;
            let entry_path = entry.path()?.into_owned();
            let Some(relative) = archive_relative_path(study_id, &entry_path)? else {
                continue;
            };
            let dest = staging_dir.join(&relative);
            if !dest.starts_with(&staging_dir) {
                return Err(BioMcpError::Api {
                    api: DATAHUB_API.to_string(),
                    message: format!(
                        "Archive entry escaped staging directory: {}",
                        entry_path.display()
                    ),
                });
            }

            match entry.header().entry_type() {
                tar::EntryType::Directory => {
                    fs::create_dir_all(&dest)?;
                }
                tar::EntryType::Regular => {
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let mut out = File::create(&dest)?;
                    std::io::copy(&mut entry, &mut out)?;
                }
                _ => {
                    return Err(BioMcpError::Api {
                        api: DATAHUB_API.to_string(),
                        message: format!(
                            "Unsupported archive entry type for {}",
                            entry_path.display()
                        ),
                    });
                }
            }
        }

        if !staging_dir.join("meta_study.txt").is_file() {
            return Err(BioMcpError::SourceUnavailable {
                source_name: DATAHUB_API.to_string(),
                reason: format!(
                    "Downloaded study archive for '{study_id}' is missing meta_study.txt"
                ),
                suggestion: "Retry the download or choose a different study.".to_string(),
            });
        }

        Ok(())
    })();

    match extract_result {
        Ok(()) => Ok(staging_root),
        Err(err) => {
            let _ = fs::remove_dir_all(&staging_root);
            Err(err)
        }
    }
}

fn is_valid_installed_study(
    root: &Path,
    study_id: &str,
    target: &Path,
) -> Result<bool, BioMcpError> {
    if !target.is_dir() || !target.join("meta_study.txt").is_file() {
        return Ok(false);
    }

    let studies = crate::sources::cbioportal_study::list_studies(root)?;
    Ok(studies
        .into_iter()
        .any(|study| study.study_id.eq_ignore_ascii_case(study_id) && study.path == target))
}

fn install_study_archive(
    root: &Path,
    study_id: &str,
    archive_path: &Path,
) -> Result<StudyInstallResult, BioMcpError> {
    fs::create_dir_all(root)?;
    let target = root.join(study_id);
    if target.exists() {
        if is_valid_installed_study(root, study_id, &target)? {
            return Ok(StudyInstallResult {
                study_id: study_id.to_string(),
                path: target,
                downloaded: false,
            });
        }
        return Err(BioMcpError::SourceUnavailable {
            source_name: DATAHUB_API.to_string(),
            reason: format!(
                "Target directory already exists but is not a valid study: {}",
                target.display()
            ),
            suggestion: "Remove the incomplete study directory and retry.".to_string(),
        });
    }

    let staging_root = extract_archive_into(root, study_id, archive_path)?;
    let staging_dir = staging_root.join(study_id);
    match fs::rename(&staging_dir, &target) {
        Ok(()) => {}
        Err(err) => {
            let _ = fs::remove_dir_all(&staging_root);
            return Err(err.into());
        }
    }
    let _ = fs::remove_dir_all(&staging_root);

    if !is_valid_installed_study(root, study_id, &target)? {
        let _ = fs::remove_dir_all(&target);
        return Err(BioMcpError::SourceUnavailable {
            source_name: DATAHUB_API.to_string(),
            reason: format!(
                "Installed study '{study_id}' could not be validated by the local study loader"
            ),
            suggestion: "Retry the download or inspect the extracted study files.".to_string(),
        });
    }

    Ok(StudyInstallResult {
        study_id: study_id.to_string(),
        path: target,
        downloaded: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_support::TempDirGuard;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;
    use std::time::Duration;
    use tar::{Builder, Header};
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;
    use tokio::sync::oneshot;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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

    struct StalledArchiveServer {
        uri: String,
        release: Option<oneshot::Sender<()>>,
        handle: tokio::task::JoinHandle<()>,
    }

    impl StalledArchiveServer {
        async fn shutdown(mut self) {
            if let Some(release) = self.release.take() {
                let _ = release.send(());
            }
            let _ = self.handle.await;
        }
    }

    async fn stalled_archive_server(study_id: &'static str) -> StalledArchiveServer {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind stalled archive server");
        let uri = format!("http://{}", listener.local_addr().expect("local addr"));
        let (release_tx, release_rx) = oneshot::channel::<()>();
        let expected_request = format!("GET /{study_id}.tar.gz ");

        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept stalled request");
            let mut request = Vec::new();
            let mut buf = [0_u8; 1024];
            loop {
                let read = stream.read(&mut buf).await.expect("read stalled request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buf[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&request);
            assert!(
                request.starts_with(&expected_request),
                "unexpected request: {request}"
            );

            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/gzip\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nabcde\r\n",
                )
                .await
                .expect("write stalled response prefix");
            stream.flush().await.expect("flush stalled response prefix");

            let _ = release_rx.await;
        });

        StalledArchiveServer {
            uri,
            release: Some(release_tx),
            handle,
        }
    }

    #[tokio::test]
    async fn list_study_ids_fetches_remote_catalog() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/study_list.json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"["msk_impact_2017","brca_tcga_pan_can_atlas_2018"]"#),
            )
            .mount(&server)
            .await;

        let client = CBioPortalDownloadClient::new_for_test(server.uri()).expect("client");
        let study_ids = client.list_study_ids().await.expect("study list");
        assert_eq!(
            study_ids,
            vec![
                "msk_impact_2017".to_string(),
                "brca_tcga_pan_can_atlas_2018".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn datahub_client_omits_total_timeout_for_slow_catalog_responses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/study_list.json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(100))
                    .set_body_string(r#"["demo_study"]"#),
            )
            .expect(2)
            .mount(&server)
            .await;

        let timed_client = CBioPortalDownloadClient {
            client: datahub_client(DATAHUB_CONNECT_TIMEOUT, Some(Duration::from_millis(50)))
                .expect("timed client"),
            base: Cow::Owned(server.uri()),
            download_idle_timeout: DATAHUB_ARCHIVE_IDLE_TIMEOUT_FOR_TEST,
        };
        let err = timed_client
            .list_study_ids()
            .await
            .expect_err("timed client should fail");
        let timed_out = match &err {
            BioMcpError::Http(source) => source.is_timeout(),
            BioMcpError::HttpMiddleware(source) => source.is_timeout(),
            _ => false,
        };
        assert!(timed_out, "expected timeout error, got {err:?}");

        let untimed_client = CBioPortalDownloadClient {
            client: datahub_client(DATAHUB_CONNECT_TIMEOUT, None).expect("untimed client"),
            base: Cow::Owned(server.uri()),
            download_idle_timeout: DATAHUB_ARCHIVE_IDLE_TIMEOUT_FOR_TEST,
        };
        let study_ids = untimed_client
            .list_study_ids()
            .await
            .expect("untimed client should succeed");
        assert_eq!(study_ids, vec!["demo_study".to_string()]);
    }

    #[tokio::test]
    async fn download_study_installs_archive_into_root() {
        let server = MockServer::start().await;
        let archive = tar_gz_bytes(&[
            (
                "demo_study/meta_study.txt",
                b"cancer_study_identifier: demo_study\nname: Demo Study\n",
            ),
            (
                "demo_study/data_mutations.txt",
                b"Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\n",
            ),
        ]);
        Mock::given(method("GET"))
            .and(path("/demo_study.tar.gz"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(archive))
            .mount(&server)
            .await;

        let root = TempRoot::new("install");
        let client = CBioPortalDownloadClient::new_for_test(server.uri()).expect("client");
        let result = client
            .download_study("demo_study", &root.path)
            .await
            .expect("download result");

        assert!(result.downloaded);
        assert_eq!(result.path, root.path.join("demo_study"));
        assert!(result.path.join("meta_study.txt").is_file());
        let studies =
            crate::sources::cbioportal_study::list_studies(&root.path).expect("local study list");
        assert_eq!(studies.len(), 1);
        assert_eq!(studies[0].study_id, "demo_study");
    }

    #[tokio::test]
    async fn download_study_stalled_archive_body_times_out_with_clear_error() {
        let server = stalled_archive_server("stalled_study").await;
        let root = TempRoot::new("stalled-archive");
        let client = CBioPortalDownloadClient::new_for_test(server.uri.clone()).expect("client");

        let result = tokio::time::timeout(
            Duration::from_millis(750),
            client.download_study("stalled_study", &root.path),
        )
        .await;
        server.shutdown().await;

        let err = match result {
            Ok(Ok(result)) => panic!("stalled download unexpectedly succeeded: {result:?}"),
            Ok(Err(err)) => err,
            Err(_) => panic!(
                "stalled download should fail with a readable idle timeout before the test harness timeout"
            ),
        };
        let message = err.to_string();
        assert!(
            message.contains("cBioPortal DataHub"),
            "stall error should name the DataHub source, got: {message}"
        );
        assert!(
            message.contains("stalled"),
            "stall error should describe a stalled archive download, got: {message}"
        );
        assert!(
            message.contains("no bytes") || message.contains("progress"),
            "stall error should explain the no-progress condition, got: {message}"
        );
        assert!(!root.path.join("stalled_study").exists());
        let remaining = fs::read_dir(&root.path)
            .expect("read temp root")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect temp root entries");
        assert!(
            remaining.is_empty(),
            "stalled download should not leave partial archive or install files behind"
        );
    }

    #[tokio::test]
    async fn download_study_returns_not_found_for_missing_remote_archive_403() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing_study.tar.gz"))
            .respond_with(ResponseTemplate::new(403).set_body_string(
                r#"<?xml version="1.0" encoding="UTF-8"?><Error><Code>AccessDenied</Code></Error>"#,
            ))
            .mount(&server)
            .await;

        let root = TempRoot::new("missing-403");
        let client = CBioPortalDownloadClient::new_for_test(server.uri()).expect("client");
        let err = client
            .download_study("missing_study", &root.path)
            .await
            .expect_err("403 should return not found");

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
        let message = err.to_string();
        assert!(message.contains("Study 'missing_study' not found."));
        assert!(message.contains("biomcp study download --list"));
        assert!(!message.contains("AccessDenied"));
        assert!(!message.contains("API error from cbioportal-datahub"));
    }

    #[tokio::test]
    async fn download_study_returns_not_found_for_missing_remote_archive_404() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing_study.tar.gz"))
            .respond_with(ResponseTemplate::new(404).set_body_string(
                r#"<?xml version="1.0" encoding="UTF-8"?><Error><Code>NoSuchKey</Code></Error>"#,
            ))
            .mount(&server)
            .await;

        let root = TempRoot::new("missing-404");
        let client = CBioPortalDownloadClient::new_for_test(server.uri()).expect("client");
        let err = client
            .download_study("missing_study", &root.path)
            .await
            .expect_err("404 should return not found");

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
        let message = err.to_string();
        assert!(message.contains("Study 'missing_study' not found."));
        assert!(message.contains("biomcp study download --list"));
        assert!(!message.contains("NoSuchKey"));
        assert!(!message.contains("API error from cbioportal-datahub"));
    }

    #[tokio::test]
    async fn download_study_skips_existing_valid_target() {
        let root = TempRoot::new("existing");
        let study_dir = root.path.join("demo_study");
        fs::create_dir_all(&study_dir).expect("create study dir");
        fs::write(
            study_dir.join("meta_study.txt"),
            "cancer_study_identifier: demo_study\nname: Demo Study\n",
        )
        .expect("write meta");

        let client =
            CBioPortalDownloadClient::new_for_test("http://127.0.0.1".to_string()).expect("client");
        let result = client
            .download_study("demo_study", &root.path)
            .await
            .expect("existing study");

        assert!(!result.downloaded);
        assert_eq!(result.path, study_dir);
    }

    #[tokio::test]
    async fn download_study_rejects_path_like_study_id() {
        let root = TempRoot::new("invalid-study-id");
        let client =
            CBioPortalDownloadClient::new_for_test("http://127.0.0.1".to_string()).expect("client");
        let err = client
            .download_study("../demo_study", &root.path)
            .await
            .expect_err("path-like study ID should fail");

        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn download_study_rejects_entries_outside_expected_top_level_directory() {
        let server = MockServer::start().await;
        let archive = tar_gz_bytes(&[
            (
                "demo_study/meta_study.txt",
                b"cancer_study_identifier: demo_study\nname: Demo Study\n",
            ),
            ("other_study/evil.txt", b"bad"),
        ]);
        Mock::given(method("GET"))
            .and(path("/demo_study.tar.gz"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(archive))
            .mount(&server)
            .await;

        let root = TempRoot::new("traversal");
        let client = CBioPortalDownloadClient::new_for_test(server.uri()).expect("client");
        let err = client
            .download_study("demo_study", &root.path)
            .await
            .expect_err("unexpected top-level directory should fail");

        assert!(matches!(err, BioMcpError::Api { .. }));
        assert!(!root.path.join("demo_study").exists());
        assert!(!root.path.join("evil.txt").exists());
        let remaining = fs::read_dir(&root.path)
            .expect("read temp root")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect temp root entries");
        assert!(
            remaining.is_empty(),
            "failed install should not leave staging files behind"
        );
    }
}
