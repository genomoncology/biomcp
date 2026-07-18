use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use http_cache_reqwest::CacheMode;
use regex::Regex;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::entities::SearchPage;
use crate::entities::drug::{
    EmaDhpcEntry, EmaDrugSearchResult, EmaPsusaEntry, EmaReferralEntry, EmaRegulatoryActivity,
    EmaRegulatoryRow, EmaSafetyInfo, EmaShortageEntry,
};
use crate::error::BioMcpError;
use crate::utils::serde::StringOrVec;

const SOURCE_NAME: &str = "EMA";
const DOWNLOAD_URL: &str =
    "https://www.ema.europa.eu/en/about-us/about-website/download-website-data-json-data-format";
const EMA_API: &str = "ema";
const EMA_REPORT_BASE: &str = "https://www.ema.europa.eu/en/documents/report";
const EMA_REPORT_BASE_ENV: &str = "BIOMCP_EMA_REPORT_BASE";
const EMA_MAX_BODY_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const EMA_STALE_AFTER: Duration = Duration::from_secs(72 * 60 * 60);
const EMA_SIZE_HINT: &str = "~11 MB";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EmaFeed {
    local_name: &'static str,
    report_name: &'static str,
}

const EMA_FEEDS: [EmaFeed; 6] = [
    EmaFeed {
        local_name: "medicines.json",
        report_name: "medicines-output-medicines_json-report_en.json",
    },
    EmaFeed {
        local_name: "post_authorisation.json",
        report_name: "medicines-output-post_authorisation_json-report_en.json",
    },
    EmaFeed {
        local_name: "referrals.json",
        report_name: "referrals-output-json-report_en.json",
    },
    EmaFeed {
        local_name: "psusas.json",
        report_name: "medicines-output-periodic_safety_update_report_single_assessments-output-json-report_en.json",
    },
    EmaFeed {
        local_name: "dhpcs.json",
        report_name: "dhpc-output-json-report_en.json",
    },
    EmaFeed {
        local_name: "shortages.json",
        report_name: "shortages-output-json-report_en.json",
    },
];

const MEDICINES_FILE: &str = EMA_FEEDS[0].local_name;
const POST_AUTHORISATION_FILE: &str = EMA_FEEDS[1].local_name;
const REFERRALS_FILE: &str = EMA_FEEDS[2].local_name;
const PSUSAS_FILE: &str = EMA_FEEDS[3].local_name;
const DHPCS_FILE: &str = EMA_FEEDS[4].local_name;
const SHORTAGES_FILE: &str = EMA_FEEDS[5].local_name;

pub(crate) const EMA_REQUIRED_FILES: &[&str] = &[
    MEDICINES_FILE,
    POST_AUTHORISATION_FILE,
    REFERRALS_FILE,
    PSUSAS_FILE,
    DHPCS_FILE,
    SHORTAGES_FILE,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmaSyncMode {
    Auto,
    Force,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeedSyncState {
    Fresh,
    Missing,
    Stale,
}

#[derive(Debug, Clone, Copy)]
struct FeedSyncPlan {
    feed: EmaFeed,
    state: FeedSyncState,
    cache_mode: CacheMode,
}

#[derive(Debug, Clone)]
pub(crate) struct EmaDrugIdentity {
    terms: Vec<String>,
}

impl EmaDrugIdentity {
    pub(crate) fn new(primary: &str) -> Self {
        Self::from_terms(vec![primary.to_string()])
    }

    pub(crate) fn with_aliases(primary: &str, canonical: Option<&str>, aliases: &[String]) -> Self {
        let mut terms = vec![primary.to_string()];
        if let Some(canonical) = canonical {
            terms.push(canonical.to_string());
        }
        terms.extend(aliases.iter().cloned());
        Self::from_terms(terms)
    }

    fn from_terms(terms: Vec<String>) -> Self {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for term in terms {
            let Some(term) = normalize_term(&term) else {
                continue;
            };
            if seen.insert(term.clone()) {
                out.push(term);
            }
        }
        Self { terms: out }
    }

    fn term_set(&self) -> HashSet<String> {
        self.terms.iter().cloned().collect()
    }

    fn search_tokens(&self) -> HashSet<String> {
        let mut out = HashSet::new();
        for term in &self.terms {
            for token in term
                .split(|ch: char| !ch.is_ascii_alphanumeric())
                .filter(|token| token.len() >= 3)
            {
                out.insert(token.to_string());
            }
        }
        out
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EmaClient {
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct EmaAnchor {
    medicines: Vec<AnchorMedicine>,
    terms: HashSet<String>,
}

#[derive(Debug, Clone)]
struct AnchorMedicine {
    medicine_name: String,
    active_substance: String,
    ema_product_number: String,
    status: String,
    holder: Option<String>,
    marketing_authorisation_date: Option<String>,
    therapeutic_indication: Option<String>,
    match_rank: u8,
}

#[derive(Debug, Deserialize)]
struct EmaWrapper<T> {
    data: Vec<T>,
}

#[derive(Debug, Default, Deserialize)]
struct EmaMedicineRow {
    #[serde(default)]
    name_of_medicine: StringOrVec,
    #[serde(default)]
    active_substance: StringOrVec,
    #[serde(default)]
    ema_product_number: StringOrVec,
    #[serde(default)]
    medicine_status: StringOrVec,
    #[serde(default)]
    category: StringOrVec,
    #[serde(default, alias = "marketing_authorisation_holder_company_name")]
    marketing_authorisation_developer_applicant_holder: StringOrVec,
    #[serde(default)]
    marketing_authorisation_date: StringOrVec,
    #[serde(default, alias = "condition_indication")]
    therapeutic_indication: StringOrVec,
}

#[derive(Debug, Default, Deserialize)]
struct EmaPostAuthorisationRow {
    #[serde(default)]
    ema_product_number: StringOrVec,
    #[serde(default)]
    first_published_date: StringOrVec,
    #[serde(default)]
    last_updated_date: StringOrVec,
}

#[derive(Debug, Default, Deserialize)]
struct EmaDhpcRow {
    #[serde(default)]
    name_of_medicine: StringOrVec,
    #[serde(default)]
    dhpc_type: StringOrVec,
    #[serde(default)]
    regulatory_outcome: StringOrVec,
    #[serde(default)]
    first_published_date: StringOrVec,
    #[serde(default)]
    last_updated_date: StringOrVec,
}

#[derive(Debug, Default, Deserialize)]
struct EmaReferralRow {
    #[serde(default)]
    referral_name: StringOrVec,
    #[serde(default)]
    international_non_proprietary_name_inn_common_name: StringOrVec,
    #[serde(default)]
    associated_names_centrally_authorised_medicines: StringOrVec,
    #[serde(default)]
    current_status: StringOrVec,
    #[serde(default)]
    safety_referral: StringOrVec,
    #[serde(default)]
    referral_type: StringOrVec,
    #[serde(default)]
    procedure_start_date: StringOrVec,
    #[serde(default)]
    prac_recommendation: StringOrVec,
    #[serde(default)]
    category: StringOrVec,
}

#[derive(Debug, Default, Deserialize)]
struct EmaPsusaRow {
    #[serde(default)]
    related_medicines: StringOrVec,
    #[serde(default)]
    active_substance: StringOrVec,
    #[serde(default)]
    procedure_number: StringOrVec,
    #[serde(default)]
    regulatory_outcome: StringOrVec,
    #[serde(default)]
    first_published_date: StringOrVec,
    #[serde(default)]
    last_updated_date: StringOrVec,
}

#[derive(Debug, Default, Deserialize)]
struct EmaShortageRow {
    #[serde(default)]
    medicine_affected: StringOrVec,
    #[serde(default)]
    supply_shortage_status: StringOrVec,
    #[serde(default)]
    availability_of_alternatives: StringOrVec,
    #[serde(default)]
    first_published_date: StringOrVec,
    #[serde(default)]
    last_updated_date: StringOrVec,
}

impl EmaClient {
    #[cfg(test)]
    fn from_root(root: PathBuf) -> Self {
        Self { root }
    }

    pub(crate) async fn ready(mode: EmaSyncMode) -> Result<Self, BioMcpError> {
        let root = resolve_ema_root();
        sync_ema_root(&root, mode).await?;
        Ok(Self { root })
    }

    pub(crate) async fn sync(mode: EmaSyncMode) -> Result<(), BioMcpError> {
        let root = resolve_ema_root();
        sync_ema_root(&root, mode).await
    }

    pub(crate) fn resolve_anchor(
        &self,
        identity: &EmaDrugIdentity,
    ) -> Result<EmaAnchor, BioMcpError> {
        let medicines = self.read_feed::<EmaMedicineRow>(MEDICINES_FILE)?;
        let terms = identity.term_set();
        let mut out = Vec::new();
        let mut seen_products = HashSet::new();

        for row in medicines {
            if !is_human_category(&row.category) {
                continue;
            }

            let Some(medicine_name) = scalar_value(&row.name_of_medicine) else {
                continue;
            };
            let Some(active_substance) = scalar_value(&row.active_substance) else {
                continue;
            };
            let Some(ema_product_number) = scalar_value(&row.ema_product_number) else {
                continue;
            };

            let matches_name = field_matches_terms(&medicine_name, &terms);
            let matches_active = field_matches_terms(&active_substance, &terms);
            if !matches_name && !matches_active {
                continue;
            }

            let product_key = ema_product_number.to_ascii_lowercase();
            if !seen_products.insert(product_key) {
                continue;
            }

            out.push(AnchorMedicine {
                medicine_name,
                active_substance,
                ema_product_number,
                status: scalar_value(&row.medicine_status).unwrap_or_else(|| "Unknown".to_string()),
                holder: scalar_value(&row.marketing_authorisation_developer_applicant_holder),
                marketing_authorisation_date: scalar_value(&row.marketing_authorisation_date),
                therapeutic_indication: scalar_ema_text(&row.therapeutic_indication),
                match_rank: if matches_name { 0 } else { 1 },
            });
        }

        out.sort_by(|a, b| {
            a.match_rank
                .cmp(&b.match_rank)
                .then_with(|| a.medicine_name.cmp(&b.medicine_name))
                .then_with(|| a.ema_product_number.cmp(&b.ema_product_number))
        });

        let mut anchor_terms = terms;
        for medicine in &out {
            if let Some(term) = normalize_term(&medicine.medicine_name) {
                anchor_terms.insert(term);
            }
            if let Some(term) = normalize_term(&medicine.active_substance) {
                anchor_terms.insert(term);
            }
        }

        Ok(EmaAnchor {
            medicines: out,
            terms: anchor_terms,
        })
    }

    pub(crate) fn search_medicines(
        &self,
        identity: &EmaDrugIdentity,
        limit: usize,
        offset: usize,
    ) -> Result<SearchPage<EmaDrugSearchResult>, BioMcpError> {
        let medicines = self.read_feed::<EmaMedicineRow>(MEDICINES_FILE)?;
        let phrase_terms = identity.term_set();
        let token_terms = identity.search_tokens();
        let mut out = Vec::new();
        let mut seen_products = HashSet::new();

        for row in medicines {
            if !is_human_category(&row.category) {
                continue;
            }

            let Some(medicine_name) = scalar_value(&row.name_of_medicine) else {
                continue;
            };
            let Some(active_substance) = scalar_value(&row.active_substance) else {
                continue;
            };
            let Some(ema_product_number) = scalar_value(&row.ema_product_number) else {
                continue;
            };
            let therapeutic_indication = scalar_ema_text(&row.therapeutic_indication);

            let match_rank = if field_matches_terms(&medicine_name, &phrase_terms) {
                0
            } else if field_matches_terms(&active_substance, &phrase_terms) {
                1
            } else if therapeutic_indication
                .as_deref()
                .is_some_and(|value| field_matches_terms(value, &phrase_terms))
            {
                2
            } else if !token_terms.is_empty()
                && field_matches_terms(&active_substance, &token_terms)
            {
                3
            } else if !token_terms.is_empty()
                && therapeutic_indication
                    .as_deref()
                    .is_some_and(|value| field_matches_terms(value, &token_terms))
            {
                4
            } else {
                continue;
            };

            let product_key = ema_product_number.to_ascii_lowercase();
            if !seen_products.insert(product_key) {
                continue;
            }

            out.push(AnchorMedicine {
                medicine_name,
                active_substance,
                ema_product_number,
                status: scalar_value(&row.medicine_status).unwrap_or_else(|| "Unknown".to_string()),
                holder: scalar_value(&row.marketing_authorisation_developer_applicant_holder),
                marketing_authorisation_date: scalar_value(&row.marketing_authorisation_date),
                therapeutic_indication,
                match_rank,
            });
        }

        out.sort_by(|a, b| {
            a.match_rank
                .cmp(&b.match_rank)
                .then_with(|| a.medicine_name.cmp(&b.medicine_name))
                .then_with(|| a.ema_product_number.cmp(&b.ema_product_number))
        });

        let total = out.len();
        let results = out
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|medicine| EmaDrugSearchResult {
                name: medicine.medicine_name,
                active_substance: medicine.active_substance,
                ema_product_number: medicine.ema_product_number,
                status: medicine.status,
            })
            .collect::<Vec<_>>();
        Ok(SearchPage::offset(results, Some(total)))
    }

    pub(crate) fn regulatory(
        &self,
        anchor: &EmaAnchor,
    ) -> Result<Vec<EmaRegulatoryRow>, BioMcpError> {
        self.require_files(&[MEDICINES_FILE, POST_AUTHORISATION_FILE])?;
        if anchor.medicines.is_empty() {
            return Ok(Vec::new());
        }

        let post_rows = self.read_feed::<EmaPostAuthorisationRow>(POST_AUTHORISATION_FILE)?;
        let mut out = Vec::new();
        for medicine in &anchor.medicines {
            let mut recent_activity = post_rows
                .iter()
                .filter(|row| {
                    scalar_value(&row.ema_product_number).is_some_and(|value| {
                        value.eq_ignore_ascii_case(&medicine.ema_product_number)
                    })
                })
                .filter_map(|row| {
                    let first_published_date = scalar_value(&row.first_published_date)?;
                    Some(EmaRegulatoryActivity {
                        first_published_date,
                        last_updated_date: scalar_value(&row.last_updated_date),
                    })
                })
                .collect::<Vec<_>>();

            recent_activity.sort_by(|a, b| {
                cmp_date_desc(
                    Some(a.first_published_date.as_str()),
                    Some(b.first_published_date.as_str()),
                )
            });
            recent_activity.truncate(5);

            out.push(EmaRegulatoryRow {
                medicine_name: medicine.medicine_name.clone(),
                active_substance: medicine.active_substance.clone(),
                ema_product_number: medicine.ema_product_number.clone(),
                status: medicine.status.clone(),
                holder: medicine.holder.clone(),
                marketing_authorisation_date: medicine.marketing_authorisation_date.clone(),
                therapeutic_indication: medicine.therapeutic_indication.clone(),
                recent_activity,
            });
        }

        Ok(out)
    }

    pub(crate) fn safety(&self, anchor: &EmaAnchor) -> Result<EmaSafetyInfo, BioMcpError> {
        self.require_files(&[DHPCS_FILE, REFERRALS_FILE, PSUSAS_FILE])?;
        if anchor.medicines.is_empty() {
            return Ok(EmaSafetyInfo::default());
        }

        let dhpcs = self
            .read_feed::<EmaDhpcRow>(DHPCS_FILE)?
            .into_iter()
            .filter_map(|row| {
                let medicine_name = scalar_value(&row.name_of_medicine)?;
                if !field_matches_terms(&medicine_name, &anchor.terms) {
                    return None;
                }
                Some(EmaDhpcEntry {
                    medicine_name,
                    dhpc_type: scalar_value(&row.dhpc_type),
                    regulatory_outcome: scalar_value(&row.regulatory_outcome),
                    first_published_date: scalar_value(&row.first_published_date),
                    last_updated_date: scalar_value(&row.last_updated_date),
                })
            })
            .collect::<Vec<_>>();

        let mut referrals = self
            .read_feed::<EmaReferralRow>(REFERRALS_FILE)?
            .into_iter()
            .filter(|row| is_human_category(&row.category))
            .filter_map(|row| {
                let referral_name = scalar_value(&row.referral_name)?;
                let active_substance =
                    scalar_value(&row.international_non_proprietary_name_inn_common_name);
                let associated_medicines =
                    scalar_value(&row.associated_names_centrally_authorised_medicines);
                let matched = field_matches_terms(&referral_name, &anchor.terms)
                    || active_substance
                        .as_deref()
                        .is_some_and(|value| field_matches_terms(value, &anchor.terms))
                    || associated_medicines
                        .as_deref()
                        .is_some_and(|value| field_matches_terms(value, &anchor.terms));
                if !matched {
                    return None;
                }
                Some(EmaReferralEntry {
                    referral_name,
                    active_substance,
                    associated_medicines,
                    current_status: scalar_value(&row.current_status),
                    safety_referral: scalar_value(&row.safety_referral),
                    referral_type: scalar_value(&row.referral_type),
                    procedure_start_date: scalar_value(&row.procedure_start_date),
                    prac_recommendation: scalar_value(&row.prac_recommendation),
                })
            })
            .collect::<Vec<_>>();

        let mut psusas = self
            .read_feed::<EmaPsusaRow>(PSUSAS_FILE)?
            .into_iter()
            .filter_map(|row| {
                let related_medicines = scalar_value(&row.related_medicines);
                let active_substance = scalar_value(&row.active_substance);
                let matched = related_medicines
                    .as_deref()
                    .is_some_and(|value| field_matches_terms(value, &anchor.terms))
                    || active_substance
                        .as_deref()
                        .is_some_and(|value| field_matches_terms(value, &anchor.terms));
                if !matched {
                    return None;
                }
                Some(EmaPsusaEntry {
                    related_medicines,
                    active_substance,
                    procedure_number: scalar_value(&row.procedure_number),
                    regulatory_outcome: scalar_value(&row.regulatory_outcome),
                    first_published_date: scalar_value(&row.first_published_date),
                    last_updated_date: scalar_value(&row.last_updated_date),
                })
            })
            .collect::<Vec<_>>();

        let mut dhpcs = dhpcs;
        dhpcs.sort_by(|a, b| {
            cmp_date_desc(
                a.first_published_date.as_deref(),
                b.first_published_date.as_deref(),
            )
        });
        referrals.sort_by(|a, b| {
            cmp_date_desc(
                a.procedure_start_date.as_deref(),
                b.procedure_start_date.as_deref(),
            )
        });
        psusas.sort_by(|a, b| {
            cmp_date_desc(
                a.first_published_date.as_deref(),
                b.first_published_date.as_deref(),
            )
        });

        Ok(EmaSafetyInfo {
            dhpcs,
            referrals,
            psusas,
        })
    }

    pub(crate) fn shortages(
        &self,
        anchor: &EmaAnchor,
    ) -> Result<Vec<EmaShortageEntry>, BioMcpError> {
        self.require_files(&[MEDICINES_FILE, SHORTAGES_FILE])?;
        if anchor.medicines.is_empty() {
            return Ok(Vec::new());
        }

        let mut out = self
            .read_feed::<EmaShortageRow>(SHORTAGES_FILE)?
            .into_iter()
            .filter_map(|row| {
                let medicine_affected = scalar_value(&row.medicine_affected)?;
                if !field_matches_terms(&medicine_affected, &anchor.terms) {
                    return None;
                }
                Some(EmaShortageEntry {
                    medicine_affected,
                    status: scalar_value(&row.supply_shortage_status),
                    availability_of_alternatives: scalar_value(&row.availability_of_alternatives),
                    first_published_date: scalar_value(&row.first_published_date),
                    last_updated_date: scalar_value(&row.last_updated_date),
                })
            })
            .collect::<Vec<_>>();

        out.sort_by(|a, b| {
            cmp_date_desc(
                a.last_updated_date
                    .as_deref()
                    .or(a.first_published_date.as_deref()),
                b.last_updated_date
                    .as_deref()
                    .or(b.first_published_date.as_deref()),
            )
        });
        Ok(out)
    }

    fn require_files(&self, files: &[&str]) -> Result<(), BioMcpError> {
        let missing = ema_missing_files(&self.root, files);
        if missing.is_empty() {
            return Ok(());
        }

        Err(BioMcpError::SourceUnavailable {
            source_name: SOURCE_NAME.to_string(),
            reason: format!(
                "Missing required EMA file(s) under {}: {}",
                self.root.display(),
                missing.join(", ")
            ),
            suggestion: format!(
                "Run `biomcp ema sync`, retry with network access, or place the EMA human-medicines JSON batch from {DOWNLOAD_URL} into {}. You can also set BIOMCP_EMA_DIR.",
                self.root.display()
            ),
        })
    }

    fn read_feed<T>(&self, file: &str) -> Result<Vec<T>, BioMcpError>
    where
        T: DeserializeOwned,
    {
        self.require_files(&[file])?;
        let reader = File::open(self.root.join(file))?;
        let wrapper: EmaWrapper<T> = serde_json::from_reader(reader)?;
        Ok(wrapper.data)
    }
}

fn ema_report_base() -> Cow<'static, str> {
    crate::sources::env_base(EMA_REPORT_BASE, EMA_REPORT_BASE_ENV)
}

fn normalize_sync_mode(mode: EmaSyncMode) -> EmaSyncMode {
    if matches!(mode, EmaSyncMode::Auto) && crate::sources::is_no_cache_enabled() {
        EmaSyncMode::Force
    } else {
        mode
    }
}

fn file_is_stale(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return true;
    };
    let Ok(modified) = metadata.modified() else {
        return true;
    };
    match SystemTime::now().duration_since(modified) {
        Ok(age) => age >= EMA_STALE_AFTER,
        Err(_) => false,
    }
}

fn feed_sync_state(root: &Path, feed: EmaFeed) -> FeedSyncState {
    let path = root.join(feed.local_name);
    if !path.is_file() {
        return FeedSyncState::Missing;
    }
    if file_is_stale(&path) {
        FeedSyncState::Stale
    } else {
        FeedSyncState::Fresh
    }
}

fn sync_plan(root: &Path, mode: EmaSyncMode) -> Vec<FeedSyncPlan> {
    let mode = normalize_sync_mode(mode);
    EMA_FEEDS
        .iter()
        .copied()
        .filter_map(|feed| {
            let state = feed_sync_state(root, feed);
            match mode {
                EmaSyncMode::Force => Some(FeedSyncPlan {
                    feed,
                    state,
                    cache_mode: CacheMode::Reload,
                }),
                EmaSyncMode::Auto => match state {
                    FeedSyncState::Fresh => None,
                    FeedSyncState::Missing => Some(FeedSyncPlan {
                        feed,
                        state,
                        cache_mode: CacheMode::Default,
                    }),
                    FeedSyncState::Stale => Some(FeedSyncPlan {
                        feed,
                        state,
                        cache_mode: CacheMode::Default,
                    }),
                },
            }
        })
        .collect()
}

fn sync_intro(plan: &[FeedSyncPlan], mode: EmaSyncMode) -> &'static str {
    if matches!(normalize_sync_mode(mode), EmaSyncMode::Force)
        || plan
            .iter()
            .any(|entry| matches!(entry.state, FeedSyncState::Stale))
    {
        "Refreshing"
    } else {
        "Downloading"
    }
}

fn has_readable_local_file(path: &Path) -> bool {
    path.is_file() && File::open(path).is_ok()
}

fn touch_file(path: &Path) -> Result<(), BioMcpError> {
    let file = std::fs::OpenOptions::new().write(true).open(path)?;
    file.set_modified(SystemTime::now())?;
    Ok(())
}

fn validate_feed_payload(feed: EmaFeed, body: &[u8]) -> Result<(), BioMcpError> {
    let payload: Value = serde_json::from_slice(body).map_err(|source| BioMcpError::ApiJson {
        api: EMA_API.to_string(),
        source,
    })?;
    let Some(object) = payload.as_object() else {
        return Err(BioMcpError::Api {
            api: EMA_API.to_string(),
            message: format!("{}: expected a top-level JSON object", feed.local_name),
        });
    };
    if !object.get("data").is_some_and(|value| value.is_array()) {
        return Err(BioMcpError::Api {
            api: EMA_API.to_string(),
            message: format!(
                "{}: expected a top-level `data` array in the EMA payload",
                feed.local_name
            ),
        });
    }
    Ok(())
}

async fn sync_feed(root: &Path, plan: FeedSyncPlan) -> Result<(), BioMcpError> {
    let client = crate::sources::shared_client()?;
    let url = format!(
        "{}/{}",
        ema_report_base().trim_end_matches('/'),
        plan.feed.report_name
    );
    let mut request = client.get(url).with_extension(plan.cache_mode);
    if matches!(plan.state, FeedSyncState::Stale) {
        // `http-cache`'s `NoCache` mode performs an unconditional network fetch.
        // `Default` plus a request `Cache-Control: no-cache` forces validator-based
        // revalidation with the cached ETag/Last-Modified metadata.
        request = request.header(reqwest::header::CACHE_CONTROL, "no-cache");
    }
    let response = crate::sources::with_response_body_limit(request, EMA_MAX_BODY_BYTES, EMA_API)
        .send_with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::EMA,
        ))
        .await?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let body = crate::sources::read_limited_source_body_with_limit(
        response,
        crate::error::SourceContext::narrow(crate::error::SourceProvider::EMA),
        EMA_MAX_BODY_BYTES,
    )
    .await?;
    let context = crate::error::SourceContext::retry(crate::error::SourceProvider::EMA);
    if !status.is_success() {
        return Err(BioMcpError::Api {
            api: EMA_API.to_string(),
            message: format!(
                "{}: HTTP {status}: {}",
                plan.feed.local_name,
                crate::sources::body_excerpt(&body)
            ),
        }
        .with_source_context(context));
    }

    crate::sources::ensure_json_content_type(context, content_type.as_ref(), &body)?;
    validate_feed_payload(plan.feed, &body).map_err(|error| error.with_source_context(context))?;

    let path = root.join(plan.feed.local_name);
    if let Ok(existing) = tokio::fs::read(&path).await
        && existing == body
    {
        touch_file(&path)?;
        return Ok(());
    }

    crate::utils::download::write_atomic_bytes(&path, &body).await
}

fn ema_sync_error(root: &Path, detail: impl Into<String>) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!(
            "Could not prepare EMA data under {}. {}",
            root.display(),
            detail.into()
        ),
        suggestion: format!(
            "Retry with network access or run `biomcp ema sync`. You can also preseed the EMA human-medicines JSON batch from {DOWNLOAD_URL} into {} or set BIOMCP_EMA_DIR.",
            root.display()
        ),
    }
}

fn write_stderr_line(line: &str) -> Result<(), BioMcpError> {
    let mut stderr = std::io::stderr().lock();
    writeln!(stderr, "{line}")?;
    Ok(())
}

async fn sync_ema_root(root: &Path, mode: EmaSyncMode) -> Result<(), BioMcpError> {
    let plan = sync_plan(root, mode);
    if plan.is_empty() {
        return Ok(());
    }

    tokio::fs::create_dir_all(root).await?;

    write_stderr_line(&format!(
        "{} EMA data ({EMA_SIZE_HINT})...",
        sync_intro(&plan, mode)
    ))?;

    let mut fatal_errors = Vec::new();
    for entry in plan {
        if let Err(err) = sync_feed(root, entry).await {
            let path = root.join(entry.feed.local_name);
            if has_readable_local_file(&path) {
                write_stderr_line(&format!(
                    "Warning: EMA refresh failed for {}: {err}. Using existing data.",
                    entry.feed.local_name
                ))?;
                continue;
            }
            fatal_errors.push(format!("{}: {err}", entry.feed.local_name));
        }
    }

    let missing = ema_missing_files(root, EMA_REQUIRED_FILES);
    if missing.is_empty() {
        return Ok(());
    }

    let detail = if fatal_errors.is_empty() {
        format!("Missing required EMA file(s): {}", missing.join(", "))
    } else {
        format!(
            "{} Missing required EMA file(s): {}",
            fatal_errors.join("; "),
            missing.join(", ")
        )
    };
    Err(ema_sync_error(root, detail))
}

pub(crate) fn ema_missing_files<'a>(root: &Path, files: &[&'a str]) -> Vec<&'a str> {
    files
        .iter()
        .filter(|file| !root.join(file).is_file())
        .copied()
        .collect()
}

pub(crate) fn resolve_ema_root() -> PathBuf {
    if let Some(path) = std::env::var("BIOMCP_EMA_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return PathBuf::from(path);
    }

    match dirs::data_dir() {
        Some(path) => path.join("biomcp").join("ema"),
        None => std::env::temp_dir().join("biomcp").join("ema"),
    }
}

fn normalize_term(value: &str) -> Option<String> {
    clean_text(value).map(|value| value.to_ascii_lowercase())
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

fn clean_text(value: &str) -> Option<String> {
    let normalized = value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn clean_ema_text(value: &str) -> Option<String> {
    clean_text(&strip_inline_html_tags(&decode_html_entities(value)))
}

fn scalar_value(value: &StringOrVec) -> Option<String> {
    value.first().and_then(clean_text)
}

fn scalar_ema_text(value: &StringOrVec) -> Option<String> {
    value.first().and_then(clean_ema_text)
}

fn is_human_category(value: &StringOrVec) -> bool {
    value
        .first()
        .and_then(normalize_term)
        .is_some_and(|category| category == "human")
}

fn field_matches_terms(value: &str, terms: &HashSet<String>) -> bool {
    let Some(field) = normalize_term(value) else {
        return false;
    };
    if terms.contains(&field) {
        return true;
    }

    for piece in field.split(['/', ',', ';', '|']).filter_map(normalize_term) {
        if terms.contains(&piece) {
            return true;
        }
    }

    terms
        .iter()
        .any(|term| contains_boundary_phrase(&field, term))
}

fn contains_boundary_phrase(field: &str, term: &str) -> bool {
    if field.is_empty() || term.is_empty() {
        return false;
    }

    let field_bytes = field.as_bytes();
    let mut search_from = 0usize;
    while let Some(pos) = field[search_from..].find(term) {
        let start = search_from + pos;
        let end = start + term.len();
        let before_ok = start == 0 || !field_bytes[start - 1].is_ascii_alphanumeric();
        let after_ok = end == field_bytes.len() || !field_bytes[end].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
        search_from = start + 1;
    }
    false
}

fn cmp_date_desc(a: Option<&str>, b: Option<&str>) -> std::cmp::Ordering {
    parse_ema_date(b).cmp(&parse_ema_date(a))
}

fn parse_ema_date(value: Option<&str>) -> Option<(u32, u32, u32)> {
    let value = value?.trim();
    let mut parts = value.split('/');
    let day = parts.next()?.parse::<u32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let year = parts.next()?.parse::<u32>().ok()?;
    Some((year, month, day))
}

#[cfg(test)]
mod tests;
