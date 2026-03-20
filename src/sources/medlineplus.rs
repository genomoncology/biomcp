use std::borrow::Cow;
use std::sync::OnceLock;

use regex::Regex;
use reqwest::header::HeaderValue;
use roxmltree::Document;

use crate::error::BioMcpError;

const MEDLINEPLUS_BASE: &str = "https://wsearch.nlm.nih.gov";
const MEDLINEPLUS_API: &str = "medlineplus";
const MEDLINEPLUS_BASE_ENV: &str = "BIOMCP_MEDLINEPLUS_BASE";

pub struct MedlinePlusClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl MedlinePlusClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MEDLINEPLUS_BASE, MEDLINEPLUS_BASE_ENV),
        })
    }

    #[cfg(test)]
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub async fn search(&self, query: &str) -> Result<Vec<MedlinePlusTopic>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let resp = crate::sources::apply_cache_mode(self.client.get(self.endpoint("ws/query")))
            .query(&[("db", "healthTopics"), ("term", query), ("retmax", "3")])
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, MEDLINEPLUS_API).await?;

        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: MEDLINEPLUS_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(&bytes)),
            });
        }

        reject_html_content_type(content_type.as_ref(), &bytes)?;

        let xml = String::from_utf8(bytes).map_err(|_| BioMcpError::Api {
            api: MEDLINEPLUS_API.to_string(),
            message: "Response body was not valid UTF-8 XML".to_string(),
        })?;

        tokio::task::spawn_blocking(move || parse_topics(&xml))
            .await
            .map_err(|err| BioMcpError::Api {
                api: MEDLINEPLUS_API.to_string(),
                message: format!("XML parse task failed: {err}"),
            })?
    }
}

#[derive(Debug, Clone)]
pub struct MedlinePlusTopic {
    pub title: String,
    pub url: String,
    pub summary_excerpt: String,
}

fn reject_html_content_type(
    content_type: Option<&HeaderValue>,
    body: &[u8],
) -> Result<(), BioMcpError> {
    let Some(content_type) = content_type else {
        return Ok(());
    };
    let Ok(raw) = content_type.to_str() else {
        return Ok(());
    };
    let media_type = raw
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if matches!(media_type.as_str(), "text/html" | "application/xhtml+xml") {
        return Err(BioMcpError::Api {
            api: MEDLINEPLUS_API.to_string(),
            message: format!(
                "Unexpected HTML response (content-type: {raw}): {}",
                crate::sources::body_excerpt(body)
            ),
        });
    }
    Ok(())
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

fn strip_inline_html_tags(value: &str) -> String {
    static HTML_TAG_RE: OnceLock<Regex> = OnceLock::new();
    let re = HTML_TAG_RE.get_or_init(|| Regex::new(r"(?is)<[^>]+>").expect("valid regex"));
    re.replace_all(value, "").to_string()
}

fn clean_text(value: &str) -> String {
    strip_inline_html_tags(&decode_html_entities(value))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_topics(xml: &str) -> Result<Vec<MedlinePlusTopic>, BioMcpError> {
    let doc = Document::parse(xml).map_err(|source| BioMcpError::Api {
        api: MEDLINEPLUS_API.to_string(),
        message: format!("Invalid XML response: {source}"),
    })?;

    let mut out = Vec::new();
    for document in doc
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("document"))
    {
        let url = document.attribute("url").unwrap_or_default().trim();
        if url.is_empty() {
            continue;
        }

        let mut title = String::new();
        let mut summary = String::new();
        for content in document
            .children()
            .filter(|child| child.is_element() && child.has_tag_name("content"))
        {
            let name = content.attribute("name").unwrap_or_default();
            let text = clean_text(content.text().unwrap_or_default());
            if text.is_empty() {
                continue;
            }
            match name {
                "title" if title.is_empty() => title = text,
                "FullSummary" if summary.is_empty() => summary = text,
                "snippet" if summary.is_empty() => summary = text,
                _ => {}
            }
        }

        if title.is_empty() {
            continue;
        }

        out.push(MedlinePlusTopic {
            title,
            url: url.to_string(),
            summary_excerpt: summary,
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{MedlinePlusClient, parse_topics};

    #[test]
    fn parse_topics_decodes_inline_markup() {
        let topics = parse_topics(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<nlmSearchResult>
  <list num="1" start="0" per="1">
    <document rank="0" url="https://medlineplus.gov/chestpain.html">
      <content name="title">&lt;span class="qt0"&gt;Chest&lt;/span&gt; Pain</content>
      <content name="FullSummary">&lt;p&gt;Chest pain summary.&lt;/p&gt;</content>
    </document>
  </list>
</nlmSearchResult>"#,
        )
        .expect("topics");

        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].title, "Chest Pain");
        assert_eq!(topics[0].summary_excerpt, "Chest pain summary.");
    }

    #[tokio::test]
    async fn search_uses_expected_query_contract() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ws/query"))
            .and(query_param("db", "healthTopics"))
            .and(query_param("term", "chest pain"))
            .and(query_param("retmax", "3"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "application/xml")
                    .set_body_string(
                        r#"<?xml version="1.0" encoding="UTF-8"?>
<nlmSearchResult>
  <list num="1" start="0" per="1">
    <document rank="0" url="https://medlineplus.gov/chestpain.html">
      <content name="title">Chest Pain</content>
      <content name="FullSummary">Summary</content>
    </document>
  </list>
</nlmSearchResult>"#,
                    ),
            )
            .mount(&server)
            .await;

        let client = MedlinePlusClient::new_for_test(server.uri()).expect("client");
        let topics = client.search("chest pain").await.expect("search");
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].title, "Chest Pain");
    }
}
