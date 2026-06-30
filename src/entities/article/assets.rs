use std::collections::{BTreeMap, BTreeSet};

use roxmltree::Node;
use sha2::{Digest, Sha256};

use crate::error::BioMcpError;
use crate::sources::figshare::{
    FigshareArticle, FigshareArticleRef, FigshareArticleSearchResult, FigshareClient, FigshareFile,
    parse_figshare_article_url,
};
use crate::sources::ncbi_idconv::NcbiIdConverterClient;
use crate::sources::pmc_oa::{
    PmcOaArchiveEntry, PmcOaArchiveManifest, PmcOaArchivePackage, PmcOaClient,
};

use super::{
    Article, ArticleAssetCoverage, ArticleAssetEntry, ArticleAssetJats, ArticleAssetsManifest,
    ArticleFulltextProvenance, ArticleFulltextProvider, ArticleFulltextReuse, ArticleNotIncluded,
    ArticleOmittedCoverage,
};

const PMC_PROVIDER_LABEL: &str = "PMC OA Archive";
const PMC_PROVIDER_SOURCE: &str = "PMC OA";
const FIGSHARE_PROVIDER_LABEL: &str = "Figshare";
const FIGSHARE_PROVIDER_SOURCE: &str = "Figshare";
const FIGSHARE_COLLECTION_RECORD_LIMIT: usize = 25;

#[derive(Clone, Debug, Default)]
struct JatsAssetFacts {
    kind: Option<&'static str>,
    label: Option<String>,
    caption: Option<String>,
    source_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct JatsFacts {
    assets: BTreeMap<String, JatsAssetFacts>,
    complex_tables: usize,
}

pub async fn article_assets_manifest(
    requested_id: &str,
) -> Result<ArticleAssetsManifest, BioMcpError> {
    let mut article = super::detail::get_article_base(requested_id).await?;
    if let Some((pmcid, package)) = try_pmc_package(&article, requested_id).await {
        return Ok(build_assets_manifest(
            requested_id,
            &article,
            &pmcid,
            package,
        ));
    }
    if let Some(manifest) = figshare_assets_manifest(requested_id, &mut article).await? {
        return Ok(manifest);
    }
    Err(no_supported_asset_source(requested_id))
}

pub async fn article_asset_bytes(
    requested_id: &str,
    filename: &str,
) -> Result<Vec<u8>, BioMcpError> {
    let mut article = super::detail::get_article_base(requested_id).await?;
    let wanted = filename.trim();
    if let Some((_, package)) = try_pmc_package(&article, requested_id).await {
        return package
            .entries
            .into_iter()
            .find(|entry| !entry.is_xml && entry.filename == wanted)
            .map(|entry| entry.bytes)
            .ok_or_else(|| article_asset_not_found(requested_id, wanted));
    }
    if let Some(bytes) = figshare_asset_bytes(&mut article, wanted).await? {
        return Ok(bytes);
    }
    Err(article_asset_not_found(requested_id, wanted))
}

pub(super) async fn attach_not_included(article: &mut Article, requested_id: &str) {
    let Some((pmcid, package)) = try_pmc_package(article, requested_id).await else {
        return;
    };
    let manifest = build_assets_manifest(requested_id, article, &pmcid, package);
    article.not_included = manifest.not_included;
}

async fn try_pmc_package(
    article: &Article,
    requested_id: &str,
) -> Option<(String, PmcOaArchivePackage)> {
    let pmcid = match resolve_article_pmcid(article, requested_id).await {
        Ok(pmcid) => pmcid,
        Err(err) => {
            tracing::warn!(
                ?err,
                requested_id,
                "Article PMC OA asset coverage unavailable"
            );
            return None;
        }
    };
    match fetch_package(&pmcid).await {
        Ok(package) => Some((pmcid, package)),
        Err(err) => {
            tracing::warn!(
                ?err,
                requested_id,
                "PMC OA package unavailable for asset coverage"
            );
            None
        }
    }
}

fn article_asset_not_found(requested_id: &str, wanted: &str) -> BioMcpError {
    BioMcpError::NotFound {
        entity: "article asset".to_string(),
        id: wanted.to_string(),
        suggestion: format!("List assets: biomcp --json get article {requested_id} assets"),
    }
}

fn no_supported_asset_source(requested_id: &str) -> BioMcpError {
    BioMcpError::NotFound {
        entity: "article asset source".to_string(),
        id: requested_id.to_string(),
        suggestion:
            "No supported article asset source found: no PMC OA package or supported Figshare asset URL."
                .to_string(),
    }
}

async fn fetch_package(pmcid: &str) -> Result<PmcOaArchivePackage, BioMcpError> {
    PmcOaClient::new()?
        .get_archive_package(pmcid)
        .await?
        .ok_or_else(|| BioMcpError::NotFound {
            entity: "PMC OA package".to_string(),
            id: pmcid.to_string(),
            suggestion: "Try fulltext or verify the article has a PMC Open Access package."
                .to_string(),
        })
}

async fn resolve_article_pmcid(
    article: &Article,
    requested_id: &str,
) -> Result<String, BioMcpError> {
    if let Some(pmcid) = article
        .pmcid
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(pmcid.to_string());
    }

    let ncbi = NcbiIdConverterClient::new()?;
    if let Some(pmid) = article.pmid.as_deref()
        && let Some(pmcid) = ncbi.pmid_to_pmcid(pmid).await?
    {
        return Ok(pmcid);
    }
    if let Some(doi) = article.doi.as_deref()
        && let Some(pmcid) = ncbi.doi_to_pmcid(doi).await?
    {
        return Ok(pmcid);
    }

    Err(BioMcpError::NotFound {
        entity: "PMC OA package".to_string(),
        id: requested_id.to_string(),
        suggestion: "Article has no resolved PMCID for OA asset retrieval.".to_string(),
    })
}

async fn figshare_collection(
    article: &mut Article,
) -> Result<Option<Vec<FigshareArticle>>, BioMcpError> {
    super::detail::enrich_article_with_semantic_scholar(article).await?;
    let Some(url) = article
        .semantic_scholar
        .as_ref()
        .and_then(|semantic| semantic.open_access_pdf.as_ref())
        .map(|pdf| pdf.url.trim())
        .filter(|url| !url.is_empty())
    else {
        return Ok(None);
    };
    let Some(reference) = parse_figshare_article_url(url) else {
        return Ok(None);
    };

    let client = FigshareClient::new()?;
    let linked = client.article(&reference).await?;
    let target_doi = article
        .doi
        .as_deref()
        .and_then(normalize_doi)
        .or_else(|| linked.doi.as_deref().and_then(normalize_doi));
    let target_title = normalize_title(&article.title)
        .or_else(|| linked.title.as_deref().and_then(normalize_title));
    let mut ids = vec![linked.article_id];
    let mut seen = BTreeSet::from([linked.article_id]);
    let doi_additions = match target_doi.as_deref() {
        Some(doi) => {
            append_figshare_search_ids(
                &client,
                doi,
                target_doi.as_deref(),
                target_title.as_deref(),
                &mut seen,
                &mut ids,
            )
            .await?
        }
        None => 0,
    };
    if doi_additions == 0
        && let Some(title) = target_title.as_deref()
    {
        append_figshare_search_ids(
            &client,
            title,
            target_doi.as_deref(),
            Some(title),
            &mut seen,
            &mut ids,
        )
        .await?;
    }

    let mut articles = vec![linked];
    for article_id in ids.into_iter().skip(1) {
        let reference = FigshareArticleRef {
            article_id,
            file_id: None,
        };
        match client.article(&reference).await {
            Ok(article) => articles.push(article),
            Err(err) => tracing::warn!(?err, article_id, "Figshare sibling article unavailable"),
        }
    }
    Ok(Some(articles))
}

async fn append_figshare_search_ids(
    client: &FigshareClient,
    query: &str,
    target_doi: Option<&str>,
    target_title: Option<&str>,
    seen: &mut BTreeSet<u64>,
    ids: &mut Vec<u64>,
) -> Result<usize, BioMcpError> {
    let rows = match client.search_articles(query).await {
        Ok(rows) => rows,
        Err(err) => {
            tracing::warn!(?err, query, "Figshare sibling search unavailable");
            return Ok(0);
        }
    };
    Ok(append_matching_figshare_ids(
        rows,
        target_doi,
        target_title,
        seen,
        ids,
    ))
}

fn append_matching_figshare_ids(
    mut rows: Vec<FigshareArticleSearchResult>,
    target_doi: Option<&str>,
    target_title: Option<&str>,
    seen: &mut BTreeSet<u64>,
    ids: &mut Vec<u64>,
) -> usize {
    rows.sort_by_key(|row| row.article_id);
    let mut additions = 0;
    for row in rows
        .into_iter()
        .filter(|row| figshare_same_paper(row, target_doi, target_title))
    {
        if seen.contains(&row.article_id) {
            continue;
        }
        if ids.len() >= FIGSHARE_COLLECTION_RECORD_LIMIT {
            tracing::warn!(
                limit = FIGSHARE_COLLECTION_RECORD_LIMIT,
                "Figshare sibling enumeration truncated"
            );
            break;
        }
        seen.insert(row.article_id);
        ids.push(row.article_id);
        additions += 1;
    }
    additions
}

fn figshare_same_paper(
    row: &FigshareArticleSearchResult,
    target_doi: Option<&str>,
    target_title: Option<&str>,
) -> bool {
    if let (Some(candidate), Some(target)) =
        (row.doi.as_deref().and_then(normalize_doi), target_doi)
        && candidate == target
    {
        return true;
    }
    if let (Some(candidate), Some(target)) =
        (row.title.as_deref().and_then(normalize_title), target_title)
    {
        return candidate == target || figshare_supplement_title_matches(&candidate, target);
    }
    false
}

fn figshare_supplement_title_matches(candidate: &str, target: &str) -> bool {
    (candidate.starts_with("supplementary ") || candidate.starts_with("supplemental "))
        && candidate.ends_with(target)
}

fn normalize_doi(raw: &str) -> Option<String> {
    let value = raw
        .trim()
        .trim_start_matches("doi:")
        .trim_start_matches("DOI:")
        .to_ascii_lowercase();
    (!value.is_empty()).then_some(value)
}

fn normalize_title(raw: &str) -> Option<String> {
    let mut text = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => {}
            _ if ch.is_alphanumeric() => text.extend(ch.to_lowercase()),
            _ => text.push(' '),
        }
    }
    let value = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!value.is_empty()).then_some(value)
}

async fn figshare_assets_manifest(
    requested_id: &str,
    article: &mut Article,
) -> Result<Option<ArticleAssetsManifest>, BioMcpError> {
    let Some(figshare_articles) = figshare_collection(article).await? else {
        return Ok(None);
    };
    let Some(first_article) = figshare_articles.first() else {
        return Ok(None);
    };
    let provider = figshare_provider();
    let provenance = figshare_provenance(first_article, article);
    let client = FigshareClient::new()?;
    let mut seen_files = BTreeSet::new();
    let mut assets = Vec::new();
    for figshare in &figshare_articles {
        let reuse = figshare_reuse(figshare);
        let asset_provenance = figshare_provenance(figshare, article);
        for file in &figshare.files {
            if !seen_files.insert(file.filename.clone()) {
                continue;
            }
            match client.download_file(file).await {
                Ok(bytes) => assets.push(figshare_asset_entry(
                    requested_id,
                    file,
                    &bytes,
                    &provider,
                    &reuse,
                    &asset_provenance,
                )),
                Err(err) => {
                    tracing::warn!(?err, filename = %file.filename, "Figshare asset unavailable")
                }
            }
        }
    }
    if assets.is_empty() {
        return Ok(None);
    }
    assets.sort_by(|left, right| left.filename.cmp(&right.filename));
    let mut manifest = ArticleAssetsManifest {
        article_id: requested_id.trim().to_string(),
        pmid: article.pmid.clone(),
        pmcid: None,
        provider,
        provenance,
        assets,
        not_included: None,
    };
    manifest.not_included = Some(not_included_from_manifest(&manifest));
    Ok(Some(manifest))
}

async fn figshare_asset_bytes(
    article: &mut Article,
    wanted: &str,
) -> Result<Option<Vec<u8>>, BioMcpError> {
    let Some(figshare_articles) = figshare_collection(article).await? else {
        return Ok(None);
    };
    let client = FigshareClient::new()?;
    let mut seen_files = BTreeSet::new();
    for figshare in &figshare_articles {
        for file in &figshare.files {
            if !seen_files.insert(file.filename.clone()) {
                continue;
            }
            if file.filename == wanted {
                return client.download_file(file).await.map(Some);
            }
        }
    }
    Ok(None)
}

fn figshare_provider() -> ArticleFulltextProvider {
    ArticleFulltextProvider {
        label: FIGSHARE_PROVIDER_LABEL.to_string(),
        source: FIGSHARE_PROVIDER_SOURCE.to_string(),
    }
}

fn figshare_reuse(article: &FigshareArticle) -> ArticleFulltextReuse {
    let license = article
        .license
        .as_ref()
        .and_then(|license| license.name.clone().or_else(|| license.url.clone()));
    match license {
        Some(license) => ArticleFulltextReuse {
            license_present: true,
            license: Some(license),
            reuse_warning: None,
        },
        None => ArticleFulltextReuse {
            license_present: false,
            license: None,
            reuse_warning: Some(
                "License/reuse status is unknown; verify rights before reuse.".to_string(),
            ),
        },
    }
}

fn figshare_provenance(figshare: &FigshareArticle, article: &Article) -> ArticleFulltextProvenance {
    ArticleFulltextProvenance {
        open_access: article.open_access,
        retracted: None,
        package_url: figshare
            .public_url
            .clone()
            .or_else(|| figshare.api_url.clone()),
        pdf_fallback_used: false,
    }
}

fn article_asset_command(article_id: &str, filename: &str) -> String {
    crate::next_command::NextCommand::biomcp()
        .args(["get", "article"])
        .arg(article_id.trim())
        .arg("asset")
        .arg(filename)
        .render_shell()
}

fn figshare_asset_entry(
    requested_id: &str,
    file: &FigshareFile,
    bytes: &[u8],
    provider: &ArticleFulltextProvider,
    reuse: &ArticleFulltextReuse,
    provenance: &ArticleFulltextProvenance,
) -> ArticleAssetEntry {
    ArticleAssetEntry {
        filename: file.filename.clone(),
        kind: figshare_kind(file).to_string(),
        size_bytes: bytes.len(),
        sha256: sha256_hex(bytes),
        provider: provider.clone(),
        reuse: reuse.clone(),
        provenance: provenance.clone(),
        jats: None,
        handle: article_asset_command(requested_id, &file.filename),
    }
}

fn figshare_kind(file: &FigshareFile) -> &'static str {
    if file
        .mimetype
        .as_deref()
        .is_some_and(|mimetype| mimetype.to_ascii_lowercase().starts_with("image/"))
    {
        return "figure-image";
    }
    filename_kind(&file.filename)
}

fn build_assets_manifest(
    requested_id: &str,
    article: &Article,
    pmcid: &str,
    package: PmcOaArchivePackage,
) -> ArticleAssetsManifest {
    let facts = jats_facts(&package.entries);
    let provider = provider();
    let reuse = reuse(&package.manifest, article);
    let provenance = provenance(&package.manifest, article);
    let mut assets = package
        .entries
        .iter()
        .filter(|entry| !entry.is_xml)
        .map(|entry| asset_entry(requested_id, entry, &facts, &provider, &reuse, &provenance))
        .collect::<Vec<_>>();
    assets.sort_by(|a, b| a.filename.cmp(&b.filename));
    let mut manifest = ArticleAssetsManifest {
        article_id: requested_id.trim().to_string(),
        pmid: article.pmid.clone(),
        pmcid: Some(pmcid.to_string()),
        provider,
        provenance,
        assets,
        not_included: None,
    };
    manifest.not_included = Some(not_included_from_manifest(&manifest));
    manifest.not_included.as_mut().unwrap().complex_tables.count = facts.complex_tables;
    manifest
}

fn provider() -> ArticleFulltextProvider {
    ArticleFulltextProvider {
        label: PMC_PROVIDER_LABEL.to_string(),
        source: PMC_PROVIDER_SOURCE.to_string(),
    }
}

fn reuse(manifest: &PmcOaArchiveManifest, article: &Article) -> ArticleFulltextReuse {
    match manifest
        .license
        .clone()
        .or_else(|| article.europepmc_license.clone())
    {
        Some(license) => ArticleFulltextReuse {
            license_present: true,
            license: Some(license),
            reuse_warning: None,
        },
        None => ArticleFulltextReuse {
            license_present: false,
            license: None,
            reuse_warning: Some(
                "License/reuse status is unknown; verify rights before reuse.".to_string(),
            ),
        },
    }
}

fn provenance(manifest: &PmcOaArchiveManifest, article: &Article) -> ArticleFulltextProvenance {
    ArticleFulltextProvenance {
        open_access: article.open_access,
        retracted: manifest.retracted.or(article.europepmc_retracted),
        package_url: Some(manifest.package_url.clone()),
        pdf_fallback_used: false,
    }
}

fn asset_entry(
    requested_id: &str,
    entry: &PmcOaArchiveEntry,
    facts: &JatsFacts,
    provider: &ArticleFulltextProvider,
    reuse: &ArticleFulltextReuse,
    provenance: &ArticleFulltextProvenance,
) -> ArticleAssetEntry {
    let jats = facts.assets.get(&entry.filename);
    let kind = jats
        .and_then(|fact| fact.kind)
        .unwrap_or_else(|| filename_kind(&entry.filename))
        .to_string();
    ArticleAssetEntry {
        filename: entry.filename.clone(),
        kind,
        size_bytes: entry.bytes.len(),
        sha256: sha256_hex(&entry.bytes),
        provider: provider.clone(),
        reuse: reuse.clone(),
        provenance: provenance.clone(),
        jats: jats.and_then(article_asset_jats),
        handle: article_asset_command(requested_id, &entry.filename),
    }
}

fn article_asset_jats(facts: &JatsAssetFacts) -> Option<ArticleAssetJats> {
    if facts.label.is_none() && facts.caption.is_none() && facts.source_id.is_none() {
        return None;
    }
    Some(ArticleAssetJats {
        label: facts.label.clone(),
        caption: facts.caption.clone(),
        source_id: facts.source_id.clone(),
    })
}

fn not_included_from_manifest(manifest: &ArticleAssetsManifest) -> ArticleNotIncluded {
    let figure_count = manifest
        .assets
        .iter()
        .filter(|asset| asset.kind == "figure-image")
        .count();
    let supplement_count = manifest
        .assets
        .iter()
        .filter(|asset| asset.kind == "supplementary-file")
        .count();
    let retrieve_with = crate::next_command::NextCommand::biomcp()
        .args(["--json", "get", "article", &manifest.article_id, "assets"])
        .render_shell();
    let mut next_commands = vec![retrieve_with.clone()];
    if let Some(handle) = manifest
        .assets
        .iter()
        .find_map(|asset| (asset.kind == "supplementary-file").then(|| asset.handle.clone()))
    {
        next_commands.push(handle);
    }
    ArticleNotIncluded {
        figure_images: ArticleAssetCoverage {
            count: figure_count,
            retrieve_with: retrieve_with.clone(),
        },
        supplementary_files: ArticleAssetCoverage {
            count: supplement_count,
            retrieve_with: retrieve_with.clone(),
        },
        complex_tables: ArticleOmittedCoverage {
            count: 0,
            retrieve_with,
        },
        next_commands,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn filename_kind(filename: &str) -> &'static str {
    let lower = filename.to_ascii_lowercase();
    if matches!(
        lower.rsplit('.').next(),
        Some("png" | "jpg" | "jpeg" | "gif" | "tif" | "tiff" | "svg" | "webp")
    ) {
        return "figure-image";
    }
    if lower.contains("supp")
        || lower.contains("suppl")
        || lower.contains("s1")
        || matches!(
            lower.rsplit('.').next(),
            Some("csv" | "tsv" | "xlsx" | "xls" | "doc" | "docx" | "pdf")
        )
    {
        return "supplementary-file";
    }
    "other"
}

fn jats_facts(entries: &[PmcOaArchiveEntry]) -> JatsFacts {
    entries
        .iter()
        .find(|entry| entry.is_xml)
        .and_then(|entry| std::str::from_utf8(&entry.bytes).ok())
        .and_then(parse_jats_facts)
        .unwrap_or_default()
}

fn parse_jats_facts(xml: &str) -> Option<JatsFacts> {
    let doc = roxmltree::Document::parse(xml).ok()?;
    let mut out = JatsFacts {
        complex_tables: doc
            .descendants()
            .filter(|node| node.is_element() && node.tag_name().name() == "table-wrap")
            .filter(|node| {
                node.descendants().any(|desc| {
                    desc.is_element()
                        && (desc.attribute("rowspan").is_some()
                            || desc.attribute("colspan").is_some())
                })
            })
            .count(),
        ..JatsFacts::default()
    };

    for node in doc.descendants().filter(|node| node.is_element()) {
        match node.tag_name().name() {
            "fig" => add_asset_facts(&mut out, node, "figure-image"),
            "supplementary-material" => add_asset_facts(&mut out, node, "supplementary-file"),
            _ => {}
        }
    }
    Some(out)
}

fn add_asset_facts(out: &mut JatsFacts, node: Node<'_, '_>, kind: &'static str) {
    let label = child_text(node, "label");
    let caption = child_text(node, "caption");
    let source_id = node.attribute("id").map(str::to_string);
    for href in node
        .descendants()
        .filter(|desc| desc.is_element())
        .filter_map(node_href)
        .filter_map(normalize_href)
    {
        let entry = out.assets.entry(href).or_default();
        entry.kind = Some(kind);
        if entry.label.is_none() {
            entry.label = label.clone();
        }
        if entry.caption.is_none() {
            entry.caption = caption.clone();
        }
        if entry.source_id.is_none() {
            entry.source_id = source_id.clone();
        }
    }
}

fn node_href<'a, 'input>(node: Node<'a, 'input>) -> Option<&'a str> {
    const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
    node.attribute((XLINK_NS, "href"))
        .or_else(|| node.attribute("href"))
}

fn normalize_href(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_start_matches("./");
    if trimmed.is_empty() || trimmed.starts_with('/') || trimmed.contains("..") {
        return None;
    }
    Some(trimmed.rsplit('/').next().unwrap_or(trimmed).to_string())
}

fn child_text(node: Node<'_, '_>, child_name: &str) -> Option<String> {
    let child = node
        .children()
        .find(|child| child.is_element() && child.tag_name().name() == child_name)?;
    let text = child
        .descendants()
        .filter(|desc| desc.is_text())
        .filter_map(|desc| desc.text())
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!text.is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_article() -> Article {
        Article {
            pmid: Some("22663011".to_string()),
            pmcid: Some("PMC123456".to_string()),
            doi: None,
            title: "Fixture article".to_string(),
            authors: Vec::new(),
            journal: None,
            date: None,
            citation_count: None,
            publication_type: None,
            open_access: Some(true),
            abstract_text: None,
            full_text_path: None,
            full_text_note: None,
            full_text_source: None,
            full_text_manifest: None,
            not_included: None,
            europepmc_license: None,
            europepmc_retracted: None,
            annotations: None,
            semantic_scholar: None,
            pubtator_fallback: false,
        }
    }

    #[test]
    fn build_manifest_hashes_binary_bytes_and_quotes_retrieval_commands() {
        let binary = vec![0, 0xff, b'P', b'N', b'G', b'\n'];
        let csv = b"time,value\n0,1\n".to_vec();
        let jats_xml = br#"
<article xmlns:xlink="http://www.w3.org/1999/xlink">
  <body>
    <fig id="f1">
      <label>Figure 1</label>
      <caption><p>Binary panel.</p></caption>
      <graphic xlink:href="fig 1.png" />
    </fig>
    <supplementary-material id="s1" xlink:href="traces-s1.csv">
      <label>Supplement S1</label>
      <caption><p>Trace data.</p></caption>
    </supplementary-material>
    <table-wrap><table><tr><td rowspan="2">x</td></tr></table></table-wrap>
  </body>
</article>
"#;
        let package = PmcOaArchivePackage {
            manifest: PmcOaArchiveManifest {
                package_url: "https://example.test/archive.tgz".to_string(),
                tgz_url: "https://example.test/archive.tgz".to_string(),
                license: Some("CC BY".to_string()),
                retracted: Some(false),
            },
            entries: vec![
                PmcOaArchiveEntry {
                    filename: "article.nxml".to_string(),
                    bytes: jats_xml.to_vec(),
                    is_xml: true,
                },
                PmcOaArchiveEntry {
                    filename: "fig 1.png".to_string(),
                    bytes: binary.clone(),
                    is_xml: false,
                },
                PmcOaArchiveEntry {
                    filename: "traces-s1.csv".to_string(),
                    bytes: csv.clone(),
                    is_xml: false,
                },
            ],
        };

        let manifest =
            build_assets_manifest("10.1000/foo bar", &sample_article(), "PMC123456", package);
        let fig = manifest
            .assets
            .iter()
            .find(|asset| asset.filename == "fig 1.png")
            .expect("figure asset should be listed");
        assert_eq!(fig.kind, "figure-image");
        assert_eq!(fig.size_bytes, binary.len());
        assert_eq!(fig.sha256, sha256_hex(&binary));
        assert_eq!(
            fig.handle,
            "biomcp get article \"10.1000/foo bar\" asset \"fig 1.png\""
        );
        assert_eq!(
            fig.jats.as_ref().and_then(|jats| jats.label.as_deref()),
            Some("Figure 1")
        );

        let not_included = manifest.not_included.expect("coverage summary");
        assert_eq!(not_included.figure_images.count, 1);
        assert_eq!(not_included.supplementary_files.count, 1);
        assert_eq!(not_included.complex_tables.count, 1);
        assert_eq!(
            not_included.figure_images.retrieve_with,
            "biomcp --json get article \"10.1000/foo bar\" assets"
        );
    }

    fn figshare_row(
        article_id: u64,
        title: Option<&str>,
        doi: Option<&str>,
    ) -> FigshareArticleSearchResult {
        FigshareArticleSearchResult {
            article_id,
            title: title.map(str::to_string),
            doi: doi.map(str::to_string),
            api_url: None,
            public_url: None,
        }
    }

    #[test]
    fn figshare_same_paper_matches_doi_or_normalized_exact_title() {
        assert!(figshare_same_paper(
            &figshare_row(1, Some("Different title"), Some("10.1000/Example")),
            Some("10.1000/example"),
            Some("target title"),
        ));
        assert!(figshare_same_paper(
            &figshare_row(2, Some(" Target   Title "), None),
            Some("10.1000/example"),
            Some("target title"),
        ));
        assert!(figshare_same_paper(
            &figshare_row(
                4,
                Some("Supplementary Table S1 from High-Throughput <i>ERBB2</i>."),
                Some("10.1000/figshare-record"),
            ),
            Some("10.1000/article"),
            Some("high throughput erbb2"),
        ));
        assert!(!figshare_same_paper(
            &figshare_row(3, Some("Unrelated"), Some("10.1000/other")),
            Some("10.1000/example"),
            Some("target title"),
        ));
    }

    #[test]
    fn append_matching_figshare_ids_dedupes_sorts_and_caps() {
        let mut rows = (0..30)
            .rev()
            .map(|offset| figshare_row(100 + offset, Some("Target Title"), None))
            .collect::<Vec<_>>();
        rows.push(figshare_row(100, Some("Target Title"), None));
        rows.push(figshare_row(999, Some("Unrelated"), None));
        let mut seen = BTreeSet::from([100]);
        let mut ids = vec![100];

        let additions =
            append_matching_figshare_ids(rows, None, Some("target title"), &mut seen, &mut ids);

        assert_eq!(additions, FIGSHARE_COLLECTION_RECORD_LIMIT - 1);
        assert_eq!(ids.len(), FIGSHARE_COLLECTION_RECORD_LIMIT);
        assert_eq!(ids[0], 100);
        assert_eq!(ids[1], 101);
        assert!(!ids.contains(&999));
    }
}
