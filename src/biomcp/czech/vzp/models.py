"""Pydantic v2 models for VZP insurance codebook data."""

from pydantic import BaseModel, Field


class CodebookEntry(BaseModel):
    """A single VZP insurance codebook entry."""

    codebook_type: str = Field(
        description=("Codebook type identifier (e.g., 'seznam_vykonu')")
    )
    code: str = Field(description="Entry code")
    name: str = Field(description="Entry name")
    description: str | None = Field(
        default=None, description="Long-form description"
    )
    valid_from: str | None = Field(
        default=None, description="Validity start date (ISO 8601)"
    )
    valid_to: str | None = Field(
        default=None, description="Validity end date (ISO 8601)"
    )
    rules: list[str] = Field(
        default_factory=list,
        description="List of billing/coverage rules",
    )
    related_codes: list[str] = Field(
        default_factory=list,
        description="Related codebook codes",
    )
    source: str = Field(default="VZP")


class CodebookSearchResult(BaseModel):
    """Paginated VZP codebook search results."""

    total: int = Field(description="Total number of matches")
    results: list[dict] = Field(
        default_factory=list,
        description=(
            "List of codebook summaries with keys: codebook_type, code, name"
        ),
    )
