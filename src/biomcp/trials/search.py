import json
import logging
from ssl import TLSVersion
from typing import Annotated

from pydantic import BaseModel, Field, field_validator, model_validator

from .. import StrEnum, ensure_list, http_client, render
from ..constants import CLINICAL_TRIALS_BASE_URL

logger = logging.getLogger(__name__)


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


class LineOfTherapy(StrEnum):
    FIRST_LINE = "1L"
    SECOND_LINE = "2L"
    THIRD_LINE_PLUS = "3L+"


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

# Line of therapy patterns for EligibilityCriteria search
LINE_OF_THERAPY_PATTERNS = {
    LineOfTherapy.FIRST_LINE: [
        '"first line"',
        '"first-line"',
        '"1st line"',
        '"frontline"',
        '"treatment naive"',
        '"previously untreated"',
    ],
    LineOfTherapy.SECOND_LINE: [
        '"second line"',
        '"second-line"',
        '"2nd line"',
        '"one prior line"',
        '"1 prior line"',
    ],
    LineOfTherapy.THIRD_LINE_PLUS: [
        '"third line"',
        '"third-line"',
        '"3rd line"',
        '"≥2 prior"',
        '"at least 2 prior"',
        '"heavily pretreated"',
    ],
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
        description="Study recruitment status. Use 'OPEN' for actively recruiting trials, 'CLOSED' for completed/terminated trials, or 'ANY' for all trials. Common aliases like 'recruiting', 'active', 'enrolling' map to 'OPEN'.",
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
        description="Latitude for location search. AI agents should geocode city/location names (e.g., 'Cleveland' → 41.4993, -81.6944) before using this parameter.",
    )
    long: float | None = Field(
        default=None,
        description="Longitude for location search. AI agents should geocode city/location names (e.g., 'Cleveland' → 41.4993, -81.6944) before using this parameter.",
    )
    distance: int | None = Field(
        default=None,
        description="Distance from lat/long in miles (default: 50 miles if lat/long provided but distance not specified)",
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
    # New eligibility-focused fields
    prior_therapies: list[str] | None = Field(
        default=None,
        description="Prior therapies to search for in eligibility criteria",
    )
    progression_on: list[str] | None = Field(
        default=None,
        description="Therapies the patient has progressed on",
    )
    required_mutations: list[str] | None = Field(
        default=None,
        description="Required mutations in eligibility criteria",
    )
    excluded_mutations: list[str] | None = Field(
        default=None,
        description="Excluded mutations in eligibility criteria",
    )
    biomarker_expression: dict[str, str] | None = Field(
        default=None,
        description="Biomarker expression requirements (e.g., {'PD-L1': '≥50%'})",
    )
    line_of_therapy: LineOfTherapy | None = Field(
        default=None,
        description="Line of therapy filter",
    )
    allow_brain_mets: bool | None = Field(
        default=None,
        description="Whether to allow trials that accept brain metastases",
    )
    return_fields: list[str] | None = Field(
        default=None,
        description="Specific fields to return in the response",
    )
    page_size: int | None = Field(
        default=None,
        description="Number of results per page",
        ge=1,
        le=1000,
    )

    @field_validator("recruiting_status", mode="before")
    @classmethod
    def normalize_recruiting_status(cls, v):
        """Normalize common recruiting status aliases to enum values."""
        if isinstance(v, str):
            v_lower = v.lower()
            # Map common aliases
            alias_map = {
                "recruiting": "OPEN",
                "active": "OPEN",
                "enrolling": "OPEN",
                "closed": "CLOSED",
                "completed": "CLOSED",
                "terminated": "CLOSED",
            }
            return alias_map.get(v_lower, v)
        return v

    # Field validators for list fields
    @model_validator(mode="before")
    def convert_list_fields(cls, data):
        """Convert string values to lists for list fields."""
        if isinstance(data, dict):
            for field_name in [
                "conditions",
                "terms",
                "interventions",
                "nct_ids",
                "prior_therapies",
                "progression_on",
                "required_mutations",
                "excluded_mutations",
                "return_fields",
            ]:
                if field_name in data and data[field_name] is not None:
                    data[field_name] = ensure_list(
                        data[field_name], split_strings=True
                    )
        return data


def _inject_ids(
    params: dict[str, list[str]], ids: list[str], has_other_filters: bool
) -> None:
    """Inject NCT IDs into params using intersection or id-only semantics.

    Args:
        params: The parameter dictionary to modify
        ids: List of NCT IDs to inject
        has_other_filters: Whether other filters are present
    """
    ids_csv = ",".join(ids)
    if has_other_filters:  # intersection path
        params["filter.ids"] = [ids_csv]
    elif len(ids_csv) < 1800:  # pure-ID & small
        params["query.id"] = [ids_csv]
    else:  # pure-ID & large
        params["filter.ids"] = [ids_csv]


def _build_prior_therapy_essie(therapies: list[str]) -> list[str]:
    """Build Essie fragments for prior therapy search."""
    fragments = []
    for therapy in therapies:
        if therapy.strip():  # Skip empty strings
            fragment = f'AREA[EligibilityCriteria]("{therapy}" AND (prior OR previous OR received))'
            fragments.append(fragment)
    return fragments


def _build_progression_essie(therapies: list[str]) -> list[str]:
    """Build Essie fragments for progression on therapy search."""
    fragments = []
    for therapy in therapies:
        if therapy.strip():  # Skip empty strings
            fragment = f'AREA[EligibilityCriteria]("{therapy}" AND (progression OR resistant OR refractory))'
            fragments.append(fragment)
    return fragments


def _build_required_mutations_essie(mutations: list[str]) -> list[str]:
    """Build Essie fragments for required mutations."""
    fragments = []
    for mutation in mutations:
        if mutation.strip():  # Skip empty strings
            fragment = f'AREA[EligibilityCriteria]("{mutation}")'
            fragments.append(fragment)
    return fragments


def _build_excluded_mutations_essie(mutations: list[str]) -> list[str]:
    """Build Essie fragments for excluded mutations."""
    fragments = []
    for mutation in mutations:
        if mutation.strip():  # Skip empty strings
            fragment = f'AREA[EligibilityCriteria](NOT "{mutation}")'
            fragments.append(fragment)
    return fragments


def _build_biomarker_expression_essie(biomarkers: dict[str, str]) -> list[str]:
    """Build Essie fragments for biomarker expression requirements."""
    fragments = []
    for marker, expression in biomarkers.items():
        if marker.strip() and expression.strip():  # Skip empty values
            fragment = (
                f'AREA[EligibilityCriteria]("{marker}" AND "{expression}")'
            )
            fragments.append(fragment)
    return fragments


def _build_line_of_therapy_essie(line: LineOfTherapy) -> str:
    """Build Essie fragment for line of therapy."""
    patterns = LINE_OF_THERAPY_PATTERNS.get(line, [])
    if patterns:
        # Join all patterns with OR within a single AREA block
        pattern_str = " OR ".join(patterns)
        return f"AREA[EligibilityCriteria]({pattern_str})"
    return ""


def _build_brain_mets_essie(allow: bool) -> str:
    """Build Essie fragment for brain metastases filter."""
    if allow is False:
        return 'AREA[EligibilityCriteria](NOT "brain metastases")'
    return ""


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

    # Track whether we have other filters (for NCT ID intersection logic)
    has_other_filters = False

    # Handle conditions, terms, interventions
    for key, val in [
        ("query.cond", query.conditions),
        ("query.term", query.terms),
        ("query.intr", query.interventions),
    ]:
        if val:
            has_other_filters = True
            if len(val) == 1:
                params[key] = [val[0]]
            else:
                # Join multiple terms with OR, wrapped in parentheses
                params[key] = [f"({' OR '.join(val)})"]

    # Collect Essie fragments for eligibility criteria
    essie_fragments: list[str] = []

    # Prior therapies
    if query.prior_therapies:
        has_other_filters = True
        essie_fragments.extend(
            _build_prior_therapy_essie(query.prior_therapies)
        )

    # Progression on therapies
    if query.progression_on:
        has_other_filters = True
        essie_fragments.extend(_build_progression_essie(query.progression_on))

    # Required mutations
    if query.required_mutations:
        has_other_filters = True
        essie_fragments.extend(
            _build_required_mutations_essie(query.required_mutations)
        )

    # Excluded mutations
    if query.excluded_mutations:
        has_other_filters = True
        essie_fragments.extend(
            _build_excluded_mutations_essie(query.excluded_mutations)
        )

    # Biomarker expression
    if query.biomarker_expression:
        has_other_filters = True
        essie_fragments.extend(
            _build_biomarker_expression_essie(query.biomarker_expression)
        )

    # Line of therapy
    if query.line_of_therapy:
        has_other_filters = True
        line_fragment = _build_line_of_therapy_essie(query.line_of_therapy)
        if line_fragment:
            essie_fragments.append(line_fragment)

    # Brain metastases filter
    if query.allow_brain_mets is not None:
        has_other_filters = True
        brain_fragment = _build_brain_mets_essie(query.allow_brain_mets)
        if brain_fragment:
            essie_fragments.append(brain_fragment)

    # Combine all Essie fragments with AND and append to query.term
    if essie_fragments:
        combined_essie = " AND ".join(essie_fragments)
        if "query.term" in params:
            # Append to existing terms with AND
            params["query.term"][0] = (
                f"{params['query.term'][0]} AND {combined_essie}"
            )
        else:
            params["query.term"] = [combined_essie]

    # Geospatial
    if query.lat is not None and query.long is not None:
        has_other_filters = True
        geo_val = f"distance({query.lat},{query.long},{query.distance}mi)"
        params["filter.geo"] = [geo_val]

    # Collect advanced filters in a list
    advanced_filters: list[str] = []

    # Date filter
    if query.date_field and (query.min_date or query.max_date):
        has_other_filters = True
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
            has_other_filters = True
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
        has_other_filters = True
        mapped = CTGOV_AGE_GROUP_MAPPING[query.age_group]
        if mapped:
            advanced_filters.append(f"AREA[StdAge]{mapped[0]}")
        else:
            advanced_filters.append(f"AREA[StdAge]{query.age_group}")

    # If we collected any advanced filters, join them with AND
    if advanced_filters:
        params["filter.advanced"] = [" AND ".join(advanced_filters)]

    # NCT IDs - now using intersection semantics
    # Must be done BEFORE recruiting status to properly detect user-set filters
    if query.nct_ids:
        _inject_ids(params, query.nct_ids, has_other_filters)

    # Recruiting status - apply AFTER NCT ID injection
    # Only count as a user filter if explicitly set to something other than default
    if query.recruiting_status not in (None, RecruitingStatus.OPEN):
        # User explicitly set a non-default status
        if query.recruiting_status is not None:  # Type guard for mypy
            statuses = CTGOV_RECRUITING_STATUS_MAPPING.get(
                query.recruiting_status
            )
            if statuses:
                params["filter.overallStatus"] = [",".join(statuses)]
    elif not query.nct_ids or has_other_filters:
        # Apply default OPEN status only if:
        # 1. No NCT IDs provided, OR
        # 2. NCT IDs provided with other filters (intersection mode)
        params["filter.overallStatus"] = [",".join(OPEN_STATUSES)]

    # Sort & paging
    if query.sort is None:
        sort_val = CTGOV_SORT_MAPPING[SortOrder.RELEVANCE]
    else:
        sort_val = CTGOV_SORT_MAPPING.get(query.sort, query.sort)

    params["sort"] = [sort_val]
    if query.next_page_hash:
        params["pageToken"] = [query.next_page_hash]

    # Finally, add fields to limit payload size
    if query.return_fields:
        # Use custom fields if specified
        params["fields"] = [",".join(query.return_fields)]
    else:
        # Use default fields
        params["fields"] = SEARCH_FIELDS_PARAM

    # Set page size
    if query.page_size:
        params["pageSize"] = [str(query.page_size)]
    else:
        params["pageSize"] = ["40"]

    return params


async def search_trials(
    query: TrialQuery,
    output_json: bool = False,
) -> str:
    """Search ClinicalTrials.gov for clinical trials."""
    params = convert_query(query)

    # Log filter mode if NCT IDs are present
    if query.nct_ids:
        # Check if we're using intersection or id-only mode
        # Only count explicit user-set filters, not defaults
        has_other_filters = any([
            query.conditions,
            query.terms,
            query.interventions,
            query.lat is not None and query.long is not None,
            query.date_field and (query.min_date or query.max_date),
            query.primary_purpose,
            query.study_type,
            query.intervention_type,
            query.sponsor_type,
            query.study_design,
            query.phase,
            query.age_group and query.age_group != AgeGroup.ALL,
            query.recruiting_status not in (None, RecruitingStatus.OPEN),
            query.prior_therapies,
            query.progression_on,
            query.required_mutations,
            query.excluded_mutations,
            query.biomarker_expression,
            query.line_of_therapy,
            query.allow_brain_mets is not None,
        ])

        if has_other_filters:
            logger.debug(
                "Filter mode: intersection (NCT IDs AND other filters)"
            )
        else:
            logger.debug("Filter mode: id-only (NCT IDs only)")

    response, error = await http_client.request_api(
        url=CLINICAL_TRIALS_BASE_URL,
        request=params,
        method="GET",
        tls_version=TLSVersion.TLSv1_2,
        domain="trial",
    )

    data = response
    if error:
        data = {"error": f"Error {error.code}: {error.message}"}

    if data and not output_json:
        return render.to_markdown(data)
    else:
        return json.dumps(data, indent=2)


async def _trial_searcher(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    conditions: Annotated[
        list[str] | str | None,
        "Condition terms (e.g., 'breast cancer') - list or comma-separated string",
    ] = None,
    terms: Annotated[
        list[str] | str | None,
        "General search terms - list or comma-separated string",
    ] = None,
    interventions: Annotated[
        list[str] | str | None,
        "Intervention names (e.g., 'pembrolizumab') - list or comma-separated string",
    ] = None,
    recruiting_status: Annotated[
        RecruitingStatus | str | None,
        "Study recruitment status (OPEN, CLOSED, ANY)",
    ] = None,
    study_type: Annotated[StudyType | str | None, "Type of study"] = None,
    nct_ids: Annotated[
        list[str] | str | None,
        "Clinical trial NCT IDs - list or comma-separated string",
    ] = None,
    lat: Annotated[
        float | None,
        "Latitude for location search. AI agents should geocode city/location names (e.g., 'Cleveland' → 41.4993, -81.6944) before using this parameter.",
    ] = None,
    long: Annotated[
        float | None,
        "Longitude for location search. AI agents should geocode city/location names (e.g., 'Cleveland' → 41.4993, -81.6944) before using this parameter.",
    ] = None,
    distance: Annotated[
        float | None,
        "Distance from lat/long in miles (default: 50 miles if lat/long provided but distance not specified)",
    ] = None,
    min_date: Annotated[
        str | None, "Minimum date for filtering (YYYY-MM-DD)"
    ] = None,
    max_date: Annotated[
        str | None, "Maximum date for filtering (YYYY-MM-DD)"
    ] = None,
    date_field: Annotated[
        DateField | str | None, "Date field to filter on"
    ] = None,
    phase: Annotated[TrialPhase | str | None, "Trial phase filter"] = None,
    age_group: Annotated[AgeGroup | str | None, "Age group filter"] = None,
    primary_purpose: Annotated[
        PrimaryPurpose | str | None, "Primary purpose of the trial"
    ] = None,
    intervention_type: Annotated[
        InterventionType | str | None, "Type of intervention"
    ] = None,
    sponsor_type: Annotated[
        SponsorType | str | None, "Type of sponsor"
    ] = None,
    study_design: Annotated[StudyDesign | str | None, "Study design"] = None,
    sort: Annotated[SortOrder | str | None, "Sort order for results"] = None,
    next_page_hash: Annotated[
        str | None, "Token to retrieve the next page of results"
    ] = None,
    prior_therapies: Annotated[
        list[str] | str | None,
        "Prior therapies to search for in eligibility criteria - list or comma-separated string",
    ] = None,
    progression_on: Annotated[
        list[str] | str | None,
        "Therapies the patient has progressed on - list or comma-separated string",
    ] = None,
    required_mutations: Annotated[
        list[str] | str | None,
        "Required mutations in eligibility criteria - list or comma-separated string",
    ] = None,
    excluded_mutations: Annotated[
        list[str] | str | None,
        "Excluded mutations in eligibility criteria - list or comma-separated string",
    ] = None,
    biomarker_expression: Annotated[
        dict[str, str] | None,
        "Biomarker expression requirements (e.g., {'PD-L1': '≥50%'})",
    ] = None,
    line_of_therapy: Annotated[
        LineOfTherapy | str | None,
        "Line of therapy filter (1L, 2L, 3L+)",
    ] = None,
    allow_brain_mets: Annotated[
        bool | None,
        "Whether to allow trials that accept brain metastases",
    ] = None,
    return_fields: Annotated[
        list[str] | str | None,
        "Specific fields to return in the response - list or comma-separated string",
    ] = None,
    page_size: Annotated[
        int | None,
        "Number of results per page (1-1000)",
    ] = None,
) -> str:
    """
    Searches for clinical trials based on specified criteria.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - conditions: Condition terms (e.g., "breast cancer") - list or comma-separated string
    - terms: General search terms - list or comma-separated string
    - interventions: Intervention names (e.g., "pembrolizumab") - list or comma-separated string
    - recruiting_status: Study recruitment status (OPEN, CLOSED, ANY)
    - study_type: Type of study
    - nct_ids: Clinical trial NCT IDs - list or comma-separated string
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
    - prior_therapies: Prior therapies to search for in eligibility criteria - list or comma-separated string
    - progression_on: Therapies the patient has progressed on - list or comma-separated string
    - required_mutations: Required mutations in eligibility criteria - list or comma-separated string
    - excluded_mutations: Excluded mutations in eligibility criteria - list or comma-separated string
    - biomarker_expression: Biomarker expression requirements (e.g., {'PD-L1': '≥50%'})
    - line_of_therapy: Line of therapy filter (1L, 2L, 3L+)
    - allow_brain_mets: Whether to allow trials that accept brain metastases
    - return_fields: Specific fields to return in the response - list or comma-separated string
    - page_size: Number of results per page (1-1000)

    Returns:
    Markdown formatted list of clinical trials
    """
    # Convert individual parameters to a TrialQuery object
    query = TrialQuery(
        conditions=ensure_list(conditions, split_strings=True),
        terms=ensure_list(terms, split_strings=True),
        interventions=ensure_list(interventions, split_strings=True),
        recruiting_status=recruiting_status,
        study_type=study_type,
        nct_ids=ensure_list(nct_ids, split_strings=True),
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
        prior_therapies=ensure_list(prior_therapies, split_strings=True),
        progression_on=ensure_list(progression_on, split_strings=True),
        required_mutations=ensure_list(required_mutations, split_strings=True),
        excluded_mutations=ensure_list(excluded_mutations, split_strings=True),
        biomarker_expression=biomarker_expression,
        line_of_therapy=line_of_therapy,
        allow_brain_mets=allow_brain_mets,
        return_fields=ensure_list(return_fields, split_strings=True),
        page_size=page_size,
    )
    return await search_trials(query, output_json=False)
