//! Narrow RefSNP coordinate resolution for variant population lookups.

use std::borrow::Cow;

use serde::Deserialize;

use crate::entities::variant::{GenomeBuild, normalize_genomic_coordinate};
use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{RequestBuilderSourceContextExt, RequestPlan, request_from_plan};

const DBSNP_BASE: &str = "https://api.ncbi.nlm.nih.gov/variation/v0";
const DBSNP_BASE_ENV: &str = "BIOMCP_DBSNP_BASE";

pub(crate) struct DbSnpClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub(crate) struct DbSnpCoordinate {
    pub(crate) id: String,
}

#[derive(Debug, Deserialize)]
struct RefSnpRecord {
    primary_snapshot_data: Option<RefSnpSnapshot>,
}

#[derive(Debug, Deserialize)]
struct RefSnpSnapshot {
    #[serde(default)]
    placements_with_allele: Vec<RefSnpPlacement>,
}

#[derive(Debug, Deserialize)]
struct RefSnpPlacement {
    placement_annot: Option<RefSnpPlacementAnnotation>,
    #[serde(default)]
    alleles: Vec<RefSnpAllele>,
}

#[derive(Debug, Deserialize)]
struct RefSnpPlacementAnnotation {
    #[serde(default)]
    seq_id_traits_by_assembly: Vec<RefSnpAssemblyTrait>,
}

#[derive(Debug, Deserialize)]
struct RefSnpAssemblyTrait {
    assembly_name: Option<String>,
    #[serde(default)]
    is_chromosome: bool,
}

#[derive(Debug, Deserialize)]
struct RefSnpAllele {
    allele: Option<RefSnpAlleleValue>,
}

#[derive(Debug, Deserialize)]
struct RefSnpAlleleValue {
    spdi: Option<RefSnpSpdi>,
}

#[derive(Debug, Deserialize)]
struct RefSnpSpdi {
    seq_id: String,
    position: u64,
    deleted_sequence: String,
    inserted_sequence: String,
}

fn numeric_rsid(rsid: &str) -> Result<&str, BioMcpError> {
    let rsid = rsid.trim();
    let Some(prefix) = rsid.get(..2) else {
        return Err(BioMcpError::InvalidArgument(
            "dbSNP requires an rs identifier with numeric digits".into(),
        ));
    };
    let Some(id) = prefix.eq_ignore_ascii_case("rs").then(|| &rsid[2..]) else {
        return Err(BioMcpError::InvalidArgument(
            "dbSNP requires an rs identifier with numeric digits".into(),
        ));
    };
    if id.is_empty() || !id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(BioMcpError::InvalidArgument(
            "dbSNP requires an rs identifier with numeric digits".into(),
        ));
    }
    Ok(id)
}

fn is_grch38_chromosome_placement(placement: &RefSnpPlacement) -> bool {
    placement
        .placement_annot
        .as_ref()
        .into_iter()
        .flat_map(|annotation| &annotation.seq_id_traits_by_assembly)
        .any(|trait_| {
            trait_.is_chromosome
                && trait_.assembly_name.as_deref().is_some_and(|name| {
                    name.eq_ignore_ascii_case("GRCh38")
                        || name
                            .get(..6)
                            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("GRCh38"))
                            && name.as_bytes().get(6) == Some(&b'.')
                })
        })
}

fn allele_change(id: &str) -> Option<&str> {
    let (_, coordinate) = id.split_once(":g.")?;
    let change = coordinate.get(coordinate.len().checked_sub(3)?..)?;
    (change.as_bytes().get(1) == Some(&b'>')
        && change
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 1 || matches!(byte, b'A' | b'C' | b'G' | b'T')))
    .then_some(change)
}

fn placement_coordinate(
    placement: &RefSnpPlacement,
    expected_coordinate: &str,
) -> Option<DbSnpCoordinate> {
    let expected_change = allele_change(expected_coordinate)?;
    let mut matches = placement
        .alleles
        .iter()
        .filter_map(|allele| allele.allele.as_ref()?.spdi.as_ref())
        .filter_map(|spdi| {
            let input = format!(
                "{}:{}:{}:{}",
                spdi.seq_id, spdi.position, spdi.deleted_sequence, spdi.inserted_sequence
            );
            normalize_genomic_coordinate(&input).ok().flatten()
        })
        .filter(|coordinate| {
            coordinate.genome_build == Some(GenomeBuild::Grch38)
                && allele_change(&coordinate.id) == Some(expected_change)
        })
        .map(|coordinate| coordinate.id)
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();
    (matches.len() == 1).then(|| DbSnpCoordinate {
        id: matches.remove(0),
    })
}

impl DbSnpClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(DBSNP_BASE, DBSNP_BASE_ENV),
        })
    }

    pub(crate) fn refsnp_plan(rsid: &str) -> Result<RequestPlan, BioMcpError> {
        Ok(RequestPlan::get(format!("refsnp/{}", numeric_rsid(rsid)?)))
    }

    async fn refsnp(&self, rsid: &str) -> Result<RefSnpRecord, BioMcpError> {
        let plan = Self::refsnp_plan(rsid)?;
        let response = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            &plan,
        ))
        .send_with_source_context(SourceContext::retry(SourceProvider::DBSNP))
        .await?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned();
        let body = crate::sources::read_limited_source_body(
            response,
            SourceContext::narrow(SourceProvider::DBSNP),
        )
        .await?;
        crate::sources::decode_json(
            SourceContext::retry(SourceProvider::DBSNP),
            status,
            content_type.as_ref(),
            &body,
            true,
        )
    }

    pub(crate) async fn resolve_grch38_coordinate(
        &self,
        rsid: &str,
        expected_coordinate: &str,
    ) -> Result<Option<DbSnpCoordinate>, BioMcpError> {
        let record = self.refsnp(rsid).await?;
        let placements = record
            .primary_snapshot_data
            .into_iter()
            .flat_map(|snapshot| snapshot.placements_with_allele)
            .filter(is_grch38_chromosome_placement)
            .collect::<Vec<_>>();
        if placements.len() != 1 {
            return Ok(None);
        }
        Ok(placement_coordinate(&placements[0], expected_coordinate))
    }
}
