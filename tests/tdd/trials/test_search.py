import pytest

from biomcp.trials.search import (
    CLOSED_STATUSES,
    AgeGroup,
    DateField,
    InterventionType,
    LineOfTherapy,
    PrimaryPurpose,
    RecruitingStatus,
    SortOrder,
    SponsorType,
    StudyDesign,
    StudyType,
    TrialPhase,
    TrialQuery,
    _build_biomarker_expression_essie,
    _build_brain_mets_essie,
    _build_excluded_mutations_essie,
    _build_line_of_therapy_essie,
    _build_prior_therapy_essie,
    _build_progression_essie,
    _build_required_mutations_essie,
    _inject_ids,
    convert_query,
)


@pytest.mark.asyncio
async def test_convert_query_basic_parameters():
    """Test basic parameter conversion from TrialQuery to API format."""
    query = TrialQuery(conditions=["lung cancer"])
    params = await convert_query(query)

    assert "markupFormat" in params
    assert params["markupFormat"] == ["markdown"]
    assert "query.cond" in params
    assert params["query.cond"] == ["lung cancer"]
    assert "filter.overallStatus" in params
    assert "RECRUITING" in params["filter.overallStatus"][0]


@pytest.mark.asyncio
async def test_convert_query_multiple_conditions():
    """Test conversion of multiple conditions to API format."""
    query = TrialQuery(conditions=["lung cancer", "metastatic"])
    params = await convert_query(query)

    assert "query.cond" in params
    # The query should contain the original terms, but may have expanded synonyms
    cond_value = params["query.cond"][0]
    assert "lung cancer" in cond_value
    assert "metastatic" in cond_value
    assert cond_value.startswith("(") and cond_value.endswith(")")


@pytest.mark.asyncio
async def test_convert_query_terms_parameter():
    """Test conversion of terms parameter to API format."""
    query = TrialQuery(terms=["immunotherapy"])
    params = await convert_query(query)

    assert "query.term" in params
    assert params["query.term"] == ["immunotherapy"]


@pytest.mark.asyncio
async def test_convert_query_interventions_parameter():
    """Test conversion of interventions parameter to API format."""
    query = TrialQuery(interventions=["pembrolizumab"])
    params = await convert_query(query)

    assert "query.intr" in params
    assert params["query.intr"] == ["pembrolizumab"]


@pytest.mark.asyncio
async def test_convert_query_nct_ids():
    """Test conversion of NCT IDs to API format."""
    query = TrialQuery(nct_ids=["NCT04179552"])
    params = await convert_query(query)

    assert "query.id" in params
    assert params["query.id"] == ["NCT04179552"]
    # Note: The implementation keeps filter.overallStatus when using nct_ids
    # So we don't assert its absence


@pytest.mark.asyncio
async def test_convert_query_recruiting_status():
    """Test conversion of recruiting status to API format."""
    # Test open status
    query = TrialQuery(recruiting_status=RecruitingStatus.OPEN)
    params = await convert_query(query)

    assert "filter.overallStatus" in params
    assert "RECRUITING" in params["filter.overallStatus"][0]

    # Test closed status
    query = TrialQuery(recruiting_status=RecruitingStatus.CLOSED)
    params = await convert_query(query)

    assert "filter.overallStatus" in params
    assert all(
        status in params["filter.overallStatus"][0]
        for status in CLOSED_STATUSES
    )

    # Test any status
    query = TrialQuery(recruiting_status=RecruitingStatus.ANY)
    params = await convert_query(query)

    assert "filter.overallStatus" not in params


@pytest.mark.asyncio
async def test_convert_query_location_parameters():
    """Test conversion of location parameters to API format."""
    query = TrialQuery(lat=40.7128, long=-74.0060, distance=10)
    params = await convert_query(query)

    assert "filter.geo" in params
    assert params["filter.geo"] == ["distance(40.7128,-74.006,10mi)"]


@pytest.mark.asyncio
async def test_convert_query_study_type():
    """Test conversion of study type to API format."""
    query = TrialQuery(study_type=StudyType.INTERVENTIONAL)
    params = await convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[StudyType]Interventional" in params["filter.advanced"][0]


@pytest.mark.asyncio
async def test_convert_query_phase():
    """Test conversion of phase to API format."""
    query = TrialQuery(phase=TrialPhase.PHASE3)
    params = await convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[Phase]PHASE3" in params["filter.advanced"][0]


@pytest.mark.asyncio
async def test_convert_query_date_range():
    """Test conversion of date range to API format."""
    query = TrialQuery(
        min_date="2020-01-01",
        max_date="2020-12-31",
        date_field=DateField.LAST_UPDATE,
    )
    params = await convert_query(query)

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
    params = await convert_query(query)

    assert "filter.advanced" in params
    assert (
        "AREA[StartDate]RANGE[2021-01-01,MAX]" in params["filter.advanced"][0]
    )


@pytest.mark.asyncio
async def test_convert_query_sort_order():
    """Test conversion of sort order to API format."""
    query = TrialQuery(sort=SortOrder.RELEVANCE)
    params = await convert_query(query)

    assert "sort" in params
    assert params["sort"] == ["@relevance"]

    query = TrialQuery(sort=SortOrder.LAST_UPDATE)
    params = await convert_query(query)

    assert "sort" in params
    assert params["sort"] == ["LastUpdatePostDate:desc"]


@pytest.mark.asyncio
async def test_convert_query_intervention_type():
    """Test conversion of intervention type to API format."""
    query = TrialQuery(intervention_type=InterventionType.DRUG)
    params = await convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[InterventionType]Drug" in params["filter.advanced"][0]


@pytest.mark.asyncio
async def test_convert_query_sponsor_type():
    """Test conversion of sponsor type to API format."""
    query = TrialQuery(sponsor_type=SponsorType.ACADEMIC)
    params = await convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[SponsorType]Academic" in params["filter.advanced"][0]


@pytest.mark.asyncio
async def test_convert_query_study_design():
    """Test conversion of study design to API format."""
    query = TrialQuery(study_design=StudyDesign.RANDOMIZED)
    params = await convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[StudyDesign]Randomized" in params["filter.advanced"][0]


@pytest.mark.asyncio
async def test_convert_query_age_group():
    """Test conversion of age group to API format."""
    query = TrialQuery(age_group=AgeGroup.ADULT)
    params = await convert_query(query)

    assert "filter.advanced" in params
    assert "AREA[StdAge]Adult" in params["filter.advanced"][0]


@pytest.mark.asyncio
async def test_convert_query_primary_purpose():
    """Test conversion of primary purpose to API format."""
    query = TrialQuery(primary_purpose=PrimaryPurpose.TREATMENT)
    params = await convert_query(query)

    assert "filter.advanced" in params
    assert (
        "AREA[DesignPrimaryPurpose]Treatment" in params["filter.advanced"][0]
    )


@pytest.mark.asyncio
async def test_convert_query_next_page_hash():
    """Test conversion of next_page_hash to API format."""
    query = TrialQuery(next_page_hash="abc123")
    params = await convert_query(query)

    assert "pageToken" in params
    assert params["pageToken"] == ["abc123"]


@pytest.mark.asyncio
async def test_convert_query_complex_parameters():
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
    params = await convert_query(query)

    assert "query.cond" in params
    # Disease synonym expansion may add synonyms to diabetes
    assert "diabetes" in params["query.cond"][0]
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
@pytest.mark.asyncio
async def test_trial_query_field_validation_combined():
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
    params = await convert_query(query)
    assert "query.cond" in params
    # The query should contain the original terms, but may have expanded synonyms
    cond_value = params["query.cond"][0]
    assert "diabetes" in cond_value
    assert "obesity" in cond_value
    assert cond_value.startswith("(") and cond_value.endswith(")")
    assert "query.intr" in params
    assert "metformin" in params["query.intr"][0]
    assert "filter.geo" in params
    assert "distance(40.7128,-74.006,10mi)" in params["filter.geo"][0]


# noinspection PyTypeChecker
@pytest.mark.asyncio
async def test_trial_query_field_validation_terms():
    """Test terms parameter validation."""
    # Single term as string
    query = TrialQuery(terms="cancer")
    assert query.terms == ["cancer"]

    # Multiple terms as list
    query = TrialQuery(terms=["cancer", "therapy"])
    assert query.terms == ["cancer", "therapy"]

    # Check parameter generation
    params = await convert_query(query)
    assert "query.term" in params
    assert "(cancer OR therapy)" in params["query.term"][0]


# noinspection PyTypeChecker
@pytest.mark.asyncio
async def test_trial_query_field_validation_nct_ids():
    """Test NCT IDs parameter validation."""
    # Single NCT ID
    query = TrialQuery(nct_ids="NCT01234567")
    assert query.nct_ids == ["NCT01234567"]

    # Multiple NCT IDs
    query = TrialQuery(nct_ids=["NCT01234567", "NCT89012345"])
    assert query.nct_ids == ["NCT01234567", "NCT89012345"]

    # Check parameter generation
    params = await convert_query(query)
    assert "query.id" in params
    assert "NCT01234567,NCT89012345" in params["query.id"][0]


# noinspection PyTypeChecker
@pytest.mark.asyncio
async def test_trial_query_field_validation_date_range():
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
    params = await convert_query(query)
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


# Tests for new Essie builder functions
def test_build_prior_therapy_essie():
    """Test building Essie fragments for prior therapies."""
    # Single therapy
    fragments = _build_prior_therapy_essie(["osimertinib"])
    assert len(fragments) == 1
    assert (
        fragments[0]
        == 'AREA[EligibilityCriteria]("osimertinib" AND (prior OR previous OR received))'
    )

    # Multiple therapies
    fragments = _build_prior_therapy_essie(["osimertinib", "erlotinib"])
    assert len(fragments) == 2
    assert (
        fragments[0]
        == 'AREA[EligibilityCriteria]("osimertinib" AND (prior OR previous OR received))'
    )
    assert (
        fragments[1]
        == 'AREA[EligibilityCriteria]("erlotinib" AND (prior OR previous OR received))'
    )

    # Empty strings are filtered out
    fragments = _build_prior_therapy_essie(["osimertinib", "", "erlotinib"])
    assert len(fragments) == 2


def test_build_progression_essie():
    """Test building Essie fragments for progression on therapy."""
    fragments = _build_progression_essie(["pembrolizumab"])
    assert len(fragments) == 1
    assert (
        fragments[0]
        == 'AREA[EligibilityCriteria]("pembrolizumab" AND (progression OR resistant OR refractory))'
    )


def test_build_required_mutations_essie():
    """Test building Essie fragments for required mutations."""
    fragments = _build_required_mutations_essie(["EGFR L858R", "T790M"])
    assert len(fragments) == 2
    assert fragments[0] == 'AREA[EligibilityCriteria]("EGFR L858R")'
    assert fragments[1] == 'AREA[EligibilityCriteria]("T790M")'


def test_build_excluded_mutations_essie():
    """Test building Essie fragments for excluded mutations."""
    fragments = _build_excluded_mutations_essie(["KRAS G12C"])
    assert len(fragments) == 1
    assert fragments[0] == 'AREA[EligibilityCriteria](NOT "KRAS G12C")'


def test_build_biomarker_expression_essie():
    """Test building Essie fragments for biomarker expression."""
    biomarkers = {"PD-L1": "≥50%", "TMB": "≥10 mut/Mb"}
    fragments = _build_biomarker_expression_essie(biomarkers)
    assert len(fragments) == 2
    assert 'AREA[EligibilityCriteria]("PD-L1" AND "≥50%")' in fragments
    assert 'AREA[EligibilityCriteria]("TMB" AND "≥10 mut/Mb")' in fragments

    # Empty values are filtered out
    biomarkers = {"PD-L1": "≥50%", "TMB": "", "HER2": "positive"}
    fragments = _build_biomarker_expression_essie(biomarkers)
    assert len(fragments) == 2


def test_build_line_of_therapy_essie():
    """Test building Essie fragment for line of therapy."""
    # First line
    fragment = _build_line_of_therapy_essie(LineOfTherapy.FIRST_LINE)
    assert (
        fragment
        == 'AREA[EligibilityCriteria]("first line" OR "first-line" OR "1st line" OR "frontline" OR "treatment naive" OR "previously untreated")'
    )

    # Second line
    fragment = _build_line_of_therapy_essie(LineOfTherapy.SECOND_LINE)
    assert (
        fragment
        == 'AREA[EligibilityCriteria]("second line" OR "second-line" OR "2nd line" OR "one prior line" OR "1 prior line")'
    )

    # Third line plus
    fragment = _build_line_of_therapy_essie(LineOfTherapy.THIRD_LINE_PLUS)
    assert (
        fragment
        == 'AREA[EligibilityCriteria]("third line" OR "third-line" OR "3rd line" OR "≥2 prior" OR "at least 2 prior" OR "heavily pretreated")'
    )


def test_build_brain_mets_essie():
    """Test building Essie fragment for brain metastases filter."""
    # Allow brain mets (no filter)
    fragment = _build_brain_mets_essie(True)
    assert fragment == ""

    # Exclude brain mets
    fragment = _build_brain_mets_essie(False)
    assert fragment == 'AREA[EligibilityCriteria](NOT "brain metastases")'


@pytest.mark.asyncio
async def test_convert_query_with_eligibility_fields():
    """Test conversion of query with new eligibility-focused fields."""
    query = TrialQuery(
        conditions=["lung cancer"],
        prior_therapies=["osimertinib"],
        progression_on=["erlotinib"],
        required_mutations=["EGFR L858R"],
        excluded_mutations=["T790M"],
        biomarker_expression={"PD-L1": "≥50%"},
        line_of_therapy=LineOfTherapy.SECOND_LINE,
        allow_brain_mets=False,
    )
    params = await convert_query(query)

    # Check that query.term contains all the Essie fragments
    assert "query.term" in params
    term = params["query.term"][0]

    # Prior therapy
    assert (
        'AREA[EligibilityCriteria]("osimertinib" AND (prior OR previous OR received))'
        in term
    )

    # Progression
    assert (
        'AREA[EligibilityCriteria]("erlotinib" AND (progression OR resistant OR refractory))'
        in term
    )

    # Required mutation
    assert 'AREA[EligibilityCriteria]("EGFR L858R")' in term

    # Excluded mutation
    assert 'AREA[EligibilityCriteria](NOT "T790M")' in term

    # Biomarker expression
    assert 'AREA[EligibilityCriteria]("PD-L1" AND "≥50%")' in term

    # Line of therapy
    assert 'AREA[EligibilityCriteria]("second line" OR "second-line"' in term

    # Brain mets exclusion
    assert 'AREA[EligibilityCriteria](NOT "brain metastases")' in term

    # All fragments should be combined with AND
    assert " AND " in term


@pytest.mark.asyncio
async def test_convert_query_with_custom_fields_and_page_size():
    """Test conversion of query with custom return fields and page size."""
    query = TrialQuery(
        conditions=["diabetes"],
        return_fields=["NCTId", "BriefTitle", "OverallStatus"],
        page_size=100,
    )
    params = await convert_query(query)

    assert "fields" in params
    assert params["fields"] == ["NCTId,BriefTitle,OverallStatus"]

    assert "pageSize" in params
    assert params["pageSize"] == ["100"]


@pytest.mark.asyncio
async def test_convert_query_eligibility_with_existing_terms():
    """Test that eligibility Essie fragments are properly combined with existing terms."""
    query = TrialQuery(
        terms=["immunotherapy"],
        prior_therapies=["chemotherapy"],
    )
    params = await convert_query(query)

    assert "query.term" in params
    term = params["query.term"][0]

    # Should contain both the original term and the new Essie fragment
    assert "immunotherapy" in term
    assert (
        'AREA[EligibilityCriteria]("chemotherapy" AND (prior OR previous OR received))'
        in term
    )
    # Should be combined with AND
    assert "immunotherapy AND AREA[EligibilityCriteria]" in term
