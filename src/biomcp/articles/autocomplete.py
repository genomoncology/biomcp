"""Find entities for a given concept using the PUBTATOR API.

Example URL:
https://www.ncbi.nlm.nih.gov/research/pubtator3-api/entity/autocomplete/?query=BRAF
"""

from typing import Literal

from pydantic import BaseModel, Field, RootModel

from .. import http_client
from ..constants import PUBTATOR3_BASE_URL

Concept = Literal["variant", "chemical", "disease", "gene"]


class EntityRequest(BaseModel):
    concept: Concept | None = None
    query: str
    limit: int = Field(default=1, ge=1, le=100)


class Entity(BaseModel):
    entity_id: str = Field(
        alias="_id",
        examples=["@GENE_BRAF"],
        description="Text-based entity following @<biotype>_<n> format.",
    )
    concept: Concept = Field(
        ...,
        alias="biotype",
        description="Entity label or concept type.",
    )
    name: str = Field(
        ...,
        description="Preferred term of entity concept.",
        examples=[
            "BRAF",
            "Adenocarcinoma of Lung",
            "Osimertinib",
            "EGFR L858R",
        ],
    )
    match: str | None = Field(
        default=None,
        description="Reason for the entity match.",
        examples=["Multiple matches", "Matched on name <m>NAME</m>"],
    )

    def __eq__(self, other) -> bool:
        return self.entity_id == other.entity_id


class EntityList(RootModel):
    root: list[Entity]

    @property
    def first(self) -> Entity | None:
        return self.root[0] if self.root else None


PUBTATOR3_AUTOCOMPLETE = f"{PUBTATOR3_BASE_URL}/entity/autocomplete/"


async def autocomplete(request: EntityRequest) -> Entity | None:
    """Given a request of biotype and query, returns the best matching Entity.
    If API call fails or returns 0 results, then None is returned.

    Example Request:
    {
        "concept": "gene",
        "query": "BRAF"
    }
    Response:
    {
        "entity_id": "@GENE_BRAF",
        "biotype": "gene",
        "name": "BRAF",
        "match": "Matched on name <m>BRAF</m>"
    }
    """
    response, _ = await http_client.request_api(
        url=PUBTATOR3_AUTOCOMPLETE,
        request=request,
        response_model_type=EntityList,
        domain="pubmed",
    )
    return response.first if response else None
