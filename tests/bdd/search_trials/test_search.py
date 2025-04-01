import asyncio
from typing import Any

from pytest_bdd import given, parsers, scenarios, then, when

from biomcp.trials.search import (
    AgeGroup,
    DateField,
    InterventionType,
    PrimaryPurpose,
    RecruitingStatus,
    SortOrder,
    SponsorType,
    StudyDesign,
    StudyType,
    TrialPhase,
    TrialQuery,
    search_trials,
)

scenarios("search.feature")


@given(
    parsers.parse('I build a trial query with condition "{condition}"'),
    target_fixture="trial_query",
)
def trial_query(condition: str) -> TrialQuery:
    return TrialQuery(conditions=[condition])


@given(
    parsers.parse('I build a trial query with term "{term}"'),
    target_fixture="trial_query",
)
def trial_query_with_term(term: str) -> TrialQuery:
    return TrialQuery(terms=[term])


@given(
    parsers.parse('I build a trial query with nct_id "{nct_id}"'),
    target_fixture="trial_query",
)
def trial_query_with_nct_id(nct_id: str) -> TrialQuery:
    return TrialQuery(nct_ids=[nct_id])


@given(parsers.parse('I add intervention "{intervention}"'))
def add_intervention(trial_query: TrialQuery, intervention: str):
    trial_query.interventions = [intervention]


@given(parsers.parse('I set recruiting status to "{status}"'))
def set_recruiting_status(trial_query: TrialQuery, status: RecruitingStatus):
    trial_query.recruiting_status = status


@given(parsers.parse('I set study type to "{study_type}"'))
def set_study_type(trial_query: TrialQuery, study_type: StudyType):
    trial_query.study_type = study_type


@given(parsers.parse('I set phase to "{phase}"'))
def set_phase(trial_query: TrialQuery, phase: TrialPhase):
    trial_query.phase = phase


@given(parsers.parse('I set sort order to "{sort_order}"'))
def set_sort_order(trial_query: TrialQuery, sort_order: SortOrder):
    trial_query.sort = sort_order


@given(
    parsers.parse(
        'I set location to latitude "{lat}" longitude "{lon}" within "{distance}" miles',
    ),
)
def set_location(trial_query: TrialQuery, lat: str, lon: str, distance: str):
    trial_query.lat = float(lat)
    trial_query.long = float(lon)
    trial_query.distance = int(distance)


@given(parsers.parse('I set age group to "{age_group}"'))
def set_age_group(trial_query: TrialQuery, age_group: AgeGroup):
    trial_query.age_group = age_group


@given(parsers.parse('I set primary purpose to "{purpose}"'))
def set_primary_purpose(trial_query: TrialQuery, purpose: PrimaryPurpose):
    trial_query.primary_purpose = purpose


@given(parsers.parse('I set min date to "{min_date}"'))
def set_min_date(trial_query: TrialQuery, min_date: str):
    trial_query.min_date = min_date


@given(parsers.parse('I set max date to "{max_date}"'))
def set_max_date(trial_query: TrialQuery, max_date: str):
    trial_query.max_date = max_date


@given(parsers.parse('I set date field to "{date_field}"'))
def set_date_field(trial_query: TrialQuery, date_field: DateField):
    trial_query.date_field = date_field


@given(parsers.parse('I set intervention type to "{intervention_type}"'))
def set_intervention_type(
    trial_query: TrialQuery, intervention_type: InterventionType
):
    trial_query.intervention_type = intervention_type


@given(parsers.parse('I set sponsor type to "{sponsor_type}"'))
def set_sponsor_type(trial_query: TrialQuery, sponsor_type: SponsorType):
    trial_query.sponsor_type = sponsor_type


@given(parsers.parse('I set study design to "{study_design}"'))
def set_study_design(trial_query: TrialQuery, study_design: StudyDesign):
    trial_query.study_design = study_design


@when("I perform a trial search", target_fixture="trial_results")
def trial_results(trial_query: TrialQuery):
    """
    Perform a trial search and convert the markdown response to JSON
    for easier parsing in the test assertions.
    """
    return asyncio.run(search_trials(trial_query, output_json=True))


@then(
    parsers.parse(
        'the response should contain a study with condition "{condition}"',
    ),
)
def check_condition(trial_results: dict[str, Any], condition: str):
    """Verify that studies are returned for the condition query."""


@then(
    parsers.parse(
        'the response should contain a study with term "{term}"',
    ),
)
def check_term(trial_results: dict[str, Any], term: str):
    """Verify that studies are returned for the term query."""


@then(
    parsers.parse(
        'the response should contain a study with NCT ID "{nct_id}"',
    ),
)
def check_specific_nct_id(trial_results: dict[str, Any], nct_id: str):
    """Verify that the specific NCT ID is in the results."""


@then("the study should have a valid NCT ID")
def check_nct_id(trial_results: dict[str, Any]):
    """Verify that the NCT ID is valid."""


@then(parsers.parse('the study should include intervention "{intervention}"'))
def check_intervention(trial_results: dict[str, Any], intervention: str):
    """Verify that studies are returned for the intervention query."""


@then(parsers.parse('the study should be of type "{study_type}"'))
def check_study_type(trial_results: dict[str, Any], study_type: str):
    """Check if the study has the expected study type."""


@then(parsers.parse('the study should be in phase "{phase}"'))
def check_phase(trial_results: dict[str, Any], phase: str):
    """Check if the study has the expected phase."""


@then(parsers.parse('the studies should be sorted by "{sort_field}"'))
def check_sort_order(trial_results: dict[str, Any], sort_field: str):
    """Verify that results are sorted in the expected order."""


@then(parsers.parse('at least one study location should be in "{state}"'))
def check_location_state(trial_results: dict[str, Any], state: str):
    """Verify that studies are returned for the location query."""


@then("the study should have required fields")
def check_required_fields(trial_results: dict[str, Any]):
    """Verify all required fields are present in the search results."""


@then(parsers.parse('the study should have recruiting status "{status}"'))
def check_recruiting_status(trial_results: dict[str, Any], status: str):
    """Check if the study has the expected recruiting status."""


@then(parsers.parse('the study should include age group "{age_group}"'))
def check_age_group(trial_results: dict[str, Any], age_group: str):
    """Check if the study includes the expected age group."""


@then(parsers.parse('the study should have primary purpose "{purpose}"'))
def check_primary_purpose(trial_results: dict[str, Any], purpose: str):
    """Check if the study has the expected primary purpose."""


@then(parsers.parse('the study should have a start date after "{min_date}"'))
def check_start_date(trial_results: dict[str, Any], min_date: str):
    """Check if the study has a start date after the specified date."""


@then(
    parsers.parse(
        'the study should have intervention type "{intervention_type}"'
    )
)
def check_intervention_type(
    trial_results: dict[str, Any], intervention_type: str
):
    """Check if the study has the expected intervention type."""


@then(
    parsers.parse('the study should have a sponsor of type "{sponsor_type}"')
)
def check_sponsor_type(trial_results: dict[str, Any], sponsor_type: str):
    """Check if the study has a sponsor of the expected type."""


@then(parsers.parse('the study should have design "{study_design}"'))
def check_study_design(trial_results: dict[str, Any], study_design: str):
    """Check if the study has the expected study design."""


@then("the response should contain studies")
def check_studies_present(trial_results: dict[str, Any]):
    """Verify that studies are returned in the response."""
