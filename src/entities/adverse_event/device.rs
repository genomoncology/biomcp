use super::{DeviceEventSearchResult, yyyymmdd_from_date};
use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::openfda::OpenFdaClient;
use crate::transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceEventSeriousness {
    Any,
    Death,
    Injury,
}

impl DeviceEventSeriousness {
    pub fn from_flag(raw: &str) -> Result<Self, BioMcpError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "any" => Ok(Self::Any),
            "death" => Ok(Self::Death),
            "injury" => Ok(Self::Injury),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown --serious value '{other}' for --type device. Expected one of: any, death, injury"
            ))),
        }
    }

    fn query_term(self) -> &'static str {
        match self {
            Self::Any => "(event_type:\"Death\" OR event_type:\"Injury\")",
            Self::Death => "event_type:\"Death\"",
            Self::Injury => "event_type:\"Injury\"",
        }
    }

    fn summary_value(self) -> &'static str {
        match self {
            Self::Any => "death_or_injury",
            Self::Death => "death",
            Self::Injury => "injury",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeviceEventSearchFilters {
    pub device: Option<String>,
    pub manufacturer: Option<String>,
    pub product_code: Option<String>,
    pub serious: Option<DeviceEventSeriousness>,
    pub since: Option<String>,
}

pub(super) fn build_device_query(
    filters: &DeviceEventSearchFilters,
) -> Result<String, BioMcpError> {
    let device = filters
        .device
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let manufacturer = filters
        .manufacturer
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let product_code = filters
        .product_code
        .as_deref()
        .and_then(normalize_product_code);

    if device.is_none() && manufacturer.is_none() && product_code.is_none() {
        return Err(BioMcpError::InvalidArgument(
            "At least one device filter is required (--device, --manufacturer, or --product-code)."
                .into(),
        ));
    }

    let mut terms = Vec::new();
    if let Some(device) = device {
        let escaped = OpenFdaClient::escape_query_value(device);
        let name_query = if device.chars().any(char::is_whitespace) {
            format!("device.brand_name:\"{escaped}\" OR device.generic_name:\"{escaped}\"")
        } else {
            format!("device.brand_name:*{escaped}* OR device.generic_name:*{escaped}*")
        };
        terms.push(format!("({name_query})"));
    }

    if let Some(manufacturer) = manufacturer {
        let escaped = OpenFdaClient::escape_query_value(manufacturer);
        let manufacturer_query = if manufacturer.chars().any(char::is_whitespace) {
            format!("manufacturer_name:\"{escaped}\" OR device.manufacturer_d_name:\"{escaped}\"")
        } else {
            format!("manufacturer_name:*{escaped}* OR device.manufacturer_d_name:*{escaped}*")
        };
        terms.push(format!("({manufacturer_query})"));
    }

    if let Some(product_code) = product_code {
        terms.push(format!(
            "device.device_report_product_code:\"{}\"",
            OpenFdaClient::escape_query_value(&product_code)
        ));
    }
    if let Some(seriousness) = filters.serious {
        terms.push(seriousness.query_term().to_string());
    }
    if let Some(since) = filters.since.as_deref() {
        let yyyymmdd = yyyymmdd_from_date(since, false)?;
        terms.push(format!("date_received:[{yyyymmdd} TO *]"));
    }
    Ok(terms.join(" AND "))
}

fn normalize_product_code(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_uppercase();
    (!normalized.is_empty()).then_some(normalized)
}

pub async fn search_device_page(
    filters: &DeviceEventSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<DeviceEventSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let query = build_device_query(filters)?;
    let client = OpenFdaClient::new()?;
    let response = client.device_event_search(&query, limit, offset).await?;
    let Some(response) = response else {
        return Ok(SearchPage::offset(Vec::new(), Some(0)));
    };
    Ok(SearchPage::offset(
        response
            .results
            .iter()
            .map(transform::adverse_event::from_openfda_device_search_result)
            .collect(),
        Some(response.meta.results.total),
    ))
}

pub fn device_query_summary(filters: &DeviceEventSearchFilters) -> String {
    let mut parts = Vec::new();
    if let Some(device) = filters
        .device
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("device={device}"));
    }
    if let Some(manufacturer) = filters
        .manufacturer
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("manufacturer={manufacturer}"));
    }
    if let Some(code) = filters
        .product_code
        .as_deref()
        .and_then(normalize_product_code)
    {
        parts.push(format!("product_code={code}"));
    }
    if let Some(seriousness) = filters.serious {
        parts.push(format!("serious={}", seriousness.summary_value()));
    }
    if let Some(since) = filters
        .since
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("since={since}"));
    }
    parts.join(", ")
}
