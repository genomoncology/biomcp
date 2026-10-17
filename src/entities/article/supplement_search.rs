//! Article supplement text search (2152).
//!
//! One resolution of the article's assets feeds a text search across its
//! xlsx, xls, csv, docx, pdf and zip supplements. Every hit carries an exact
//! locator — worksheet and row, page, paragraph, or archive member path —
//! plus the source file's checksum, and every file that could not be
//! extracted is reported with its format and reason so an empty search stays
//! distinguishable from an unread file. Matches are exact text only.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::transform::article::{
    SupplementExtractionProblem, SupplementFormat, SupplementTextBlock, SupplementTextBudget,
    SupplementTextExtraction, extract_supplement_text, supplement_format_for_filename,
};

use super::{
    ArticleAssetEntry, ArticleSupplementSearch, ArticleSupplementSearchFile,
    ArticleSupplementSearchFileOutcome, ArticleSupplementSearchHit,
};

// The design recorded in the ticket: 64 MiB per source file and per archive
// member, 256 MiB aggregate per source file, 500 PDF pages, one nested
// archive level past the source zip, 20 s per source file, 200 reported hits.
const SUPPLEMENT_SEARCH_MAX_FILE_BYTES: usize = 64 * 1024 * 1024;
const SUPPLEMENT_SEARCH_BUDGET: SupplementTextBudget = SupplementTextBudget {
    max_member_bytes: 64 * 1024 * 1024,
    max_aggregate_bytes: 256 * 1024 * 1024,
    max_pdf_pages: 500,
    archive_depth: 2,
};
const SUPPLEMENT_SEARCH_FILE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);
const SUPPLEMENT_SEARCH_HIT_LIMIT: usize = 200;
const SUPPLEMENT_SEARCH_CONTEXT_CHARS: usize = 400;
const SUPPLEMENT_SEARCH_NEEDLE_MAX_BYTES: usize = 1024;
const SUPPLEMENT_TEXT_CACHE_SCHEMA: u8 = 1;
const SUPPLEMENT_TEXT_CACHE_MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Deserialize, Serialize)]
struct CachedSupplementText {
    schema: u8,
    extraction: SupplementTextExtraction,
}

/// Searches an article's supplement text for an exact substring.
///
/// `asset_filter` restricts the search to one manifest asset by asset key or
/// unambiguous file name; without it every retrievable supplement with a
/// searchable file extension is searched.
pub async fn article_supplement_search(
    requested_id: &str,
    needle: &str,
    asset_filter: Option<&str>,
) -> Result<ArticleSupplementSearch, BioMcpError> {
    let needle = needle.trim();
    if needle.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "supplement-search requires a non-empty search string".into(),
        ));
    }
    if needle.len() > SUPPLEMENT_SEARCH_NEEDLE_MAX_BYTES {
        return Err(BioMcpError::InvalidArgument(
            "supplement-search string is longer than 1024 bytes".into(),
        ));
    }
    let article = super::detail::get_article_base(requested_id, false).await?;
    let resolved = super::assets::resolve_article_assets(requested_id, article).await?;
    let manifest = &resolved.manifest;
    let bytes = &resolved.bytes;

    let selected: Vec<&ArticleAssetEntry> = match asset_filter {
        Some(wanted) => {
            let wanted = wanted.trim();
            match super::assets::select_manifest_asset(&manifest.assets, wanted) {
                Some(asset) => vec![asset],
                // A key no retrievable asset answers reaches the same typed
                // refusal or not-found error the raw `asset` download gives.
                None => {
                    let refused = super::assets::article_asset_bytes_from_resolution(
                        requested_id,
                        wanted,
                        super::assets::ResolvedArticleAssets {
                            manifest: manifest.clone(),
                            bytes: BTreeMap::new(),
                        },
                    )
                    .err();
                    return Err(refused.unwrap_or_else(|| {
                        BioMcpError::InvalidArgument(
                            "supplement-search asset resolved without bytes".into(),
                        )
                    }));
                }
            }
        }
        None => manifest
            .assets
            .iter()
            .filter(|asset| supplement_format_for_filename(&asset.filename).is_some())
            .collect(),
    };

    let mut files = Vec::new();
    let mut hits = Vec::new();
    let mut hits_truncated = false;
    for asset in selected {
        let format = supplement_format_for_filename(&asset.filename);
        let Some(bytes) = bytes.get(&asset.asset_key) else {
            continue;
        };
        if bytes.len() > SUPPLEMENT_SEARCH_MAX_FILE_BYTES {
            files.push(ArticleSupplementSearchFile {
                asset_key: asset.asset_key.clone(),
                filename: asset.filename.clone(),
                member: None,
                format: format
                    .map(|format| format.as_str())
                    .unwrap_or("other")
                    .to_string(),
                sha256: asset.sha256.clone(),
                outcome: ArticleSupplementSearchFileOutcome::Unextractable,
                truncated: false,
                reason: Some(format!(
                    "file exceeds the {SUPPLEMENT_SEARCH_MAX_FILE_BYTES} byte extraction limit"
                )),
            });
            continue;
        }
        let extraction = supplement_text_for_bytes(&asset.sha256, format, bytes).await;
        if let Some(problem) = extraction.problems.iter().find(|problem| problem.member.is_none())
        {
            files.push(ArticleSupplementSearchFile {
                asset_key: asset.asset_key.clone(),
                filename: asset.filename.clone(),
                member: None,
                format: problem.format.clone(),
                sha256: asset.sha256.clone(),
                outcome: if problem.empty_text {
                    ArticleSupplementSearchFileOutcome::EmptyText
                } else {
                    ArticleSupplementSearchFileOutcome::Unextractable
                },
                truncated: false,
                reason: Some(problem.reason.clone()),
            });
            continue;
        }
        files.extend(file_outcome_rows(asset, &extraction));
        for block in &extraction.blocks {
            if !block.text.contains(needle) {
                continue;
            }
            if hits.len() >= SUPPLEMENT_SEARCH_HIT_LIMIT {
                hits_truncated = true;
                break;
            }
            hits.push(ArticleSupplementSearchHit {
                asset_key: asset.asset_key.clone(),
                filename: asset.filename.clone(),
                member: block.member.clone(),
                format: format
                    .map(|format| format.as_str())
                    .unwrap_or("other")
                    .to_string(),
                sha256: asset.sha256.clone(),
                worksheet: block.worksheet.clone(),
                row: block.row,
                page: block.page,
                paragraph: block.paragraph,
                locator: block_locator_text(block),
                context: truncate_chars(&block.text, SUPPLEMENT_SEARCH_CONTEXT_CHARS),
            });
        }
    }

    Ok(ArticleSupplementSearch {
        article_id: manifest.article_id.clone(),
        pmid: manifest.pmid.clone(),
        pmcid: manifest.pmcid.clone(),
        query: needle.to_string(),
        files,
        hits,
        hits_truncated,
    })
}

/// One row per searched file, and one row per archive member inside it.
fn file_outcome_rows(
    asset: &ArticleAssetEntry,
    extraction: &SupplementTextExtraction,
) -> Vec<ArticleSupplementSearchFile> {
    let format = supplement_format_for_filename(&asset.filename)
        .map(|format| format.as_str())
        .unwrap_or("other");
    let row = |member: Option<String>,
                   outcome: ArticleSupplementSearchFileOutcome,
                   reason: Option<String>| {
        let member_format = match &member {
            Some(member) => member_leaf_format(member).unwrap_or(format),
            None => format,
        };
        ArticleSupplementSearchFile {
            asset_key: asset.asset_key.clone(),
            filename: asset.filename.clone(),
            member,
            format: member_format.to_string(),
            sha256: asset.sha256.clone(),
            outcome,
            truncated: extraction.truncated,
            reason,
        }
    };
    if format != "zip" {
        let outcome = if extraction.blocks.is_empty() {
            ArticleSupplementSearchFileOutcome::EmptyText
        } else {
            ArticleSupplementSearchFileOutcome::Searched
        };
        return vec![row(None, outcome, None)];
    }
    let mut rows = Vec::new();
    for problem in &extraction.problems {
        rows.push(row(
            problem.member.clone(),
            if problem.empty_text {
                ArticleSupplementSearchFileOutcome::EmptyText
            } else {
                ArticleSupplementSearchFileOutcome::Unextractable
            },
            Some(problem.reason.clone()),
        ));
    }
    let mut searched_members = extraction
        .blocks
        .iter()
        .filter_map(|block| block.member.clone())
        .collect::<Vec<_>>();
    searched_members.sort();
    searched_members.dedup();
    for member in searched_members {
        if rows
            .iter()
            .any(|existing| existing.member.as_deref() == Some(member.as_str()))
        {
            continue;
        }
        rows.push(row(
            Some(member),
            ArticleSupplementSearchFileOutcome::Searched,
            None,
        ));
    }
    rows
}

fn member_leaf_format(member: &str) -> Option<&'static str> {
    let leaf = member.rsplit('/').next().unwrap_or(member);
    supplement_format_for_filename(leaf).map(|format| format.as_str())
}

fn block_locator_text(block: &SupplementTextBlock) -> String {
    let mut parts = Vec::new();
    if let Some(member) = &block.member {
        parts.push(format!("member '{member}'"));
    }
    if let Some(worksheet) = &block.worksheet {
        parts.push(worksheet.clone());
    }
    if let Some(row) = block.row {
        parts.push(format!("row {row}"));
    }
    if let Some(page) = block.page {
        parts.push(format!("page {page}"));
    }
    if let Some(paragraph) = block.paragraph {
        parts.push(format!("paragraph {paragraph}"));
    }
    parts.join(" ")
}

fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let cut: String = text.chars().take(limit).collect();
    format!("{cut}…")
}

/// Reads the cached extraction for these bytes, or extracts, budgets, and
/// caches it. A time-limit failure is never cached.
async fn supplement_text_for_bytes(
    sha256: &str,
    format: Option<SupplementFormat>,
    bytes: &[u8],
) -> SupplementTextExtraction {
    if !crate::sources::is_no_cache_enabled()
        && let Some(extraction) = read_cached_supplement_text(sha256).await
    {
        return extraction;
    }
    let Some(format) = format else {
        return SupplementExtractionProblem::unread(
            None,
            "other",
            "unsupported format".to_string(),
        )
        .into_extraction();
    };
    let extraction = match tokio::time::timeout(
        SUPPLEMENT_SEARCH_FILE_TIMEOUT,
        tokio::task::spawn_blocking({
            let bytes = bytes.to_vec();
            move || extract_supplement_text(format, &bytes, &SUPPLEMENT_SEARCH_BUDGET)
        }),
    )
    .await
    {
        Ok(Ok(extraction)) => extraction,
        Ok(Err(error)) => SupplementExtractionProblem::unread(
            None,
            format.as_str(),
            format!("extraction worker failed: {error}"),
        )
        .into_extraction(),
        Err(_) => SupplementExtractionProblem::unread(
            None,
            format.as_str(),
            "extraction time limit exceeded".to_string(),
        )
        .into_extraction(),
    };
    if !extraction_timed_out(&extraction) && !crate::sources::is_no_cache_enabled() {
        write_cached_supplement_text(sha256, &extraction).await;
    }
    extraction
}

fn extraction_timed_out(extraction: &SupplementTextExtraction) -> bool {
    extraction
        .problems
        .iter()
        .any(|problem| problem.reason == "extraction time limit exceeded")
}

fn supplement_text_cache_path(sha256: &str) -> Result<std::path::PathBuf, BioMcpError> {
    let config = crate::cache::resolve_cache_config()?;
    Ok(config
        .cache_root
        .join("sessions")
        .join("article-supplement-text-v1")
        .join(format!("{sha256}.json")))
}

async fn read_cached_supplement_text(sha256: &str) -> Option<SupplementTextExtraction> {
    let path = supplement_text_cache_path(sha256).ok()?;
    let metadata = match tokio::fs::symlink_metadata(&path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            tracing::warn!("Supplement text cache metadata unavailable: {error}");
            return None;
        }
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > SUPPLEMENT_TEXT_CACHE_MAX_BYTES as u64
    {
        let _ = tokio::fs::remove_file(&path).await;
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            tracing::warn!("Ignoring linked supplement text cache entry");
            return None;
        }
    }
    let bytes = tokio::fs::read(&path).await.ok()?;
    let cached: CachedSupplementText = match serde_json::from_slice(&bytes) {
        Ok(cached) => cached,
        Err(_) => {
            let _ = tokio::fs::remove_file(&path).await;
            return None;
        }
    };
    if cached.schema != SUPPLEMENT_TEXT_CACHE_SCHEMA {
        let _ = tokio::fs::remove_file(&path).await;
        return None;
    }
    if let Some(parent) = path.parent()
        && let Err(error) = crate::cache::secure_managed_tree(parent, true, None)
    {
        tracing::warn!("Supplement text cache permissions could not be repaired: {error}");
        return None;
    }
    Some(cached.extraction)
}

async fn write_cached_supplement_text(sha256: &str, extraction: &SupplementTextExtraction) {
    let cached = CachedSupplementText {
        schema: SUPPLEMENT_TEXT_CACHE_SCHEMA,
        extraction: extraction.clone(),
    };
    let Ok(bytes) = serde_json::to_vec(&cached) else {
        tracing::warn!("Supplement text cache serialization failed");
        return;
    };
    if bytes.len() > SUPPLEMENT_TEXT_CACHE_MAX_BYTES {
        tracing::warn!("Supplement text cache entry exceeded its size budget");
        return;
    }
    let Ok(path) = supplement_text_cache_path(sha256) else {
        return;
    };
    if let Err(error) = crate::utils::download::write_atomic_bytes(&path, &bytes).await {
        tracing::warn!("Supplement text cache write failed: {error}");
    }
}
