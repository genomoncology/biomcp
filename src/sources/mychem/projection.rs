//! Original-byte drug identity custody and temporary product compatibility.
//! Remove this compatibility view when product presentation uses drug documents,
//! and before public release. Enrichment never supplies an identity value.
use super::*;
use crate::error::{BioMcpError, SourceContext, SourceProvider};
use biodata::{MyChemFieldState, MyChemPage, MyChemProfile, MyChemRow, parse_mychem_identity};
use std::sync::Arc;

#[derive(Deserialize)]
struct Enrichment {
    drugbank: Option<MyChemDrugBank>,
    chembl: Option<MyChemChembl>,
    drugcentral: Option<MyChemDrugCentral>,
    gtopdb: Option<MyChemGtoPdb>,
    ndc: Option<MyChemNdcField>,
    unii: Option<MyChemUniiField>,
    chebi: Option<MyChemChebiField>,
    openfda: Option<MyChemOpenfda>,
}

pub(crate) fn failure(message: &'static str) -> BioMcpError {
    BioMcpError::Api {
        api: "MyChem.info".into(),
        message: message.into(),
    }
    .with_source_context(SourceContext::retry(SourceProvider::MYCHEM))
}
pub(crate) fn validate_transport(
    status: reqwest::StatusCode,
    content_type: Option<&reqwest::header::HeaderValue>,
    _bytes: &[u8],
) -> Result<(), BioMcpError> {
    if !status.is_success() {
        return Err(failure("MyChem response status rejected"));
    }
    let media = content_type
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    if media.is_some_and(|value| {
        !(value.eq_ignore_ascii_case("application/json")
            || value.eq_ignore_ascii_case("text/json")
            || value.to_ascii_lowercase().ends_with("+json"))
    }) {
        return Err(failure("MyChem response content type rejected"));
    }
    Ok(())
}

fn values(row: &MyChemRow, section: &str, field: &str, index: Option<usize>) -> Vec<String> {
    row.source()
        .fields()
        .iter()
        .find(|value| {
            value.section() == section && value.name() == field && value.section_index() == index
        })
        .and_then(|value| match value.state() {
            MyChemFieldState::Values(values) => Some(values.clone()),
            _ => None,
        })
        .unwrap_or_default()
}
fn first(row: &MyChemRow, section: &str, field: &str, index: Option<usize>) -> Option<String> {
    values(row, section, field, index).into_iter().next()
}
fn names(row: &MyChemRow, field: &str) -> StringOrVec {
    let Some(source) = row
        .source()
        .fields()
        .iter()
        .find(|value| value.section() == "openfda" && value.name() == field)
    else {
        return StringOrVec::None;
    };
    match source.state() {
        MyChemFieldState::Values(values) if source.was_array() => {
            StringOrVec::Multiple(values.clone())
        }
        MyChemFieldState::Values(values) => values
            .first()
            .cloned()
            .map(StringOrVec::Single)
            .unwrap_or_default(),
        MyChemFieldState::EmptyArray => StringOrVec::Multiple(Vec::new()),
        _ => StringOrVec::None,
    }
}
fn convert(row: &MyChemRow, page: Arc<MyChemPage>) -> Result<MyChemHit, BioMcpError> {
    let mut enrichment: Enrichment =
        serde_json::from_str(row.source().raw()).map_err(|source| {
            BioMcpError::ApiJson {
                api: "MyChem.info".into(),
                source,
            }
            .with_source_context(SourceContext::retry(SourceProvider::MYCHEM))
        })?;
    if let Some(value) = &mut enrichment.drugbank {
        value.id = first(row, "drugbank", "id", None);
        value.name = first(row, "drugbank", "name", None);
        value.synonyms = values(row, "drugbank", "synonyms", None);
    }
    if let Some(value) = &mut enrichment.chembl {
        value.molecule_chembl_id = first(row, "chembl", "molecule_chembl_id", None);
        value.pref_name = first(row, "chembl", "pref_name", None);
    }
    if let Some(value) = &mut enrichment.gtopdb {
        value.name = first(row, "gtopdb", "name", None);
    }
    if let Some(value) = &mut enrichment.openfda {
        value.generic_name = names(row, "generic_name");
        value.brand_name = names(row, "brand_name");
    }
    if let Some(value) = &mut enrichment.ndc {
        match value {
            MyChemNdcField::One(value) => {
                value.nonproprietaryname = first(row, "ndc", "nonproprietaryname", None)
            }
            MyChemNdcField::Many(values) => {
                for (index, value) in values.iter_mut().enumerate() {
                    value.nonproprietaryname = first(row, "ndc", "nonproprietaryname", Some(index));
                }
            }
        }
    }
    if let Some(value) = &mut enrichment.unii {
        match value {
            MyChemUniiField::One(value) => {
                value.unii = first(row, "unii", "unii", None);
                value.display_name = first(row, "unii", "display_name", None);
            }
            MyChemUniiField::Many(values) => {
                for (index, value) in values.iter_mut().enumerate() {
                    value.unii = first(row, "unii", "unii", Some(index));
                    value.display_name = first(row, "unii", "display_name", Some(index));
                }
            }
        }
    }
    if let Some(value) = &mut enrichment.chebi {
        match value {
            MyChemChebiField::One(value) => value.name = first(row, "chebi", "name", None),
            MyChemChebiField::Many(values) => {
                for (index, value) in values.iter_mut().enumerate() {
                    value.name = first(row, "chebi", "name", Some(index));
                }
            }
        }
    }
    Ok(MyChemHit {
        drugbank: enrichment.drugbank,
        chembl: enrichment.chembl,
        drugcentral: enrichment.drugcentral,
        gtopdb: enrichment.gtopdb,
        ndc: enrichment.ndc,
        unii: enrichment.unii,
        chebi: enrichment.chebi,
        openfda: enrichment.openfda,
        row: row.clone(),
        page,
        conversion: Default::default(),
    })
}
pub(crate) fn decode(
    bytes: &[u8],
    profile: MyChemProfile,
) -> Result<MyChemQueryResponse, BioMcpError> {
    let page = Arc::new(
        parse_mychem_identity(bytes, profile)
            .map_err(|_| failure("MyChem identity response rejected"))?,
    );
    let rows = page
        .require_complete()
        .map_err(|_| failure("MyChem identity page rejected"))?;
    let total = checked_total::<usize>(page.total())?;
    let hits = rows
        .into_iter()
        .map(|row| convert(row, Arc::clone(&page)))
        .collect::<Result<_, _>>()?;
    Ok(MyChemQueryResponse { total, hits })
}
/// Degrade only on absence or positively identified unavailable transport.
pub(crate) fn optional_failure(error: &BioMcpError) -> bool {
    crate::sources::mydisease::optional_failure(error)
}
/// Preserve terminal classification without retaining source diagnostic text.
pub(crate) fn terminal_failure(error: BioMcpError) -> BioMcpError {
    match error {
        BioMcpError::WithSourceContext { context, source } => {
            terminal_failure(*source).with_source_context(context)
        }
        BioMcpError::Api { .. } => BioMcpError::Api {
            api: "MyChem.info".into(),
            message: "MyChem request rejected".into(),
        },
        BioMcpError::SourceUnavailable { .. } => BioMcpError::SourceUnavailable {
            source_name: "MyChem.info".into(),
            reason: "MyChem source unavailable".into(),
            suggestion: "Review source configuration and retry.".into(),
        },
        other => other,
    }
}
#[cfg(test)]
impl<'de> Deserialize<'de> for MyChemHit {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hit = serde_json::Value::deserialize(deserializer)?;
        let bytes = serde_json::to_vec(&serde_json::json!({"total": 1, "hits": [hit]}))
            .map_err(serde::de::Error::custom)?;
        decode(&bytes, MyChemProfile::Get)
            .map_err(serde::de::Error::custom)?
            .hits
            .into_iter()
            .next()
            .ok_or_else(|| serde::de::Error::custom("MyChem test hit missing"))
    }
}
#[cfg(test)]
impl<'de> Deserialize<'de> for MyChemQueryResponse {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let page = serde_json::Value::deserialize(deserializer)?;
        let bytes = serde_json::to_vec(&page).map_err(serde::de::Error::custom)?;
        decode(&bytes, MyChemProfile::Search).map_err(serde::de::Error::custom)
    }
}

pub(super) fn checked_total<T: TryFrom<u64> + TryInto<usize>>(
    total: u64,
) -> Result<T, BioMcpError> {
    T::try_from(total).map_err(|_| BioMcpError::ProviderResponseLimit {
        source_name: "MyChem.info".into(),
        limit: if std::mem::size_of::<T>() == 4 {
            u32::MAX as usize
        } else {
            usize::MAX
        },
        unit: "results",
    })
}
