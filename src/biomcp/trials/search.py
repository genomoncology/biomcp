import json
from ssl import TLSVersion
from typing import Any

from pydantic import BaseModel, Field, field_validator

from .. import StrEnum, const, ensure_list, http_client, mcp_app, render


class SortOrder(StrEnum):
    RELEVANCE = "RELEVANCE"
    LAST_UPDATE = "LAST_UPDATE"
    ENROLLMENT = "ENROLLMENT"
    START_DATE = "START_DATE"
    COMPLETION_DATE = "COMPLETION_DATE"
    SUBMITTED_DATE = "SUBMITTED_DATE"


class TrialPhase(StrEnum):
    EARLY_PHASE1 = "EARLY_PHASE1"
    PHASE1 = "PHASE1"
    PHASE2 = "PHASE2"
    PHASE3 = "PHASE3"
    PHASE4 = "PHASE4"
    NOT_APPLICABLE = "NOT_APPLICABLE"


class RecruitingStatus(StrEnum):
    OPEN = "OPEN"
    CLOSED = "CLOSED"
    ANY = "ANY"


class StudyType(StrEnum):
    INTERVENTIONAL = "INTERVENTIONAL"
    OBSERVATIONAL = "OBSERVATIONAL"
    EXPANDED_ACCESS = "EXPANDED_ACCESS"
    OTHER = "OTHER"


class InterventionType(StrEnum):
    DRUG = "DRUG"
    DEVICE = "DEVICE"
    BIOLOGICAL = "BIOLOGICAL"
    PROCEDURE = "PROCEDURE"
    RADIATION = "RADIATION"
    BEHAVIORAL = "BEHAVIORAL"
    GENETIC = "GENETIC"
    DIETARY = "DIETARY"
    DIAGNOSTIC_TEST = "DIAGNOSTIC_TEST"
    OTHER = "OTHER"


class SponsorType(StrEnum):
    INDUSTRY = "INDUSTRY"
    GOVERNMENT = "GOVERNMENT"
    ACADEMIC = "ACADEMIC"
    OTHER = "OTHER"


class StudyDesign(StrEnum):
    RANDOMIZED = "RANDOMIZED"
    NON_RANDOMIZED = "NON_RANDOMIZED"
    OBSERVATIONAL = "OBSERVATIONAL"


class DateField(StrEnum):
    LAST_UPDATE = "LAST_UPDATE"
    STUDY_START = "STUDY_START"
    PRIMARY_COMPLETION = "PRIMARY_COMPLETION"
    OUTCOME_POSTING = "OUTCOME_POSTING"
    COMPLETION = "COMPLETION"
    FIRST_POSTING = "FIRST_POSTING"
    SUBMITTED_DATE = "SUBMITTED_DATE"


class PrimaryPurpose(StrEnum):
    TREATMENT = "TREATMENT"
    PREVENTION = "PREVENTION"
    DIAGNOSTIC = "DIAGNOSTIC"
    SUPPORTIVE_CARE = "SUPPORTIVE_CARE"
    SCREENING = "SCREENING"
    HEALTH_SERVICES = "HEALTH_SERVICES"
    BASIC_SCIENCE = "BASIC_SCIENCE"
    DEVICE_FEASIBILITY = "DEVICE_FEASIBILITY"
    OTHER = "OTHER"


class AgeGroup(StrEnum):
    CHILD = "CHILD"
    ADULT = "ADULT"
    SENIOR = "SENIOR"
    ALL = "ALL"


CTGOV_SORT_MAPPING = {
    SortOrder.RELEVANCE: "@relevance",
    SortOrder.LAST_UPDATE: "LastUpdatePostDate:desc",
    SortOrder.ENROLLMENT: "EnrollmentCount:desc",
    SortOrder.START_DATE: "StudyStartDate:desc",
    SortOrder.COMPLETION_DATE: "PrimaryCompletionDate:desc",
    SortOrder.SUBMITTED_DATE: "StudyFirstSubmitDate:desc",
}

CTGOV_PHASE_MAPPING = {
    TrialPhase.EARLY_PHASE1: ("EARLY_PHASE1",),
    TrialPhase.PHASE1: ("PHASE1",),
    TrialPhase.PHASE2: ("PHASE2",),
    TrialPhase.PHASE3: ("PHASE3",),
    TrialPhase.PHASE4: ("PHASE4",),
    TrialPhase.NOT_APPLICABLE: ("NOT_APPLICABLE",),
}

OPEN_STATUSES = (
    "AVAILABLE",
    "ENROLLING_BY_INVITATION",
    "NOT_YET_RECRUITING",
    "RECRUITING",
)
CLOSED_STATUSES = (
    "ACTIVE_NOT_RECRUITING",
    "COMPLETED",
    "SUSPENDED",
    "TERMINATED",
    "WITHDRAWN",
)
CTGOV_RECRUITING_STATUS_MAPPING = {
    RecruitingStatus.OPEN: OPEN_STATUSES,
    RecruitingStatus.CLOSED: CLOSED_STATUSES,
    RecruitingStatus.ANY: None,
}

CTGOV_STUDY_TYPE_MAPPING = {
    StudyType.INTERVENTIONAL: ("Interventional",),
    StudyType.OBSERVATIONAL: ("Observational",),
    StudyType.EXPANDED_ACCESS: ("Expanded Access",),
    StudyType.OTHER: ("Other",),
}

CTGOV_INTERVENTION_TYPE_MAPPING = {
    InterventionType.DRUG: ("Drug",),
    InterventionType.DEVICE: ("Device",),
    InterventionType.BIOLOGICAL: ("Biological",),
    InterventionType.PROCEDURE: ("Procedure",),
    InterventionType.RADIATION: ("Radiation",),
    InterventionType.BEHAVIORAL: ("Behavioral",),
    InterventionType.GENETIC: ("Genetic",),
    InterventionType.DIETARY: ("Dietary",),
    InterventionType.DIAGNOSTIC_TEST: ("Diagnostic Test",),
    InterventionType.OTHER: ("Other",),
}

CTGOV_SPONSOR_TYPE_MAPPING = {
    SponsorType.INDUSTRY: ("Industry",),
    SponsorType.GOVERNMENT: ("Government",),
    SponsorType.ACADEMIC: ("Academic",),
    SponsorType.OTHER: ("Other",),
}

CTGOV_STUDY_DESIGN_MAPPING = {
    StudyDesign.RANDOMIZED: ("Randomized",),
    StudyDesign.NON_RANDOMIZED: ("Non-Randomized",),
    StudyDesign.OBSERVATIONAL: ("Observational",),
}

CTGOV_DATE_FIELD_MAPPING = {
    DateField.LAST_UPDATE: "LastUpdatePostDate",
    DateField.STUDY_START: "StartDate",
    DateField.PRIMARY_COMPLETION: "PrimaryCompletionDate",
    DateField.OUTCOME_POSTING: "ResultsFirstPostDate",
    DateField.COMPLETION: "CompletionDate",
    DateField.FIRST_POSTING: "StudyFirstPostDate",
    DateField.SUBMITTED_DATE: "StudyFirstSubmitDate",
}

CTGOV_PRIMARY_PURPOSE_MAPPING = {
    PrimaryPurpose.TREATMENT: ("Treatment",),
    PrimaryPurpose.PREVENTION: ("Prevention",),
    PrimaryPurpose.DIAGNOSTIC: ("Diagnostic",),
    PrimaryPurpose.SUPPORTIVE_CARE: ("Supportive Care",),
    PrimaryPurpose.SCREENING: ("Screening",),
    PrimaryPurpose.HEALTH_SERVICES: ("Health Services",),
    PrimaryPurpose.BASIC_SCIENCE: ("Basic Science",),
    PrimaryPurpose.DEVICE_FEASIBILITY: ("Device Feasibility",),
    PrimaryPurpose.OTHER: ("Other",),
}

CTGOV_AGE_GROUP_MAPPING = {
    AgeGroup.CHILD: ("Child",),
    AgeGroup.ADULT: ("Adult",),
    AgeGroup.SENIOR: ("Older Adult",),
    AgeGroup.ALL: None,
}

DEFAULT_FORMAT = "csv"
DEFAULT_MARKUP = "markdown"

SEARCH_FIELDS = [
    "NCT Number",
    "Study Title",
    "Study URL",
    "Study Status",
    "Brief Summary",
    "Study Results",
    "Conditions",
    "Interventions",
    "Phases",
    "Enrollment",
    "Study Type",
    "Study Design",
    "Start Date",
    "Completion Date",
]

SEARCH_FIELDS_PARAM = [",".join(SEARCH_FIELDS)]


class TrialQuery(BaseModel):
    """Parameters for querying clinical trial data from ClinicalTrials.gov."""

    conditions: list[str] | None = Field(
        default=None,
        description="List of condition terms.",
    )
    terms: list[str] | None = Field(
        default=None,
        description="General search terms that don't fit specific categories.",
    )
    interventions: list[str] | None = Field(
        default=None,
        description="Intervention names.",
    )
    recruiting_status: RecruitingStatus | None = Field(
        default=None,
        description="Study recruitment status.",
    )
    study_type: StudyType | None = Field(
        default=None,
        description="Type of study.",
    )
    nct_ids: list[str] | None = Field(
        default=None,
        description="Clinical trial NCT IDs",
    )
    lat: float | None = Field(
        default=None,
        description="Latitude for location search",
    )
    long: float | None = Field(
        default=None,
        description="Longitude for location search",
    )
    distance: int | None = Field(
        default=None,
        description="Distance from lat/long in miles",
    )
    min_date: str | None = Field(
        default=None,
        description="Minimum date for filtering",
    )
    max_date: str | None = Field(
        default=None,
        description="Maximum date for filtering",
    )
    date_field: DateField | None = Field(
        default=None,
        description="Date field to filter on",
    )
    phase: TrialPhase | None = Field(
        default=None,
        description="Trial phase filter",
    )
    age_group: AgeGroup | None = Field(
        default=None,
        description="Age group filter",
    )
    primary_purpose: PrimaryPurpose | None = Field(
        default=None,
        description="Primary purpose of the trial",
    )
    intervention_type: InterventionType | None = Field(
        default=None,
        description="Type of intervention",
    )
    sponsor_type: SponsorType | None = Field(
        default=None,
        description="Type of sponsor",
    )
    study_design: StudyDesign | None = Field(
        default=None,
        description="Study design",
    )
    sort: SortOrder | None = Field(
        default=None,
        description="Sort order for results",
    )
    next_page_hash: str | None = Field(
        default=None,
        description="Token to retrieve the next page of results",
    )

    # Field validators for list fields
    @field_validator(
        "conditions",
        "terms",
        "interventions",
        "nct_ids",
        mode="before",
    )
    @classmethod
    def validate_list_fields(cls, value: Any) -> list[Any]:
        """Convert any field to a list or None."""
        return ensure_list(value)


def convert_query(query: TrialQuery) -> dict[str, list[str]]:  # noqa: C901
    """Convert a TrialQuery object into a dict of query params
    for the ClinicalTrials.gov API (v2). Each key maps to one or
    more strings in a list, consistent with parse_qs outputs.
    """
    # Start with required fields
    params: dict[str, list[str]] = {
        "format": [DEFAULT_FORMAT],
        "markupFormat": [DEFAULT_MARKUP],
    }

    # Handle conditions, terms, interventions
    for key, val in [
        ("query.cond", query.conditions),
        ("query.term", query.terms),
        ("query.intr", query.interventions),
    ]:
        if val:
            if len(val) == 1:
                params[key] = [val[0]]
            else:
                # Join multiple terms with OR, wrapped in parentheses
                params[key] = [f"({' OR '.join(val)})"]

    # Geospatial
    if query.lat is not None and query.long is not None:
        geo_val = f"distance({query.lat},{query.long},{query.distance}mi)"
        params["filter.geo"] = [geo_val]

    # Collect advanced filters in a list
    advanced_filters: list[str] = []

    # Date filter
    if query.date_field and (query.min_date or query.max_date):
        date_field = CTGOV_DATE_FIELD_MAPPING[query.date_field]
        min_val = query.min_date or "MIN"
        max_val = query.max_date or "MAX"
        advanced_filters.append(
            f"AREA[{date_field}]RANGE[{min_val},{max_val}]",
        )

    # Prepare a map of "AREA[...] -> (query_value, mapping_dict)"
    advanced_map = {
        "DesignPrimaryPurpose": (
            query.primary_purpose,
            CTGOV_PRIMARY_PURPOSE_MAPPING,
        ),
        "StudyType": (query.study_type, CTGOV_STUDY_TYPE_MAPPING),
        "InterventionType": (
            query.intervention_type,
            CTGOV_INTERVENTION_TYPE_MAPPING,
        ),
        "SponsorType": (query.sponsor_type, CTGOV_SPONSOR_TYPE_MAPPING),
        "StudyDesign": (query.study_design, CTGOV_STUDY_DESIGN_MAPPING),
        "Phase": (query.phase, CTGOV_PHASE_MAPPING),
    }

    # Append advanced filters
    for area, (qval, mapping) in advanced_map.items():
        if qval:
            # Check if mapping is a dict before using get method
            mapped = (
                mapping.get(qval)
                if mapping and isinstance(mapping, dict)
                else None
            )
            # Use the first mapped value if available, otherwise the literal
            value = mapped[0] if mapped else qval
            advanced_filters.append(f"AREA[{area}]{value}")

    # Age group
    if query.age_group and query.age_group != "ALL":
        mapped = CTGOV_AGE_GROUP_MAPPING[query.age_group]
        if mapped:
            advanced_filters.append(f"AREA[StdAge]{mapped[0]}")
        else:
            advanced_filters.append(f"AREA[StdAge]{query.age_group}")

    # If we collected any advanced filters, join them with AND
    if advanced_filters:
        params["filter.advanced"] = [" AND ".join(advanced_filters)]

    # Recruiting status
    if (
        query.recruiting_status is None
        or query.recruiting_status == RecruitingStatus.OPEN
    ):
        params["filter.overallStatus"] = [",".join(OPEN_STATUSES)]
    elif query.recruiting_status != RecruitingStatus.ANY:
        statuses = CTGOV_RECRUITING_STATUS_MAPPING.get(query.recruiting_status)
        if statuses:
            params["filter.overallStatus"] = [",".join(statuses)]

    # NCT IDs
    if query.nct_ids:
        params["query.id"] = [",".join(query.nct_ids)]

    # Sort & paging
    if query.sort is None:
        sort_val = CTGOV_SORT_MAPPING[SortOrder.RELEVANCE]
    else:
        sort_val = CTGOV_SORT_MAPPING.get(query.sort, query.sort)

    params["sort"] = [sort_val]
    if query.next_page_hash:
        params["pageToken"] = [query.next_page_hash]

    # Finally, add fields to limit payload size
    params["fields"] = SEARCH_FIELDS_PARAM
    params["pageSize"] = ["40"]

    return params


async def search_trials(
    query: TrialQuery,
    output_json: bool = False,
) -> str:
    """Search ClinicalTrials.gov for clinical trials."""
    params = convert_query(query)

    response, error = await http_client.request_api(
        url=const.CT_GOV_STUDIES,
        request=params,
        method="GET",
        tls_version=TLSVersion.TLSv1_2,
    )

    data = response
    if error:
        data = {"error": f"Error {error.code}: {error.message}"}

    if data and not output_json:
        return render.to_markdown(data)
    else:
        return json.dumps(data, indent=2)


@mcp_app.tool()
async def trial_searcher(
    conditions=None,
    terms=None,
    interventions=None,
    recruiting_status=None,
    study_type=None,
    nct_ids=None,
    lat=None,
    long=None,
    distance=None,
    min_date=None,
    max_date=None,
    date_field=None,
    phase=None,
    age_group=None,
    primary_purpose=None,
    intervention_type=None,
    sponsor_type=None,
    study_design=None,
    sort=None,
    next_page_hash=None,
) -> str:
    """
    Searches for clinical trials based on specified criteria.

    Parameters:
    - conditions: Condition terms (e.g., "breast cancer")
    - terms: General search terms
    - interventions: Intervention names (e.g., "pembrolizumab")
    - recruiting_status: Study recruitment status (OPEN, CLOSED, ANY)
    - study_type: Type of study
    - nct_ids: Clinical trial NCT IDs
    - lat: Latitude for location search
    - long: Longitude for location search
    - distance: Distance from lat/long in miles
    - min_date: Minimum date for filtering (YYYY-MM-DD)
    - max_date: Maximum date for filtering (YYYY-MM-DD)
    - date_field: Date field to filter on
    - phase: Trial phase filter
    - age_group: Age group filter
    - primary_purpose: Primary purpose of the trial
    - intervention_type: Type of intervention
    - sponsor_type: Type of sponsor
    - study_design: Study design
    - sort: Sort order for results
    - next_page_hash: Token to retrieve the next page of results

    Returns:
    Markdown formatted list of clinical trials
    """
    # Convert individual parameters to a TrialQuery object
    query = TrialQuery(
        conditions=conditions,
        terms=terms,
        interventions=interventions,
        recruiting_status=recruiting_status,
        study_type=study_type,
        nct_ids=nct_ids,
        lat=lat,
        long=long,
        distance=distance,
        min_date=min_date,
        max_date=max_date,
        date_field=date_field,
        phase=phase,
        age_group=age_group,
        primary_purpose=primary_purpose,
        intervention_type=intervention_type,
        sponsor_type=sponsor_type,
        study_design=study_design,
        sort=sort,
        next_page_hash=next_page_hash,
    )
    return await search_trials(query, output_json=False)
