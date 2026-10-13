//! One place that decides which upstream sources are disabled.
//!
//! `--disable-sources` and `BIOMCP_DISABLE_SOURCES` take a comma- or
//! pipe-separated list of source ids and group names. The inventory and
//! each group's membership come from `docs/reference/sources.json`
//! (embedded below), so a source never appears in a list here: when the
//! recorded terms change, the groups redraw themselves from the file.
//! `BIOMCP_DISABLE_KEGG=1` remains a working alias for `kegg`.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::error::{BioMcpError, DISABLED_SOURCE_REASON_PREFIX};

pub(crate) const DISABLE_SOURCES_ENV: &str = "BIOMCP_DISABLE_SOURCES";
pub(crate) const DISABLE_KEGG_ENV: &str = "BIOMCP_DISABLE_KEGG";

/// The recorded commercial-terms shape of a source. The two shapes name
/// their own groups, and `commercial-restricted` is their union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CommercialTerms {
    LicenseRequired,
    NonCommercial,
}

impl CommercialTerms {
    fn description(self) -> &'static str {
        match self {
            Self::LicenseRequired => {
                "the recorded terms require a commercial license for commercial use"
            }
            Self::NonCommercial => "the recorded terms allow non-commercial use only",
        }
    }
}

/// A group name resolves to a recorded commercial-terms shape; `None`
/// means every source that records a shape (the union group).
const GROUPS: &[(&str, Option<CommercialTerms>)] = &[
    ("commercial-restricted", None),
    ("license-required", Some(CommercialTerms::LicenseRequired)),
    ("non-commercial", Some(CommercialTerms::NonCommercial)),
];

const SOURCES_JSON: &str = include_str!("../../docs/reference/sources.json");

#[derive(Debug, Deserialize)]
struct SourceRecord {
    id: String,
    name: String,
    #[serde(default)]
    commercial_terms: Option<CommercialTerms>,
}

#[derive(Debug)]
pub(crate) struct Registry {
    records: Vec<SourceRecord>,
}

impl Registry {
    fn load() -> Self {
        let records: Vec<SourceRecord> = serde_json::from_str(SOURCES_JSON)
            .expect("embedded docs/reference/sources.json stays schema-conformant");
        Self { records }
    }

    pub(crate) fn get() -> &'static Self {
        static REGISTRY: OnceLock<Registry> = OnceLock::new();
        REGISTRY.get_or_init(Self::load)
    }

    fn find_id(&self, token: &str) -> Option<&SourceRecord> {
        self.records
            .iter()
            .find(|record| record.id.eq_ignore_ascii_case(token))
    }

    /// Maps a health-catalog display name back to its inventory id, so
    /// health rows ask the registry instead of carrying their own copy of
    /// the name list.
    pub(crate) fn id_for_name(&self, name: &str) -> Option<&str> {
        self.records
            .iter()
            .find(|record| record.name.eq_ignore_ascii_case(name))
            .map(|record| record.id.as_str())
    }

    /// The inventory display name for one id, for `biomcp health` rows.
    pub(crate) fn display_name(&self, id: &str) -> Option<&str> {
        self.records
            .iter()
            .find(|record| record.id == id)
            .map(|record| record.name.as_str())
    }

    fn find_group(&self, token: &str) -> Option<(&'static str, Option<CommercialTerms>)> {
        GROUPS
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(token))
            .map(|(name, shape)| (*name, *shape))
    }

    fn ids_for_group(&self, shape: Option<CommercialTerms>) -> Vec<&str> {
        self.records
            .iter()
            .filter(|record| match shape {
                Some(expected) => record.commercial_terms == Some(expected),
                None => record.commercial_terms.is_some(),
            })
            .map(|record| record.id.as_str())
            .collect()
    }

    fn id_list(&self) -> String {
        self.records
            .iter()
            .map(|record| record.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn group_list(&self) -> String {
        GROUPS
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// One disabled source with the setting that turned it off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DisabledEntry {
    pub(crate) id: String,
    pub(crate) reason: String,
}

/// The resolved set of disabled sources for one process.
#[derive(Debug, Default)]
pub(crate) struct DisabledSources {
    entries: BTreeMap<String, String>,
}

impl DisabledSources {
    #[cfg(test)]
    pub(crate) fn is_disabled(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    #[cfg(test)]
    pub(crate) fn entries(&self) -> Vec<DisabledEntry> {
        self.entries
            .iter()
            .map(|(id, reason)| DisabledEntry {
                id: id.clone(),
                reason: reason.clone(),
            })
            .collect()
    }
}

fn unknown_token_error(token: &str) -> BioMcpError {
    let registry = Registry::get();
    BioMcpError::InvalidArgument(format!(
        "Unknown source or group \"{token}\" for --disable-sources. \
Valid sources: {}. Valid groups: {}.",
        registry.id_list(),
        registry.group_list()
    ))
}

/// Parses one `--disable-sources`/`BIOMCP_DISABLE_SOURCES` value: comma- or
/// pipe-separated, case-insensitive, whitespace trimmed.
pub(crate) fn parse_disable_spec(raw: &str, origin: &str) -> Result<DisabledSources, BioMcpError> {
    let registry = Registry::get();
    let mut entries: BTreeMap<String, String> = BTreeMap::new();
    for token in raw.split([',', '|']) {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        if let Some(record) = registry.find_id(&token.to_ascii_lowercase()) {
            entries.insert(record.id.clone(), format!("disabled by {origin}={token}"));
        } else if let Some((group, shape)) = registry.find_group(&token.to_ascii_lowercase()) {
            let reason = match shape {
                Some(shape) => {
                    format!("disabled by the {group} group ({})", shape.description())
                }
                None => "disabled by the commercial-restricted group (recorded commercial terms restrict commercial use)"
                    .to_string(),
            };
            for id in registry.ids_for_group(shape) {
                entries
                    .entry(id.to_string())
                    .or_insert_with(|| reason.clone());
            }
        } else {
            return Err(unknown_token_error(token));
        }
    }
    Ok(DisabledSources { entries })
}

fn kegg_alias_from_env_value(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}

fn merge(entries: &mut BTreeMap<String, String>, extra: DisabledSources) {
    for (id, reason) in extra.entries {
        entries.entry(id).or_insert(reason);
    }
}

fn resolve_from_env() -> Result<DisabledSources, BioMcpError> {
    let mut entries: BTreeMap<String, String> = BTreeMap::new();
    if let Ok(raw) = std::env::var(DISABLE_SOURCES_ENV) {
        merge(&mut entries, parse_disable_spec(&raw, DISABLE_SOURCES_ENV)?);
    }
    if kegg_alias_from_env_value(std::env::var(DISABLE_KEGG_ENV).ok().as_deref()) {
        entries.insert(
            "kegg".to_string(),
            format!("disabled by {DISABLE_KEGG_ENV}=1"),
        );
    }
    Ok(DisabledSources { entries })
}

#[derive(Debug)]
struct State {
    entries: BTreeMap<String, String>,
}

static STATE: OnceLock<State> = OnceLock::new();

/// Resolves the disabled-source state for this process: the environment
/// variable and the `BIOMCP_DISABLE_KEGG` alias, plus the one-shot CLI
/// flag. Entry points that can fail validate the spec here, so an
/// unknown name stops the process before any source is queried.
pub(crate) fn init(flag: Option<&str>) -> Result<(), BioMcpError> {
    let mut entries = resolve_from_env()?.entries;
    if let Some(raw) = flag {
        merge(&mut entries, parse_disable_spec(raw, "--disable-sources")?);
    }
    if let Some(state) = STATE.get() {
        if state.entries == entries {
            return Ok(());
        }
        return Err(BioMcpError::InvalidArgument(format!(
            "Disabled sources were already resolved for this process by {DISABLE_SOURCES_ENV} \
or {DISABLE_KEGG_ENV}; pass --disable-sources only to a one-shot CLI invocation"
        )));
    }
    let _ = STATE.set(State { entries });
    Ok(())
}

fn state() -> &'static State {
    STATE.get_or_init(|| {
        // Entry points validate the spec; a library or test caller that
        // skips init resolves leniently so an invalid value cannot wedge
        // a mid-flight command.
        let entries = match resolve_from_env() {
            Ok(disabled) => disabled.entries,
            Err(err) => {
                tracing::warn!("ignoring invalid disabled-sources setting: {err}");
                BTreeMap::new()
            }
        };
        State { entries }
    })
}

/// Asks whether one source id is disabled. This is the one place every
/// caller consults.
pub(crate) fn is_disabled(id: &str) -> bool {
    state().entries.contains_key(id)
}

/// The recorded reason one source is off, so error paths can name the
/// setting that disabled it.
pub(crate) fn reason(id: &str) -> Option<&'static str> {
    state().entries.get(id).map(String::as_str)
}

// Every reason this module records starts with the shared prefix in
// `crate::error::DISABLED_SOURCE_REASON_PREFIX`, so error projections can
// tell a disabled source from a down one.

/// The error a direct caller returns when its source is disabled: the
/// reason records the setting so downstream surfaces can repeat it.
pub(crate) fn source_disabled_error(source_name: &str, id: &str) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: source_name.to_string(),
        reason: reason(id)
            .unwrap_or("disabled by --disable-sources")
            .to_string(),
        suggestion: format!(
            "Remove {id} from --disable-sources/BIOMCP_DISABLE_SOURCES to query {source_name}."
        ),
    }
}

/// Extracts the disable reason from a `SourceUnavailable` error built by
/// [`source_disabled_error`], so section outcomes can say why a source
/// never answered.
pub(crate) fn unavailable_disabled_reason(err: &BioMcpError) -> Option<&str> {
    match err.underlying() {
        BioMcpError::SourceUnavailable { reason, .. }
            if reason.starts_with(DISABLED_SOURCE_REASON_PREFIX) =>
        {
            Some(reason)
        }
        _ => None,
    }
}

/// The disabled sources with the reason each is off, for `biomcp health`
/// and the MCP server description.
pub(crate) fn disabled_entries() -> Vec<DisabledEntry> {
    state()
        .entries
        .iter()
        .map(|(id, reason)| DisabledEntry {
            id: id.clone(),
            reason: reason.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_message(raw: &str) -> String {
        parse_disable_spec(raw, "--disable-sources")
            .expect_err("unknown token must fail")
            .to_string()
    }

    #[test]
    fn parse_splits_comma_and_pipe_and_trims_and_lowercases() {
        let parsed = parse_disable_spec(" KEGG | enrichr ,\tmygene-info", "--disable-sources")
            .expect("mixed separators parse");
        assert!(parsed.is_disabled("kegg"));
        assert!(parsed.is_disabled("enrichr"));
        assert!(parsed.is_disabled("mygene-info"));
        assert!(!parsed.is_disabled("reactome"));
    }

    #[test]
    fn parse_records_the_origin_in_each_reason() {
        let parsed = parse_disable_spec("kegg", "BIOMCP_DISABLE_SOURCES").unwrap();
        let entries = parsed.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "kegg");
        assert_eq!(entries[0].reason, "disabled by BIOMCP_DISABLE_SOURCES=kegg");
    }

    #[test]
    fn unknown_token_error_lists_valid_sources_and_groups() {
        let message = err_message("nope");
        assert!(message.contains("\"nope\""), "{message}");
        assert!(message.contains("Valid sources: alphafold-db"), "{message}");
        assert!(message.contains(", kegg,"), "{message}");
        assert!(message.contains("enrichr"), "{message}");
        assert!(message.contains("drugbank"), "{message}");
        assert!(
            message
                .contains("Valid groups: commercial-restricted, license-required, non-commercial"),
            "{message}"
        );
    }

    #[test]
    fn empty_tokens_are_skipped_so_blank_values_disable_nothing() {
        for raw in ["", " , ", "||"] {
            let parsed = parse_disable_spec(raw, "--disable-sources").unwrap();
            assert!(parsed.entries().is_empty(), "{raw:?}");
        }
    }

    #[test]
    fn license_required_group_covers_the_four_recorded_sources() {
        let parsed = parse_disable_spec("license-required", "--disable-sources").unwrap();
        let entries = parsed.entries();
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["drugbank", "enrichr", "kegg", "oncokb"]);
        assert!(entries.iter().all(|entry| {
            entry
                .reason
                .contains("license-required group (the recorded terms require a commercial license")
        }));
    }

    #[test]
    fn non_commercial_group_covers_the_two_recorded_sources() {
        let parsed = parse_disable_spec("non-commercial", "--disable-sources").unwrap();
        let entries = parsed.entries();
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["ddinter", "pharmacodb"]);
    }

    #[test]
    fn commercial_restricted_group_is_the_union_of_both_shapes() {
        let union = parse_disable_spec("commercial-restricted", "--disable-sources").unwrap();
        let license = parse_disable_spec("license-required", "--disable-sources").unwrap();
        let non_commercial = parse_disable_spec("non-commercial", "--disable-sources").unwrap();

        let mut member_ids: Vec<String> = license
            .entries()
            .into_iter()
            .chain(non_commercial.entries())
            .map(|entry| entry.id)
            .collect();
        member_ids.sort();
        assert_eq!(member_ids.len(), 6, "union covers both shapes");
        for id in &member_ids {
            assert!(union.is_disabled(id), "{id} must join the union group");
        }
        assert!(!union.is_disabled("reactome"));
        assert!(!union.is_disabled("cosmic"));
    }

    #[test]
    fn registry_ids_match_the_embedded_inventory() {
        let registry = Registry::get();
        assert!(registry.find_id("kegg").is_some());
        assert!(registry.find_id("NOPE").is_none());
        assert!(registry.id_for_name("KEGG").is_some_and(|id| id == "kegg"));
        assert!(
            registry
                .id_for_name("Enrichr")
                .is_some_and(|id| id == "enrichr")
        );
        assert!(
            registry
                .id_for_name("DDInter")
                .is_some_and(|id| id == "ddinter")
        );
        assert!(registry.ids_for_group(None).len() == 6);
    }

    #[test]
    fn kegg_alias_keeps_its_historical_value_set() {
        assert!(kegg_alias_from_env_value(Some("1")));
        assert!(kegg_alias_from_env_value(Some("true")));
        assert!(kegg_alias_from_env_value(Some("TRUE")));
        assert!(kegg_alias_from_env_value(Some("yes")));
        assert!(kegg_alias_from_env_value(Some("YES")));
        assert!(!kegg_alias_from_env_value(Some("0")));
        assert!(!kegg_alias_from_env_value(Some("")));
        assert!(!kegg_alias_from_env_value(None));
    }
}
