import json
import re
from ssl import TLSVersion
from typing import Annotated, Any

from pydantic import BaseModel, Field, computed_field

from .. import http_client, render
from ..constants import PUBTATOR3_FULLTEXT_URL
from ..http_client import RequestError


class PassageInfo(BaseModel):
    section_type: str | None = Field(
        None,
        description="Type of the section.",
    )
    passage_type: str | None = Field(
        None,
        alias="type",
        description="Type of the passage.",
    )


class Passage(BaseModel):
    info: PassageInfo | None = Field(
        None,
        alias="infons",
    )
    text: str | None = None

    @property
    def section_type(self) -> str:
        section_type = None
        if self.info is not None:
            section_type = self.info.section_type or self.info.passage_type
        section_type = section_type or "UNKNOWN"
        return section_type.upper()

    @property
    def is_title(self) -> bool:
        return self.section_type == "TITLE"

    @property
    def is_abstract(self) -> bool:
        return self.section_type == "ABSTRACT"

    @property
    def is_text(self) -> bool:
        return self.section_type in {
            "INTRO",
            "RESULTS",
            "METHODS",
            "DISCUSS",
            "CONCL",
            "FIG",
            "TABLE",
        }


class Article(BaseModel):
    pmid: int | None = Field(
        None,
        description="PubMed ID of the reference article.",
    )
    pmcid: str | None = Field(
        None,
        description="PubMed Central ID of the reference article.",
    )
    date: str | None = Field(
        None,
        description="Date of the reference article's publication.",
    )
    journal: str | None = Field(
        None,
        description="Journal name.",
    )
    authors: list[str] | None = Field(
        None,
        description="List of authors.",
    )
    passages: list[Passage] = Field(
        ...,
        alias="passages",
        description="List of passages in the reference article.",
        exclude=True,
    )

    @computed_field
    def title(self) -> str:
        lines = []
        for passage in filter(lambda p: p.is_title, self.passages):
            if passage.text:
                lines.append(passage.text)
        return " ... ".join(lines) or f"Article: {self.pmid}"

    @computed_field
    def abstract(self) -> str:
        lines = []
        for passage in filter(lambda p: p.is_abstract, self.passages):
            if passage.text:
                lines.append(passage.text)
        return "\n\n".join(lines) or f"Article: {self.pmid}"

    @computed_field
    def full_text(self) -> str:
        lines = []
        for passage in filter(lambda p: p.is_text, self.passages):
            if passage.text:
                lines.append(passage.text)
        return "\n\n".join(lines) or ""

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


class FetchArticlesResponse(BaseModel):
    articles: list[Article] = Field(
        ...,
        alias="PubTator3",
        description="List of full texts Articles retrieved from PubTator3.",
    )

    def get_abstract(self, pmid: int | None) -> str | None:
        for article in self.articles:
            if pmid and article.pmid == pmid:
                return str(article.abstract)
        return None


async def call_pubtator_api(
    pmids: list[int],
    full: bool,
) -> tuple[FetchArticlesResponse | None, RequestError | None]:
    """Fetch the text of a list of PubMed IDs."""

    request = {
        "pmids": ",".join(str(pmid) for pmid in pmids),
        "full": str(full).lower(),
    }

    response, error = await http_client.request_api(
        url=PUBTATOR3_FULLTEXT_URL,
        request=request,
        response_model_type=FetchArticlesResponse,
        tls_version=TLSVersion.TLSv1_2,
        domain="pubmed",
    )
    return response, error


async def fetch_articles(
    pmids: list[int],
    full: bool,
    output_json: bool = False,
) -> str:
    """Fetch the text of a list of PubMed IDs."""

    response, error = await call_pubtator_api(pmids, full)

    # PubTator API returns full text even when full=False
    exclude_fields = {"full_text"} if not full else set()

    # noinspection DuplicatedCode
    if error:
        data: list[dict[str, Any]] = [
            {"error": f"Error {error.code}: {error.message}"}
        ]
    else:
        data = [
            article.model_dump(
                mode="json",
                exclude_none=True,
                exclude=exclude_fields,
            )
            for article in (response.articles if response else [])
        ]

    if data and not output_json:
        return render.to_markdown(data)
    else:
        return json.dumps(data, indent=2)


def is_doi(identifier: str) -> bool:
    """Check if the identifier is a DOI."""
    # DOI pattern: starts with 10. followed by numbers/slash/alphanumeric
    doi_pattern = r"^10\.\d{4,9}/[\-._;()/:\w]+$"
    return bool(re.match(doi_pattern, str(identifier)))


def is_pmid(identifier: str) -> bool:
    """Check if the identifier is a PubMed ID."""
    # PMID is a numeric string
    return str(identifier).isdigit()


async def _article_details(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    pmid,
) -> str:
    """
    Retrieves details for a single article given its identifier.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - pmid: An article identifier - either a PubMed ID (e.g., 34397683) or DOI (e.g., 10.1101/2024.01.20.23288905)

    Process:
    - For PMIDs: Calls the PubTator3 API to fetch the article's title, abstract, and full text (if available)
    - For DOIs: Calls Europe PMC API to fetch preprint details

    Output: A JSON formatted string containing the retrieved article content.
    """
    identifier = str(pmid)

    # Check if it's a DOI (Europe PMC preprint)
    if is_doi(identifier):
        from .preprints import fetch_europe_pmc_article

        return await fetch_europe_pmc_article(identifier, output_json=True)
    # Check if it's a PMID (PubMed article)
    elif is_pmid(identifier):
        return await fetch_articles(
            [int(identifier)], full=True, output_json=True
        )
    else:
        # Unknown identifier format
        return json.dumps(
            [
                {
                    "error": f"Invalid identifier format: {identifier}. Expected either a PMID (numeric) or DOI (10.xxxx/xxxx format)."
                }
            ],
            indent=2,
        )
