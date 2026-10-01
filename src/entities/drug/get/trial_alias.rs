//! Trial aliases preserve source priority, cache policy and terminal failures.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrialAliasSource {
    Requested,
    Canonical,
    OpenFdaBrand,
    DrugBankSynonym,
}

impl TrialAliasSource {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Canonical => "canonical",
            Self::OpenFdaBrand => "openfda_brand",
            Self::DrugBankSynonym => "drugbank_synonym",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrialAlias {
    pub(crate) label: String,
    pub(crate) source: TrialAliasSource,
}

#[derive(Clone)]
pub(super) struct TrialAliasResolution {
    pub(super) canonical_name: String,
    pub(super) aliases: Vec<TrialAlias>,
}

pub(super) struct TrialAliasLookup {
    pub(super) canonical_name: String,
    pub(super) candidates: Vec<TrialAlias>,
}

static TRIAL_ALIAS_CACHE: OnceLock<Mutex<HashMap<String, TrialAliasResolution>>> = OnceLock::new();

pub(super) fn trial_alias_cache() -> &'static Mutex<HashMap<String, TrialAliasResolution>> {
    TRIAL_ALIAS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn trial_alias_cache_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

pub(super) fn is_investigational_code(alias: &str) -> bool {
    static CODE_RE: OnceLock<Regex> = OnceLock::new();
    CODE_RE
        .get_or_init(|| {
            Regex::new(r"(?i)^[a-z]+[ -]\d{2}[a-z0-9-]*$")
                .expect("valid investigational code regex")
        })
        .is_match(alias)
}

pub(super) fn has_free_base_descriptor(alias: &str) -> bool {
    static FREE_BASE_RE: OnceLock<Regex> = OnceLock::new();
    FREE_BASE_RE
        .get_or_init(|| {
            Regex::new(r"(?i)\bfree(?:\s+|-+)base\b").expect("valid free-base descriptor regex")
        })
        .is_match(alias)
}

pub(super) fn is_simple_trial_name(alias: &str) -> bool {
    alias.chars().count() <= 64
        && alias.split_whitespace().count() <= 4
        && alias
            .chars()
            .all(|ch| ch.is_alphanumeric() || ch.is_whitespace() || matches!(ch, '\'' | '-'))
}

pub(super) fn eligible_drugbank_trial_alias(alias: &str) -> bool {
    !has_free_base_descriptor(alias)
        && (is_investigational_code(alias) || is_simple_trial_name(alias))
}

pub(super) fn push_trial_alias(
    aliases: &mut Vec<TrialAlias>,
    seen: &mut HashSet<String>,
    alias: &str,
    source: TrialAliasSource,
) {
    let alias = alias.trim();
    if alias.is_empty() {
        return;
    }
    if seen.insert(alias.to_ascii_lowercase()) {
        aliases.push(TrialAlias {
            label: alias.to_string(),
            source,
        });
    }
}

pub(super) fn build_trial_aliases(
    requested_name: &str,
    canonical_name: Option<&str>,
    candidates: &[TrialAlias],
) -> Vec<TrialAlias> {
    let mut aliases = Vec::new();
    let mut seen = HashSet::new();

    push_trial_alias(
        &mut aliases,
        &mut seen,
        requested_name,
        TrialAliasSource::Requested,
    );
    if let Some(canonical_name) = canonical_name {
        push_trial_alias(
            &mut aliases,
            &mut seen,
            canonical_name,
            TrialAliasSource::Canonical,
        );
    }

    let mut provider_aliases = 0;
    for source in [
        TrialAliasSource::OpenFdaBrand,
        TrialAliasSource::DrugBankSynonym,
    ] {
        let mut source_candidates = candidates
            .iter()
            .filter(|candidate| candidate.source == source)
            .filter(|candidate| {
                source == TrialAliasSource::OpenFdaBrand
                    || eligible_drugbank_trial_alias(candidate.label.trim())
            })
            .collect::<Vec<_>>();
        source_candidates.sort_by(|left, right| {
            left.label
                .trim()
                .to_ascii_lowercase()
                .cmp(&right.label.trim().to_ascii_lowercase())
                .then_with(|| left.label.trim().cmp(right.label.trim()))
        });
        for candidate in source_candidates {
            if provider_aliases >= 3 {
                break;
            }
            let previous_len = aliases.len();
            push_trial_alias(&mut aliases, &mut seen, &candidate.label, candidate.source);
            provider_aliases += usize::from(aliases.len() > previous_len);
        }
    }

    aliases
}

pub(super) fn trial_alias_candidates_from_hits(hits: &[&MyChemHit]) -> Vec<TrialAlias> {
    let mut candidates = Vec::new();
    for hit in hits {
        if let Some(openfda) = &hit.openfda {
            candidates.extend(
                openfda
                    .brand_name
                    .clone()
                    .into_vec()
                    .into_iter()
                    .map(|label| TrialAlias {
                        label,
                        source: TrialAliasSource::OpenFdaBrand,
                    }),
            );
        }
        if let Some(drugbank) = &hit.drugbank {
            candidates.extend(drugbank.synonyms.iter().cloned().map(|label| TrialAlias {
                label,
                source: TrialAliasSource::DrugBankSynonym,
            }));
        }
    }
    candidates
}

pub(super) fn trial_alias_resolution_from_lookup_result(
    requested_name: &str,
    result: Result<TrialAliasLookup, BioMcpError>,
) -> Result<(TrialAliasResolution, bool), BioMcpError> {
    match result {
        Ok(resolved) => Ok((
            TrialAliasResolution {
                canonical_name: resolved.canonical_name.clone(),
                aliases: build_trial_aliases(
                    requested_name,
                    Some(&resolved.canonical_name),
                    &resolved.candidates,
                ),
            },
            true,
        )),
        Err(error) if crate::sources::mychem::optional_failure(&error) => {
            let cacheable = error.is_not_found();
            Ok((
                TrialAliasResolution {
                    canonical_name: requested_name.to_string(),
                    aliases: vec![TrialAlias {
                        label: requested_name.to_string(),
                        source: TrialAliasSource::Requested,
                    }],
                },
                cacheable,
            ))
        }
        Err(error) => Err(error),
    }
}

pub(super) async fn optional_lookup(
    name: &str,
) -> Result<Option<crate::sources::mychem::MyChemQueryResponse>, BioMcpError> {
    match direct_drug_lookup(name).await {
        Ok(response) => Ok(Some(response)),
        Err(error) if crate::sources::mychem::optional_failure(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

pub(super) async fn resolve_trial_alias_resolution(
    name: &str,
) -> Result<TrialAliasResolution, BioMcpError> {
    resolve_trial_alias_resolution_with_lookup(name, async {
        resolve_drug_base(name.trim(), false, false)
            .await
            .map(|resolved| TrialAliasLookup {
                canonical_name: resolved.drug.name,
                candidates: resolved.trial_alias_candidates,
            })
    })
    .await
}
pub(super) async fn resolve_trial_alias_resolution_with_lookup(
    name: &str,
    lookup: impl std::future::Future<Output = Result<TrialAliasLookup, BioMcpError>>,
) -> Result<TrialAliasResolution, BioMcpError> {
    let requested_name = name.trim();
    if requested_name.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Trial intervention alias expansion requires a non-empty drug name".into(),
        ));
    }

    let cache_key = trial_alias_cache_key(requested_name);
    let cached_resolution = {
        // Scoped: the recovered guard must not live across the
        // resolution await below.
        let cache = crate::utils::sync::recover_poison(trial_alias_cache().lock());
        cache.get(&cache_key).cloned()
    };
    if let Some(mut resolution) = cached_resolution {
        if let Some(requested_alias) = resolution.aliases.first_mut() {
            requested_alias.label = requested_name.to_string();
        }
        return Ok(resolution);
    }

    let (resolution, cacheable) =
        trial_alias_resolution_from_lookup_result(requested_name, lookup.await)?;

    if cacheable {
        let mut cache = crate::utils::sync::recover_poison(trial_alias_cache().lock());
        cache.insert(cache_key, resolution.clone());
    }

    Ok(resolution)
}

pub(crate) async fn resolve_trial_aliases(name: &str) -> Result<Vec<String>, BioMcpError> {
    Ok(resolve_trial_alias_resolution(name)
        .await?
        .aliases
        .into_iter()
        .map(|alias| alias.label)
        .collect())
}

pub(crate) async fn resolve_trial_aliases_with_sources(
    name: &str,
) -> Result<Vec<TrialAlias>, BioMcpError> {
    Ok(resolve_trial_alias_resolution(name).await?.aliases)
}

pub(crate) async fn resolve_trial_canonical_name(name: &str) -> Result<String, BioMcpError> {
    Ok(resolve_trial_alias_resolution(name).await?.canonical_name)
}
