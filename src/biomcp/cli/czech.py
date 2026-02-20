"""CLI commands for Czech healthcare modules."""

import asyncio
import json
from enum import Enum
from typing import Annotated

import typer

czech_app = typer.Typer(
    help="Czech healthcare data tools (SUKL, MKN-10, NRPZS, SZV, VZP).",
)


class OutputFormat(str, Enum):
    json = "json"
    human = "human"


# Sub-apps for each module
sukl_app = typer.Typer(help="SUKL drug registry commands.")
mkn_app = typer.Typer(help="MKN-10 diagnosis code commands.")
nrpzs_app = typer.Typer(help="NRPZS provider registry commands.")
szv_app = typer.Typer(help="SZV health procedure commands.")
vzp_app = typer.Typer(help="VZP insurance codebook commands.")

czech_app.add_typer(sukl_app, name="sukl", no_args_is_help=True)
czech_app.add_typer(mkn_app, name="mkn", no_args_is_help=True)
czech_app.add_typer(nrpzs_app, name="nrpzs", no_args_is_help=True)
czech_app.add_typer(szv_app, name="szv", no_args_is_help=True)
czech_app.add_typer(vzp_app, name="vzp", no_args_is_help=True)


def _output(data: dict | list | str, fmt: OutputFormat) -> None:
    """Output data in the requested format."""
    if fmt == OutputFormat.json:
        if isinstance(data, str):
            typer.echo(data)
        else:
            typer.echo(json.dumps(data, indent=2, ensure_ascii=False))
    else:
        if isinstance(data, str):
            typer.echo(data)
        elif isinstance(data, dict):
            for k, v in data.items():
                typer.echo(f"{k}: {v}")
        elif isinstance(data, list):
            for item in data:
                if isinstance(item, dict):
                    for k, v in item.items():
                        typer.echo(f"  {k}: {v}")
                    typer.echo("---")
                else:
                    typer.echo(f"  {item}")


# ---------------------------------------------------------------------------
# SUKL CLI commands
# ---------------------------------------------------------------------------

FMT_OPT = Annotated[
    OutputFormat,
    typer.Option(
        "--format",
        "-f",
        help="Output format",
    ),
]


@sukl_app.command("search")
def sukl_search(
    query: Annotated[
        str, typer.Option("--query", "-q", help="Drug name, substance, or ATC code")
    ],
    page: Annotated[int, typer.Option(help="Page number")] = 1,
    page_size: Annotated[int, typer.Option(help="Results per page")] = 10,
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Search Czech drug registry."""
    from biomcp.czech.sukl.search import _sukl_drug_search

    result = asyncio.run(_sukl_drug_search(query, page, page_size))
    data = json.loads(result)
    _output(data, fmt)


@sukl_app.command("get")
def sukl_get(
    code: Annotated[str, typer.Argument(help="SUKL drug code")],
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Get full drug details by SUKL code."""
    from biomcp.czech.sukl.getter import _sukl_drug_details

    result = asyncio.run(_sukl_drug_details(code))
    data = json.loads(result)
    _output(data, fmt)


@sukl_app.command("spc")
def sukl_spc(
    code: Annotated[str, typer.Argument(help="SUKL drug code")],
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Get SmPC document for a drug."""
    from biomcp.czech.sukl.getter import _sukl_spc_getter

    result = asyncio.run(_sukl_spc_getter(code))
    data = json.loads(result)
    _output(data, fmt)


@sukl_app.command("pil")
def sukl_pil(
    code: Annotated[str, typer.Argument(help="SUKL drug code")],
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Get PIL document for a drug."""
    from biomcp.czech.sukl.getter import _sukl_pil_getter

    result = asyncio.run(_sukl_pil_getter(code))
    data = json.loads(result)
    _output(data, fmt)


@sukl_app.command("availability")
def sukl_availability(
    code: Annotated[str, typer.Argument(help="SUKL drug code")],
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Check drug availability status."""
    from biomcp.czech.sukl.availability import _sukl_availability_check

    result = asyncio.run(_sukl_availability_check(code))
    data = json.loads(result)
    _output(data, fmt)


# -----------------------------------------------------------
# MKN-10 CLI commands
# -----------------------------------------------------------


@mkn_app.command("search")
def mkn_search(
    query: Annotated[
        str,
        typer.Option(
            "--query", "-q", help="MKN-10 code or text"
        ),
    ],
    max_results: Annotated[
        int, typer.Option(help="Max results")
    ] = 10,
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Search MKN-10 diagnoses."""
    from biomcp.czech.mkn.search import _mkn_search

    result = asyncio.run(_mkn_search(query, max_results))
    data = json.loads(result)
    _output(data, fmt)


@mkn_app.command("get")
def mkn_get(
    code: Annotated[
        str, typer.Argument(help="MKN-10 code")
    ],
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Get diagnosis details by MKN-10 code."""
    from biomcp.czech.mkn.search import _mkn_get

    result = asyncio.run(_mkn_get(code))
    data = json.loads(result)
    _output(data, fmt)


@mkn_app.command("browse")
def mkn_browse(
    code: Annotated[
        str | None, typer.Argument(help="Category code")
    ] = None,
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Browse MKN-10 category hierarchy."""
    from biomcp.czech.mkn.search import _mkn_browse

    result = asyncio.run(_mkn_browse(code))
    data = json.loads(result)
    _output(data, fmt)


# -----------------------------------------------------------
# NRPZS CLI commands
# -----------------------------------------------------------


@nrpzs_app.command("search")
def nrpzs_search(
    query: Annotated[
        str | None,
        typer.Option("--query", "-q", help="Provider name"),
    ] = None,
    city: Annotated[
        str | None,
        typer.Option("--city", help="City"),
    ] = None,
    specialty: Annotated[
        str | None,
        typer.Option("--specialty", help="Specialty"),
    ] = None,
    page: Annotated[
        int, typer.Option(help="Page number")
    ] = 1,
    page_size: Annotated[
        int, typer.Option(help="Results per page")
    ] = 10,
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Search healthcare providers."""
    from biomcp.czech.nrpzs.search import _nrpzs_search

    result = asyncio.run(
        _nrpzs_search(query, city, specialty, page, page_size)
    )
    data = json.loads(result)
    _output(data, fmt)


@nrpzs_app.command("get")
def nrpzs_get(
    provider_id: Annotated[
        str, typer.Argument(help="Provider ID")
    ],
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Get provider details by ID."""
    from biomcp.czech.nrpzs.search import _nrpzs_get

    result = asyncio.run(_nrpzs_get(provider_id))
    data = json.loads(result)
    _output(data, fmt)


# -----------------------------------------------------------
# SZV CLI commands
# -----------------------------------------------------------


@szv_app.command("search")
def szv_search(
    query: Annotated[
        str,
        typer.Option(
            "--query", "-q", help="Procedure code or name"
        ),
    ],
    max_results: Annotated[
        int, typer.Option(help="Max results")
    ] = 10,
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Search health procedures."""
    from biomcp.czech.szv.search import _szv_search

    result = asyncio.run(_szv_search(query, max_results))
    data = json.loads(result)
    _output(data, fmt)


@szv_app.command("get")
def szv_get(
    code: Annotated[
        str, typer.Argument(help="Procedure code")
    ],
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Get procedure details."""
    from biomcp.czech.szv.search import _szv_get

    result = asyncio.run(_szv_get(code))
    data = json.loads(result)
    _output(data, fmt)


# -----------------------------------------------------------
# VZP CLI commands
# -----------------------------------------------------------


@vzp_app.command("search")
def vzp_search(
    query: Annotated[
        str,
        typer.Option("--query", "-q", help="Search term"),
    ],
    codebook_type: Annotated[
        str | None,
        typer.Option("--type", help="Codebook type"),
    ] = None,
    max_results: Annotated[
        int, typer.Option(help="Max results")
    ] = 10,
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Search VZP codebooks."""
    from biomcp.czech.vzp.search import _vzp_search

    result = asyncio.run(
        _vzp_search(query, codebook_type, max_results)
    )
    data = json.loads(result)
    _output(data, fmt)


@vzp_app.command("get")
def vzp_get(
    codebook_type: Annotated[
        str, typer.Argument(help="Codebook type")
    ],
    code: Annotated[
        str, typer.Argument(help="Entry code")
    ],
    fmt: FMT_OPT = OutputFormat.json,
) -> None:
    """Get codebook entry details."""
    from biomcp.czech.vzp.search import _vzp_get

    result = asyncio.run(_vzp_get(codebook_type, code))
    data = json.loads(result)
    _output(data, fmt)
