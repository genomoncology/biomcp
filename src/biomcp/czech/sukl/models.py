"""Pydantic v2 models for SUKL drug registry data."""

from pydantic import BaseModel, Field


class ActiveSubstance(BaseModel):
    """Active substance in a drug."""

    name: str = Field(description="Substance name")
    strength: str | None = Field(
        default=None, description="Strength (e.g., '400 mg')"
    )


class AvailabilityStatus(BaseModel):
    """Drug market availability status."""

    status: str = Field(description="'available', 'limited', or 'unavailable'")
    last_checked: str | None = Field(
        default=None, description="ISO 8601 datetime"
    )
    note: str | None = Field(
        default=None, description="Additional availability note"
    )


class Drug(BaseModel):
    """Full drug record from SUKL registry."""

    sukl_code: str = Field(description="7-digit SUKL identifier")
    name: str = Field(description="Trade name")
    active_substances: list[ActiveSubstance] = Field(default_factory=list)
    pharmaceutical_form: str | None = Field(
        default=None, description="Dosage form"
    )
    atc_code: str | None = Field(
        default=None, description="ATC classification code"
    )
    registration_number: str | None = Field(
        default=None, description="Marketing authorization number"
    )
    mah: str | None = Field(
        default=None, description="Marketing Authorization Holder"
    )
    registration_valid_to: str | None = Field(
        default=None, description="Validity date (ISO 8601)"
    )
    availability: AvailabilityStatus | None = Field(default=None)
    spc_url: str | None = Field(
        default=None, description="URL to SmPC document"
    )
    pil_url: str | None = Field(
        default=None, description="URL to PIL document"
    )
    source: str = Field(default="SUKL")


class DrugSummary(BaseModel):
    """Summary drug info for search results."""

    sukl_code: str
    name: str
    active_substance: str | None = Field(
        default=None, description="Primary active substance"
    )
    atc_code: str | None = None
    pharmaceutical_form: str | None = None


class DrugSearchResult(BaseModel):
    """Paginated drug search results."""

    total: int = Field(description="Total number of matches")
    page: int = Field(description="Current page number")
    page_size: int = Field(description="Results per page")
    results: list[DrugSummary] = Field(default_factory=list)
