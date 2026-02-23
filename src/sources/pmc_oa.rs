use std::borrow::Cow;
use std::sync::OnceLock;

use regex::Regex;

use crate::error::BioMcpError;

// PubMed Central Open Access (OA) service
// Docs: https://www.ncbi.nlm.nih.gov/pmc/tools/oa/
const PMC_OA_BASE: &str = "https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi";
const PMC_OA_API: &str = "pmc-oa";
const PMC_OA_BASE_ENV: &str = "BIOMCP_PMC_OA_BASE";
const MAX_TGZ_BYTES: usize = 64 * 1024 * 1024;
const MAX_ARCHIVE_ENTRY_BYTES: u64 = 8 * 1024 * 1024;

static TGZ_HREF_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Clone)]
pub struct PmcOaClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl PmcOaClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(PMC_OA_BASE, PMC_OA_BASE_ENV),
            api_key: crate::sources::ncbi_api_key(),
        })
    }

    #[cfg(test)]
    fn new_for_test(base: String, api_key: Option<String>) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
            api_key: api_key
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
        })
    }

    fn endpoint(&self) -> String {
        self.base.as_ref().trim_end_matches('/').to_string()
    }

    async fn get_text(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<String, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, self.api_key.is_some())
            .send()
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, PMC_OA_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: PMC_OA_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    async fn oa_tgz_url(&self, pmcid: &str) -> Result<Option<String>, BioMcpError> {
        let pmcid = pmcid.trim();
        if pmcid.is_empty() {
            return Ok(None);
        }
        if pmcid.len() > 64 {
            return Err(BioMcpError::InvalidArgument("PMCID is too long.".into()));
        }

        let url = self.endpoint();
        let req = self.client.get(&url).query(&[("id", pmcid)]);
        let req = crate::sources::append_ncbi_api_key(req, self.api_key.as_deref());
        let xml = self.get_text(req).await?;

        let re = TGZ_HREF_RE.get_or_init(|| {
            Regex::new(r#"<link[^>]*format="tgz"[^>]*href="([^"]+)""#)
                .expect("valid tgz href regex")
        });

        let Some(caps) = re.captures(&xml) else {
            return Ok(None);
        };
        let Some(raw_href) = caps
            .get(1)
            .map(|m| m.as_str().trim())
            .filter(|s| !s.is_empty())
        else {
            return Ok(None);
        };

        let href = if raw_href.starts_with("ftp://ftp.ncbi.nlm.nih.gov/") {
            raw_href.replacen(
                "ftp://ftp.ncbi.nlm.nih.gov/",
                "https://ftp.ncbi.nlm.nih.gov/",
                1,
            )
        } else if raw_href.starts_with("ftp://") {
            raw_href.replacen("ftp://", "https://", 1)
        } else {
            raw_href.to_string()
        };

        Ok(Some(href))
    }

    pub async fn get_full_text_xml(&self, pmcid: &str) -> Result<Option<String>, BioMcpError> {
        let Some(tgz_url) = self.oa_tgz_url(pmcid).await? else {
            return Ok(None);
        };

        let resp = crate::sources::apply_cache_mode(self.client.get(&tgz_url))
            .send()
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, PMC_OA_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: PMC_OA_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        let bytes = bytes.to_vec();
        let xml = tokio::task::spawn_blocking(move || extract_first_nxml(&bytes))
            .await
            .map_err(|err| BioMcpError::Api {
                api: PMC_OA_API.to_string(),
                message: format!("Task join error: {err}"),
            })??;

        Ok(xml)
    }
}

fn extract_first_nxml(tgz_bytes: &[u8]) -> Result<Option<String>, BioMcpError> {
    use std::io::Read;

    if tgz_bytes.len() > MAX_TGZ_BYTES {
        return Err(BioMcpError::Api {
            api: PMC_OA_API.to_string(),
            message: format!("PMC OA archive exceeded {MAX_TGZ_BYTES} bytes"),
        });
    }

    let gz = flate2::read::GzDecoder::new(tgz_bytes);
    let mut archive = tar::Archive::new(gz);
    let entries = archive.entries()?;

    for entry in entries {
        let entry = entry?;
        if entry.size() > MAX_ARCHIVE_ENTRY_BYTES {
            continue;
        }
        let path = entry.path()?;
        let Some(file_name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !(file_name.ends_with(".nxml") || file_name.ends_with(".xml")) {
            continue;
        }

        let mut out: Vec<u8> = Vec::new();
        let mut reader = entry.take(MAX_ARCHIVE_ENTRY_BYTES + 1);
        reader.read_to_end(&mut out)?;
        if out.len() as u64 > MAX_ARCHIVE_ENTRY_BYTES {
            continue;
        }
        if out.is_empty() {
            continue;
        }
        return Ok(Some(String::from_utf8_lossy(&out).to_string()));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;
    use tar::{Builder, Header};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn oa_tgz_url_rewrites_ftp_to_https() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .and(query_param("id", "PMC123"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"<records><record><link format="tgz" href="ftp://ftp.ncbi.nlm.nih.gov/pub/pmc/file.tar.gz"/></record></records>"#,
            ))
            .mount(&server)
            .await;

        let client = PmcOaClient::new_for_test(server.uri(), None).unwrap();
        let href = client.oa_tgz_url("PMC123").await.unwrap().unwrap();
        assert_eq!(href, "https://ftp.ncbi.nlm.nih.gov/pub/pmc/file.tar.gz");
    }

    #[tokio::test]
    async fn oa_tgz_url_includes_api_key_when_configured() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .and(query_param("id", "PMC123"))
            .and(query_param("api_key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"<records><record><link format="tgz" href="ftp://ftp.ncbi.nlm.nih.gov/pub/pmc/file.tar.gz"/></record></records>"#,
            ))
            .expect(1)
            .mount(&server)
            .await;

        let client = PmcOaClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let href = client.oa_tgz_url("PMC123").await.unwrap().unwrap();
        assert_eq!(href, "https://ftp.ncbi.nlm.nih.gov/pub/pmc/file.tar.gz");
    }

    #[test]
    fn extract_first_nxml_reads_xml_entry() {
        let mut tar_buf = Vec::new();
        {
            let mut builder = Builder::new(&mut tar_buf);
            let contents = b"<article><body>ok</body></article>";
            let mut header = Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "sample.nxml", &contents[..])
                .unwrap();
            builder.finish().unwrap();
        }

        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&tar_buf).unwrap();
        let tgz = gz.finish().unwrap();

        let xml = extract_first_nxml(&tgz).unwrap().unwrap();
        assert!(xml.contains("<article>"));
    }
}
