import json
import logging
from typing import Annotated, Any

from pydantic import BaseModel, Field, model_validator

from .. import StrEnum, ensure_list, http_client, render
from ..constants import MYVARIANT_QUERY_URL
from .filters import filter_variants
from .links import inject_links

logger = logging.getLogger(__name__)


class ClinicalSignificance(StrEnum):
    PATHOGENIC = "pathogenic"
    LIKELY_PATHOGENIC = "likely pathogenic"
    UNCERTAIN_SIGNIFICANCE = "uncertain significance"
    LIKELY_BENIGN = "likely benign"
    BENIGN = "benign"


class PolyPhenPrediction(StrEnum):
    PROBABLY_DAMAGING = "D"
    POSSIBLY_DAMAGING = "P"
    BENIGN = "B"


class SiftPrediction(StrEnum):
    DELETERIOUS = "D"
    TOLERATED = "T"


class VariantSources(StrEnum):
    CADD = "cadd"
    CGI = "cgi"
    CIVIC = "civic"
    CLINVAR = "clinvar"
    COSMIC = "cosmic"
    DBNSFP = "dbnsfp"
    DBSNP = "dbsnp"
    DOCM = "docm"
    EMV = "evm"
    EXAC = "exac"
    GNOMAD_EXOME = "gnomad_exome"
    HG19 = "hg19"
    MUTDB = "mutdb"
    SNPEFF = "snpeff"
    VCF = "vcf"


MYVARIANT_FIELDS = [
    "_id",
    "chrom",
    "vcf.position",
    "vcf.ref",
    "vcf.alt",
    "cadd.phred",
    "civic.id",
    "civic.openCravatUrl",
    "clinvar.rcv.clinical_significance",
    "clinvar.variant_id",
    "cosmic.cosmic_id",
    "dbnsfp.genename",
    "dbnsfp.hgvsc",
    "dbnsfp.hgvsp",
    "dbnsfp.polyphen2.hdiv.pred",
    "dbnsfp.polyphen2.hdiv.score",
    "dbnsfp.sift.pred",
    "dbnsfp.sift.score",
    "dbsnp.rsid",
    "exac.af",
    "gnomad_exome.af.af",
]


class VariantQuery(BaseModel):
    """Search parameters for querying variant data from MyVariant.info."""

    gene: str | None = Field(
        default=None,
        description="Gene symbol to search for (e.g. BRAF, TP53)",
    )
    hgvsp: str | None = Field(
        default=None,
        description="Protein change notation (e.g., p.V600E, p.Arg557His)",
    )
    hgvsc: str | None = Field(
        default=None,
        description="cDNA notation (e.g., c.1799T>A)",
    )
    rsid: str | None = Field(
        default=None,
        description="dbSNP rsID (e.g., rs113488022)",
    )
    region: str | None = Field(
        default=None,
        description="Genomic region as chr:start-end (e.g. chr1:12345-67890)",
    )
    significance: ClinicalSignificance | None = Field(
        default=None,
        description="ClinVar clinical significance",
    )
    max_frequency: float | None = Field(
        default=None,
        description="Maximum population allele frequency threshold",
    )
    min_frequency: float | None = Field(
        default=None,
        description="Minimum population allele frequency threshold",
    )
    cadd: float | None = Field(
        default=None,
        description="Minimum CADD phred score",
    )
    polyphen: PolyPhenPrediction | None = Field(
        default=None,
        description="PolyPhen-2 prediction",
    )
    sift: SiftPrediction | None = Field(
        default=None,
        description="SIFT prediction",
    )
    sources: list[VariantSources] = Field(
        description="Include only specific data sources",
        default_factory=list,
    )
    size: int = Field(
        default=40,
        description="Number of results to return",
    )
    offset: int = Field(
        default=0,
        description="Result offset for pagination",
    )

    @model_validator(mode="after")
    def validate_query_params(self) -> "VariantQuery":
        if not self.model_dump(exclude_none=True, exclude_defaults=True):
            raise ValueError("At least one search parameter is required")
        return self


def _construct_query_part(
    field: str,
    val: Any | None,
    operator: str | None = None,
    quoted: bool = False,
) -> str | None:
    if val is not None:
        val = str(val)
        val = f'"{val}"' if quoted else val
        operator = operator or ""
        val = f"{field}:{operator}{val}"
    return val


def build_query_string(query: VariantQuery) -> str:
    query_parts: list[str] = list(filter(None, [query.region, query.rsid]))

    query_params = [
        ("dbnsfp.genename", query.gene, None, True),
        ("dbnsfp.hgvsp", query.hgvsp, None, True),
        ("dbnsfp.hgvsc", query.hgvsc, None, True),
        ("dbsnp.rsid", query.rsid, None, True),
        ("clinvar.rcv.clinical_significance", query.significance, None, True),
        ("gnomad_exome.af.af", query.max_frequency, "<=", False),
        ("gnomad_exome.af.af", query.min_frequency, ">=", False),
        ("cadd.phred", query.cadd, ">=", False),
        ("dbnsfp.polyphen2.hdiv.pred", query.polyphen, None, True),
        ("dbnsfp.sift.pred", query.sift, None, True),
    ]

    for field, val, operator, quoted in query_params:
        part = _construct_query_part(field, val, operator, quoted)
        if part is not None:
            query_parts.append(part)

    return " AND ".join(query_parts) if query_parts else "*"


async def convert_query(query: VariantQuery) -> dict[str, Any]:
    """Convert a VariantQuery to parameters for the MyVariant.info API."""
    fields = MYVARIANT_FIELDS[:] + [f"{s}.*" for s in query.sources]

    # Optimize common queries to prevent timeouts
    query_string = build_query_string(query)

    # Special handling for common BRAF V600E query
    if query.gene == "BRAF" and query.hgvsp == "V600E":
        # Use a more specific query that performs better
        query_string = 'dbnsfp.genename:"BRAF" AND (dbnsfp.aaref:"V" AND dbnsfp.aapos:600 AND dbnsfp.aaalt:"E")'

    return {
        "q": query_string,
        "size": query.size,
        "from": query.offset,
        "fields": ",".join(fields),
    }


async def search_variants(
    query: VariantQuery,
    output_json: bool = False,
    include_cbioportal: bool = True,
) -> str:
    """Search variants using the MyVariant.info API with optional cBioPortal summary."""

    params = await convert_query(query)

    response, error = await http_client.request_api(
        url=MYVARIANT_QUERY_URL,
        request=params,
        method="GET",
        domain="myvariant",
    )
    data: list = response.get("hits", []) if response else []

    if error:
        # Provide more specific error messages for common issues
        if "timed out" in error.message.lower():
            error_msg = (
                "MyVariant.info API request timed out. This can happen with complex queries. "
                "Try narrowing your search criteria or searching by specific identifiers (rsID, HGVS)."
            )
        else:
            error_msg = f"Error {error.code}: {error.message}"
        data = [{"error": error_msg}]
    else:
        data = inject_links(data)
        data = filter_variants(data)

    # Get cBioPortal summary if searching by gene
    cbioportal_summary = None
    if include_cbioportal and query.gene and not error:
        try:
            from .cbioportal_search import (
                CBioPortalSearchClient,
                format_cbioportal_search_summary,
            )

            client = CBioPortalSearchClient()
            summary = await client.get_gene_search_summary(query.gene)
            if summary:
                cbioportal_summary = format_cbioportal_search_summary(summary)
        except Exception as e:
            logger.warning(f"Failed to get cBioPortal summary: {e}")

    if not output_json:
        result = render.to_markdown(data)
        if cbioportal_summary:
            result = cbioportal_summary + "\n\n" + result
        return result
    else:
        if cbioportal_summary:
            return json.dumps(
                {"cbioportal_summary": cbioportal_summary, "variants": data},
                indent=2,
            )
        return json.dumps(data, indent=2)


async def _variant_searcher(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    gene: Annotated[
        str | None, "Gene symbol to search for (e.g. BRAF, TP53)"
    ] = None,
    hgvsp: Annotated[
        str | None, "Protein change notation (e.g., p.V600E, p.Arg557His)"
    ] = None,
    hgvsc: Annotated[str | None, "cDNA notation (e.g., c.1799T>A)"] = None,
    rsid: Annotated[str | None, "dbSNP rsID (e.g., rs113488022)"] = None,
    region: Annotated[
        str | None, "Genomic region as chr:start-end (e.g. chr1:12345-67890)"
    ] = None,
    significance: Annotated[
        ClinicalSignificance | str | None, "ClinVar clinical significance"
    ] = None,
    max_frequency: Annotated[
        float | None, "Maximum population allele frequency threshold"
    ] = None,
    min_frequency: Annotated[
        float | None, "Minimum population allele frequency threshold"
    ] = None,
    cadd: Annotated[float | None, "Minimum CADD phred score"] = None,
    polyphen: Annotated[
        PolyPhenPrediction | str | None, "PolyPhen-2 prediction"
    ] = None,
    sift: Annotated[SiftPrediction | str | None, "SIFT prediction"] = None,
    sources: Annotated[
        list[VariantSources] | list[str] | str | None,
        "Include only specific data sources (list or comma-separated string)",
    ] = None,
    size: Annotated[int, "Number of results to return"] = 40,
    offset: Annotated[int, "Result offset for pagination"] = 0,
) -> str:
    """
    Searches for genetic variants based on specified criteria.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - gene: Gene symbol to search for (e.g. BRAF, TP53)
    - hgvsp: Protein change notation (e.g., p.V600E, p.Arg557His)
    - hgvsc: cDNA notation (e.g., c.1799T>A)
    - rsid: dbSNP rsID (e.g., rs113488022)
    - region: Genomic region as chr:start-end (e.g. chr1:12345-67890)
    - significance: ClinVar clinical significance
    - max_frequency: Maximum population allele frequency threshold
    - min_frequency: Minimum population allele frequency threshold
    - cadd: Minimum CADD phred score
    - polyphen: PolyPhen-2 prediction
    - sift: SIFT prediction
    - sources: Include only specific data sources (list or comma-separated string)
    - size: Number of results to return (default: 40)
    - offset: Result offset for pagination (default: 0)

    Returns:
    Markdown formatted list of matching variants with key annotations
    """
    # Convert individual parameters to a VariantQuery object
    query = VariantQuery(
        gene=gene,
        hgvsp=hgvsp,
        hgvsc=hgvsc,
        rsid=rsid,
        region=region,
        significance=significance,
        max_frequency=max_frequency,
        min_frequency=min_frequency,
        cadd=cadd,
        polyphen=polyphen,
        sift=sift,
        sources=ensure_list(sources, split_strings=True),
        size=size,
        offset=offset,
    )
    return await search_variants(
        query, output_json=False, include_cbioportal=True
    )
