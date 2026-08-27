use crate::cli::CommandOutcome;

#[derive(Clone, Copy)]
pub(super) enum AssetOutput<'a> {
    Exact(&'a std::path::Path),
    Directory(&'a std::path::Path),
}

impl<'a> AssetOutput<'a> {
    pub(super) fn from_paths(
        output: Option<&'a std::path::Path>,
        out: Option<&'a std::path::Path>,
    ) -> Option<Self> {
        output.map(Self::Exact).or_else(|| out.map(Self::Directory))
    }
}

pub(super) async fn handle_asset_get(
    id: &str,
    sections: &[String],
    json_output: bool,
    destination: Option<AssetOutput<'_>>,
    view: &str,
    explicit_limit: Option<usize>,
    offset: usize,
) -> anyhow::Result<Option<CommandOutcome>> {
    if let Some(asset_name) = article_asset_request(sections)? {
        let (bytes, media_type, filename) =
            crate::entities::article::article_asset_bytes(id, &asset_name).await?;
        let output_path = match destination {
            Some(AssetOutput::Exact(path)) => Some(path.to_path_buf()),
            Some(AssetOutput::Directory(directory)) => {
                Some(asset_export_path(directory, &filename)?)
            }
            None => None,
        };
        return Ok(Some(CommandOutcome::stdout_article_asset(
            bytes,
            media_type,
            output_path,
        )));
    }
    if matches!(destination, Some(AssetOutput::Exact(_))) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--output requires asset <asset-key> as the sole article section".into(),
        )
        .into());
    }
    if !article_assets_request(sections)? {
        if view != "compact" || explicit_limit.is_some() || offset != 0 {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "--asset-view, --asset-limit, and --asset-offset require assets as the sole section".into(),
            ).into());
        }
        return Ok(None);
    }
    if view == "compact" && (explicit_limit.is_some() || offset != 0) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "compact asset view rejects --asset-limit and nonzero --asset-offset".into(),
        )
        .into());
    }
    let limit = explicit_limit.unwrap_or(25);
    if !(1..=100).contains(&limit) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--asset-limit must be between 1 and 100".into(),
        )
        .into());
    }
    offset.checked_add(limit).ok_or_else(|| {
        crate::error::BioMcpError::InvalidArgument("asset offset plus limit overflows".into())
    })?;
    if !json_output {
        anyhow::bail!(crate::error::BioMcpError::InvalidArgument(
            "Article asset manifests are JSON-only; rerun with --json (example: biomcp --json get article 22663011 assets)"
                .into(),
        ));
    }

    let manifest = crate::entities::article::article_assets_manifest(id).await?;
    let commands = manifest_next_commands(&manifest);
    let (assets, coverage) = bounded_manifest_rows(&manifest)?;
    let mut value = serde_json::to_value(&manifest)?;
    let object = value
        .as_object_mut()
        .expect("manifest serializes as object");
    object.remove("assets");
    object.remove("coverage");
    let asset_total = assets.len();
    let coverage_total = coverage.len();
    let (asset_offset, asset_limit) = if view == "retrievable" {
        (offset, limit)
    } else {
        (0, 25)
    };
    let (coverage_offset, coverage_limit) = if view == "coverage" {
        (offset, limit)
    } else {
        (0, 10)
    };
    let asset_page = page_values(&assets, asset_offset, asset_limit);
    let coverage_page = page_values(&coverage, coverage_offset, coverage_limit);
    if view != "coverage" {
        object.insert("assets".into(), asset_page.clone().into());
    }
    if view != "retrievable" {
        object.insert("coverage".into(), coverage_page.clone().into());
    }
    if view == "compact" {
        object.insert(
            "asset_page".into(),
            pagination(
                id,
                "retrievable",
                asset_offset,
                asset_limit,
                asset_page.len(),
                asset_total,
            ),
        );
        object.insert(
            "coverage_page".into(),
            pagination(
                id,
                "coverage",
                coverage_offset,
                coverage_limit,
                coverage_page.len(),
                coverage_total,
            ),
        );
    } else {
        let total = if view == "retrievable" {
            asset_total
        } else {
            coverage_total
        };
        let returned = if view == "retrievable" {
            asset_page.len()
        } else {
            coverage_page.len()
        };
        object.insert(
            "pagination".into(),
            pagination(id, view, offset, limit, returned, total),
        );
    }
    object.insert(
        "_meta".into(),
        serde_json::to_value(crate::cli::search_meta(commands))?,
    );
    Ok(Some(CommandOutcome::stdout(
        crate::render::json::to_pretty(&value)?,
    )))
}

fn dedupe_values(values: &mut Vec<serde_json::Value>) {
    let mut seen = std::collections::HashSet::new();
    values.retain(|value| seen.insert(serde_json::to_string(value).unwrap_or_default()));
}

fn bounded_manifest_rows(
    manifest: &crate::entities::article::ArticleAssetsManifest,
) -> Result<(Vec<serde_json::Value>, Vec<serde_json::Value>), serde_json::Error> {
    let mut assets = manifest
        .assets
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    dedupe_values(&mut assets);

    let mut seen = Vec::new();
    let coverage = manifest
        .coverage
        .iter()
        .filter(|row| {
            !manifest.assets.iter().any(|asset| {
                asset.filename == row.filename
                    && asset.provider == row.provider
                    && asset.discovery_routes.iter().any(|route| {
                        route.provider == row.provider
                            && route.source_document == row.source_document
                    })
            })
        })
        .filter(|row| {
            let key = (
                row.provider.clone(),
                row.source_document,
                row.filename.clone(),
                row.outcome,
            );
            if seen.contains(&key) {
                false
            } else {
                seen.push(key);
                true
            }
        })
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((assets, coverage))
}

fn page_values(
    values: &[serde_json::Value],
    offset: usize,
    limit: usize,
) -> Vec<serde_json::Value> {
    values.iter().skip(offset).take(limit).cloned().collect()
}

fn pagination(
    id: &str,
    view: &str,
    offset: usize,
    limit: usize,
    returned: usize,
    total: usize,
) -> serde_json::Value {
    let has_more = offset.saturating_add(returned) < total;
    let next_offset = has_more.then_some(offset.saturating_add(returned));
    serde_json::json!({
        "returned": returned, "total": total, "has_more": has_more, "next_offset": next_offset,
        "continuation_command": next_offset.map(|next| format!("biomcp --json get article {} --asset-view {view} --asset-limit {limit} --asset-offset {next} assets", crate::render::markdown::shell_quote_arg(id)))
    })
}

fn asset_export_path(
    directory: &std::path::Path,
    filename: &str,
) -> Result<std::path::PathBuf, crate::error::BioMcpError> {
    let mut components = std::path::Path::new(filename).components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Article asset filename must be one non-empty normal path component".into(),
        ));
    }
    Ok(directory.join(filename))
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
#[path = "../../../tests/unit/cli/article_assets.rs"]
mod tests;
