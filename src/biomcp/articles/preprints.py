"""Preprint search functionality for bioRxiv/medRxiv and Europe PMC."""

import asyncio
import json
import logging
from datetime import datetime
from typing import Any

from pydantic import BaseModel, Field

from .. import http_client, render
from ..constants import (
    BIORXIV_BASE_URL,
    BIORXIV_DEFAULT_DAYS_BACK,
    BIORXIV_MAX_PAGES,
    BIORXIV_RESULTS_PER_PAGE,
    EUROPE_PMC_BASE_URL,
    EUROPE_PMC_PAGE_SIZE,
    MEDRXIV_BASE_URL,
    SYSTEM_PAGE_SIZE,
)
from ..core import PublicationState
from .search import PubmedRequest, ResultItem, SearchResponse

logger = logging.getLogger(__name__)


class BiorxivRequest(BaseModel):
    """Request parameters for bioRxiv/medRxiv API."""

    query: str
    interval: str = Field(
        default="", description="Date interval in YYYY-MM-DD/YYYY-MM-DD format"
    )
    cursor: int = Field(default=0, description="Starting position")


class BiorxivResult(BaseModel):
    """Individual result from bioRxiv/medRxiv."""

    doi: str | None = None
    title: str | None = None
    authors: str | None = None
    author_corresponding: str | None = None
    author_corresponding_institution: str | None = None
    date: str | None = None
    version: int | None = None
    type: str | None = None
    license: str | None = None
    category: str | None = None
    jatsxml: str | None = None
    abstract: str | None = None
    published: str | None = None
    server: str | None = None

    def to_result_item(self) -> ResultItem:
        """Convert to standard ResultItem format."""
        authors_list = []
        if self.authors:
            authors_list = [
                author.strip() for author in self.authors.split(";")
            ]

        return ResultItem(
            pmid=None,
            pmcid=None,
            title=self.title,
            journal=f"{self.server or 'bioRxiv'} (preprint)",
            authors=authors_list,
            date=self.date,
            doi=self.doi,
            abstract=self.abstract,
            publication_state=PublicationState.PREPRINT,
            source=self.server or "bioRxiv",
        )


class BiorxivResponse(BaseModel):
    """Response from bioRxiv/medRxiv API."""

    collection: list[BiorxivResult] = Field(default_factory=list)
    messages: list[dict[str, Any]] = Field(default_factory=list)
    total: int = Field(default=0, alias="total")


class EuropePMCRequest(BaseModel):
    """Request parameters for Europe PMC API."""

    query: str
    format: str = "json"
    pageSize: int = Field(default=25, le=1000)
    cursorMark: str = Field(default="*")
    src: str = Field(default="PPR", description="Source: PPR for preprints")


class EuropePMCResult(BaseModel):
    """Individual result from Europe PMC."""

    id: str | None = None
    source: str | None = None
    pmid: str | None = None
    pmcid: str | None = None
    doi: str | None = None
    title: str | None = None
    authorString: str | None = None
    journalTitle: str | None = None
    pubYear: str | None = None
    firstPublicationDate: str | None = None
    abstractText: str | None = None

    def to_result_item(self) -> ResultItem:
        """Convert to standard ResultItem format."""
        authors_list = []
        if self.authorString:
            authors_list = [
                author.strip() for author in self.authorString.split(",")
            ]

        return ResultItem(
            pmid=int(self.pmid) if self.pmid and self.pmid.isdigit() else None,
            pmcid=self.pmcid,
            title=self.title,
            journal=f"{self.journalTitle or 'Preprint Server'} (preprint)",
            authors=authors_list,
            date=self.firstPublicationDate or self.pubYear,
            doi=self.doi,
            abstract=self.abstractText,
            publication_state=PublicationState.PREPRINT,
            source="Europe PMC",
        )


class EuropePMCResponse(BaseModel):
    """Response from Europe PMC API."""

    hitCount: int = Field(default=0)
    nextCursorMark: str | None = None
    resultList: dict[str, Any] = Field(default_factory=dict)

    @property
    def results(self) -> list[EuropePMCResult]:
        result_data = self.resultList.get("result", [])
        return [EuropePMCResult(**r) for r in result_data]


class PreprintSearcher:
    """Handles searching across multiple preprint sources."""

    def __init__(self):
        self.biorxiv_client = BiorxivClient()
        self.europe_pmc_client = EuropePMCClient()

    async def search(
        self,
        request: PubmedRequest,
        include_biorxiv: bool = True,
        include_europe_pmc: bool = True,
    ) -> SearchResponse:
        """Search across preprint sources and merge results."""
        query = self._build_query(request)

        tasks = []
        if include_biorxiv:
            tasks.append(self.biorxiv_client.search(query))
        if include_europe_pmc:
            tasks.append(self.europe_pmc_client.search(query))

        results_lists = await asyncio.gather(*tasks, return_exceptions=True)

        all_results = []
        for results in results_lists:
            if isinstance(results, list):
                all_results.extend(results)

        # Remove duplicates based on DOI
        seen_dois = set()
        unique_results = []
        for result in all_results:
            if result.doi and result.doi in seen_dois:
                continue
            if result.doi:
                seen_dois.add(result.doi)
            unique_results.append(result)

        # Sort by date (newest first)
        unique_results.sort(key=lambda x: x.date or "0000-00-00", reverse=True)

        # Limit results
        limited_results = unique_results[:SYSTEM_PAGE_SIZE]

        return SearchResponse(
            results=limited_results,
            page_size=len(limited_results),
            current=0,
            count=len(limited_results),
            total_pages=1,
        )

    def _build_query(self, request: PubmedRequest) -> str:
        """Build query string from structured request.

        Note: Preprint servers use plain text search, not PubMed syntax.
        """
        query_parts = []

        if request.keywords:
            query_parts.extend(request.keywords)
        if request.genes:
            query_parts.extend(request.genes)
        if request.diseases:
            query_parts.extend(request.diseases)
        if request.chemicals:
            query_parts.extend(request.chemicals)
        if request.variants:
            query_parts.extend(request.variants)

        return " ".join(query_parts) if query_parts else ""


class BiorxivClient:
    """Client for bioRxiv/medRxiv API.

    IMPORTANT LIMITATION: bioRxiv/medRxiv APIs do not provide a search endpoint.
    This implementation works around this limitation by:
    1. Fetching articles from a date range (last 365 days by default)
    2. Filtering results client-side based on query match in title/abstract

    This approach has limitations but is optimized for performance:
    - Searches up to 1 year of preprints by default (configurable)
    - Uses pagination to avoid fetching all results at once
    - May still miss older preprints beyond the date range

    Consider using Europe PMC for more comprehensive preprint search capabilities,
    as it has proper search functionality without date limitations.
    """

    async def search(  # noqa: C901
        self,
        query: str,
        server: str = "biorxiv",
        days_back: int = BIORXIV_DEFAULT_DAYS_BACK,
    ) -> list[ResultItem]:
        """Search bioRxiv or medRxiv for articles.

        Note: Due to API limitations, this performs client-side filtering on
        recent articles only. See class docstring for details.
        """
        if not query:
            return []

        base_url = (
            BIORXIV_BASE_URL if server == "biorxiv" else MEDRXIV_BASE_URL
        )

        # Optimize by only fetching recent articles (last 30 days by default)
        from datetime import timedelta

        today = datetime.now()
        start_date = today - timedelta(days=days_back)
        interval = f"{start_date.year}-{start_date.month:02d}-{start_date.day:02d}/{today.year}-{today.month:02d}-{today.day:02d}"

        # Prepare query terms for better matching
        query_terms = query.lower().split()

        filtered_results = []
        cursor = 0
        max_pages = (
            BIORXIV_MAX_PAGES  # Limit pagination to avoid excessive API calls
        )

        for page in range(max_pages):
            request = BiorxivRequest(
                query=query, interval=interval, cursor=cursor
            )
            url = f"{base_url}/{request.interval}/{request.cursor}"

            response, error = await http_client.request_api(
                url=url,
                method="GET",
                request={},
                response_model_type=BiorxivResponse,
                domain="biorxiv",
                cache_ttl=300,  # Cache for 5 minutes
            )

            if error or not response:
                logger.warning(
                    f"Failed to fetch {server} articles page {page} for query '{query}': {error if error else 'No response'}"
                )
                break

            # Filter results based on query
            page_filtered = 0
            for result in response.collection:
                # Create searchable text from title and abstract
                searchable_text = ""
                if result.title:
                    searchable_text += result.title.lower() + " "
                if result.abstract:
                    searchable_text += result.abstract.lower()

                # Check if all query terms are present (AND logic)
                if all(term in searchable_text for term in query_terms):
                    filtered_results.append(result.to_result_item())
                    page_filtered += 1

                    # Stop if we have enough results
                    if len(filtered_results) >= SYSTEM_PAGE_SIZE:
                        return filtered_results[:SYSTEM_PAGE_SIZE]

            # If this page had no matches and we have some results, stop pagination
            if page_filtered == 0 and filtered_results:
                break

            # Move to next page
            cursor += len(response.collection)

            # Stop if we've processed all available results
            if (
                len(response.collection) < BIORXIV_RESULTS_PER_PAGE
            ):  # bioRxiv typically returns this many per page
                break

        return filtered_results[:SYSTEM_PAGE_SIZE]


class EuropePMCClient:
    """Client for Europe PMC API."""

    async def search(
        self, query: str, max_results: int = SYSTEM_PAGE_SIZE
    ) -> list[ResultItem]:
        """Search Europe PMC for preprints with pagination support."""
        results: list[ResultItem] = []
        cursor_mark = "*"
        page_size = min(
            EUROPE_PMC_PAGE_SIZE, max_results
        )  # Europe PMC optimal page size

        while len(results) < max_results:
            request = EuropePMCRequest(
                query=f"(SRC:PPR) AND ({query})" if query else "SRC:PPR",
                pageSize=page_size,
                cursorMark=cursor_mark,
            )

            params = request.model_dump(exclude_none=True)

            response, error = await http_client.request_api(
                url=EUROPE_PMC_BASE_URL,
                method="GET",
                request=params,
                response_model_type=EuropePMCResponse,
                domain="europepmc",
                cache_ttl=300,  # Cache for 5 minutes
            )

            if error or not response:
                logger.warning(
                    f"Failed to fetch Europe PMC preprints for query '{query}': {error if error else 'No response'}"
                )
                break

            # Add results
            page_results = [
                result.to_result_item() for result in response.results
            ]
            results.extend(page_results)

            # Check if we have more pages
            if (
                not response.nextCursorMark
                or response.nextCursorMark == cursor_mark
            ):
                break

            # Check if we got fewer results than requested (last page)
            if len(page_results) < page_size:
                break

            cursor_mark = response.nextCursorMark

            # Adjust page size for last request if needed
            remaining = max_results - len(results)
            if remaining < page_size:
                page_size = remaining

        return results[:max_results]


async def fetch_europe_pmc_article(
    doi: str,
    output_json: bool = False,
) -> str:
    """Fetch a single article from Europe PMC by DOI."""
    # Europe PMC search API can fetch article details by DOI
    request = EuropePMCRequest(
        query=f'DOI:"{doi}"',
        pageSize=1,
        src="PPR",  # Preprints source
    )

    params = request.model_dump(exclude_none=True)

    response, error = await http_client.request_api(
        url=EUROPE_PMC_BASE_URL,
        method="GET",
        request=params,
        response_model_type=EuropePMCResponse,
        domain="europepmc",
    )

    if error:
        data: list[dict[str, Any]] = [
            {"error": f"Error {error.code}: {error.message}"}
        ]
    elif response and response.results:
        # Convert Europe PMC result to Article format for consistency
        europe_pmc_result = response.results[0]
        article_data = {
            "pmid": None,  # Europe PMC preprints don't have PMIDs
            "pmcid": europe_pmc_result.pmcid,
            "doi": europe_pmc_result.doi,
            "title": europe_pmc_result.title,
            "journal": f"{europe_pmc_result.journalTitle or 'Preprint Server'} (preprint)",
            "date": europe_pmc_result.firstPublicationDate
            or europe_pmc_result.pubYear,
            "authors": [
                author.strip()
                for author in (europe_pmc_result.authorString or "").split(",")
            ],
            "abstract": europe_pmc_result.abstractText,
            "full_text": "",  # Europe PMC API doesn't provide full text for preprints
            "pubmed_url": None,
            "pmc_url": f"https://europepmc.org/article/PPR/{doi}"
            if doi
            else None,
            "source": "Europe PMC",
        }
        data = [article_data]
    else:
        data = [{"error": "Article not found in Europe PMC"}]

    if data and not output_json:
        return render.to_markdown(data)
    else:
        return json.dumps(data, indent=2)


async def search_preprints(
    request: PubmedRequest,
    include_biorxiv: bool = True,
    include_europe_pmc: bool = True,
    output_json: bool = False,
) -> str:
    """Search for preprints across multiple sources."""
    searcher = PreprintSearcher()
    response = await searcher.search(
        request,
        include_biorxiv=include_biorxiv,
        include_europe_pmc=include_europe_pmc,
    )

    if response and response.results:
        data = [
            result.model_dump(mode="json", exclude_none=True)
            for result in response.results
        ]
    else:
        data = []

    if data and not output_json:
        return render.to_markdown(data)
    else:
        return json.dumps(data, indent=2)
