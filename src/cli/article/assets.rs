use crate::cli::CommandOutcome;

pub(super) async fn handle_asset_get(
    id: &str,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<Option<CommandOutcome>> {
    if let Some(asset_name) = article_asset_request(sections)? {
        let bytes = crate::entities::article::article_asset_bytes(id, &asset_name).await?;
        return Ok(Some(CommandOutcome::stdout_bytes(bytes)));
    }
    if !article_assets_request(sections)? {
        return Ok(None);
    }
    if !json_output {
        anyhow::bail!(crate::error::BioMcpError::InvalidArgument(
            "Article asset manifests are JSON-only; rerun with --json (example: biomcp --json get article 22663011 assets)"
                .into(),
        ));
    }

    let manifest = crate::entities::article::article_assets_manifest(id).await?;
    let commands = manifest_next_commands(&manifest);
    #[derive(serde::Serialize)]
    struct AssetsResponse {
        #[serde(flatten)]
        manifest: crate::entities::article::ArticleAssetsManifest,
        #[serde(skip_serializing_if = "Option::is_none")]
        _meta: Option<super::super::SearchJsonMeta>,
    }
    Ok(Some(CommandOutcome::stdout(
        crate::render::json::to_pretty(&AssetsResponse {
            manifest,
            _meta: crate::cli::search_meta(commands),
        })?,
    )))
}

pub(super) fn article_asset_route(sections: &[String]) -> bool {
    sections.iter().any(|section| {
        let normalized = section.trim().to_ascii_lowercase();
        normalized == "asset" || normalized == "assets"
    })
}

fn article_assets_request(sections: &[String]) -> Result<bool, crate::error::BioMcpError> {
    let has_assets = sections
        .iter()
        .any(|section| section.trim().eq_ignore_ascii_case("assets"));
    if !has_assets {
        return Ok(false);
    }
    if sections.len() != 1 {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "assets is a standalone JSON-only article section; do not combine it with other sections"
                .into(),
        ));
    }
    Ok(true)
}

fn article_asset_request(sections: &[String]) -> Result<Option<String>, crate::error::BioMcpError> {
    let Some((index, _)) = sections
        .iter()
        .enumerate()
        .find(|(_, section)| section.trim().eq_ignore_ascii_case("asset"))
    else {
        return Ok(None);
    };
    if sections
        .iter()
        .any(|section| section.trim().eq_ignore_ascii_case("assets"))
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "asset <asset-key> is a standalone raw-byte retrieval form; do not combine it with assets"
                .into(),
        ));
    }
    if index + 2 != sections.len() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "asset requires exactly one asset key (example: biomcp get article 22663011 asset traces-s1.csv)"
                .into(),
        ));
    }
    sections
        .get(index + 1)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| Ok(Some(value.to_string())))
        .unwrap_or_else(|| {
            Err(crate::error::BioMcpError::InvalidArgument(
                "asset requires exactly one asset key (example: biomcp get article 22663011 asset traces-s1.csv)"
                    .into(),
            ))
        })
}

fn manifest_next_commands(
    manifest: &crate::entities::article::ArticleAssetsManifest,
) -> Vec<String> {
    let mut commands = vec![
        crate::next_command::NextCommand::biomcp()
            .args(["--json", "get", "article", &manifest.article_id, "assets"])
            .render_shell(),
    ];
    commands.extend(manifest.assets.iter().map(|asset| asset.handle.clone()));
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sections(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn assets_is_standalone_json_only_route() {
        assert!(article_asset_route(&sections(&["assets"])));
        assert!(article_assets_request(&sections(&["assets"])).unwrap());

        let err = article_assets_request(&sections(&["assets", "fulltext"])).unwrap_err();
        assert!(err.to_string().contains("standalone JSON-only"));
    }

    #[test]
    fn asset_requires_exactly_one_key_and_no_assets_section() {
        assert_eq!(
            article_asset_request(&sections(&["asset", "supplement.pdf"]))
                .unwrap()
                .as_deref(),
            Some("supplement.pdf")
        );

        for bad in [
            sections(&["asset"]),
            sections(&["asset", "supplement.pdf", "fulltext"]),
            sections(&["asset", "supplement.pdf", "assets"]),
            sections(&["asset", " "]),
        ] {
            let err = article_asset_request(&bad).unwrap_err();
            assert!(err.to_string().contains("asset"));
        }
    }
}
