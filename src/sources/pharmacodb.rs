//! PharmacoDB, the GraphQL route to published in-vitro drug-response metrics.
//!
//! PharmacoDB carries GDSC1, GDSC2, CTRPv2, PRISM, gCSI and the rest, and it is
//! BioMCP's route to GDSC data now that `cancerrxgene.org` returns HTTP 410.
//! The client exposes three lookups and two experiment reads and nothing else.
//! BioMCP reports every metric as PharmacoDB published it. It never ranks,
//! thresholds, averages, or labels a cell line as sensitive or resistant.
//!
//! One PharmacoDB body can be far larger than any other source BioMCP reads.
//! The measured worst cases are K-562 at 19.4 MB and doxorubicin at 18.6 MB, so
//! the experiment reads carry their own 32 MiB cap rather than the shared 8 MiB
//! default. The cap is checked chunk by chunk, so an oversized body fails while
//! it streams and is never fully buffered.

use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const PHARMACODB_BASE: &str = "https://pharmacodb.ca";
const PHARMACODB_BASE_ENV: &str = "BIOMCP_PHARMACODB_BASE";
const PHARMACODB_API: &str = "pharmacodb";
const PHARMACODB_PATH: &str = "graphql";

/// The cap the experiment reads use, above every measured PharmacoDB body. The
/// shared 8 MiB default stays for every other source.
pub(crate) const PHARMACODB_MAX_BODY_BYTES: usize = 32 * 1024 * 1024;
/// What a section says when a body passes the cap.
pub(crate) const PHARMACODB_BODY_LIMIT_MESSAGE: &str = "PharmacoDB response exceeds 32 MiB";

/// The datasets PharmacoDB carries, read from its own `datasets` query on
/// 2026-09-18. The list is held here so an unknown `--dataset` fails before any
/// request and can still name every accepted value.
pub(crate) const PHARMACODB_DATASET_NAMES: &[&str] = &[
    "CCLE",
    "CTRPv2",
    "FIMM",
    "GDSC1",
    "GDSC2",
    "GRAY",
    "NCI60",
    "PRISM",
    "UHNBreast",
    "gCSI",
];

/// The operation names BioMCP uses in a PharmacoDB error message.
const OPERATION_CELL_LINE: &str = "cell_line";
const OPERATION_COMPOUND: &str = "compound";
const OPERATION_EXPERIMENT_COUNTS: &str = "experiment counts";
const OPERATION_EXPERIMENTS: &str = "experiments";

/// The upstream message for a cell line PharmacoDB does not hold.
const CELL_LINE_MISS_MESSAGE: &str = "Please provide a valid cell ID, Name or UID.";
/// The upstream message for a compound PharmacoDB does not hold. PharmacoDB
/// answers a missing compound with a non-null violation rather than a message
/// of its own, observed 2026-09-18.
const COMPOUND_MISS_MESSAGE: &str = "Cannot return null for non-nullable field";

const CELL_LINE_BY_UID_QUERY: &str = r"query PharmacoCellLineByUid($cellUID: String) {
  cell_line(cellUID: $cellUID) {
    id
    uid
    name
    accession_id
  }
}
";

const CELL_LINE_BY_NAME_QUERY: &str = r"query PharmacoCellLineByName($cellName: String) {
  cell_line(cellName: $cellName) {
    id
    uid
    name
    accession_id
  }
}
";

const COMPOUND_BY_NAME_QUERY: &str = r"query PharmacoCompoundByName($compoundName: String) {
  compound(compoundName: $compoundName) {
    compound {
      id
      uid
      name
    }
  }
}
";

/// The counts projection. Only the row id and its dataset name are asked for,
/// so a section pays the smallest body PharmacoDB can answer with.
const EXPERIMENT_COUNTS_QUERY: &str = r"query PharmacoExperimentCounts($cellLineId: Int, $compoundId: Int) {
  experiments(cellLineId: $cellLineId, compoundId: $compoundId, all: true) {
    id
    dataset {
      name
    }
  }
}
";

/// The row projection. `DSS2` and `DSS3` are left out because no BioMCP output
/// prints them.
const EXPERIMENTS_QUERY: &str = r"query PharmacoExperiments($cellLineId: Int, $compoundId: Int) {
  experiments(cellLineId: $cellLineId, compoundId: $compoundId, all: true) {
    id
    cell_line {
      id
      uid
      name
    }
    compound {
      id
      uid
      name
    }
    dataset {
      name
    }
    tissue {
      name
    }
    profile {
      AAC
      IC50
      EC50
      Einf
      HS
      DSS1
    }
  }
}
";

/// Which side of PharmacoDB an experiment read is scoped to. There is no
/// unscoped read: `experiments` with no id argument is the 19.4 MB case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PharmacoFilter {
    CellLine(i64),
    Compound(i64),
    Pair { cell_line_id: i64, compound_id: i64 },
}

impl PharmacoFilter {
    /// The GraphQL variables one filter sends. Only the ids the filter names are
    /// sent, so the other argument stays absent rather than null.
    fn variables(self) -> serde_json::Value {
        match self {
            Self::CellLine(cell_line_id) => serde_json::json!({ "cellLineId": cell_line_id }),
            Self::Compound(compound_id) => serde_json::json!({ "compoundId": compound_id }),
            Self::Pair {
                cell_line_id,
                compound_id,
            } => serde_json::json!({
                "cellLineId": cell_line_id,
                "compoundId": compound_id,
            }),
        }
    }
}

/// One PharmacoDB cell line, with the Cellosaurus accession it claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PharmacoCellLine {
    pub id: i64,
    pub uid: String,
    pub name: String,
    pub accession_id: Option<String>,
}

/// One PharmacoDB compound.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PharmacoCompound {
    pub id: i64,
    pub uid: String,
    pub name: String,
}

/// How many experiments one dataset holds for the requested side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PharmacoDatasetCount {
    pub name: String,
    pub count: usize,
}

/// One published experiment. Every metric is optional, and a null stays null.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PharmacoExperiment {
    pub experiment_id: i64,
    pub dataset: String,
    pub cell_line_id: i64,
    pub cell_line_uid: String,
    pub cell_line_name: String,
    pub compound_id: i64,
    pub compound_uid: String,
    pub compound_name: String,
    pub tissue: Option<String>,
    pub aac: Option<f64>,
    pub ic50: Option<f64>,
    pub ec50: Option<f64>,
    pub einf: Option<f64>,
    pub hs: Option<f64>,
    pub dss1: Option<f64>,
}

pub struct PharmacoDbClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl PharmacoDbClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(PHARMACODB_BASE, PHARMACODB_BASE_ENV),
        })
    }

    fn query_plan(query: &str, variables: serde_json::Value) -> RequestPlan {
        RequestPlan::post(PHARMACODB_PATH).json(serde_json::json!({
            "query": query,
            "variables": variables,
        }))
    }

    pub(crate) fn cell_line_by_uid_plan(uid: &str) -> Result<RequestPlan, BioMcpError> {
        let uid = required_value("PharmacoDB cell line UID", uid)?;
        Ok(Self::query_plan(
            CELL_LINE_BY_UID_QUERY,
            serde_json::json!({ "cellUID": uid }),
        ))
    }

    pub(crate) fn cell_line_by_name_plan(name: &str) -> Result<RequestPlan, BioMcpError> {
        let name = required_value("PharmacoDB cell line name", name)?;
        Ok(Self::query_plan(
            CELL_LINE_BY_NAME_QUERY,
            serde_json::json!({ "cellName": name }),
        ))
    }

    pub(crate) fn compound_by_name_plan(name: &str) -> Result<RequestPlan, BioMcpError> {
        let name = required_value("PharmacoDB compound name", name)?;
        Ok(Self::query_plan(
            COMPOUND_BY_NAME_QUERY,
            serde_json::json!({ "compoundName": name }),
        ))
    }

    pub(crate) fn experiment_counts_plan(filter: PharmacoFilter) -> RequestPlan {
        Self::query_plan(EXPERIMENT_COUNTS_QUERY, filter.variables())
    }

    pub(crate) fn experiments_plan(filter: PharmacoFilter) -> RequestPlan {
        Self::query_plan(EXPERIMENTS_QUERY, filter.variables())
    }

    /// Decode one GraphQL body. The two upstream miss messages become `None`,
    /// and every other GraphQL error is a source error.
    fn decode_optional<T: DeserializeOwned>(
        url: &str,
        operation: &str,
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<Option<T>, BioMcpError> {
        let resp: GraphQlResponse<T> = decode_graphql_body(url, operation, status, bytes)?;
        if let Some(message) = graphql_error_message(resp.errors.as_deref()) {
            if is_miss(&message) {
                return Ok(None);
            }
            return Err(graphql_error(url, operation, &message));
        }
        Ok(resp.data)
    }

    pub(crate) fn decode_cell_line(
        url: &str,
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<Option<PharmacoCellLine>, BioMcpError> {
        let data: Option<CellLineData> =
            Self::decode_optional(url, OPERATION_CELL_LINE, status, bytes)?;
        Ok(data
            .and_then(|data| data.cell_line)
            .map(|node| PharmacoCellLine {
                id: node.id,
                uid: clean(node.uid),
                name: clean(node.name),
                accession_id: clean_optional(node.accession_id),
            }))
    }

    pub(crate) fn decode_compound(
        url: &str,
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<Option<PharmacoCompound>, BioMcpError> {
        let data: Option<CompoundData> =
            Self::decode_optional(url, OPERATION_COMPOUND, status, bytes)?;
        Ok(data
            .and_then(|data| data.compound)
            .and_then(|wrapper| wrapper.compound)
            .map(|node| PharmacoCompound {
                id: node.id,
                uid: clean(node.uid),
                name: clean(node.name),
            }))
    }

    /// Sum the counts projection into one row per dataset, ordered by name.
    pub(crate) fn decode_experiment_counts(
        url: &str,
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<Vec<PharmacoDatasetCount>, BioMcpError> {
        let data: Option<CountsData> =
            Self::decode_optional(url, OPERATION_EXPERIMENT_COUNTS, status, bytes)?;
        let rows = data.map(|data| data.experiments).unwrap_or_default();
        let mut counts: Vec<PharmacoDatasetCount> = Vec::new();
        for row in rows {
            let name = clean(row.dataset.and_then(|node| node.name));
            match counts
                .iter_mut()
                .find(|entry| entry.name.eq_ignore_ascii_case(&name))
            {
                Some(entry) => entry.count += 1,
                None => counts.push(PharmacoDatasetCount { name, count: 1 }),
            }
        }
        counts.sort_by(|left, right| {
            left.name
                .to_ascii_lowercase()
                .cmp(&right.name.to_ascii_lowercase())
                .then_with(|| left.name.cmp(&right.name))
        });
        Ok(counts)
    }

    pub(crate) fn decode_experiments(
        url: &str,
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<Vec<PharmacoExperiment>, BioMcpError> {
        let data: Option<ExperimentsData> =
            Self::decode_optional(url, OPERATION_EXPERIMENTS, status, bytes)?;
        Ok(data
            .map(|data| data.experiments)
            .unwrap_or_default()
            .into_iter()
            .map(PharmacoExperiment::from_node)
            .collect())
    }

    /// Read one experiment body under the PharmacoDB cap. The cap is applied
    /// chunk by chunk, so an oversized body never lands in memory whole.
    pub(crate) async fn read_experiment_body(
        resp: reqwest::Response,
    ) -> Result<Vec<u8>, BioMcpError> {
        crate::sources::read_limited_body_with_limit(
            resp,
            PHARMACODB_API,
            PHARMACODB_MAX_BODY_BYTES,
        )
        .await
    }

    async fn send(&self, plan: &RequestPlan) -> Result<(String, reqwest::Response), BioMcpError> {
        let url = crate::sources::join_base_path(self.base.as_ref(), &plan.path);
        let req = request_from_plan(&self.client, self.base.as_ref(), plan);
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::PHARMACODB,
            ))
            .await?;
        Ok((url, resp))
    }

    /// One small lookup: the shared 8 MiB default is plenty.
    async fn lookup_bytes(
        &self,
        plan: &RequestPlan,
    ) -> Result<(String, StatusCode, Vec<u8>), BioMcpError> {
        let (url, resp) = self.send(plan).await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::PHARMACODB),
        )
        .await?;
        Ok((url, status, bytes))
    }

    /// One experiment read, under the PharmacoDB cap.
    async fn experiment_bytes(
        &self,
        plan: &RequestPlan,
    ) -> Result<(String, StatusCode, Vec<u8>), BioMcpError> {
        let (url, resp) = self.send(plan).await?;
        let status = resp.status();
        let bytes = Self::read_experiment_body(resp).await?;
        Ok((url, status, bytes))
    }

    /// The cell line one PharmacoDB UID names, or `None`.
    pub(crate) async fn cell_line_by_uid(
        &self,
        uid: &str,
    ) -> Result<Option<PharmacoCellLine>, BioMcpError> {
        let plan = Self::cell_line_by_uid_plan(uid)?;
        let (url, status, bytes) = self.lookup_bytes(&plan).await?;
        Self::decode_cell_line(&url, status, &bytes)
    }

    /// The cell line one PharmacoDB name names, or `None`. PharmacoDB spells
    /// its cell lines the way Cellosaurus does, so this reaches the lines that
    /// Cellosaurus carries no PharmacoDB cross-reference for.
    pub(crate) async fn cell_line_by_name(
        &self,
        name: &str,
    ) -> Result<Option<PharmacoCellLine>, BioMcpError> {
        let plan = Self::cell_line_by_name_plan(name)?;
        let (url, status, bytes) = self.lookup_bytes(&plan).await?;
        Self::decode_cell_line(&url, status, &bytes)
    }

    /// The compound one name names, or `None`. PharmacoDB matches the name
    /// case-insensitively.
    pub(crate) async fn compound_by_name(
        &self,
        name: &str,
    ) -> Result<Option<PharmacoCompound>, BioMcpError> {
        let plan = Self::compound_by_name_plan(name)?;
        let (url, status, bytes) = self.lookup_bytes(&plan).await?;
        Self::decode_compound(&url, status, &bytes)
    }

    /// How many experiments each dataset holds for one side.
    pub(crate) async fn experiment_counts(
        &self,
        filter: PharmacoFilter,
    ) -> Result<Vec<PharmacoDatasetCount>, BioMcpError> {
        let plan = Self::experiment_counts_plan(filter);
        let (url, status, bytes) = self.experiment_bytes(&plan).await?;
        Self::decode_experiment_counts(&url, status, &bytes)
    }

    /// Every experiment row for one side or one pair. PharmacoDB has no dataset
    /// argument, so a dataset filter is applied to these rows afterwards.
    pub(crate) async fn experiments(
        &self,
        filter: PharmacoFilter,
    ) -> Result<Vec<PharmacoExperiment>, BioMcpError> {
        let plan = Self::experiments_plan(filter);
        let (url, status, bytes) = self.experiment_bytes(&plan).await?;
        Self::decode_experiments(&url, status, &bytes)
    }
}

impl PharmacoExperiment {
    fn from_node(node: ExperimentNode) -> Self {
        let cell_line = node.cell_line.unwrap_or_default();
        let compound = node.compound.unwrap_or_default();
        let profile = node.profile.unwrap_or_default();
        Self {
            experiment_id: node.id,
            dataset: clean(node.dataset.and_then(|row| row.name)),
            cell_line_id: cell_line.id.unwrap_or_default(),
            cell_line_uid: clean(cell_line.uid),
            cell_line_name: clean(cell_line.name),
            compound_id: compound.id.unwrap_or_default(),
            compound_uid: clean(compound.uid),
            compound_name: clean(compound.name),
            tissue: clean_optional(node.tissue.and_then(|row| row.name)),
            aac: profile.aac,
            ic50: profile.ic50,
            ec50: profile.ec50,
            einf: profile.einf,
            hs: profile.hs,
            dss1: profile.dss1,
        }
    }
}

/// Is one upstream GraphQL message a miss rather than a failure?
fn is_miss(message: &str) -> bool {
    message.contains(CELL_LINE_MISS_MESSAGE) || message.contains(COMPOUND_MISS_MESSAGE)
}

fn graphql_error_message(errors: Option<&[GraphQlError]>) -> Option<String> {
    let message = errors?
        .iter()
        .filter_map(|row| row.message.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("; ");
    (!message.is_empty()).then_some(message)
}

fn graphql_error(url: &str, operation: &str, message: &str) -> BioMcpError {
    BioMcpError::Api {
        api: PHARMACODB_API.to_string(),
        message: format!("PharmacoDB GraphQL error for {operation} at {url}: {message}"),
    }
}

/// Decode one GraphQL body, rejecting a non-JSON answer.
///
/// An HTTP 200 whose body is an HTML human-verification page where a GraphQL
/// JSON body was expected is a provider error that names the URL and the
/// operation and stops the command. BioMCP never retries through a check and
/// never scrapes the page.
fn decode_graphql_body<T: DeserializeOwned>(
    url: &str,
    operation: &str,
    status: StatusCode,
    bytes: &[u8],
) -> Result<GraphQlResponse<T>, BioMcpError> {
    if status.is_success() && looks_like_html(bytes) {
        return Err(BioMcpError::Api {
            api: PHARMACODB_API.to_string(),
            message: format!(
                "PharmacoDB returned an HTML page where a GraphQL JSON body was expected \
for {operation} at {url}: {}",
                crate::sources::body_excerpt(bytes)
            ),
        });
    }
    crate::sources::decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PHARMACODB),
        status,
        None,
        bytes,
        false,
    )
}

fn looks_like_html(bytes: &[u8]) -> bool {
    let sniff = String::from_utf8_lossy(&bytes[..bytes.len().min(128)])
        .trim_start()
        .to_ascii_lowercase();
    sniff.starts_with("<!doctype html") || sniff.starts_with("<html")
}

fn required_value(label: &str, value: &str) -> Result<String, BioMcpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(BioMcpError::InvalidArgument(format!(
            "{label} is required."
        )));
    }
    if trimmed.len() > 256 {
        return Err(BioMcpError::InvalidArgument(format!(
            "{label} is too long."
        )));
    }
    Ok(trimmed.to_string())
}

fn clean(value: Option<String>) -> String {
    value
        .map(|value| value.trim().to_string())
        .unwrap_or_default()
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CellLineData {
    cell_line: Option<CellLineNode>,
}

#[derive(Debug, Deserialize)]
struct CellLineNode {
    id: i64,
    uid: Option<String>,
    name: Option<String>,
    accession_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CompoundData {
    compound: Option<CompoundWrapper>,
}

#[derive(Debug, Deserialize)]
struct CompoundWrapper {
    compound: Option<CompoundNode>,
}

#[derive(Debug, Deserialize)]
struct CompoundNode {
    id: i64,
    uid: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CountsData {
    #[serde(default)]
    experiments: Vec<CountsRow>,
}

#[derive(Debug, Deserialize)]
struct CountsRow {
    dataset: Option<NameNode>,
}

#[derive(Debug, Deserialize)]
struct ExperimentsData {
    #[serde(default)]
    experiments: Vec<ExperimentNode>,
}

#[derive(Debug, Deserialize)]
struct ExperimentNode {
    id: i64,
    cell_line: Option<RefNode>,
    compound: Option<RefNode>,
    dataset: Option<NameNode>,
    tissue: Option<NameNode>,
    profile: Option<ProfileNode>,
}

#[derive(Debug, Default, Deserialize)]
struct RefNode {
    id: Option<i64>,
    uid: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NameNode {
    name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ProfileNode {
    #[serde(rename = "AAC")]
    aac: Option<f64>,
    #[serde(rename = "IC50")]
    ic50: Option<f64>,
    #[serde(rename = "EC50")]
    ec50: Option<f64>,
    #[serde(rename = "Einf")]
    einf: Option<f64>,
    #[serde(rename = "HS")]
    hs: Option<f64>,
    #[serde(rename = "DSS1")]
    dss1: Option<f64>,
}

#[cfg(test)]
mod tests {
    mod construction;
    mod limits;
    mod parsing;
}
