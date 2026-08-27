use std::fmt::Write as _;
use std::path::Path;

use crate::entities::article::Article;
use crate::error::BioMcpError;

pub(super) fn validate_request(
    enabled: bool,
    sections: &[String],
    outline: bool,
    lines: Option<&str>,
) -> Result<(), BioMcpError> {
    if enabled
        && (outline
            || lines.is_some()
            || !((sections.len() == 1 && sections[0].eq_ignore_ascii_case("fulltext"))
                || (sections.len() == 2
                    && sections[0].eq_ignore_ascii_case("asset")
                    && !sections[1].trim().is_empty())))
    {
        return Err(BioMcpError::InvalidArgument(
            "--out requires fulltext or asset <asset-key> as the sole article retrieval form and does not support --outline or --lines"
                .into(),
        ));
    }
    Ok(())
}

pub(super) async fn export_fulltext(
    article: &Article,
    directory: Option<&Path>,
) -> Result<(), BioMcpError> {
    let Some(directory) = directory else {
        return Ok(());
    };
    let pmid = article.pmid.as_deref().ok_or_else(|| {
        BioMcpError::InvalidArgument(
            "--out requires the resolved article to have a canonical PMID".into(),
        )
    })?;
    let path = article
        .full_text_path
        .as_deref()
        .ok_or_else(|| BioMcpError::NotFound {
            entity: "article fulltext".into(),
            id: pmid.into(),
            suggestion: "Full text was not available from the configured providers.".into(),
        })?;
    let source = article.full_text_source.as_ref().ok_or_else(|| {
        BioMcpError::InvalidArgument("Resolved full text has no source rung".into())
    })?;
    let body = super::fulltext_view::read_managed_text(path)?;
    let filename = format!("{}-{}.md", pmid, title_slug(&article.title));
    let retrieved_at = chrono::Utc::now().to_rfc3339();
    let mut document = String::from("---\n");
    for (key, value) in [
        ("pmid", Some(pmid)),
        ("pmcid", article.pmcid.as_deref()),
        ("doi", article.doi.as_deref()),
        ("title", Some(article.title.as_str())),
        ("journal", article.journal.as_deref()),
        ("date", article.date.as_deref()),
        ("retrieved-at", Some(retrieved_at.as_str())),
        ("source-rung", Some(source.label.as_str())),
    ] {
        let value = yaml_scalar(value)?;
        writeln!(document, "{key}: {value}").expect("writing to a string cannot fail");
    }
    document.push_str("---\n");
    document.push_str(&body);
    crate::utils::download::write_user_atomic_bytes(&directory.join(filename), document.as_bytes())
        .await
}

fn yaml_scalar(value: Option<&str>) -> Result<String, BioMcpError> {
    value
        .map(serde_json::to_string)
        .transpose()
        .map(|value| value.unwrap_or_else(|| "null".into()))
        .map_err(Into::into)
}

fn title_slug(title: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    for character in title.chars() {
        if character.is_alphanumeric() {
            if separator && !slug.is_empty() {
                slug.push('-');
            }
            slug.extend(character.to_lowercase());
            separator = false;
        } else if !slug.is_empty() {
            separator = true;
        }
    }
    if slug.is_empty() {
        "article".into()
    } else {
        slug
    }
}
