import json
from ssl import TLSVersion
from typing import Annotated, Any

from .. import StrEnum, const, http_client, mcp_app, render


class Module(StrEnum):
    PROTOCOL = "Protocol"
    LOCATIONS = "Locations"
    REFERENCES = "References"
    OUTCOMES = "Outcomes"


modules: dict[Module, list[str]] = {
    Module.PROTOCOL: [
        "IdentificationModule",
        "StatusModule",
        "SponsorCollaboratorsModule",
        "OversightModule",
        "DescriptionModule",
        "ConditionsModule",
        "DesignModule",
        "ArmsInterventionsModule",
        "EligibilityModule",
    ],
    Module.LOCATIONS: ["ContactsLocationsModule"],
    Module.REFERENCES: ["ReferencesModule"],
    Module.OUTCOMES: ["OutcomesModule", "ResultsSection"],
}


async def get_trial(
    nct_id: str,
    module: Module = Module.PROTOCOL,
    output_json: bool = False,
) -> str:
    """Get details of a clinical trial by module."""
    fields = ",".join(modules[module])
    params = {"fields": fields}
    url = f"{const.CT_GOV_STUDIES}/{nct_id}"

    parsed_data: dict[str, Any] | None
    error_obj: http_client.RequestError | None
    parsed_data, error_obj = await http_client.request_api(
        url=url,
        request=params,
        method="GET",
        tls_version=TLSVersion.TLSv1_2,
        response_model_type=None,
    )

    data_to_return: dict[str, Any]

    if error_obj:
        data_to_return = {
            "error": f"API Error {error_obj.code}",
            "details": error_obj.message,
        }
    elif parsed_data:
        data_to_return = parsed_data
        data_to_return["URL"] = f"https://clinicaltrials.gov/study/{nct_id}"
    else:
        data_to_return = {
            "error": f"No data found for {nct_id} with module {module.value}"
        }

    if output_json:
        return json.dumps(data_to_return, indent=2)
    else:
        return render.to_markdown(data_to_return)


@mcp_app.tool()
async def trial_protocol(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    nct_id: str,
):
    """
    Retrieves core protocol information for a single clinical
    trial identified by its NCT ID.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - nct_id: A single NCT ID (string, e.g., "NCT04280705")

    Process: Fetches standard "Protocol" view modules (like ID,
             Status, Sponsor, Design, Eligibility) from the
             ClinicalTrials.gov v2 API.
    Output: A Markdown formatted string detailing title, status,
            sponsor, purpose, study design, phase, interventions,
            eligibility criteria, etc. Returns error if invalid.
    """
    return await get_trial(nct_id, Module.PROTOCOL)


@mcp_app.tool()
async def trial_locations(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    nct_id: str,
) -> str:
    """
    Retrieves contact and location details for a single
    clinical trial identified by its NCT ID.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - nct_id: A single NCT ID (string, e.g., "NCT04280705")

    Process: Fetches the `ContactsLocationsModule` from the
             ClinicalTrials.gov v2 API for the given NCT ID.
    Output: A Markdown formatted string detailing facility names,
            addresses (city, state, country), and contact info.
            Returns an error message if the NCT ID is invalid.
    """
    return await get_trial(nct_id, Module.LOCATIONS)


@mcp_app.tool()
async def trial_outcomes(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    nct_id: str,
) -> str:
    """
    Retrieves outcome measures, results (if available), and
    adverse event data for a single clinical trial.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - nct_id: A single NCT ID (string, e.g., "NCT04280705")

    Process: Fetches the `OutcomesModule` and `ResultsSection`
             from the ClinicalTrials.gov v2 API for the NCT ID.
    Output: A Markdown formatted string detailing primary/secondary
            outcomes, participant flow, results tables (if posted),
            and adverse event summaries. Returns an error if invalid.
    """
    return await get_trial(nct_id, Module.OUTCOMES)


@mcp_app.tool()
async def trial_references(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    nct_id: str,
):
    """
    Retrieves publications and other references associated with
    a single clinical trial identified by its NCT ID.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - nct_id: A single NCT ID (string, e.g., "NCT04280705")

    Process: Fetches the `ReferencesModule` from the
             ClinicalTrials.gov v2 API for the NCT ID.
    Output: A Markdown formatted string listing citations,
            associated PubMed IDs (PMIDs), and reference types
            (e.g., result publication). Returns error if invalid.
    """
    return await get_trial(nct_id, Module.REFERENCES)
