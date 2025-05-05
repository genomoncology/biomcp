import json
from typing import Any, Optional

from pydantic import BaseModel, Field, model_validator

from .. import StrEnum, const, http_client, mcp_app, render
from .filters import filter_variants
from .links import inject_links

# MyVariant.info API URL
MYVARIANT_QUERY_ENDPOINT = f"{const.MYVARIANT_BASE_URL}/query"


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

    gene: Optional[str] = Field(
        default=None,
        description="Gene symbol to search for (e.g. BRAF, TP53)",
    )
    hgvsp: Optional[str] = Field(
        default=None,
        description="Protein change notation (e.g., p.V600E, p.Arg557His)",
    )
    hgvsc: Optional[str] = Field(
        default=None,
        description="cDNA notation (e.g., c.1799T>A)",
    )
    rsid: Optional[str] = Field(
        default=None,
        description="dbSNP rsID (e.g., rs113488022)",
    )
    region: Optional[str] = Field(
        default=None,
        description="Genomic region as chr:start-end (e.g. chr1:12345-67890)",
    )
    significance: Optional[ClinicalSignificance] = Field(
        default=None,
        description="ClinVar clinical significance",
    )
    max_frequency: Optional[float] = Field(
        default=None,
        description="Maximum population allele frequency threshold",
    )
    min_frequency: Optional[float] = Field(
        default=None,
        description="Minimum population allele frequency threshold",
    )
    cadd: Optional[float] = Field(
        default=None,
        description="Minimum CADD phred score",
    )
    polyphen: Optional[PolyPhenPrediction] = Field(
        default=None,
        description="PolyPhen-2 prediction",
    )
    sift: Optional[SiftPrediction] = Field(
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

    return {
        "q": build_query_string(query),
        "size": query.size,
        "from": query.offset,
        "fields": ",".join(fields),
    }


async def search_variants(
    query: VariantQuery,
    output_json: bool = False,
) -> str:
    """Search variants using the MyVariant.info API."""

    params = await convert_query(query)

    response, error = await http_client.request_api(
        url=MYVARIANT_QUERY_ENDPOINT,
        request=params,
        method="GET",
    )
    data: list = response.get("hits", []) if response else []

    if error:
        data = [{"error": f"Error {error.code}: {error.message}"}]
    else:
        data = inject_links(data)
        data = filter_variants(data)

    if not output_json:
        return render.to_markdown(data)
    else:
        return json.dumps(data, indent=2)


@mcp_app.tool()
async def variant_searcher(
    gene=None,
    hgvsp=None,
    hgvsc=None,
    rsid=None,
    region=None,
    significance=None,
    max_frequency=None,
    min_frequency=None,
    cadd=None,
    polyphen=None,
    sift=None,
    sources=None,
    size=40,
    offset=0,
) -> str:
    """
    Searches for genetic variants based on specified criteria.

    Parameters:
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
    - sources: Include only specific data sources
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
        sources=sources or [],
        size=size,
        offset=offset,
    )
    return await search_variants(query, output_json=False)
