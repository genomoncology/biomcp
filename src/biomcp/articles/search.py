import json
from collections.abc import Generator
from typing import Any, get_args

from pydantic import BaseModel, Field, computed_field

from .. import const, http_client, mcp_app, render
from .autocomplete import Concept, EntityRequest, autocomplete
from .fetch import call_pubtator_api

PUBTATOR3_SEARCH = f"{const.PUBTATOR3_BASE}/search/"

concepts: list[Concept] = sorted(get_args(Concept))
fields: list[str] = [concept + "s" for concept in concepts]


class PubmedRequest(BaseModel):
    chemicals: list[str] = Field(
        default_factory=list,
        description="List of chemicals for filtering results.",
    )
    diseases: list[str] = Field(
        default_factory=list,
        description="Diseases such as Hypertension, Lung Adenocarcinoma, etc.",
    )
    genes: list[str] = Field(
        default_factory=list,
        description="List of genes for filtering results.",
    )
    keywords: list[str] = Field(
        default_factory=list,
        description="List of other keywords for filtering results.",
    )
    variants: list[str] = Field(
        default_factory=list,
        description="List of variants for filtering results.",
    )

    def iter_concepts(self) -> Generator[tuple[Concept, str], None, None]:
        for concept in concepts:
            field = concept + "s"
            values = getattr(self, field, []) or []
            for value in values:
                yield concept, value


class PubtatorRequest(BaseModel):
    text: str
    size: int = 50


class ResultItem(BaseModel):
    pmid: int | None = None
    pmcid: str | None = None
    title: str | None = None
    journal: str | None = None
    authors: list[str] | None = None
    date: str | None = None
    doi: str | None = None
    abstract: str | None = None

    @computed_field
    def pubmed_url(self) -> str | None:
        url = None
        if self.pmid:
            url = f"https://pubmed.ncbi.nlm.nih.gov/{self.pmid}/"
        return url

    @computed_field
    def pmc_url(self) -> str | None:
        """Generates the PMC URL if PMCID exists."""
        url = None
        if self.pmcid:
            url = f"https://www.ncbi.nlm.nih.gov/pmc/articles/{self.pmcid}/"
        return url

    @computed_field
    def doi_url(self) -> str | None:
        """Generates the DOI URL if DOI exists."""
        url = None
        if self.doi:
            url = f"https://doi.org/{self.doi}"
        return url


class SearchResponse(BaseModel):
    results: list[ResultItem]
    page_size: int
    current: int
    count: int
    total_pages: int


async def convert_request(request: PubmedRequest) -> PubtatorRequest:
    query_parts = request.keywords[:]

    for concept, value in request.iter_concepts():
        entity = await autocomplete(
            request=EntityRequest(concept=concept, query=value),
        )
        if entity:
            query_parts.append(entity.entity_id)
        else:
            query_parts.append(value)

    query_text = " AND ".join(query_parts)

    return PubtatorRequest(text=query_text, size=const.SYSTEM_PAGE_SIZE)


async def add_abstracts(response: SearchResponse) -> None:
    pmids = [pr.pmid for pr in response.results if pr.pmid]
    abstract_response, _ = await call_pubtator_api(pmids, full=False)

    if abstract_response:
        for result in response.results:
            result.abstract = abstract_response.get_abstract(result.pmid)


def clean_authors(record):
    """Keep only the first and last author if > 4 authors."""
    authors = record.get("authors")
    if authors and len(authors) > 4:
        record["authors"] = [authors[0], "...", authors[-1]]
    return record


async def search_articles(
    request: PubmedRequest,
    output_json: bool = False,
) -> str:
    pubtator_request = await convert_request(request)

    response, error = await http_client.request_api(
        url=PUBTATOR3_SEARCH,
        request=pubtator_request,
        response_model_type=SearchResponse,
    )

    if response:
        await add_abstracts(response)

    # noinspection DuplicatedCode
    if error:
        data: list[dict[str, Any]] = [
            {"error": f"Error {error.code}: {error.message}"}
        ]
    else:
        data = list(
            map(
                clean_authors,
                [
                    result.model_dump(mode="json", exclude_none=True)
                    for result in (response.results if response else [])
                ],
            )
        )

    if data and not output_json:
        return render.to_markdown(data)
    else:
        return json.dumps(data, indent=2)


@mcp_app.tool()
async def article_searcher(
    chemicals=None, diseases=None, genes=None, keywords=None, variants=None
) -> str:
    """
    Searches PubMed articles using structured criteria.

    Parameters:
    - chemicals: List of chemicals for filtering results
    - diseases: Diseases such as Hypertension, Lung Adenocarcinoma, etc.
    - genes: List of genes for filtering results
    - keywords: List of other keywords for filtering results
    - variants: List of variants for filtering results

    Notes:
    - Use full terms ("Non-small cell lung carcinoma") over abbreviations ("NSCLC")
    - Use keywords to specify terms that don't fit in disease, gene ("EGFR"),
      chemical ("Cisplatin"), or variant ("BRAF V600E") categories

    Returns:
    Markdown formatted list of matching articles (PMID, title, abstract, etc.)
    Limited to max 40 results.
    """
    # Convert individual parameters to a PubmedRequest object
    request = PubmedRequest(
        chemicals=chemicals or [],
        diseases=diseases or [],
        genes=genes or [],
        keywords=keywords or [],
        variants=variants or [],
    )
    return await search_articles(request)
