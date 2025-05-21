"""Getter module for retrieving variant details."""

import json
from typing import Annotated

from .. import const, ensure_list, http_client, mcp_app, render
from .filters import filter_variants
from .links import inject_links

MYVARIANT_GET_ENDPOINT = f"{const.MYVARIANT_BASE_URL}/variant"


async def get_variant(
    variant_id: str,
    output_json: bool = False,
) -> str:
    """
    Get variant details from MyVariant.info using the variant identifier.

    The identifier can be a full HGVS-style string (e.g. "chr7:g.140453136A>T")
    or an rsID (e.g. "rs113488022"). The API response is expected to include a
    "hits" array; this function extracts the first hit.

    If output_json is True, the result is returned as a formatted JSON string;
    otherwise, it is rendered as Markdown.
    """
    response, error = await http_client.request_api(
        url=f"{MYVARIANT_GET_ENDPOINT}/{variant_id}",
        request={"fields": "all"},
        method="GET",
    )

    data_to_return: list = ensure_list(response)

    # Inject database links into the variant data
    if not error:
        data_to_return = inject_links(data_to_return)
        data_to_return = filter_variants(data_to_return)

    if error:
        data_to_return = [{"error": f"Error {error.code}: {error.message}"}]

    if output_json:
        return json.dumps(data_to_return, indent=2)
    else:
        return render.to_markdown(data_to_return)


@mcp_app.tool()
async def variant_details(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    variant_id: str,
) -> str:
    """
    Retrieves detailed information for a *single* genetic variant.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - variant_id: A variant identifier ("chr7:g.140453136A>T")

    Process: Queries the MyVariant.info GET endpoint
    Output: A Markdown formatted string containing comprehensive
            variant annotations (genomic context, frequencies,
            predictions, clinical data). Returns error if invalid.
    Note: Use the variant_searcher to find the variant id first.
    """
    return await get_variant(variant_id, output_json=False)
