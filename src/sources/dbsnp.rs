use std::borrow::Cow;

use serde::Deserialize;

use crate::entities::variant::{GenomeBuild, normalize_genomic_coordinate};
use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{RequestBuilderSourceContextExt, RequestPlan, request_from_plan};

const DBSNP_BASE: &str = "https://api.ncbi.nlm.nih.gov/variation/v0/beta";
const DBSNP_BASE_ENV: &str = "BIOMCP_DBSNP_BASE";

pub(crate) struct DbSnpClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbSnpCoordinate {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct RefSnpResponse {
    primary_snapshot_data: Option<PrimarySnapshotData>,
}

#[derive(Debug, Deserialize)]
struct PrimarySnapshotData {
    #[serde(default)]
    placements_with_allele: Vec<PlacementWithAllele>,
}

#[derive(Debug, Deserialize)]
struct PlacementWithAllele {
    placement_annot: Option<PlacementAnnotation>,
    #[serde(default)]
    alleles: Vec<PlacementAllele>,
}

#[derive(Debug, Deserialize)]
struct PlacementAnnotation {
    #[serde(default)]
    seq_id_traits_by_assembly: Vec<AssemblyTrait>,
}

#[derive(Debug, Deserialize)]
struct AssemblyTrait {
    assembly_name: Option<String>,
    #[serde(default)]
    is_chromosome: bool,
}

#[derive(Debug, Deserialize)]
struct PlacementAllele {
    allele: Option<Allele>,
}

#[derive(Debug, Deserialize)]
struct Allele {
    spdi: Option<Spdi>,
}

#[derive(Debug, Deserialize)]
struct Spdi {
    seq_id: Option<String>,
    position: Option<u64>,
    deleted_sequence: Option<String>,
    inserted_sequence: Option<String>,
}

impl DbSnpClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(DBSNP_BASE, DBSNP_BASE_ENV),
        })
    }

    pub(crate) fn refsnp_plan(rsid: &str) -> Result<RequestPlan, BioMcpError> {
        let rsid = rsid.trim();
        let Some((prefix, numeric_id)) = rsid.get(..2).zip(rsid.get(2..)) else {
            return Err(BioMcpError::InvalidArgument(
                "dbSNP requires an rsID".into(),
            ));
        };
        if !prefix.eq_ignore_ascii_case("rs")
            || numeric_id.is_empty()
            || numeric_id.len() > 20
            || !numeric_id.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(BioMcpError::InvalidArgument(
                "dbSNP requires an rsID".into(),
            ));
        }
        Ok(RequestPlan::get(format!("refsnp/{numeric_id}")))
    }

    pub(crate) async fn resolve_grch38(
        &self,
        rsid: &str,
        expected_variant_id: &str,
    ) -> Result<Option<DbSnpCoordinate>, BioMcpError> {
        let plan = Self::refsnp_plan(rsid)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let response = crate::sources::apply_cache_mode(req)
            .send_with_source_context(SourceContext::retry(SourceProvider::DBSNP))
            .await?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned();
        let bytes = crate::sources::read_limited_source_body(
            response,
            SourceContext::narrow(SourceProvider::DBSNP),
        )
        .await?;
        let response: RefSnpResponse = crate::sources::decode_json(
            SourceContext::retry(SourceProvider::DBSNP),
            status,
            content_type.as_ref(),
            &bytes,
            true,
        )?;
        Ok(select_grch38_coordinate(response, expected_variant_id))
    }
}

fn is_grch38_chromosome(annotation: &PlacementAnnotation) -> bool {
    annotation.seq_id_traits_by_assembly.iter().any(|trait_| {
        trait_.is_chromosome
            && trait_
                .assembly_name
                .as_deref()
                .map(str::trim)
                .is_some_and(|name| {
                    name.split_once('.')
                        .map_or(name, |(assembly, _)| assembly)
                        .eq_ignore_ascii_case("GRCh38")
                })
    })
}

fn allele_change(id: &str) -> Option<&str> {
    let (_, change) = id.split_once(":g.")?;
    let first_allele = change.find(|character: char| !character.is_ascii_digit())?;
    Some(&change[first_allele..])
}

fn select_grch38_coordinate(
    response: RefSnpResponse,
    expected_variant_id: &str,
) -> Option<DbSnpCoordinate> {
    let expected_change = allele_change(expected_variant_id)?;
    let mut candidates = Vec::new();

    for placement in response
        .primary_snapshot_data?
        .placements_with_allele
        .into_iter()
        .filter(|placement| {
            placement
                .placement_annot
                .as_ref()
                .is_some_and(is_grch38_chromosome)
        })
    {
        for allele in placement.alleles {
            let Some(spdi) = allele.allele.and_then(|allele| allele.spdi) else {
                continue;
            };
            let (Some(seq_id), Some(position), Some(deleted_sequence), Some(inserted_sequence)) = (
                spdi.seq_id,
                spdi.position,
                spdi.deleted_sequence,
                spdi.inserted_sequence,
            ) else {
                continue;
            };
            let spdi = format!("{seq_id}:{position}:{deleted_sequence}:{inserted_sequence}");
            let Ok(Some(coordinate)) = normalize_genomic_coordinate(&spdi) else {
                continue;
            };
            if coordinate.genome_build == Some(GenomeBuild::Grch38)
                && allele_change(&coordinate.id)
                    .is_some_and(|change| change.eq_ignore_ascii_case(expected_change))
            {
                candidates.push(coordinate.id);
            }
        }
    }

    (candidates.len() == 1).then(|| DbSnpCoordinate {
        id: candidates.pop().expect("one candidate"),
    })
}
