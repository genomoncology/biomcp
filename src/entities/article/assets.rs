use std::collections::{BTreeMap, BTreeSet};

use futures::StreamExt;
use roxmltree::Node;
use sha2::{Digest, Sha256};

use crate::error::BioMcpError;
use crate::sources::europepmc::{
    EuropePmcClient, EuropePmcSupplementaryEntry, EuropePmcSupplementaryPackage,
};
use crate::sources::figshare::{
    FigshareArticle, FigshareArticleRef, FigshareArticleSearchResult, FigshareClient, FigshareFile,
    parse_figshare_article_url,
};
use crate::sources::ncbi_efetch::NcbiEfetchClient;
use crate::sources::ncbi_idconv::NcbiIdConverterClient;
use crate::sources::pmc_article::{PmcArticleClient, PmcLinkedFetch};
use crate::sources::pmc_oa::{
    PmcOaArchiveEntry, PmcOaArchiveManifest, PmcOaArchivePackage, PmcOaClient,
};
use crate::xml::{ARTICLE_XML_NODE_LIMIT, parse_external_xml};

use super::{
    Article, ArticleAssetCoverage, ArticleAssetDiscoveryRoute, ArticleAssetEntry, ArticleAssetJats,
    ArticleAssetNamedCoverage, ArticleAssetNamedOutcome, ArticleAssetSourceDocument,
    ArticleAssetsManifest, ArticleFulltextProvenance, ArticleFulltextProvider,
    ArticleFulltextReuse, ArticleNotIncluded, ArticleOmittedCoverage,
};

const PMC_PROVIDER_LABEL: &str = "PMC OA Archive";
const PMC_PROVIDER_SOURCE: &str = "PMC OA";
const EUROPE_PMC_PROVIDER_LABEL: &str = "Europe PMC Supplementary Files";
const EUROPE_PMC_PROVIDER_SOURCE: &str = "Europe PMC";
const FIGSHARE_PROVIDER_LABEL: &str = "Figshare";
const FIGSHARE_PROVIDER_SOURCE: &str = "Figshare";
const FIGSHARE_COLLECTION_RECORD_LIMIT: usize = 25;

enum SourceAttempt<T> {
    Success(T),
    Absent,
    Failed,
}

enum ArchivePackage {
    Pmc {
        pmcid: String,
        package: PmcOaArchivePackage,
    },
    EuropePmc {
        pmcid: String,
        package: EuropePmcSupplementaryPackage,
        pmc_manifest: Option<PmcOaArchiveManifest>,
    },
}

struct FigshareCollection {
    articles: Vec<FigshareArticle>,
    failed: bool,
}

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

const LINKED_CANDIDATE_LIMIT: usize = 256;
const LINKED_FETCH_CONCURRENCY: usize = 8;
const LINKED_AGGREGATE_LIMIT: usize = 64 * 1024 * 1024;

struct ResolvedArticleAssets {
    manifest: ArticleAssetsManifest,
    bytes: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone)]
struct LinkedCandidate {
    href: String,
    filename: String,
    label: Option<String>,
    media_type: Option<String>,
    route: ArticleAssetDiscoveryRoute,
    additional_routes: Vec<ArticleAssetDiscoveryRoute>,
    relative_to_bin: bool,
}

struct ResolvedAsset {
    canonical_identity: String,
    precedence: u8,
    entry: ArticleAssetEntry,
    bytes: Vec<u8>,
}

struct PendingCoverage {
    canonical_identity: String,
    row: ArticleAssetNamedCoverage,
}

struct PmcPackageObservation {
    manifest: PmcOaArchiveManifest,
    package: Option<PmcOaArchivePackage>,
    failed: bool,
}

struct EuropePackageObservation {
    package: EuropePmcSupplementaryPackage,
    pmc_manifest: Option<PmcOaArchiveManifest>,
}

struct FigshareAssetsObservation {
    assets: Vec<ResolvedAsset>,
    nonretrievable: Vec<PendingCoverage>,
    failed: bool,
}

pub async fn article_assets_manifest(
    requested_id: &str,
) -> Result<ArticleAssetsManifest, BioMcpError> {
    let article = super::detail::get_article_base(requested_id).await?;
    Ok(resolve_article_assets(requested_id, article)
        .await?
        .manifest)
}

pub async fn article_asset_bytes(
    requested_id: &str,
    asset_key: &str,
) -> Result<Vec<u8>, BioMcpError> {
    let article = super::detail::get_article_base(requested_id).await?;
    resolve_article_assets(requested_id, article)
        .await?
        .bytes
        .remove(asset_key.trim())
        .ok_or_else(|| article_asset_not_found(requested_id, asset_key.trim()))
}

pub(super) async fn attach_not_included(article: &mut Article, requested_id: &str) {
    let manifest = match resolve_archive_package(article).await {
        SourceAttempt::Success(ArchivePackage::Pmc { pmcid, package }) => {
            build_assets_manifest(requested_id, article, &pmcid, package)
        }
        SourceAttempt::Success(ArchivePackage::EuropePmc {
            pmcid,
            package,
            pmc_manifest,
        }) => build_europe_pmc_manifest(
            requested_id,
            article,
            &pmcid,
            package,
            pmc_manifest.as_ref(),
        ),
        SourceAttempt::Absent | SourceAttempt::Failed => return,
    };
    article.not_included = manifest.not_included;
}

async fn resolve_article_assets(
    requested_id: &str,
    mut article: Article,
) -> Result<ResolvedArticleAssets, BioMcpError> {
    let (pmcid, mut any_failed) = match resolve_article_pmcid(&article).await {
        Ok(pmcid) => (pmcid, false),
        Err(_) => {
            tracing::warn!("Article asset identity resolution failed");
            (None, true)
        }
    };

    let mut assets = Vec::<ResolvedAsset>::new();
    let mut candidates = Vec::<LinkedCandidate>::new();
    let mut nonretrievable = Vec::<PendingCoverage>::new();
    let mut complex_tables = 0usize;

    if let Some(pmcid) = pmcid.as_deref() {
        let mut figshare_article = article.clone();
        let (pmc, europe, europe_xml, ncbi_xml, html, figshare) = tokio::join!(
            observe_pmc_package(pmcid),
            observe_europe_package(pmcid),
            observe_europe_xml(pmcid),
            observe_ncbi_xml(pmcid),
            observe_pmc_html(pmcid),
            observe_figshare_assets(requested_id, &mut figshare_article),
        );

        let retained_pmc_manifest = match pmc {
            SourceAttempt::Success(observation) => {
                any_failed |= observation.failed;
                if let Some(package) = observation.package {
                    let facts = jats_facts(&package.entries);
                    complex_tables = facts.complex_tables;
                    if let Some(xml_entry) = package.entries.iter().find(|entry| entry.is_xml) {
                        match std::str::from_utf8(&xml_entry.bytes) {
                            Ok(xml) => {
                                any_failed |= append_jats_candidates(
                                    &mut candidates,
                                    xml,
                                    jats_route(provider()),
                                )
                                .is_err();
                            }
                            Err(_) => any_failed = true,
                        }
                    }
                    let package_root = package
                        .entries
                        .iter()
                        .find(|entry| entry.is_xml)
                        .and_then(|entry| entry.filename.rsplit_once('/').map(|(root, _)| root));
                    let reuse = reuse(&observation.manifest, &article);
                    let provenance = provenance(&observation.manifest, &article);
                    let route = ArticleAssetDiscoveryRoute {
                        provider: provider(),
                        source_document: ArticleAssetSourceDocument::PmcOaArchive,
                    };
                    for entry in package.entries.iter().filter(|entry| !entry.is_xml) {
                        let identity = package_identity(
                            "pmc",
                            pmcid,
                            package_relative_path(&entry.filename, package_root),
                        );
                        let mut public = asset_entry(
                            requested_id,
                            entry,
                            &facts,
                            &route.provider,
                            &reuse,
                            &provenance,
                        );
                        public.asset_key = sha256_hex(identity.as_bytes());
                        assets.push(ResolvedAsset {
                            canonical_identity: identity,
                            precedence: 0,
                            entry: public,
                            bytes: entry.bytes.clone(),
                        });
                    }
                }
                Some(observation.manifest)
            }
            SourceAttempt::Absent => None,
            SourceAttempt::Failed => {
                any_failed = true;
                None
            }
        };

        match europe {
            SourceAttempt::Success(observation) => {
                let provider = ArticleFulltextProvider {
                    label: EUROPE_PMC_PROVIDER_LABEL.to_string(),
                    source: EUROPE_PMC_PROVIDER_SOURCE.to_string(),
                };
                let pmc_manifest = observation
                    .pmc_manifest
                    .as_ref()
                    .or(retained_pmc_manifest.as_ref());
                let reuse = europe_pmc_reuse(pmc_manifest, &article);
                let provenance = ArticleFulltextProvenance {
                    open_access: article.open_access,
                    retracted: pmc_manifest
                        .and_then(|manifest| manifest.retracted)
                        .or(article.europepmc_retracted),
                    package_url: None,
                    pdf_fallback_used: false,
                };
                for entry in &observation.package.entries {
                    let identity = package_identity("europe-pmc", pmcid, &entry.filename);
                    let mut public =
                        europe_pmc_asset_entry(requested_id, entry, &provider, &reuse, &provenance);
                    public.asset_key = sha256_hex(identity.as_bytes());
                    assets.push(ResolvedAsset {
                        canonical_identity: identity,
                        precedence: 1,
                        entry: public,
                        bytes: entry.bytes.clone(),
                    });
                }
            }
            SourceAttempt::Absent => {}
            SourceAttempt::Failed => any_failed = true,
        }

        match europe_xml {
            SourceAttempt::Success(xml) => {
                any_failed |= append_jats_candidates(
                    &mut candidates,
                    &xml,
                    jats_route(ArticleFulltextProvider {
                        label: "Europe PMC XML".to_string(),
                        source: "Europe PMC".to_string(),
                    }),
                )
                .is_err();
            }
            SourceAttempt::Absent => {}
            SourceAttempt::Failed => any_failed = true,
        }
        match ncbi_xml {
            SourceAttempt::Success(xml) => {
                any_failed |= append_jats_candidates(
                    &mut candidates,
                    &xml,
                    jats_route(ArticleFulltextProvider {
                        label: "NCBI EFetch PMC XML".to_string(),
                        source: "NCBI EFetch".to_string(),
                    }),
                )
                .is_err();
            }
            SourceAttempt::Absent => {}
            SourceAttempt::Failed => any_failed = true,
        }
        match html {
            SourceAttempt::Success(html) => {
                match crate::transform::article::extract_pmc_supplement_links(&html) {
                    Ok(links) => candidates.extend(links.into_iter().map(|link| LinkedCandidate {
                        href: link.href,
                        filename: link.filename,
                        label: link.label,
                        media_type: link.media_type,
                        route: ArticleAssetDiscoveryRoute {
                            provider: linked_provider(),
                            source_document: ArticleAssetSourceDocument::PmcHtml,
                        },
                        additional_routes: Vec::new(),
                        relative_to_bin: false,
                    })),
                    Err(_) => any_failed = true,
                }
            }
            SourceAttempt::Absent => {}
            SourceAttempt::Failed => any_failed = true,
        }

        resolve_linked_candidates(
            requested_id,
            &article,
            pmcid,
            candidates,
            &mut assets,
            &mut nonretrievable,
        )
        .await;
        fold_figshare_observation(figshare, &mut assets, &mut nonretrievable, &mut any_failed);
    } else {
        let figshare = observe_figshare_assets(requested_id, &mut article).await;
        fold_figshare_observation(figshare, &mut assets, &mut nonretrievable, &mut any_failed);
    }

    finish_resolution(
        requested_id,
        &article,
        pmcid.as_deref(),
        assets,
        nonretrievable,
        complex_tables,
        any_failed,
    )
}

fn fold_figshare_observation(
    observation: SourceAttempt<FigshareAssetsObservation>,
    assets: &mut Vec<ResolvedAsset>,
    nonretrievable: &mut Vec<PendingCoverage>,
    any_failed: &mut bool,
) {
    match observation {
        SourceAttempt::Success(mut found) => {
            assets.append(&mut found.assets);
            nonretrievable.append(&mut found.nonretrievable);
            *any_failed |= found.failed;
        }
        SourceAttempt::Absent => {}
        SourceAttempt::Failed => *any_failed = true,
    }
}

fn linked_provider() -> ArticleFulltextProvider {
    ArticleFulltextProvider {
        label: "PMC Linked Article Asset".to_string(),
        source: "PMC".to_string(),
    }
}

fn jats_route(provider: ArticleFulltextProvider) -> ArticleAssetDiscoveryRoute {
    ArticleAssetDiscoveryRoute {
        provider,
        source_document: ArticleAssetSourceDocument::JatsXml,
    }
}

fn package_identity(provider: &str, pmcid: &str, filename: &str) -> String {
    format!("{provider}:{pmcid}:{}", filename.replace('\\', "/"))
}

fn package_relative_path<'a>(filename: &'a str, root: Option<&str>) -> &'a str {
    root.and_then(|root| filename.strip_prefix(root)?.strip_prefix('/'))
        .unwrap_or(filename)
}

async fn resolve_linked_candidates(
    requested_id: &str,
    article: &Article,
    pmcid: &str,
    candidates: Vec<LinkedCandidate>,
    assets: &mut Vec<ResolvedAsset>,
    nonretrievable: &mut Vec<PendingCoverage>,
) {
    let client = match PmcArticleClient::new(pmcid) {
        Ok(client) => client,
        Err(_) => {
            nonretrievable.extend(candidates.into_iter().map(|candidate| PendingCoverage {
                canonical_identity: rejected_identity(&candidate),
                row: named_coverage(&candidate, ArticleAssetNamedOutcome::SourceUnavailable),
            }));
            return;
        }
    };

    let mut classified = candidates
        .into_iter()
        .map(|candidate| {
            let target = client
                .linked_target(&candidate.href, candidate.relative_to_bin)
                .ok();
            let identity = target
                .as_ref()
                .map(|target| target.canonical_identity.clone())
                .unwrap_or_else(|| rejected_identity(&candidate));
            (identity, target, candidate)
        })
        .collect::<Vec<_>>();
    classified.sort_by(|left, right| {
        (left.0.as_str(), left.2.href.as_str()).cmp(&(right.0.as_str(), right.2.href.as_str()))
    });
    let overflow = split_candidate_overflow(&mut classified, LINKED_CANDIDATE_LIMIT);
    for (identity, _, candidate) in overflow {
        nonretrievable.push(PendingCoverage {
            canonical_identity: identity,
            row: named_coverage(&candidate, ArticleAssetNamedOutcome::SourceUnavailable),
        });
    }

    let mut canonical = BTreeMap::<String, (Vec<_>, LinkedCandidate)>::new();
    for (identity, target, candidate) in classified {
        let Some(target) = target else {
            nonretrievable.push(PendingCoverage {
                canonical_identity: identity,
                row: named_coverage(&candidate, ArticleAssetNamedOutcome::UnsupportedOrigin),
            });
            continue;
        };
        match canonical.entry(identity) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert((vec![target], candidate));
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                let (targets, existing) = entry.get_mut();
                if !targets.iter().any(|current| current.url == target.url) {
                    targets.push(target);
                }
                merge_candidate(existing, candidate);
            }
        }
    }

    let mut already_resolved = BTreeMap::<String, usize>::new();
    for (index, asset) in assets.iter().enumerate() {
        already_resolved
            .entry(asset.canonical_identity.clone())
            .or_insert(index);
    }
    let mut to_fetch = Vec::new();
    for (identity, (targets, candidate)) in canonical {
        if let Some(index) = already_resolved.get(&identity).copied() {
            merge_candidate_into_entry(&mut assets[index].entry, &candidate);
        } else {
            to_fetch.push((identity, targets, candidate));
        }
    }

    // Buffered preserves canonical order while allowing up to eight independent identities to
    // make progress. Folding results in that order keeps aggregate-budget victims deterministic.
    let mut fetches =
        futures::stream::iter(to_fetch.into_iter().map(|(identity, targets, candidate)| {
            let client = client.clone();
            async move {
                let result = client.fetch_first_available(&targets).await;
                (identity, candidate, result)
            }
        }))
        .buffered(LINKED_FETCH_CONCURRENCY);
    let mut aggregate = 0usize;
    while let Some((identity, candidate, result)) = fetches.next().await {
        match result {
            PmcLinkedFetch::Bytes { bytes, media_type }
                if linked_aggregate_accepts(aggregate, bytes.len()) =>
            {
                aggregate += bytes.len();
                let routes = candidate_routes(&candidate);
                let provider = candidate.route.provider.clone();
                let mut entry = ArticleAssetEntry {
                    filename: candidate.filename.clone(),
                    asset_key: sha256_hex(identity.as_bytes()),
                    kind: filename_kind(&candidate.filename).to_string(),
                    media_type: candidate.media_type.clone().or(media_type),
                    size_bytes: bytes.len(),
                    sha256: sha256_hex(&bytes),
                    provider: provider.clone(),
                    reuse: ArticleFulltextReuse {
                        license_present: false,
                        license: None,
                        license_source: None,
                        reuse_warning: Some(
                            "License/reuse status is unknown; verify rights before reuse."
                                .to_string(),
                        ),
                    },
                    provenance: ArticleFulltextProvenance {
                        open_access: article.open_access,
                        retracted: article.europepmc_retracted,
                        package_url: None,
                        pdf_fallback_used: false,
                    },
                    jats: candidate.label.clone().map(|label| ArticleAssetJats {
                        label: Some(label),
                        caption: None,
                        source_id: None,
                    }),
                    discovery_routes: routes,
                    handle: article_asset_command(requested_id, &candidate.filename),
                };
                entry.discovery_routes.sort();
                entry.discovery_routes.dedup();
                assets.push(ResolvedAsset {
                    canonical_identity: identity,
                    precedence: 2,
                    entry,
                    bytes,
                });
            }
            PmcLinkedFetch::Bytes { .. } | PmcLinkedFetch::SourceUnavailable => {
                nonretrievable.push(PendingCoverage {
                    canonical_identity: identity,
                    row: named_coverage(&candidate, ArticleAssetNamedOutcome::SourceUnavailable),
                });
            }
            PmcLinkedFetch::HealthyAbsent => nonretrievable.push(PendingCoverage {
                canonical_identity: identity,
                row: named_coverage(&candidate, ArticleAssetNamedOutcome::HealthyAbsent),
            }),
            PmcLinkedFetch::AccessOrLicenceDenied => nonretrievable.push(PendingCoverage {
                canonical_identity: identity,
                row: named_coverage(&candidate, ArticleAssetNamedOutcome::AccessOrLicenceDenied),
            }),
        }
    }
}

fn split_candidate_overflow<T>(values: &mut Vec<T>, limit: usize) -> Vec<T> {
    values.split_off(values.len().min(limit))
}

fn linked_aggregate_accepts(current: usize, next: usize) -> bool {
    current.saturating_add(next) <= LINKED_AGGREGATE_LIMIT
}

fn merge_candidate(existing: &mut LinkedCandidate, candidate: LinkedCandidate) {
    existing.additional_routes.push(candidate.route);
    existing
        .additional_routes
        .extend(candidate.additional_routes);
    existing.additional_routes.sort();
    existing.additional_routes.dedup();
    if existing.label.is_none() {
        existing.label = candidate.label;
    }
    if existing.media_type.is_none() {
        existing.media_type = candidate.media_type;
    }
}

fn merge_candidate_into_entry(entry: &mut ArticleAssetEntry, candidate: &LinkedCandidate) {
    entry.discovery_routes.extend(candidate_routes(candidate));
    entry.discovery_routes.sort();
    entry.discovery_routes.dedup();
    if entry.media_type.is_none() {
        entry.media_type = candidate.media_type.clone();
    }
    if entry.jats.is_none() && candidate.label.is_some() {
        entry.jats = Some(ArticleAssetJats {
            label: candidate.label.clone(),
            caption: None,
            source_id: None,
        });
    }
}

fn rejected_identity(candidate: &LinkedCandidate) -> String {
    format!("rejected:{}", sha256_hex(candidate.href.trim().as_bytes()))
}

fn append_jats_candidates(
    candidates: &mut Vec<LinkedCandidate>,
    xml: &str,
    route: ArticleAssetDiscoveryRoute,
) -> Result<(), ()> {
    let links = crate::transform::article::extract_jats_supplement_links(xml).map_err(|_| ())?;
    candidates.extend(links.into_iter().map(|link| LinkedCandidate {
        href: link.href,
        filename: link.filename,
        label: link.label,
        media_type: link.media_type,
        route: route.clone(),
        additional_routes: Vec::new(),
        relative_to_bin: true,
    }));
    Ok(())
}

fn candidate_routes(candidate: &LinkedCandidate) -> Vec<ArticleAssetDiscoveryRoute> {
    let mut routes = vec![candidate.route.clone()];
    routes.extend(candidate.additional_routes.clone());
    routes.sort();
    routes.dedup();
    routes
}

fn named_coverage(
    candidate: &LinkedCandidate,
    outcome: ArticleAssetNamedOutcome,
) -> ArticleAssetNamedCoverage {
    ArticleAssetNamedCoverage {
        filename: candidate.filename.clone(),
        asset_key: None,
        label: candidate.label.clone(),
        media_type: candidate.media_type.clone(),
        provider: candidate.route.provider.clone(),
        source_document: candidate.route.source_document,
        outcome,
        handle: None,
        discovery_routes: candidate_routes(candidate),
    }
}

fn finish_resolution(
    requested_id: &str,
    article: &Article,
    pmcid: Option<&str>,
    assets: Vec<ResolvedAsset>,
    nonretrievable: Vec<PendingCoverage>,
    complex_tables: usize,
    any_failed: bool,
) -> Result<ResolvedArticleAssets, BioMcpError> {
    let mut assets = merge_resolved_assets(assets);
    let successful_identities = assets
        .iter()
        .map(|asset| asset.canonical_identity.clone())
        .collect::<BTreeSet<_>>();
    let mut pending = BTreeMap::<String, ArticleAssetNamedCoverage>::new();
    for mut observation in nonretrievable {
        if successful_identities.contains(&observation.canonical_identity) {
            continue;
        }
        observation.row.discovery_routes.sort();
        observation.row.discovery_routes.dedup();
        match pending.entry(observation.canonical_identity) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(observation.row);
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                let existing = entry.get_mut();
                existing
                    .discovery_routes
                    .extend(observation.row.discovery_routes);
                existing.discovery_routes.sort();
                existing.discovery_routes.dedup();
                if named_outcome_priority(observation.row.outcome)
                    > named_outcome_priority(existing.outcome)
                {
                    existing.outcome = observation.row.outcome;
                }
                if existing.label.is_none() {
                    existing.label = observation.row.label;
                }
                if existing.media_type.is_none() {
                    existing.media_type = observation.row.media_type;
                }
            }
        }
    }

    if assets.is_empty() && pending.is_empty() {
        return Err(final_asset_source_error(requested_id, any_failed));
    }

    assign_asset_keys(requested_id, &mut assets);
    let primary = assets.first();
    let fallback_coverage = pending
        .values()
        .min_by_key(|coverage| source_document_precedence(coverage.source_document));
    let provider = primary
        .map(|asset| asset.entry.provider.clone())
        .or_else(|| fallback_coverage.map(|coverage| coverage.provider.clone()))
        .expect("assets or named coverage establishes a provider");
    let provenance = primary
        .map(|asset| asset.entry.provenance.clone())
        .unwrap_or(ArticleFulltextProvenance {
            open_access: article.open_access,
            retracted: article.europepmc_retracted,
            package_url: None,
            pdf_fallback_used: false,
        });

    let mut coverage = assets.iter().map(retrievable_coverage).collect::<Vec<_>>();
    coverage.extend(pending.into_values());
    coverage.sort_by(|left, right| {
        (
            left.filename.as_str(),
            left.asset_key.as_deref().unwrap_or(""),
            left.source_document,
        )
            .cmp(&(
                right.filename.as_str(),
                right.asset_key.as_deref().unwrap_or(""),
                right.source_document,
            ))
    });

    let mut bytes = BTreeMap::new();
    let mut entries = Vec::with_capacity(assets.len());
    for asset in assets {
        bytes.insert(asset.entry.asset_key.clone(), asset.bytes);
        entries.push(asset.entry);
    }
    entries.sort_by(|left, right| left.asset_key.cmp(&right.asset_key));
    let mut manifest = ArticleAssetsManifest {
        article_id: requested_id.trim().to_string(),
        pmid: article.pmid.clone(),
        pmcid: pmcid.map(str::to_string).or_else(|| article.pmcid.clone()),
        provider,
        provenance,
        assets: entries,
        coverage,
        not_included: None,
    };
    manifest.not_included = Some(not_included_from_manifest(&manifest));
    if let Some(not_included) = manifest.not_included.as_mut() {
        not_included.complex_tables.count = complex_tables;
    }
    Ok(ResolvedArticleAssets { manifest, bytes })
}

fn merge_resolved_assets(mut assets: Vec<ResolvedAsset>) -> Vec<ResolvedAsset> {
    assets.sort_by(|left, right| {
        (left.precedence, left.canonical_identity.as_str())
            .cmp(&(right.precedence, right.canonical_identity.as_str()))
    });

    let mut by_identity = BTreeMap::<String, ResolvedAsset>::new();
    for asset in assets {
        match by_identity.entry(asset.canonical_identity.clone()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(asset);
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                merge_entry_facts(&mut entry.get_mut().entry, &asset.entry);
            }
        }
    }

    let mut by_hash = BTreeMap::<String, ResolvedAsset>::new();
    for asset in by_identity.into_values() {
        match by_hash.entry(asset.entry.sha256.clone()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(asset);
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                let existing = entry.get_mut();
                if (asset.precedence, asset.canonical_identity.as_str())
                    < (existing.precedence, existing.canonical_identity.as_str())
                {
                    let mut replacement = asset;
                    merge_entry_facts(&mut replacement.entry, &existing.entry);
                    *existing = replacement;
                } else {
                    merge_entry_facts(&mut existing.entry, &asset.entry);
                }
            }
        }
    }
    let mut merged = by_hash.into_values().collect::<Vec<_>>();
    merged.sort_by(|left, right| {
        (left.precedence, left.canonical_identity.as_str())
            .cmp(&(right.precedence, right.canonical_identity.as_str()))
    });
    merged
}

fn merge_entry_facts(primary: &mut ArticleAssetEntry, other: &ArticleAssetEntry) {
    primary
        .discovery_routes
        .extend(other.discovery_routes.clone());
    primary.discovery_routes.sort();
    primary.discovery_routes.dedup();
    if primary.media_type.is_none() {
        primary.media_type = other.media_type.clone();
    }
    if primary.jats.is_none() {
        primary.jats = other.jats.clone();
    }
}

fn assign_asset_keys(requested_id: &str, assets: &mut [ResolvedAsset]) {
    let mut counts = BTreeMap::new();
    for asset in assets.iter() {
        *counts.entry(asset.entry.filename.clone()).or_insert(0usize) += 1;
    }
    for asset in assets {
        let filename = asset.entry.filename.clone();
        asset.entry.asset_key = if counts.get(&filename) == Some(&1) {
            filename.clone()
        } else {
            format!(
                "{}-{}--{}",
                provider_slug(&asset.entry.provider.source),
                sha256_hex(asset.canonical_identity.as_bytes()),
                filename
            )
        };
        asset.entry.handle = article_asset_command(requested_id, &asset.entry.asset_key);
    }
}

fn retrievable_coverage(asset: &ResolvedAsset) -> ArticleAssetNamedCoverage {
    let route = asset
        .entry
        .discovery_routes
        .iter()
        .filter(|route| route.provider == asset.entry.provider)
        .min_by_key(|route| source_document_precedence(route.source_document))
        .or_else(|| {
            asset
                .entry
                .discovery_routes
                .iter()
                .min_by_key(|route| source_document_precedence(route.source_document))
        })
        .cloned()
        .unwrap_or(ArticleAssetDiscoveryRoute {
            provider: asset.entry.provider.clone(),
            source_document: ArticleAssetSourceDocument::JatsXml,
        });
    ArticleAssetNamedCoverage {
        filename: asset.entry.filename.clone(),
        asset_key: Some(asset.entry.asset_key.clone()),
        label: asset
            .entry
            .jats
            .as_ref()
            .and_then(|jats| jats.label.clone()),
        media_type: asset.entry.media_type.clone(),
        provider: asset.entry.provider.clone(),
        source_document: route.source_document,
        outcome: ArticleAssetNamedOutcome::Retrievable,
        handle: Some(asset.entry.handle.clone()),
        discovery_routes: asset.entry.discovery_routes.clone(),
    }
}

fn source_document_precedence(source: ArticleAssetSourceDocument) -> u8 {
    match source {
        ArticleAssetSourceDocument::PmcOaArchive => 0,
        ArticleAssetSourceDocument::EuropePmcZip => 1,
        ArticleAssetSourceDocument::JatsXml | ArticleAssetSourceDocument::PmcHtml => 2,
        ArticleAssetSourceDocument::Figshare => 3,
    }
}

fn named_outcome_priority(outcome: ArticleAssetNamedOutcome) -> u8 {
    match outcome {
        ArticleAssetNamedOutcome::Retrievable => 5,
        ArticleAssetNamedOutcome::SourceUnavailable => 4,
        ArticleAssetNamedOutcome::AccessOrLicenceDenied => 3,
        ArticleAssetNamedOutcome::UnsupportedOrigin => 2,
        ArticleAssetNamedOutcome::HealthyAbsent => 1,
    }
}

fn provider_slug(provider: &str) -> String {
    let slug = provider
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    slug.split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

async fn observe_pmc_package(pmcid: &str) -> SourceAttempt<PmcPackageObservation> {
    let client = match PmcOaClient::new() {
        Ok(client) => client,
        Err(_) => return SourceAttempt::Failed,
    };
    let manifest = match client.oa_archive_manifest(pmcid).await {
        Ok(Some(manifest)) => manifest,
        Ok(None) => return SourceAttempt::Absent,
        Err(_) => return SourceAttempt::Failed,
    };
    match client.archive_package(manifest.clone()).await {
        Ok(package) => SourceAttempt::Success(PmcPackageObservation {
            manifest,
            package: Some(package),
            failed: false,
        }),
        Err(_) => SourceAttempt::Success(PmcPackageObservation {
            manifest,
            package: None,
            failed: true,
        }),
    }
}

async fn observe_europe_package(pmcid: &str) -> SourceAttempt<EuropePackageObservation> {
    let client = match EuropePmcClient::new() {
        Ok(client) => client,
        Err(_) => return SourceAttempt::Failed,
    };
    match client.get_supplementary_package(pmcid).await {
        Ok(Some(package)) => SourceAttempt::Success(EuropePackageObservation {
            package,
            pmc_manifest: None,
        }),
        Ok(None) => SourceAttempt::Absent,
        Err(_) => SourceAttempt::Failed,
    }
}

async fn observe_europe_xml(pmcid: &str) -> SourceAttempt<String> {
    let client = match EuropePmcClient::new() {
        Ok(client) => client,
        Err(_) => return SourceAttempt::Failed,
    };
    match client.get_full_text_xml("PMC", pmcid).await {
        Ok(Some(xml)) => SourceAttempt::Success(xml),
        Ok(None) => SourceAttempt::Absent,
        Err(_) => SourceAttempt::Failed,
    }
}

async fn observe_ncbi_xml(pmcid: &str) -> SourceAttempt<String> {
    let client = match NcbiEfetchClient::new() {
        Ok(client) => client,
        Err(_) => return SourceAttempt::Failed,
    };
    match client.get_full_text_xml(pmcid).await {
        Ok(Some(xml)) => SourceAttempt::Success(xml),
        Ok(None) => SourceAttempt::Absent,
        Err(_) => SourceAttempt::Failed,
    }
}

async fn observe_pmc_html(pmcid: &str) -> SourceAttempt<String> {
    match crate::sources::pmc_article::html(pmcid).await {
        Ok(Some(html)) => SourceAttempt::Success(html),
        Ok(None) => SourceAttempt::Absent,
        Err(_) => SourceAttempt::Failed,
    }
}

async fn resolve_archive_package(article: &Article) -> SourceAttempt<ArchivePackage> {
    let pmcid = match resolve_article_pmcid(article).await {
        Ok(Some(pmcid)) => pmcid,
        Ok(None) => return SourceAttempt::Absent,
        Err(_) => {
            tracing::warn!("Article asset identity resolution failed");
            return SourceAttempt::Failed;
        }
    };
    let pmc = match PmcOaClient::new() {
        Ok(client) => client,
        Err(_) => {
            tracing::warn!("PMC OA client unavailable for article assets");
            return SourceAttempt::Failed;
        }
    };
    let mut failed = false;
    let mut pmc_manifest = match pmc.oa_archive_manifest(&pmcid).await {
        Ok(manifest) => manifest,
        Err(_) => {
            tracing::warn!("PMC OA manifest request failed for article assets");
            failed = true;
            None
        }
    };
    if let Some(manifest) = pmc_manifest.clone() {
        match pmc.archive_package(manifest).await {
            Ok(package) => {
                return SourceAttempt::Success(ArchivePackage::Pmc { pmcid, package });
            }
            Err(_) => {
                tracing::warn!("PMC OA archive request failed for article assets");
                failed = true;
            }
        }
    }

    let europe_pmc = match EuropePmcClient::new() {
        Ok(client) => client,
        Err(_) => {
            tracing::warn!("Europe PMC client unavailable for article assets");
            return SourceAttempt::Failed;
        }
    };
    match europe_pmc.get_supplementary_package(&pmcid).await {
        Ok(Some(package)) => SourceAttempt::Success(ArchivePackage::EuropePmc {
            pmcid,
            package,
            pmc_manifest: pmc_manifest.take(),
        }),
        Ok(None) if !failed => SourceAttempt::Absent,
        Ok(None) => SourceAttempt::Failed,
        Err(_) => {
            tracing::warn!("Europe PMC supplementary request failed for article assets");
            SourceAttempt::Failed
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
        suggestion: "No supported article asset source found: no PMC OA, Europe PMC, or supported Figshare package."
            .to_string(),
    }
}

fn final_asset_source_error(requested_id: &str, failed: bool) -> BioMcpError {
    if failed {
        asset_sources_unavailable()
    } else {
        no_supported_asset_source(requested_id)
    }
}

fn asset_sources_unavailable() -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: "article asset providers".to_string(),
        reason: "Asset availability could not be confirmed because a consulted source failed."
            .to_string(),
        suggestion: "Retry later or inspect the article at its source.".to_string(),
    }
}

async fn resolve_article_pmcid(article: &Article) -> Result<Option<String>, BioMcpError> {
    if let Some(pmcid) = article
        .pmcid
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(Some(pmcid.to_string()));
    }

    let ncbi = NcbiIdConverterClient::new()?;
    if let Some(pmid) = article.pmid.as_deref()
        && let Some(pmcid) = ncbi.pmid_to_pmcid(pmid).await?
    {
        return Ok(Some(pmcid));
    }
    if let Some(doi) = article.doi.as_deref()
        && let Some(pmcid) = ncbi.doi_to_pmcid(doi).await?
    {
        return Ok(Some(pmcid));
    }

    Ok(None)
}

async fn figshare_collection(
    article: &mut Article,
) -> Result<Option<FigshareCollection>, BioMcpError> {
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
    let (doi_additions, mut failed) = match target_doi.as_deref() {
        Some(doi) => {
            append_figshare_search_ids(
                &client,
                doi,
                target_doi.as_deref(),
                target_title.as_deref(),
                &mut seen,
                &mut ids,
            )
            .await
        }
        None => (0, false),
    };
    if doi_additions == 0
        && let Some(title) = target_title.as_deref()
    {
        failed |= append_figshare_search_ids(
            &client,
            title,
            target_doi.as_deref(),
            Some(title),
            &mut seen,
            &mut ids,
        )
        .await
        .1;
    }

    let mut articles = vec![linked];
    for article_id in ids.into_iter().skip(1) {
        let reference = FigshareArticleRef {
            article_id,
            file_id: None,
        };
        match client.article(&reference).await {
            Ok(article) => articles.push(article),
            Err(_) => {
                failed = true;
                tracing::warn!("Figshare sibling article request failed");
            }
        }
    }
    Ok(Some(FigshareCollection { articles, failed }))
}

async fn append_figshare_search_ids(
    client: &FigshareClient,
    query: &str,
    target_doi: Option<&str>,
    target_title: Option<&str>,
    seen: &mut BTreeSet<u64>,
    ids: &mut Vec<u64>,
) -> (usize, bool) {
    let rows = match client.search_articles(query).await {
        Ok(rows) => rows,
        Err(_) => {
            tracing::warn!("Figshare sibling search request failed");
            return (0, true);
        }
    };
    (
        append_matching_figshare_ids(rows, target_doi, target_title, seen, ids),
        false,
    )
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

async fn observe_figshare_assets(
    requested_id: &str,
    article: &mut Article,
) -> SourceAttempt<FigshareAssetsObservation> {
    let collection = match figshare_collection(article).await {
        Ok(Some(collection)) => collection,
        Ok(None) => return SourceAttempt::Absent,
        Err(_) => return SourceAttempt::Failed,
    };
    let client = match FigshareClient::new() {
        Ok(client) => client,
        Err(_) => return SourceAttempt::Failed,
    };
    let provider = figshare_provider();
    let route = ArticleAssetDiscoveryRoute {
        provider: provider.clone(),
        source_document: ArticleAssetSourceDocument::Figshare,
    };
    let mut seen = BTreeSet::new();
    let mut assets = Vec::new();
    let mut nonretrievable = Vec::new();
    let mut failed = collection.failed;
    let mut named_files = 0usize;
    for figshare in &collection.articles {
        let reuse = figshare_reuse(figshare);
        let provenance = figshare_provenance(figshare, article);
        for file in &figshare.files {
            let identity = format!("figshare:{}:{}", figshare.article_id, file.id);
            if !seen.insert(identity.clone()) {
                continue;
            }
            named_files += 1;
            match client.download_file(file).await {
                Ok(bytes) => {
                    let mut entry = figshare_asset_entry(
                        requested_id,
                        file,
                        &bytes,
                        &provider,
                        &reuse,
                        &provenance,
                    );
                    entry.asset_key = sha256_hex(identity.as_bytes());
                    assets.push(ResolvedAsset {
                        canonical_identity: identity,
                        precedence: 3,
                        entry,
                        bytes,
                    });
                }
                Err(_) => {
                    failed = true;
                    nonretrievable.push(PendingCoverage {
                        canonical_identity: identity,
                        row: ArticleAssetNamedCoverage {
                            filename: file.filename.clone(),
                            asset_key: None,
                            label: None,
                            media_type: file.mimetype.clone(),
                            provider: provider.clone(),
                            source_document: ArticleAssetSourceDocument::Figshare,
                            outcome: ArticleAssetNamedOutcome::SourceUnavailable,
                            handle: None,
                            discovery_routes: vec![route.clone()],
                        },
                    });
                }
            }
        }
    }
    if named_files == 0 {
        if failed {
            SourceAttempt::Failed
        } else {
            SourceAttempt::Absent
        }
    } else {
        SourceAttempt::Success(FigshareAssetsObservation {
            assets,
            nonretrievable,
            failed,
        })
    }
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
            license_source: None,
            reuse_warning: None,
        },
        None => ArticleFulltextReuse {
            license_present: false,
            license: None,
            license_source: None,
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
        asset_key: file.filename.clone(),
        kind: figshare_kind(file).to_string(),
        media_type: file.mimetype.clone(),
        size_bytes: bytes.len(),
        sha256: sha256_hex(bytes),
        provider: provider.clone(),
        reuse: reuse.clone(),
        provenance: provenance.clone(),
        jats: None,
        discovery_routes: vec![ArticleAssetDiscoveryRoute {
            provider: provider.clone(),
            source_document: ArticleAssetSourceDocument::Figshare,
        }],
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

fn build_europe_pmc_manifest(
    requested_id: &str,
    article: &Article,
    pmcid: &str,
    package: EuropePmcSupplementaryPackage,
    pmc_manifest: Option<&PmcOaArchiveManifest>,
) -> ArticleAssetsManifest {
    let provider = ArticleFulltextProvider {
        label: EUROPE_PMC_PROVIDER_LABEL.to_string(),
        source: EUROPE_PMC_PROVIDER_SOURCE.to_string(),
    };
    let reuse = europe_pmc_reuse(pmc_manifest, article);
    let provenance = ArticleFulltextProvenance {
        open_access: article.open_access,
        retracted: pmc_manifest
            .and_then(|manifest| manifest.retracted)
            .or(article.europepmc_retracted),
        package_url: None,
        pdf_fallback_used: false,
    };
    let mut assets = package
        .entries
        .iter()
        .map(|entry| europe_pmc_asset_entry(requested_id, entry, &provider, &reuse, &provenance))
        .collect::<Vec<_>>();
    assets.sort_by(|left, right| left.filename.cmp(&right.filename));
    let mut manifest = ArticleAssetsManifest {
        article_id: requested_id.trim().to_string(),
        pmid: article.pmid.clone(),
        pmcid: Some(pmcid.to_string()),
        provider,
        provenance,
        assets,
        coverage: Vec::new(),
        not_included: None,
    };
    manifest.not_included = Some(not_included_from_manifest(&manifest));
    manifest
}

fn europe_pmc_reuse(
    pmc_manifest: Option<&PmcOaArchiveManifest>,
    article: &Article,
) -> ArticleFulltextReuse {
    if let Some(license) = pmc_manifest.and_then(|manifest| manifest.license.clone()) {
        return ArticleFulltextReuse {
            license_present: true,
            license: Some(license),
            license_source: Some(provider()),
            reuse_warning: None,
        };
    }
    match article.europepmc_license.clone() {
        Some(license) => ArticleFulltextReuse {
            license_present: true,
            license: Some(license),
            license_source: None,
            reuse_warning: None,
        },
        None => ArticleFulltextReuse {
            license_present: false,
            license: None,
            license_source: None,
            reuse_warning: Some(
                "License/reuse status is unknown; verify rights before reuse.".to_string(),
            ),
        },
    }
}

fn europe_pmc_asset_entry(
    requested_id: &str,
    entry: &EuropePmcSupplementaryEntry,
    provider: &ArticleFulltextProvider,
    reuse: &ArticleFulltextReuse,
    provenance: &ArticleFulltextProvenance,
) -> ArticleAssetEntry {
    ArticleAssetEntry {
        filename: entry.filename.clone(),
        asset_key: entry.filename.clone(),
        kind: filename_kind(&entry.filename).to_string(),
        media_type: None,
        size_bytes: entry.bytes.len(),
        sha256: sha256_hex(&entry.bytes),
        provider: provider.clone(),
        reuse: reuse.clone(),
        provenance: provenance.clone(),
        jats: None,
        discovery_routes: vec![ArticleAssetDiscoveryRoute {
            provider: provider.clone(),
            source_document: ArticleAssetSourceDocument::EuropePmcZip,
        }],
        handle: article_asset_command(requested_id, &entry.filename),
    }
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
        coverage: Vec::new(),
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
            license_source: None,
            reuse_warning: None,
        },
        None => ArticleFulltextReuse {
            license_present: false,
            license: None,
            license_source: None,
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
        asset_key: entry.filename.clone(),
        kind,
        media_type: None,
        size_bytes: entry.bytes.len(),
        sha256: sha256_hex(&entry.bytes),
        provider: provider.clone(),
        reuse: reuse.clone(),
        provenance: provenance.clone(),
        jats: jats.and_then(article_asset_jats),
        discovery_routes: vec![ArticleAssetDiscoveryRoute {
            provider: provider.clone(),
            source_document: ArticleAssetSourceDocument::PmcOaArchive,
        }],
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
    let doc = parse_external_xml(xml, ARTICLE_XML_NODE_LIMIT).ok()?;
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
    use crate::entities::article::test_support::{
        TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
    };
    use crate::test_support::TempDirGuard;

    fn sample_article() -> Article {
        Article {
            section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
                crate::entities::article::ARTICLE_OUTCOME_KEYS,
            ),
            pmid: Some("22663011".to_string()),
            pmcid: Some("PMC123456".to_string()),
            doi: None,
            title: "Fixture article".to_string(),
            authors: Vec::new(),
            author_count: 0,
            author_completeness: crate::entities::article::ArticleAuthorCompleteness::Unavailable,
            author_source: crate::entities::article::ArticleSource::PubTator,
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
            full_text_coverage: None,
            not_included: None,
            europepmc_license: None,
            europepmc_retracted: None,
            annotations: None,
            indexing: None,
            semantic_scholar: None,
            pubtator_fallback: false,
        }
    }

    #[test]
    fn build_manifest_hashes_binary_bytes_and_quotes_retrieval_commands() {
        let binary = vec![0, 0xff, b'P', b'N', b'G', b'\n'];
        let csv = b"time,value\n0,1\n".to_vec();
        let jats_xml = br#"<?xml version="1.0"?>
<!DOCTYPE article PUBLIC
  "-//NLM//DTD JATS Journal Archiving DTD v1.4 20241031//EN"
  "https://example.invalid/JATS-archivearticle1-4.dtd">
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

    #[test]
    fn final_source_classification_distinguishes_absence_from_failure() {
        assert!(matches!(
            final_asset_source_error("22663011", false),
            BioMcpError::NotFound { .. }
        ));
        assert!(matches!(
            final_asset_source_error("22663011", true),
            BioMcpError::SourceUnavailable { .. }
        ));
    }

    #[serial_test::serial(article_resolver_env)]
    #[tokio::test]
    async fn manifest_and_bytes_preserve_induced_archive_failure_after_figshare_miss() {
        let fixture = TestHttpFixture::spawn(|request| {
            let (status, content_type, body) = if request.starts_with(
                "GET /publications/export/biocjson?",
            ) {
                (
                    "200 OK",
                    "application/json",
                    r#"{"PubTator3":[{"pmid":4242,"pmcid":"PMC4242","authors":[],"passages":[{"infons":{"type":"title"},"text":"Asset failure fixture"},{"infons":{"type":"abstract"},"text":"Fixture abstract"}]}]}"#,
                )
            } else if request.starts_with("GET /?id=PMC4242") {
                ("200 OK", "application/xml", "<records>")
            } else if request.starts_with("GET /PMC4242/supplementaryFiles") {
                ("404 Not Found", "text/plain", "not found")
            } else if request.starts_with("GET /graph/v1/paper/PMID:4242") {
                (
                    "200 OK",
                    "application/json",
                    r#"{"paperId":"paper-4242","title":"Asset failure fixture","openAccessPdf":{"url":"https://aacr.figshare.com/articles/dataset/Fixture/4242"}}"#,
                )
            } else if request.starts_with("GET /v2/articles/4242") {
                (
                    "200 OK",
                    "application/json",
                    r#"{"id":4242,"title":"Asset failure fixture","files":[]}"#,
                )
            } else if request.starts_with("POST /v2/articles/search") {
                ("200 OK", "application/json", "[]")
            } else {
                ("404 Not Found", "application/json", r#"{"error":"not found"}"#)
            };
            TestHttpReply::Bytes(test_http_response(status, content_type, body.as_bytes()))
        })
        .await;
        let mut env = TestEnv::new();
        let cache = TempDirGuard::new("article-asset-failure-fold");
        for key in [
            "BIOMCP_TEST_UNPACED_ORIGIN",
            "BIOMCP_PUBTATOR_BASE",
            "BIOMCP_EUROPEPMC_BASE",
            "BIOMCP_PMC_OA_BASE",
            "BIOMCP_S2_BASE",
            "BIOMCP_FIGSHARE_BASE",
        ] {
            env.set(key, &fixture.base);
        }
        env.set("BIOMCP_CACHE_DIR", cache.path());

        let manifest_err = article_assets_manifest("4242")
            .await
            .expect_err("failed archive plus missing Figshare manifest must be unavailable");
        assert!(matches!(
            manifest_err,
            BioMcpError::SourceUnavailable { .. }
        ));

        let bytes_err = article_asset_bytes("4242", "missing.csv")
            .await
            .expect_err("failed archive plus missing Figshare file must be unavailable");
        assert!(matches!(bytes_err, BioMcpError::SourceUnavailable { .. }));
    }

    #[test]
    fn europe_pmc_manifest_retains_pmc_license_source_and_exact_member_facts() {
        let bytes = b"exact supplementary bytes".to_vec();
        let package = EuropePmcSupplementaryPackage {
            entries: vec![EuropePmcSupplementaryEntry {
                filename: "supplement.docx".to_string(),
                bytes: bytes.clone(),
            }],
        };
        let pmc_manifest = PmcOaArchiveManifest {
            package_url: "https://example.test/stale.tgz".to_string(),
            tgz_url: "https://example.test/stale.tgz".to_string(),
            license: Some("CC BY".to_string()),
            retracted: Some(false),
        };

        let manifest = build_europe_pmc_manifest(
            "22663011",
            &sample_article(),
            "PMC123456",
            package,
            Some(&pmc_manifest),
        );
        assert_eq!(manifest.provider.source, EUROPE_PMC_PROVIDER_SOURCE);
        let asset = manifest.assets.first().unwrap();
        assert_eq!(asset.size_bytes, bytes.len());
        assert_eq!(asset.sha256, sha256_hex(&bytes));
        assert_eq!(asset.reuse.license.as_deref(), Some("CC BY"));
        assert_eq!(
            asset
                .reuse
                .license_source
                .as_ref()
                .map(|source| source.source.as_str()),
            Some(PMC_PROVIDER_SOURCE)
        );
    }

    fn resolved_asset(
        identity: &str,
        filename: &str,
        bytes: &[u8],
        precedence: u8,
        source_document: ArticleAssetSourceDocument,
    ) -> ResolvedAsset {
        let provider = ArticleFulltextProvider {
            label: format!("provider-{precedence}"),
            source: format!("source-{precedence}"),
        };
        ResolvedAsset {
            canonical_identity: identity.to_string(),
            precedence,
            entry: ArticleAssetEntry {
                filename: filename.to_string(),
                asset_key: String::new(),
                kind: "supplementary-file".to_string(),
                media_type: None,
                size_bytes: bytes.len(),
                sha256: sha256_hex(bytes),
                provider: provider.clone(),
                reuse: ArticleFulltextReuse {
                    license_present: false,
                    license: None,
                    license_source: None,
                    reuse_warning: None,
                },
                provenance: ArticleFulltextProvenance::default(),
                jats: None,
                discovery_routes: vec![ArticleAssetDiscoveryRoute {
                    provider,
                    source_document,
                }],
                handle: String::new(),
            },
            bytes: bytes.to_vec(),
        }
    }

    #[test]
    fn malformed_jats_is_a_source_failure_instead_of_silent_absence() {
        let mut candidates = Vec::new();
        let route = jats_route(ArticleFulltextProvider {
            label: "test XML".to_string(),
            source: "test".to_string(),
        });
        assert!(append_jats_candidates(&mut candidates, "<article>", route).is_err());
        assert!(candidates.is_empty());
    }

    #[test]
    fn linked_budgets_are_deterministic_at_the_exact_boundaries() {
        assert!(linked_aggregate_accepts(0, LINKED_AGGREGATE_LIMIT));
        assert!(!linked_aggregate_accepts(1, LINKED_AGGREGATE_LIMIT));

        let mut candidates = vec![0, 1, 2, 3];
        let overflow = split_candidate_overflow(&mut candidates, 2);
        assert_eq!(candidates, vec![0, 1]);
        assert_eq!(overflow, vec![2, 3]);
    }

    #[test]
    fn identity_then_hash_merge_keeps_primary_bytes_and_all_routes() {
        let bytes = b"same supplement";
        let merged = merge_resolved_assets(vec![
            resolved_asset(
                "figshare:1:2",
                "copy.csv",
                bytes,
                3,
                ArticleAssetSourceDocument::Figshare,
            ),
            resolved_asset(
                "pmc:PMC1:copy.csv",
                "copy.csv",
                bytes,
                0,
                ArticleAssetSourceDocument::PmcOaArchive,
            ),
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].precedence, 0);
        assert_eq!(merged[0].bytes, bytes);
        assert_eq!(merged[0].entry.discovery_routes.len(), 2);
    }

    #[test]
    fn collision_keys_are_stable_while_unique_names_keep_legacy_handles() {
        let mut assets = vec![
            resolved_asset(
                "pmc:PMC1:same.csv",
                "same.csv",
                b"one",
                0,
                ArticleAssetSourceDocument::PmcOaArchive,
            ),
            resolved_asset(
                "figshare:2:3",
                "same.csv",
                b"two",
                3,
                ArticleAssetSourceDocument::Figshare,
            ),
            resolved_asset(
                "pmc:PMC1:unique.xlsx",
                "unique.xlsx",
                b"three",
                0,
                ArticleAssetSourceDocument::PmcOaArchive,
            ),
        ];
        assign_asset_keys("42", &mut assets);

        assert_eq!(assets[2].entry.asset_key, "unique.xlsx");
        assert_eq!(
            assets[2].entry.handle,
            "biomcp get article 42 asset unique.xlsx"
        );
        for asset in &assets[..2] {
            assert!(asset.entry.asset_key.ends_with("--same.csv"));
            assert!(asset.entry.handle.ends_with(&asset.entry.asset_key));
        }
        assert_ne!(assets[0].entry.asset_key, assets[1].entry.asset_key);
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
