"""BioMCP Command Line Interface for clinical trials."""

import asyncio
from typing import Annotated

import typer

from ..trials.getter import Module, get_trial
from ..trials.search import (
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
    search_trials,
)

trial_app = typer.Typer(help="Clinical trial operations")


@trial_app.command("get")
def get_trial_cli(
    nct_id: str,
    module: Annotated[
        Module | None,
        typer.Argument(
            help="Module to retrieve: Protocol, Locations, References, or Outcomes",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = Module.PROTOCOL,
    output_json: Annotated[
        bool,
        typer.Option(
            "--json",
            "-j",
            help="Render in JSON format",
            case_sensitive=False,
        ),
    ] = False,
):
    """Get trial information by NCT ID and optional module."""
    result = asyncio.run(
        get_trial(nct_id, module or Module.PROTOCOL, output_json)
    )
    typer.echo(result)


@trial_app.command("search")
def search_trials_cli(
    condition: Annotated[
        list[str] | None,
        typer.Option(
            "--condition",
            "-c",
            help="Medical condition to search for (can specify multiple)",
        ),
    ] = None,
    intervention: Annotated[
        list[str] | None,
        typer.Option(
            "--intervention",
            "-i",
            help="Treatment or intervention to search for (can specify multiple)",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    term: Annotated[
        list[str] | None,
        typer.Option(
            "--term",
            "-t",
            help="General search terms (can specify multiple)",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    nct_id: Annotated[
        list[str] | None,
        typer.Option(
            "--nct-id",
            "-n",
            help="Clinical trial NCT ID (can specify multiple)",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    recruiting_status: Annotated[
        RecruitingStatus | None,
        typer.Option(
            "--status",
            "-s",
            help="Recruiting status.",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    study_type: Annotated[
        StudyType | None,
        typer.Option(
            "--type",
            help="Study type",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    phase: Annotated[
        TrialPhase | None,
        typer.Option(
            "--phase",
            "-p",
            help="Trial phase",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    sort_order: Annotated[
        SortOrder | None,
        typer.Option(
            "--sort",
            help="Sort order",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    age_group: Annotated[
        AgeGroup | None,
        typer.Option(
            "--age-group",
            "-a",
            help="Age group filter",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    primary_purpose: Annotated[
        PrimaryPurpose | None,
        typer.Option(
            "--purpose",
            help="Primary purpose filter",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    min_date: Annotated[
        str | None,
        typer.Option(
            "--min-date",
            help="Minimum date for filtering (YYYY-MM-DD format)",
        ),
    ] = None,
    max_date: Annotated[
        str | None,
        typer.Option(
            "--max-date",
            help="Maximum date for filtering (YYYY-MM-DD format)",
        ),
    ] = None,
    date_field: Annotated[
        DateField | None,
        typer.Option(
            "--date-field",
            help="Date field to filter",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = DateField.STUDY_START,
    intervention_type: Annotated[
        InterventionType | None,
        typer.Option(
            "--intervention-type",
            help="Intervention type filter",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    sponsor_type: Annotated[
        SponsorType | None,
        typer.Option(
            "--sponsor-type",
            help="Sponsor type filter",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    study_design: Annotated[
        StudyDesign | None,
        typer.Option(
            "--study-design",
            help="Study design filter",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    next_page_hash: Annotated[
        str | None,
        typer.Option(
            "--next-page",
            help="Next page hash for pagination",
        ),
    ] = None,
    latitude: Annotated[
        float | None,
        typer.Option(
            "--lat",
            help="Latitude for location-based search. For city names, geocode first (e.g., Cleveland: 41.4993)",
        ),
    ] = None,
    longitude: Annotated[
        float | None,
        typer.Option(
            "--lon",
            help="Longitude for location-based search. For city names, geocode first (e.g., Cleveland: -81.6944)",
        ),
    ] = None,
    distance: Annotated[
        int | None,
        typer.Option(
            "--distance",
            "-d",
            help="Distance in miles for location-based search (default: 50 miles if lat/lon provided)",
        ),
    ] = None,
    output_json: Annotated[
        bool,
        typer.Option(
            "--json",
            "-j",
            help="Render in JSON format",
            case_sensitive=False,
        ),
    ] = False,
    prior_therapy: Annotated[
        list[str] | None,
        typer.Option(
            "--prior-therapy",
            help="Prior therapies to search for in eligibility criteria (can specify multiple)",
        ),
    ] = None,
    progression_on: Annotated[
        list[str] | None,
        typer.Option(
            "--progression-on",
            help="Therapies the patient has progressed on (can specify multiple)",
        ),
    ] = None,
    required_mutation: Annotated[
        list[str] | None,
        typer.Option(
            "--required-mutation",
            help="Required mutations in eligibility criteria (can specify multiple)",
        ),
    ] = None,
    excluded_mutation: Annotated[
        list[str] | None,
        typer.Option(
            "--excluded-mutation",
            help="Excluded mutations in eligibility criteria (can specify multiple)",
        ),
    ] = None,
    biomarker: Annotated[
        list[str] | None,
        typer.Option(
            "--biomarker",
            help="Biomarker expression requirements in format 'MARKER:EXPRESSION' (e.g., 'PD-L1:≥50%')",
        ),
    ] = None,
    line_of_therapy: Annotated[
        LineOfTherapy | None,
        typer.Option(
            "--line-of-therapy",
            help="Line of therapy filter",
            show_choices=True,
            show_default=True,
            case_sensitive=False,
        ),
    ] = None,
    allow_brain_mets: Annotated[
        bool | None,
        typer.Option(
            "--allow-brain-mets/--no-brain-mets",
            help="Whether to allow trials that accept brain metastases",
        ),
    ] = None,
    return_field: Annotated[
        list[str] | None,
        typer.Option(
            "--return-field",
            help="Specific fields to return in the response (can specify multiple)",
        ),
    ] = None,
    page_size: Annotated[
        int | None,
        typer.Option(
            "--page-size",
            help="Number of results per page (1-1000)",
            min=1,
            max=1000,
        ),
    ] = None,
):
    """Search for clinical trials."""
    # Parse biomarker expression from CLI format
    biomarker_expression = None
    if biomarker:
        biomarker_expression = {}
        for item in biomarker:
            if ":" in item:
                marker, expr = item.split(":", 1)
                biomarker_expression[marker] = expr

    query = TrialQuery(
        conditions=condition,
        interventions=intervention,
        terms=term,
        nct_ids=nct_id,
        recruiting_status=recruiting_status,
        study_type=study_type,
        phase=phase,
        sort=sort_order,
        age_group=age_group,
        primary_purpose=primary_purpose,
        min_date=min_date,
        max_date=max_date,
        date_field=date_field,
        intervention_type=intervention_type,
        sponsor_type=sponsor_type,
        study_design=study_design,
        next_page_hash=next_page_hash,
        lat=latitude,
        long=longitude,
        distance=distance,
        prior_therapies=prior_therapy,
        progression_on=progression_on,
        required_mutations=required_mutation,
        excluded_mutations=excluded_mutation,
        biomarker_expression=biomarker_expression,
        line_of_therapy=line_of_therapy,
        allow_brain_mets=allow_brain_mets,
        return_fields=return_field,
        page_size=page_size,
    )

    result = asyncio.run(search_trials(query, output_json))
    typer.echo(result)
