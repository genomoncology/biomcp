"""
OpenFDA CLI commands for BioMCP.
"""

import asyncio
from typing import Annotated

import typer
from rich.console import Console

from ..openfda import (
    get_adverse_event,
    get_device_event,
    get_drug_approval,
    get_drug_label,
    get_drug_recall,
    get_drug_shortage,
    search_adverse_events,
    search_device_events,
    search_drug_approvals,
    search_drug_labels,
    search_drug_recalls,
    search_drug_shortages,
)

console = Console()

# Create separate Typer apps for each subdomain
adverse_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve FDA drug adverse event reports (FAERS)",
)

label_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve FDA drug product labels (SPL)",
)

device_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve FDA device adverse event reports (MAUDE)",
)

approval_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve FDA drug approval records (Drugs@FDA)",
)

recall_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve FDA drug recall records (Enforcement)",
)

shortage_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve FDA drug shortage information",
)


# Adverse Events Commands
@adverse_app.command("search")
def search_adverse_events_cli(
    drug: Annotated[
        str | None,
        typer.Option("--drug", "-d", help="Drug name to search for"),
    ] = None,
    reaction: Annotated[
        str | None,
        typer.Option(
            "--reaction", "-r", help="Adverse reaction to search for"
        ),
    ] = None,
    serious: Annotated[
        bool | None,
        typer.Option("--serious/--all", help="Filter for serious events only"),
    ] = None,
    limit: Annotated[
        int, typer.Option("--limit", "-l", help="Maximum number of results")
    ] = 25,
    page: Annotated[
        int, typer.Option("--page", "-p", help="Page number (1-based)")
    ] = 1,
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Search FDA adverse event reports for drugs."""
    skip = (page - 1) * limit

    try:
        results = asyncio.run(
            search_adverse_events(
                drug=drug,
                reaction=reaction,
                serious=serious,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        )
        console.print(results)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


@adverse_app.command("get")
def get_adverse_event_cli(
    report_id: Annotated[str, typer.Argument(help="Safety report ID")],
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Get detailed information for a specific adverse event report."""
    try:
        result = asyncio.run(get_adverse_event(report_id, api_key=api_key))
        console.print(result)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


# Drug Label Commands
@label_app.command("search")
def search_drug_labels_cli(
    name: Annotated[
        str | None,
        typer.Option("--name", "-n", help="Drug name to search for"),
    ] = None,
    indication: Annotated[
        str | None,
        typer.Option(
            "--indication",
            "-i",
            help="Search for drugs indicated for this condition",
        ),
    ] = None,
    boxed_warning: Annotated[
        bool,
        typer.Option(
            "--boxed-warning", help="Filter for drugs with boxed warnings"
        ),
    ] = False,
    section: Annotated[
        str | None,
        typer.Option(
            "--section", "-s", help="Specific label section to search"
        ),
    ] = None,
    limit: Annotated[
        int, typer.Option("--limit", "-l", help="Maximum number of results")
    ] = 25,
    page: Annotated[
        int, typer.Option("--page", "-p", help="Page number (1-based)")
    ] = 1,
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Search FDA drug product labels."""
    skip = (page - 1) * limit

    try:
        results = asyncio.run(
            search_drug_labels(
                name=name,
                indication=indication,
                boxed_warning=boxed_warning,
                section=section,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        )
        console.print(results)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


@label_app.command("get")
def get_drug_label_cli(
    set_id: Annotated[str, typer.Argument(help="Label set ID")],
    sections: Annotated[
        str | None,
        typer.Option(
            "--sections", help="Comma-separated list of sections to retrieve"
        ),
    ] = None,
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Get detailed drug label information."""
    section_list = None
    if sections:
        section_list = [s.strip() for s in sections.split(",")]

    try:
        result = asyncio.run(
            get_drug_label(set_id, section_list, api_key=api_key)
        )
        console.print(result)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


# Device Event Commands
@device_app.command("search")
def search_device_events_cli(
    device: Annotated[
        str | None,
        typer.Option("--device", "-d", help="Device name to search for"),
    ] = None,
    manufacturer: Annotated[
        str | None,
        typer.Option("--manufacturer", "-m", help="Manufacturer name"),
    ] = None,
    problem: Annotated[
        str | None,
        typer.Option("--problem", "-p", help="Device problem description"),
    ] = None,
    product_code: Annotated[
        str | None, typer.Option("--product-code", help="FDA product code")
    ] = None,
    genomics_only: Annotated[
        bool,
        typer.Option(
            "--genomics-only/--all-devices",
            help="Filter to genomic/diagnostic devices",
        ),
    ] = True,
    limit: Annotated[
        int, typer.Option("--limit", "-l", help="Maximum number of results")
    ] = 25,
    page: Annotated[
        int, typer.Option("--page", help="Page number (1-based)")
    ] = 1,
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Search FDA device adverse event reports."""
    skip = (page - 1) * limit

    try:
        results = asyncio.run(
            search_device_events(
                device=device,
                manufacturer=manufacturer,
                problem=problem,
                product_code=product_code,
                genomics_only=genomics_only,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        )
        console.print(results)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


@device_app.command("get")
def get_device_event_cli(
    mdr_report_key: Annotated[str, typer.Argument(help="MDR report key")],
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Get detailed information for a specific device event report."""
    try:
        result = asyncio.run(get_device_event(mdr_report_key, api_key=api_key))
        console.print(result)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


# Drug Approval Commands
@approval_app.command("search")
def search_drug_approvals_cli(
    drug: Annotated[
        str | None,
        typer.Option("--drug", "-d", help="Drug name to search for"),
    ] = None,
    application: Annotated[
        str | None,
        typer.Option(
            "--application", "-a", help="NDA or BLA application number"
        ),
    ] = None,
    year: Annotated[
        str | None,
        typer.Option("--year", "-y", help="Approval year (YYYY format)"),
    ] = None,
    limit: Annotated[
        int, typer.Option("--limit", "-l", help="Maximum number of results")
    ] = 25,
    page: Annotated[
        int, typer.Option("--page", "-p", help="Page number (1-based)")
    ] = 1,
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Search FDA drug approval records."""
    skip = (page - 1) * limit

    try:
        results = asyncio.run(
            search_drug_approvals(
                drug=drug,
                application_number=application,
                approval_year=year,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        )
        console.print(results)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


@approval_app.command("get")
def get_drug_approval_cli(
    application: Annotated[
        str, typer.Argument(help="NDA or BLA application number")
    ],
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Get detailed drug approval information."""
    try:
        result = asyncio.run(get_drug_approval(application, api_key=api_key))
        console.print(result)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


# Drug Recall Commands
@recall_app.command("search")
def search_drug_recalls_cli(
    drug: Annotated[
        str | None,
        typer.Option("--drug", "-d", help="Drug name to search for"),
    ] = None,
    recall_class: Annotated[
        str | None,
        typer.Option(
            "--class", "-c", help="Recall classification (1, 2, or 3)"
        ),
    ] = None,
    status: Annotated[
        str | None,
        typer.Option(
            "--status", "-s", help="Recall status (ongoing, completed)"
        ),
    ] = None,
    reason: Annotated[
        str | None,
        typer.Option("--reason", "-r", help="Search in recall reason"),
    ] = None,
    since: Annotated[
        str | None,
        typer.Option("--since", help="Show recalls after date (YYYYMMDD)"),
    ] = None,
    limit: Annotated[
        int, typer.Option("--limit", "-l", help="Maximum number of results")
    ] = 25,
    page: Annotated[
        int, typer.Option("--page", "-p", help="Page number (1-based)")
    ] = 1,
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Search FDA drug recall records."""
    skip = (page - 1) * limit

    try:
        results = asyncio.run(
            search_drug_recalls(
                drug=drug,
                recall_class=recall_class,
                status=status,
                reason=reason,
                since_date=since,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        )
        console.print(results)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


@recall_app.command("get")
def get_drug_recall_cli(
    recall_number: Annotated[str, typer.Argument(help="FDA recall number")],
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Get detailed drug recall information."""
    try:
        result = asyncio.run(get_drug_recall(recall_number, api_key=api_key))
        console.print(result)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


# Drug Shortage Commands
@shortage_app.command("search")
def search_drug_shortages_cli(
    drug: Annotated[
        str | None,
        typer.Option("--drug", "-d", help="Drug name to search for"),
    ] = None,
    status: Annotated[
        str | None,
        typer.Option(
            "--status", "-s", help="Shortage status (current, resolved)"
        ),
    ] = None,
    category: Annotated[
        str | None,
        typer.Option("--category", "-c", help="Therapeutic category"),
    ] = None,
    limit: Annotated[
        int, typer.Option("--limit", "-l", help="Maximum number of results")
    ] = 25,
    page: Annotated[
        int, typer.Option("--page", "-p", help="Page number (1-based)")
    ] = 1,
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Search FDA drug shortage records."""
    skip = (page - 1) * limit

    try:
        results = asyncio.run(
            search_drug_shortages(
                drug=drug,
                status=status,
                therapeutic_category=category,
                limit=limit,
                skip=skip,
                api_key=api_key,
            )
        )
        console.print(results)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


@shortage_app.command("get")
def get_drug_shortage_cli(
    drug: Annotated[str, typer.Argument(help="Drug name")],
    api_key: Annotated[
        str | None,
        typer.Option(
            "--api-key",
            help="OpenFDA API key (overrides OPENFDA_API_KEY env var)",
        ),
    ] = None,
):
    """Get detailed drug shortage information."""
    try:
        result = asyncio.run(get_drug_shortage(drug, api_key=api_key))
        console.print(result)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Exit(1) from e


# Main OpenFDA app that combines all subcommands
openfda_app = typer.Typer(
    no_args_is_help=True,
    help="Search and retrieve data from FDA's openFDA API",
)

# Add subcommands
openfda_app.add_typer(
    adverse_app, name="adverse", help="Drug adverse events (FAERS)"
)
openfda_app.add_typer(
    label_app, name="label", help="Drug product labels (SPL)"
)
openfda_app.add_typer(
    device_app, name="device", help="Device adverse events (MAUDE)"
)
openfda_app.add_typer(
    approval_app, name="approval", help="Drug approvals (Drugs@FDA)"
)
openfda_app.add_typer(
    recall_app, name="recall", help="Drug recalls (Enforcement)"
)
openfda_app.add_typer(shortage_app, name="shortage", help="Drug shortages")
