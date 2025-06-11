import pytest

from biomcp.trials.search import (
    CLOSED_STATUSES,
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
    _inject_ids,
    convert_query,
)


def test_convert_query_basic_parameters():
    """Test basic parameter conversion from TrialQuery to API format."""
    query = TrialQuery(conditions=["lung cancer"])
    params = convert_query(query)

    assert "markupFormat" in params
    assert params["markupFormat"] == ["markdown"]
    assert "query.cond" in params
    assert params["query.cond"] == ["lung cancer"]
    assert "filter.overallStatus" in params
    assert "RECRUITING" in params["filter.overallStatus"][0]


def test_convert_query_multiple_conditions():
    """Test conversion of multiple conditions to API format."""
    query = TrialQuery(conditions=["lung cancer", "metastatic"])
    params = convert_query(query)

    assert "query.cond" in params
    assert "(lung cancer OR metastatic)" in params["query.cond"][0]


def test_convert_query_terms_parameter():
    """Test conversion of terms parameter to API format."""
    query = TrialQuery(terms=["immunotherapy"])
    params = convert_query(query)

    assert "query.term" in params
    assert params["query.term"] == ["immunotherapy"]


def test_convert_query_interventions_parameter():
    """Test conversion of interventions parameter to API format."""
    query = TrialQuery(interventions=["pembrolizumab"])
    params = convert_query(query)

    assert "query.intr" in params
    assert params["query.intr"] == ["pembrolizumab"]


def test_convert_query_nct_ids():
    """Test conversion of NCT IDs to API format."""
    query = TrialQuery(nct_ids=["NCT04179552"])
    params = convert_query(query)

    assert "query.id" in params
    assert params["query.id"] == ["NCT04179552"]
    # Note: The implementation keeps filter.overallStatus when using nct_ids
    # So we don't assert its absence


def test_convert_query_recruiting_status():
    """Test conversion of recruiting status to API format."""
    # Test open status
    query = TrialQuery(recruiting_status=RecruitingStatus.OPEN)
    params = convert_query(query)

    assert "filter.overallStatus" in params
    assert "RECRUITING" in params["filter.overallStatus"][0]

    # Test closed status
    query = TrialQuery(recruiting_status=RecruitingStatus.CLOSED)
    params = convert_query(query)

    assert "filter.overallStatus" in params
    assert all(
        status in params["filter.overallStatus"][0]
        for status in CLOSED_STATUSES
    )

    # Test any status
    query = TrialQuery(recruiting_status=RecruitingStatus.ANY)
    params = convert_query(query)

    assert "filter.overallStatus" not in params


def test_convert_query_location_parameters():
    """Test conversion of location parameters to API format."""
    query = TrialQuery(lat=40.7128, long=-74.0060, distance=10)
    params = convert_query(query)

    assert "filter.geo" in params
    assert params["filter.geo"] == ["distance(40.7128,-74.006,10mi)"]


def test_convert_query_study_type():
    """Test conversion of study type to API format."""
    query = TrialQuery(study_type=StudyType.INTERVENTIONAL)
    params = convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[StudyType]Interventional" in params["filter.advanced"][0]


def test_convert_query_phase():
    """Test conversion of phase to API format."""
    query = TrialQuery(phase=TrialPhase.PHASE3)
    params = convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[Phase]PHASE3" in params["filter.advanced"][0]


def test_convert_query_date_range():
    """Test conversion of date range to API format."""
    query = TrialQuery(
        min_date="2020-01-01",
        max_date="2020-12-31",
        date_field=DateField.LAST_UPDATE,
    )
    params = convert_query(query)

    assert "filter.advanced" in params
    assert (
        "AREA[LastUpdatePostDate]RANGE[2020-01-01,2020-12-31]"
        in params["filter.advanced"][0]
    )

    # Test min date only
    query = TrialQuery(
        min_date="2021-01-01",
        date_field=DateField.STUDY_START,
    )
    params = convert_query(query)

    assert "filter.advanced" in params
    assert (
        "AREA[StartDate]RANGE[2021-01-01,MAX]" in params["filter.advanced"][0]
    )


def test_convert_query_sort_order():
    """Test conversion of sort order to API format."""
    query = TrialQuery(sort=SortOrder.RELEVANCE)
    params = convert_query(query)

    assert "sort" in params
    assert params["sort"] == ["@relevance"]

    query = TrialQuery(sort=SortOrder.LAST_UPDATE)
    params = convert_query(query)

    assert "sort" in params
    assert params["sort"] == ["LastUpdatePostDate:desc"]


def test_convert_query_intervention_type():
    """Test conversion of intervention type to API format."""
    query = TrialQuery(intervention_type=InterventionType.DRUG)
    params = convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[InterventionType]Drug" in params["filter.advanced"][0]


def test_convert_query_sponsor_type():
    """Test conversion of sponsor type to API format."""
    query = TrialQuery(sponsor_type=SponsorType.ACADEMIC)
    params = convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[SponsorType]Academic" in params["filter.advanced"][0]


def test_convert_query_study_design():
    """Test conversion of study design to API format."""
    query = TrialQuery(study_design=StudyDesign.RANDOMIZED)
    params = convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[StudyDesign]Randomized" in params["filter.advanced"][0]


def test_convert_query_age_group():
    """Test conversion of age group to API format."""
    query = TrialQuery(age_group=AgeGroup.ADULT)
    params = convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[StdAge]Adult" in params["filter.advanced"][0]


def test_convert_query_primary_purpose():
    """Test conversion of primary purpose to API format."""
    query = TrialQuery(primary_purpose=PrimaryPurpose.TREATMENT)
    params = convert_query(query)

    assert "filter.advanced" in params
    assert (
        "AREA[DesignPrimaryPurpose]Treatment" in params["filter.advanced"][0]
    )


def test_convert_query_next_page_hash():
    """Test conversion of next_page_hash to API format."""
    query = TrialQuery(next_page_hash="abc123")
    params = convert_query(query)

    assert "pageToken" in params
    assert params["pageToken"] == ["abc123"]


def test_convert_query_complex_parameters():
    """Test conversion of multiple parameters to API format."""
    query = TrialQuery(
        conditions=["diabetes"],
        terms=["obesity"],
        interventions=["metformin"],
        primary_purpose=PrimaryPurpose.TREATMENT,
        study_type=StudyType.INTERVENTIONAL,
        intervention_type=InterventionType.DRUG,
        recruiting_status=RecruitingStatus.OPEN,
        phase=TrialPhase.PHASE3,
        age_group=AgeGroup.ADULT,
        sort=SortOrder.RELEVANCE,
    )
    params = convert_query(query)

    assert "query.cond" in params
    assert params["query.cond"] == ["diabetes"]
    assert "query.term" in params
    assert params["query.term"] == ["obesity"]
    assert "query.intr" in params
    assert params["query.intr"] == ["metformin"]
    assert "filter.advanced" in params
    assert (
        "AREA[DesignPrimaryPurpose]Treatment" in params["filter.advanced"][0]
    )
    assert "AREA[StudyType]Interventional" in params["filter.advanced"][0]
    assert "AREA[InterventionType]Drug" in params["filter.advanced"][0]
    assert "AREA[Phase]PHASE3" in params["filter.advanced"][0]
    assert "AREA[StdAge]Adult" in params["filter.advanced"][0]
    assert "filter.overallStatus" in params
    assert "RECRUITING" in params["filter.overallStatus"][0]
    assert "sort" in params
    assert params["sort"] == ["@relevance"]


# Test TrialQuery field validation for CLI input processing
# noinspection PyTypeChecker
def test_trial_query_field_validation_basic():
    """Test basic field validation for TrialQuery."""
    # Test list fields conversion
    query = TrialQuery(conditions="diabetes")
    assert query.conditions == ["diabetes"]

    query = TrialQuery(interventions="metformin")
    assert query.interventions == ["metformin"]

    query = TrialQuery(terms="blood glucose")
    assert query.terms == ["blood glucose"]

    query = TrialQuery(nct_ids="NCT01234567")
    assert query.nct_ids == ["NCT01234567"]


# noinspection PyTypeChecker
def test_trial_query_field_validation_recruiting_status():
    """Test recruiting status field validation."""
    # Exact match uppercase
    query = TrialQuery(recruiting_status="OPEN")
    assert query.recruiting_status == RecruitingStatus.OPEN

    # Exact match lowercase
    query = TrialQuery(recruiting_status="closed")
    assert query.recruiting_status == RecruitingStatus.CLOSED

    # Invalid value
    with pytest.raises(ValueError) as excinfo:
        TrialQuery(recruiting_status="invalid")
    assert "validation error for TrialQuery" in str(excinfo.value)


# noinspection PyTypeChecker
def test_trial_query_field_validation_combined():
    """Test combined parameters validation."""
    query = TrialQuery(
        conditions=["diabetes", "obesity"],
        interventions="metformin",
        recruiting_status="open",
        study_type="interventional",
        lat=40.7128,
        long=-74.0060,
        distance=10,
    )

    assert query.conditions == ["diabetes", "obesity"]
    assert query.interventions == ["metformin"]
    assert query.recruiting_status == RecruitingStatus.OPEN
    assert query.study_type == StudyType.INTERVENTIONAL
    assert query.lat == 40.7128
    assert query.long == -74.0060
    assert query.distance == 10

    # Check that the query can be converted to parameters properly
    params = convert_query(query)
    assert "query.cond" in params
    assert "(diabetes OR obesity)" in params["query.cond"][0]
    assert "query.intr" in params
    assert "metformin" in params["query.intr"][0]
    assert "filter.geo" in params
    assert "distance(40.7128,-74.006,10mi)" in params["filter.geo"][0]


# noinspection PyTypeChecker
def test_trial_query_field_validation_terms():
    """Test terms parameter validation."""
    # Single term as string
    query = TrialQuery(terms="cancer")
    assert query.terms == ["cancer"]

    # Multiple terms as list
    query = TrialQuery(terms=["cancer", "therapy"])
    assert query.terms == ["cancer", "therapy"]

    # Check parameter generation
    params = convert_query(query)
    assert "query.term" in params
    assert "(cancer OR therapy)" in params["query.term"][0]


# noinspection PyTypeChecker
def test_trial_query_field_validation_nct_ids():
    """Test NCT IDs parameter validation."""
    # Single NCT ID
    query = TrialQuery(nct_ids="NCT01234567")
    assert query.nct_ids == ["NCT01234567"]

    # Multiple NCT IDs
    query = TrialQuery(nct_ids=["NCT01234567", "NCT89012345"])
    assert query.nct_ids == ["NCT01234567", "NCT89012345"]

    # Check parameter generation
    params = convert_query(query)
    assert "query.id" in params
    assert "NCT01234567,NCT89012345" in params["query.id"][0]


# noinspection PyTypeChecker
def test_trial_query_field_validation_date_range():
    """Test date range parameters validation."""
    # Min date only with date field
    query = TrialQuery(min_date="2020-01-01", date_field=DateField.STUDY_START)
    assert query.min_date == "2020-01-01"
    assert query.date_field == DateField.STUDY_START

    # Min and max date with date field using lazy mapping
    query = TrialQuery(
        min_date="2020-01-01",
        max_date="2021-12-31",
        date_field="last update",  # space not underscore.
    )
    assert query.min_date == "2020-01-01"
    assert query.max_date == "2021-12-31"
    assert query.date_field == DateField.LAST_UPDATE

    # Check parameter generation
    params = convert_query(query)
    assert "filter.advanced" in params
    assert (
        "AREA[LastUpdatePostDate]RANGE[2020-01-01,2021-12-31]"
        in params["filter.advanced"][0]
    )


# noinspection PyTypeChecker
def test_trial_query_field_validation_primary_purpose():
    """Test primary purpose parameter validation."""
    # Exact match uppercase
    query = TrialQuery(primary_purpose=PrimaryPurpose.TREATMENT)
    assert query.primary_purpose == PrimaryPurpose.TREATMENT

    # Exact match lowercase
    query = TrialQuery(primary_purpose=PrimaryPurpose.PREVENTION)
    assert query.primary_purpose == PrimaryPurpose.PREVENTION

    # Case-insensitive
    query = TrialQuery(primary_purpose="ScReeNING")
    assert query.primary_purpose == PrimaryPurpose.SCREENING

    # Invalid
    with pytest.raises(ValueError):
        TrialQuery(primary_purpose="invalid")


def test_inject_ids_with_many_ids_and_condition():
    """Test _inject_ids function with 300 IDs and a condition to ensure filter.ids is used."""
    # Create a params dict with a condition (indicating other filters present)
    params = {
        "query.cond": ["melanoma"],
        "format": ["json"],
        "markupFormat": ["markdown"],
    }

    # Generate 300 NCT IDs
    nct_ids = [f"NCT{str(i).zfill(8)}" for i in range(1, 301)]

    # Call _inject_ids with has_other_filters=True
    _inject_ids(params, nct_ids, has_other_filters=True)

    # Assert that filter.ids is used (not query.id)
    assert "filter.ids" in params
    assert "query.id" not in params

    # Verify the IDs are properly formatted
    ids_param = params["filter.ids"][0]
    assert ids_param.startswith("NCT")
    assert "NCT00000001" in ids_param
    assert "NCT00000300" in ids_param

    # Verify it's a comma-separated list
    assert "," in ids_param
    assert ids_param.count(",") == 299  # 300 IDs = 299 commas


def test_inject_ids_without_other_filters():
    """Test _inject_ids function with only NCT IDs (no other filters)."""
    # Create a minimal params dict
    params = {
        "format": ["json"],
        "markupFormat": ["markdown"],
    }

    # Use a small number of NCT IDs
    nct_ids = ["NCT00000001", "NCT00000002", "NCT00000003"]

    # Call _inject_ids with has_other_filters=False
    _inject_ids(params, nct_ids, has_other_filters=False)

    # Assert that query.id is used (not filter.ids) for small lists
    assert "query.id" in params
    assert "filter.ids" not in params

    # Verify the format
    assert params["query.id"][0] == "NCT00000001,NCT00000002,NCT00000003"


def test_inject_ids_large_list_without_filters():
    """Test _inject_ids with a large ID list but no other filters."""
    params = {
        "format": ["json"],
        "markupFormat": ["markdown"],
    }

    # Generate enough IDs to exceed 1800 character limit
    nct_ids = [f"NCT{str(i).zfill(8)}" for i in range(1, 201)]  # ~2200 chars

    # Call _inject_ids with has_other_filters=False
    _inject_ids(params, nct_ids, has_other_filters=False)

    # Assert that filter.ids is used for large lists even without other filters
    assert "filter.ids" in params
    assert "query.id" not in params
