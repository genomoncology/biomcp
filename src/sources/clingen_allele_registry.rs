use std::borrow::Cow;

use reqwest::{StatusCode, Url};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::entities::variant::{
    CarAliasCollection, CarNormalizationBatchResponse, CarNormalizationItem,
    CarNormalizationStatus, CarProvenance,
};
use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{RequestBuilderSourceContextExt, RequestPlan, request_from_plan};

const CAR_BASE: &str = "https://reg.genome.network";
const CAR_BASE_ENV: &str = "BIOMCP_CLINGEN_CAR_BASE";
const CAR_BODY_LIMIT: usize = 256 * 1024;
const CAR_BATCH_BODY_LIMIT: usize = 2 * 1024 * 1024;
const CAR_FIELDS: &str = "none @id communityStandardTitle genomicAlleles transcriptAlleles.MANE externalRecords.dbSNP externalRecords.ClinVarVariations";
type ProjectedAliases = (
    Option<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    CarAliasCollection,
);

pub(crate) struct ClinGenAlleleRegistryClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

fn response_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn with_response_sha256(
    mut response: CarNormalizationBatchResponse,
    bytes: &[u8],
) -> CarNormalizationBatchResponse {
    let hash = response_sha256(bytes);
    for item in &mut response.items {
        item.provenance.response_sha256 = Some(hash.clone());
    }
    response
}

impl ClinGenAlleleRegistryClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        let base = crate::sources::env_base(CAR_BASE, CAR_BASE_ENV);
        let parsed = Url::parse(base.as_ref()).map_err(|_| {
            BioMcpError::InvalidArgument("invalid ClinGen Allele Registry base URL".into())
        })?;
        let policy = crate::sources::provider_url_policy::ProviderUrlPolicy::clingen_car(&parsed)?;
        Ok(Self {
            client: crate::sources::provider_url_client(&policy)?,
            base,
        })
    }

    pub(crate) async fn new_with_deadline(
        deadline: &crate::sources::VariantArticleDeadline,
    ) -> Result<Self, BioMcpError> {
        let base = crate::sources::env_base(CAR_BASE, CAR_BASE_ENV);
        let parsed = Url::parse(base.as_ref()).map_err(|_| {
            BioMcpError::InvalidArgument("invalid ClinGen Allele Registry base URL".into())
        })?;
        let policy = crate::sources::provider_url_policy::ProviderUrlPolicy::clingen_car(&parsed)?;
        Ok(Self {
            client: crate::sources::provider_url_client_with_deadline(&policy, deadline).await?,
            base,
        })
    }

    #[cfg(test)]
    pub(crate) fn with_test_client(
        client: reqwest_middleware::ClientWithMiddleware,
        base: &'static str,
    ) -> Self {
        Self {
            client,
            base: Cow::Borrowed(base),
        }
    }

    pub(crate) fn normalize_plan(hgvs: &str) -> RequestPlan {
        RequestPlan::get("allele")
            .query("hgvs", hgvs)
            .query("fields", CAR_FIELDS)
    }

    pub(crate) fn caid_plan(caid: &str) -> RequestPlan {
        RequestPlan::get(caid_path(caid)).query("fields", CAR_FIELDS)
    }

    pub(crate) fn normalize_batch_plan(inputs: &[String]) -> RequestPlan {
        RequestPlan::post("alleles")
            .query("file", "hgvs")
            .query("fields", CAR_FIELDS)
            .header(
                reqwest::header::CONTENT_TYPE.as_str(),
                "text/plain; charset=utf-8",
            )
            .text(format!("{}\n", inputs.join("\n")))
    }

    pub(crate) async fn normalize(&self, input: &str) -> Result<CarNormalizationItem, BioMcpError> {
        let response = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            &Self::normalize_plan(input),
        ))
        .send_with_source_context(SourceContext::retry(SourceProvider::CLINGEN_CAR))
        .await;
        let Ok(response) = response else {
            return Ok(empty(
                input,
                CarNormalizationStatus::Unavailable,
                false,
                Some("ClinGen Allele Registry request failed".into()),
                None,
            ));
        };
        let status = response.status();
        let version = response
            .headers()
            .get("X-CAR-Version")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_CAR),
            CAR_BODY_LIMIT,
        )
        .await;
        match bytes {
            Ok(bytes) => {
                let mut item = decode_normalize_response(input, status, version, &bytes);
                item.provenance.response_sha256 = Some(response_sha256(&bytes));
                Ok(item)
            }
            Err(_) => Ok(empty(
                input,
                CarNormalizationStatus::Unavailable,
                false,
                Some("ClinGen Allele Registry response was unavailable".into()),
                version,
            )),
        }
    }

    pub(crate) async fn gene_for_caid(&self, caid: &str) -> Option<String> {
        let response = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            &Self::caid_plan(caid),
        ))
        .send_with_source_context(SourceContext::retry(SourceProvider::CLINGEN_CAR))
        .await
        .ok()?;
        let status = response.status();
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_CAR),
            CAR_BODY_LIMIT,
        )
        .await
        .ok()?;
        gene_from_caid_response(caid, status, &bytes)
    }

    pub(crate) async fn normalize_batch(
        &self,
        inputs: &[String],
    ) -> Result<CarNormalizationBatchResponse, BioMcpError> {
        let response = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            &Self::normalize_batch_plan(inputs),
        ))
        .send_with_source_context(SourceContext::retry(SourceProvider::CLINGEN_CAR))
        .await;
        let Ok(response) = response else {
            return Ok(unavailable_batch(inputs, None));
        };
        let status = response.status();
        let version = response
            .headers()
            .get("X-CAR-Version")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_CAR),
            CAR_BATCH_BODY_LIMIT,
        )
        .await;
        let Ok(bytes) = bytes else {
            return Ok(unavailable_batch(inputs, version));
        };
        if !status.is_success() {
            return Ok(with_response_sha256(
                unavailable_batch(inputs, version),
                &bytes,
            ));
        }
        Ok(with_response_sha256(
            decode_batch_response(inputs, status, version, &bytes),
            &bytes,
        ))
    }
}

fn caid_path(caid: &str) -> String {
    let mut url = Url::parse(CAR_BASE).expect("static CAR origin is valid");
    url.path_segments_mut()
        .expect("static CAR origin accepts path segments")
        .extend(["allele", caid]);
    url.path().trim_start_matches('/').to_owned()
}

fn gene_from_caid_response(caid: &str, status: StatusCode, bytes: &[u8]) -> Option<String> {
    if !status.is_success() {
        return None;
    }
    let value = serde_json::from_slice::<Value>(bytes).ok()?;
    let identity = value.get("@id").and_then(Value::as_str)?;
    if identity.rsplit('/').next() != Some(caid) {
        return None;
    }
    let titles = value.get("communityStandardTitle")?.as_array()?;
    let [title] = titles.as_slice() else {
        return None;
    };
    gene_from_title(title.as_str()?)
}

fn gene_from_title(title: &str) -> Option<String> {
    let mut genes = title
        .split('(')
        .skip(1)
        .filter_map(|component| component.split_once(')').map(|(gene, _)| gene))
        .filter(|gene| crate::sources::is_valid_gene_symbol(gene));
    let gene = genes.next()?;
    genes.next().is_none().then(|| gene.to_owned())
}

fn decode_batch_response(
    inputs: &[String],
    status: StatusCode,
    version: Option<String>,
    bytes: &[u8],
) -> CarNormalizationBatchResponse {
    let rows = serde_json::from_slice::<Vec<Value>>(bytes).ok();
    let items: Vec<CarNormalizationItem> = rows
        .filter(|rows| rows.len() == inputs.len())
        .map(|rows| {
            rows.iter()
                .zip(inputs)
                .map(|(row, input)| {
                    decode_normalize_response(
                        input,
                        status,
                        version.clone(),
                        &serde_json::to_vec(row).unwrap_or_default(),
                    )
                })
                .collect()
        })
        .unwrap_or_else(|| {
            inputs
                .iter()
                .map(|input| {
                    empty(
                        input,
                        CarNormalizationStatus::Indeterminate,
                        false,
                        Some("CAR batch response did not preserve input positions".into()),
                        version.clone(),
                    )
                })
                .collect()
        });
    let complete = items.iter().all(|item| {
        !matches!(
            item.status,
            CarNormalizationStatus::Indeterminate | CarNormalizationStatus::Unavailable
        )
    });
    CarNormalizationBatchResponse {
        items,
        complete,
        provider: "ClinGen Allele Registry".into(),
    }
}

fn unavailable_batch(inputs: &[String], version: Option<String>) -> CarNormalizationBatchResponse {
    CarNormalizationBatchResponse {
        items: inputs
            .iter()
            .map(|input| {
                empty(
                    input,
                    CarNormalizationStatus::Unavailable,
                    false,
                    Some("ClinGen Allele Registry response was unavailable".into()),
                    version.clone(),
                )
            })
            .collect(),
        complete: false,
        provider: "ClinGen Allele Registry".into(),
    }
}

fn empty(
    input: &str,
    status: CarNormalizationStatus,
    exhaustive: bool,
    error: Option<String>,
    version: Option<String>,
) -> CarNormalizationItem {
    CarNormalizationItem {
        input: input.to_owned(),
        status,
        exhaustive,
        caid: None,
        canonical_title: None,
        genomic_aliases: CarAliasCollection::default(),
        transcript_aliases: CarAliasCollection::default(),
        protein_aliases: CarAliasCollection::default(),
        external_ids: CarAliasCollection::default(),
        source: "clingen_car".into(),
        query: input.to_owned(),
        warnings: Vec::new(),
        error,
        provenance: CarProvenance {
            request_template_version: "1".into(),
            car_version: version,
            response_sha256: None,
        },
    }
}

pub(crate) fn decode_normalize_response(
    input: &str,
    status: StatusCode,
    version: Option<String>,
    bytes: &[u8],
) -> CarNormalizationItem {
    if status == StatusCode::BAD_REQUEST
        && serde_json::from_slice::<Value>(bytes)
            .ok()
            .is_some_and(|value| value.to_string().contains("HgvsParsingError"))
    {
        return empty(
            input,
            CarNormalizationStatus::Invalid,
            true,
            Some("CAR rejected the HGVS input".into()),
            version,
        );
    }
    if !status.is_success() {
        return empty(
            input,
            CarNormalizationStatus::Unavailable,
            false,
            Some(format!("CAR returned HTTP {}", status.as_u16())),
            version,
        );
    }
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return empty(
            input,
            CarNormalizationStatus::Indeterminate,
            false,
            Some("CAR returned invalid JSON".into()),
            version,
        );
    };
    let Some(id) = value.get("@id").and_then(Value::as_str) else {
        return empty(
            input,
            CarNormalizationStatus::Indeterminate,
            false,
            Some("CAR response lacked a canonical identity".into()),
            version,
        );
    };
    let Ok((canonical_title, genomic, transcripts, proteins, external)) = projected_aliases(&value)
    else {
        return empty(
            input,
            CarNormalizationStatus::Indeterminate,
            false,
            Some("CAR response did not match the projected schema".into()),
            version,
        );
    };
    if id == "_:CA" {
        return empty(input, CarNormalizationStatus::NotFound, true, None, version);
    }
    let caid = id
        .rsplit('/')
        .next()
        .filter(|value| {
            value.strip_prefix("CA").is_some_and(|digits| {
                !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
            })
        })
        .map(str::to_owned);
    let Some(caid) = caid else {
        return empty(
            input,
            CarNormalizationStatus::Indeterminate,
            false,
            Some("CAR response had an unrecognized canonical identity".into()),
            version,
        );
    };
    let mut item = empty(input, CarNormalizationStatus::Resolved, true, None, version);
    item.caid = Some(caid);
    item.canonical_title = canonical_title;
    item.genomic_aliases = CarAliasCollection::bounded(genomic, 12);
    item.transcript_aliases = CarAliasCollection::bounded(transcripts, 12);
    item.protein_aliases = CarAliasCollection::bounded(proteins, 8);
    item.external_ids = external;
    item
}

fn projected_aliases(value: &Value) -> Result<ProjectedAliases, ()> {
    let title = match value.get("communityStandardTitle") {
        Some(Value::Array(values)) => match values.first() {
            Some(value) => Some(value.as_str().ok_or(())?.to_owned()),
            None => None,
        },
        Some(_) => return Err(()),
        None => None,
    };
    let mut genomic = Vec::new();
    if let Some(alleles) = value.get("genomicAlleles") {
        for allele in alleles.as_array().ok_or(())? {
            let rank = match allele.get("referenceGenome") {
                Some(Value::String(reference)) => match reference.as_str() {
                    "GRCh38" => 0,
                    "GRCh37" => 1,
                    "NCBI36" => 2,
                    _ => 3,
                },
                None => 3,
                Some(_) => return Err(()),
            };
            let hgvs = allele.get("hgvs").and_then(Value::as_array).ok_or(())?;
            for alias in hgvs {
                genomic.push((rank, alias.as_str().ok_or(())?.to_owned()));
            }
        }
    }
    genomic.sort();
    let mut transcripts = Vec::new();
    let mut proteins = Vec::new();
    if let Some(alleles) = value.get("transcriptAlleles") {
        for allele in alleles.as_array().ok_or(())? {
            let mane = allele.get("MANE").ok_or(())?;
            let status = mane.get("maneStatus").and_then(Value::as_str).ok_or(())?;
            mane.get("maneVersion").and_then(Value::as_str).ok_or(())?;
            let rank = match status {
                "MANE Select" => 0,
                "MANE Plus Clinical" => 1,
                _ => 2,
            };
            if let Some(hgvs) = mane.pointer("/nucleotide/RefSeq/hgvs") {
                transcripts.push((rank, hgvs.as_str().ok_or(())?.to_owned()));
            }
            if let Some(hgvs) = mane.pointer("/protein/RefSeq/hgvs") {
                proteins.push(hgvs.as_str().ok_or(())?.to_owned());
            }
        }
    }
    transcripts.sort();
    proteins.sort();
    let external_records = match value.get("externalRecords") {
        Some(records) => Some(records.as_object().ok_or(())?),
        None => None,
    };
    let mut external = Vec::new();
    let mut source_count = 0;
    let mut truncated = false;
    for (path, prefix) in [("dbSNP", "rs"), ("ClinVarVariations", "ClinVar:")] {
        let mut ids = Vec::new();
        if let Some(records) = external_records.and_then(|records| records.get(path)) {
            for record in records.as_array().ok_or(())? {
                let key = if path == "dbSNP" { "rs" } else { "variationId" };
                ids.push(record.get(key).and_then(Value::as_u64).ok_or(())?);
            }
        }
        ids.sort_unstable();
        ids.dedup();
        source_count += ids.len();
        truncated |= ids.len() > 8;
        external.extend(ids.into_iter().take(8).map(|id| format!("{prefix}{id}")));
    }
    Ok((
        title,
        genomic.into_iter().map(|(_, alias)| alias).collect(),
        transcripts.into_iter().map(|(_, alias)| alias).collect(),
        proteins,
        CarAliasCollection {
            values: external,
            source_count,
            truncated,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::{HttpMethod, RequestBody};

    #[test]
    fn received_response_hashes_preserve_exact_body_bytes() {
        assert_eq!(
            response_sha256(b"CAR response"),
            "23930aafbb13d87cda75bba884ca09a706e4112a029c71416fc0b669fedae75d"
        );
        assert_ne!(
            response_sha256(b"CAR response"),
            response_sha256(b"CAR response!")
        );
    }

    #[test]
    fn received_batch_response_hashes_preserve_exact_body_bytes() {
        let body = br#"[{"@id":"_:CA"}]"#;
        let response = with_response_sha256(
            unavailable_batch(&["NM_000546.6:c.215C>G".into()], None),
            body,
        );
        assert_eq!(
            response.items[0].provenance.response_sha256.as_deref(),
            Some(response_sha256(body).as_str())
        );
    }

    #[test]
    fn direct_plan_uses_only_the_projected_read_route() {
        let plan = ClinGenAlleleRegistryClient::normalize_plan("NM_000546.6:c.215C>G");
        assert_eq!(plan.method, HttpMethod::Get);
        assert_eq!(plan.path, "allele");
        assert_eq!(plan.query_value("hgvs"), Some("NM_000546.6:c.215C>G"));
        assert_eq!(plan.query_value("fields"), Some(CAR_FIELDS));
    }

    #[test]
    fn batch_plan_uses_post_and_preserves_input_order_and_duplicates() {
        let inputs = vec![
            "NM_000546.6:c.215C>G".into(),
            "NM_000038.6:c.847C>G".into(),
            "NM_000546.6:c.215C>G".into(),
        ];
        let plan = ClinGenAlleleRegistryClient::normalize_batch_plan(&inputs);

        assert_eq!(plan.method, HttpMethod::Post);
        assert_eq!(plan.path, "alleles");
        assert_eq!(plan.query_value("file"), Some("hgvs"));
        assert_eq!(plan.query_value("fields"), Some(CAR_FIELDS));
        assert_eq!(
            plan.header_value("content-type"),
            Some("text/plain; charset=utf-8")
        );
        assert_eq!(
            plan.body,
            RequestBody::Text(
                "NM_000546.6:c.215C>G\nNM_000038.6:c.847C>G\nNM_000546.6:c.215C>G\n".into()
            )
        );
    }

    #[test]
    fn batch_cardinality_mismatch_is_incomplete() {
        let inputs = vec!["NM_000546.6:c.215C>G".into(), "NM_000038.6:c.847C>G".into()];
        let response = decode_batch_response(
            &inputs,
            StatusCode::OK,
            None,
            br#"[{"@id":"https://reg.genome.network/allele/CA123"}]"#,
        );

        assert!(!response.complete);
        assert_eq!(
            response
                .items
                .iter()
                .map(|item| &item.input)
                .collect::<Vec<_>>(),
            inputs.iter().collect::<Vec<_>>()
        );
        assert!(response.items.iter().all(|item| {
            item.status == CarNormalizationStatus::Indeterminate
                && item.error.as_deref()
                    == Some("CAR batch response did not preserve input positions")
        }));
    }

    #[test]
    fn decoder_orders_aliases_and_rejects_schema_drift() {
        let item = decode_normalize_response(
            "NM_1.1:c.1A>G",
            StatusCode::OK,
            None,
            include_bytes!("../../testdata/sources/clingen_allele_registry/resolved.json"),
        );
        assert_eq!(item.status, CarNormalizationStatus::Resolved);
        assert_eq!(item.caid.as_deref(), Some("CA123"));
        assert_eq!(item.genomic_aliases.values[0], "NC_000017.11:g.1A>G");
        assert_eq!(item.transcript_aliases.values[0], "NM_1.1:c.1A>G");
        assert_eq!(item.external_ids.values, vec!["rs2", "rs20", "ClinVar:10"]);

        let drift = decode_normalize_response(
            "NM_1.1:c.1A>G",
            StatusCode::OK,
            None,
            br#"{"@id":"CA123","genomicAlleles":"wrong"}"#,
        );
        assert_eq!(drift.status, CarNormalizationStatus::Indeterminate);
    }

    #[test]
    fn blank_node_with_malformed_projected_fact_is_indeterminate() {
        let item = decode_normalize_response(
            "NM_1.1:c.1A>G",
            StatusCode::OK,
            None,
            br#"{"@id":"_:CA","genomicAlleles":"wrong"}"#,
        );

        assert_eq!(item.status, CarNormalizationStatus::Indeterminate);
        assert!(!item.exhaustive);
        assert!(item.caid.is_none());
        assert!(item.genomic_aliases.values.is_empty());

        let item = decode_normalize_response(
            "NM_1.1:c.1A>G",
            StatusCode::OK,
            None,
            br#"{"@id":"_:CA","externalRecords":"wrong"}"#,
        );

        assert_eq!(item.status, CarNormalizationStatus::Indeterminate);
        assert!(!item.exhaustive);
        assert!(item.caid.is_none());
        assert!(item.external_ids.values.is_empty());
    }

    #[test]
    fn external_ids_keep_full_source_metadata_before_per_source_caps() {
        let item = decode_normalize_response(
            "NM_1.1:c.1A>G",
            StatusCode::OK,
            None,
            br#"{
                "@id":"https://reg.genome.network/allele/CA123",
                "externalRecords": {
                    "dbSNP": [{"rs":9},{"rs":2},{"rs":2},{"rs":1},{"rs":8},{"rs":3},{"rs":7},{"rs":4},{"rs":6},{"rs":5}],
                    "ClinVarVariations": [{"variationId":19},{"variationId":12},{"variationId":12},{"variationId":11},{"variationId":18},{"variationId":13},{"variationId":17},{"variationId":14},{"variationId":16},{"variationId":15}]
                }
            }"#,
        );

        assert_eq!(item.status, CarNormalizationStatus::Resolved);
        assert_eq!(item.external_ids.values.len(), 16);
        assert_eq!(item.external_ids.source_count, 18);
        assert!(item.external_ids.truncated);
        assert_eq!(
            item.external_ids.values,
            [
                "rs1",
                "rs2",
                "rs3",
                "rs4",
                "rs5",
                "rs6",
                "rs7",
                "rs8",
                "ClinVar:11",
                "ClinVar:12",
                "ClinVar:13",
                "ClinVar:14",
                "ClinVar:15",
                "ClinVar:16",
                "ClinVar:17",
                "ClinVar:18",
            ]
        );
    }

    #[test]
    fn decoder_does_not_accept_an_empty_caid() {
        let item =
            decode_normalize_response("NM_1.1:c.1A>G", StatusCode::OK, None, br#"{"@id":"CA"}"#);
        assert_eq!(item.status, CarNormalizationStatus::Indeterminate);
    }

    #[test]
    fn receipt_backed_car_capture_decodes_a_resolved_transcript_identity() {
        let input = "NM_000546.6:c.215C>G";
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_allele_registry/tp53-nm_000546.6-c.215c-g.json"
        ))
        .expect("ticket 662 must add the recorded CAR transcript response");

        let item = decode_normalize_response(
            input,
            StatusCode::OK,
            Some("captured-car-version".into()),
            &bytes,
        );

        assert_eq!(item.status, CarNormalizationStatus::Resolved);
        assert!(item.exhaustive);
        assert!(item.caid.as_deref().is_some_and(|caid| {
            caid.strip_prefix("CA").is_some_and(|digits| {
                !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
            })
        }));
        assert_eq!(item.query, input);
        assert_eq!(item.source, "clingen_car");
        assert!(!item.transcript_aliases.values.is_empty());
        assert_eq!(
            item.provenance.car_version.as_deref(),
            Some("captured-car-version")
        );

        let empty = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_allele_registry/tp53-nm_000546.6-c.215c-g-empty.json"
        ))
        .expect("ticket 662 must add the recorded CAR empty response");
        let empty = decode_normalize_response(input, StatusCode::OK, None, &empty);
        assert_eq!(empty.status, CarNormalizationStatus::Indeterminate);
        assert!(!empty.exhaustive);
        assert!(empty.caid.is_none());

        let malformed = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_allele_registry/tp53-nm_000546.6-c.215c-g-malformed.json"
        ))
        .expect("ticket 662 must add the recorded CAR malformed response");
        let malformed =
            decode_normalize_response("not-hgvs", StatusCode::BAD_REQUEST, None, &malformed);
        assert_eq!(malformed.status, CarNormalizationStatus::Invalid);
        assert!(malformed.exhaustive);
        assert!(malformed.caid.is_none());
    }
}
