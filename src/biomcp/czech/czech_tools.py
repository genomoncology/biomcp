"""MCP tool registrations for Czech healthcare modules.

Tools are registered here using @mcp_app.tool() decorator.
This module is imported by czech/__init__.py to auto-register
all Czech tools when the package is loaded.

Tools are added incrementally per user story:
- US1: SUKL (5 tools)
- US2: MKN-10 (3 tools)
- US3: NRPZS (2 tools)
- US4: SZV + VZP (4 tools)
"""

import logging
from typing import Annotated

from pydantic import Field

from biomcp.core import mcp_app
from biomcp.czech.mkn.search import (
    _mkn_browse,
    _mkn_get,
    _mkn_search,
)
from biomcp.czech.nrpzs.search import (
    _nrpzs_get,
    _nrpzs_search,
)
from biomcp.czech.sukl.availability import (
    _sukl_availability_check,
)
from biomcp.czech.sukl.getter import (
    _sukl_drug_details,
    _sukl_pil_getter,
    _sukl_spc_getter,
)
from biomcp.czech.sukl.search import _sukl_drug_search
from biomcp.czech.szv.search import _szv_get, _szv_search
from biomcp.czech.vzp.search import _vzp_get, _vzp_search
from biomcp.metrics import track_performance

logger = logging.getLogger(__name__)

# -------------------------------------------------------------------
# US1: SUKL Drug Registry Tools (5 tools)
# -------------------------------------------------------------------


@mcp_app.tool()
@track_performance("czechmedmcp.sukl_drug_searcher")
async def sukl_drug_searcher(
    query: Annotated[
        str,
        Field(description=("Drug name, active substance, or ATC code")),
    ],
    page: Annotated[
        int,
        Field(description="Page number (1-based)", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(
            description="Results per page (1-100)",
            ge=1,
            le=100,
        ),
    ] = 10,
) -> str:
    """Search Czech drug registry (SUKL)."""
    return await _sukl_drug_search(query, page, page_size)


@mcp_app.tool()
@track_performance("czechmedmcp.sukl_drug_getter")
async def sukl_drug_getter(
    sukl_code: Annotated[
        str,
        Field(description="SUKL drug identifier (7-digit code)"),
    ],
) -> str:
    """Get full drug details from Czech SUKL registry."""
    return await _sukl_drug_details(sukl_code)


@mcp_app.tool()
@track_performance("czechmedmcp.sukl_spc_getter")
async def sukl_spc_getter(
    sukl_code: Annotated[
        str,
        Field(description="SUKL drug identifier"),
    ],
) -> str:
    """Get SmPC for a Czech drug."""
    return await _sukl_spc_getter(sukl_code)


@mcp_app.tool()
@track_performance("czechmedmcp.sukl_pil_getter")
async def sukl_pil_getter(
    sukl_code: Annotated[
        str,
        Field(description="SUKL drug identifier"),
    ],
) -> str:
    """Get PIL for a Czech drug."""
    return await _sukl_pil_getter(sukl_code)


@mcp_app.tool()
@track_performance("czechmedmcp.sukl_availability_checker")
async def sukl_availability_checker(
    sukl_code: Annotated[
        str,
        Field(description="SUKL drug identifier"),
    ],
) -> str:
    """Check current market availability of a Czech drug."""
    return await _sukl_availability_check(sukl_code)


# -------------------------------------------------------------------
# US2: MKN-10 Diagnosis Code Tools (3 tools)
# -------------------------------------------------------------------


@mcp_app.tool()
@track_performance("czechmedmcp.mkn_diagnosis_searcher")
async def mkn_diagnosis_searcher(
    query: Annotated[
        str,
        Field(description="MKN-10 code or free text in Czech"),
    ],
    max_results: Annotated[
        int,
        Field(description="Maximum results", ge=1, le=100),
    ] = 10,
) -> str:
    """Search Czech ICD-10 (MKN-10) diagnoses."""
    return await _mkn_search(query, max_results)


@mcp_app.tool()
@track_performance("czechmedmcp.mkn_diagnosis_getter")
async def mkn_diagnosis_getter(
    code: Annotated[
        str,
        Field(description='MKN-10 code (e.g., "J06.9")'),
    ],
) -> str:
    """Get full diagnosis details including hierarchy."""
    return await _mkn_get(code)


@mcp_app.tool()
@track_performance("czechmedmcp.mkn_category_browser")
async def mkn_category_browser(
    code: Annotated[
        str | None,
        Field(
            description=("Category code to browse (omit for root/chapters)")
        ),
    ] = None,
) -> str:
    """Browse MKN-10 category hierarchy."""
    return await _mkn_browse(code)


# -------------------------------------------------------------------
# US3: NRPZS Provider Registry Tools (2 tools)
# -------------------------------------------------------------------


@mcp_app.tool()
@track_performance("czechmedmcp.nrpzs_provider_searcher")
async def nrpzs_provider_searcher(
    query: Annotated[
        str | None,
        Field(description="Provider name or keyword"),
    ] = None,
    city: Annotated[
        str | None,
        Field(description="City name"),
    ] = None,
    specialty: Annotated[
        str | None,
        Field(description="Medical specialty"),
    ] = None,
    page: Annotated[
        int,
        Field(description="Page number", ge=1),
    ] = 1,
    page_size: Annotated[
        int,
        Field(description="Results per page", ge=1, le=100),
    ] = 10,
) -> str:
    """Search Czech healthcare providers (NRPZS)."""
    return await _nrpzs_search(query, city, specialty, page, page_size)


@mcp_app.tool()
@track_performance("czechmedmcp.nrpzs_provider_getter")
async def nrpzs_provider_getter(
    provider_id: Annotated[
        str,
        Field(description="NRPZS provider identifier"),
    ],
) -> str:
    """Get full provider details including workplaces."""
    return await _nrpzs_get(provider_id)


# -------------------------------------------------------------------
# US4: SZV + VZP Tools (4 tools)
# -------------------------------------------------------------------


@mcp_app.tool()
@track_performance("czechmedmcp.szv_procedure_searcher")
async def szv_procedure_searcher(
    query: Annotated[
        str,
        Field(description="Procedure code or name"),
    ],
    max_results: Annotated[
        int,
        Field(description="Maximum results", ge=1, le=100),
    ] = 10,
) -> str:
    """Search Czech health procedures (SZV)."""
    return await _szv_search(query, max_results)


@mcp_app.tool()
@track_performance("czechmedmcp.szv_procedure_getter")
async def szv_procedure_getter(
    code: Annotated[
        str,
        Field(description='Procedure code (e.g., "09513")'),
    ],
) -> str:
    """Get full procedure details with point value."""
    return await _szv_get(code)


@mcp_app.tool()
@track_performance("czechmedmcp.vzp_codebook_searcher")
async def vzp_codebook_searcher(
    query: Annotated[
        str,
        Field(description="Search term"),
    ],
    codebook_type: Annotated[
        str | None,
        Field(description="Filter by codebook type"),
    ] = None,
    max_results: Annotated[
        int,
        Field(description="Maximum results", ge=1, le=100),
    ] = 10,
) -> str:
    """Search VZP insurance codebooks."""
    return await _vzp_search(query, codebook_type, max_results)


@mcp_app.tool()
@track_performance("czechmedmcp.vzp_codebook_getter")
async def vzp_codebook_getter(
    codebook_type: Annotated[
        str,
        Field(description="Codebook type identifier"),
    ],
    code: Annotated[
        str,
        Field(description="Entry code"),
    ],
) -> str:
    """Get codebook entry details."""
    return await _vzp_get(codebook_type, code)
