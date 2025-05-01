"""Health check command for BioMCP CLI.

This module provides a command to check the health of API endpoints and system resources.
"""

import asyncio
import platform
import socket
from typing import Any, Optional

import httpx
import typer
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from .. import const, http_client

# Try to import psutil, but handle case where it's not installed
try:
    import psutil

    PSUTIL_AVAILABLE = True
except ImportError:
    PSUTIL_AVAILABLE = False

health_app = typer.Typer(help="Health check operations")
console = Console()


async def check_api_endpoint(
    url: str,
    name: str,
    params: Optional[dict[Any, Any]] = None,
    method: str = "GET",
) -> dict:
    """Check if an API endpoint is accessible and responding."""
    try:
        status, content = await http_client.call_http(
            method, url, params or {}
        )
        return {
            "name": name,
            "url": url,
            "status": status,
            "accessible": status == 200,
            "message": "OK" if status == 200 else f"Error: HTTP {status}",
            "content": content[:500]
            if len(content) > 500
            else content,  # Truncate long responses
        }
    except Exception as e:
        return {
            "name": name,
            "url": url,
            "status": 0,
            "accessible": False,
            "message": f"Error: {e!s}",
            "content": str(e),
        }


async def check_all_api_endpoints() -> list[dict]:
    """Check all known API endpoints."""
    endpoints: list[dict[str, Any]] = [
        # PubTator3 API endpoints
        {
            "url": f"{const.PUBTATOR3_BASE}/entity/autocomplete/",
            "name": "PubTator3 Autocomplete",
            "params": {"query": "BRAF", "concept": "gene", "limit": 2},
        },
        {
            "url": f"{const.PUBTATOR3_BASE}/publications/export/biocjson",
            "name": "PubTator3 Publications",
            "params": {"pmids": "29355051", "full": "false"},
        },
        {
            "url": f"{const.PUBTATOR3_BASE}/search/",
            "name": "PubTator3 Search",
            "params": {
                "query": "BRAF",
                "concepts": "gene",
                "page": 1,
                "size": 1,
                "text": "@CHEMICAL_remdesivir",
            },
        },
        # ClinicalTrials.gov API endpoints
        {
            "url": f"{const.CT_GOV_STUDIES}",
            "name": "ClinicalTrials.gov Search API",
            "params": {"query.term": "cancer", "pageSize": "1"},
        },
        {
            "url": f"{const.CT_GOV_STUDIES}/NCT04280705",
            "name": "ClinicalTrials.gov Study API",
            "params": {"fields": "IdentificationModule,StatusModule"},
        },
        # MyVariant.info API endpoints
        {
            "url": f"{const.MYVARIANT_BASE_URL}/query",
            "name": "MyVariant.info Query API",
            "params": {"q": "rs113488022", "size": 1},
        },
        {
            "url": f"{const.MYVARIANT_BASE_URL}/variant/rs113488022",
            "name": "MyVariant.info Variant API",
            "params": {"fields": "all"},
        },
    ]

    tasks = []
    for endpoint in endpoints:
        url = endpoint["url"]
        name = endpoint["name"]
        params = endpoint.get("params")
        tasks.append(check_api_endpoint(url, name, params))

    return await asyncio.gather(*tasks)


def check_network_connectivity() -> dict:
    """Check basic network connectivity."""
    try:
        # Try to connect to Google's DNS to check internet connectivity
        socket.create_connection(("8.8.8.8", 53), timeout=3)
        return {
            "status": "Connected",
            "message": "Internet connection is available",
        }
    except OSError:
        return {
            "status": "Disconnected",
            "message": "No internet connection detected",
        }


def check_system_resources() -> dict:
    """Check system resources like CPU, memory, and disk space."""
    if not PSUTIL_AVAILABLE:
        return {
            "error": "psutil package not installed. Install with: pip install psutil"
        }

    return {
        "cpu_usage": psutil.cpu_percent(interval=1),
        "memory": {
            "total": psutil.virtual_memory().total / (1024**3),  # GB
            "available": psutil.virtual_memory().available / (1024**3),  # GB
            "percent_used": psutil.virtual_memory().percent,
        },
        "disk": {
            "total": psutil.disk_usage("/").total / (1024**3),  # GB
            "free": psutil.disk_usage("/").free / (1024**3),  # GB
            "percent_used": psutil.disk_usage("/").percent,
        },
    }


def check_python_environment() -> dict:
    """Check Python environment and installed packages."""
    env_info = {
        "python_version": platform.python_version(),
        "platform": platform.platform(),
        "system": platform.system(),
        "httpx_version": httpx.__version__,
    }

    if PSUTIL_AVAILABLE:
        env_info["psutil_version"] = psutil.__version__
    else:
        env_info["psutil_version"] = "Not installed"

    return env_info


def display_api_health(results: list[dict], verbose: bool = False) -> None:
    """Display API health check results in a table."""
    table = Table(title="API Endpoints Health")
    table.add_column("Endpoint", style="cyan")
    table.add_column("URL", style="blue")
    table.add_column("Status", style="magenta")
    table.add_column("Message", style="green")

    for result in results:
        "green" if result["accessible"] else "red"
        table.add_row(
            result["name"],
            result["url"],
            f"{result['status']}",
            result["message"],
            style=None if result["accessible"] else "red",
        )

    console.print(table)

    # Display detailed response content if verbose mode is enabled
    if verbose:
        for result in results:
            if not result["accessible"]:
                console.print(
                    f"\n[bold red]Detailed error for {result['name']}:[/bold red]"
                )
                console.print(
                    Panel(
                        result["content"],
                        title=f"{result['name']} Response",
                        border_style="red",
                    )
                )


def display_system_health(
    system_info: dict, network_info: dict, env_info: dict
) -> None:
    """Display system health information in a table."""
    # System resources table
    resource_table = Table(title="System Resources")
    resource_table.add_column("Resource", style="cyan")
    resource_table.add_column("Value", style="green")

    if "error" in system_info:
        resource_table.add_row("Error", system_info["error"], style="red")
    else:
        resource_table.add_row("CPU Usage", f"{system_info['cpu_usage']}%")
        resource_table.add_row(
            "Memory Total", f"{system_info['memory']['total']:.2f} GB"
        )
        resource_table.add_row(
            "Memory Available", f"{system_info['memory']['available']:.2f} GB"
        )
        resource_table.add_row(
            "Memory Usage",
            f"{system_info['memory']['percent_used']}%",
            style="green"
            if system_info["memory"]["percent_used"] < 90
            else "red",
        )
        resource_table.add_row(
            "Disk Total", f"{system_info['disk']['total']:.2f} GB"
        )
        resource_table.add_row(
            "Disk Free", f"{system_info['disk']['free']:.2f} GB"
        )
        resource_table.add_row(
            "Disk Usage",
            f"{system_info['disk']['percent_used']}%",
            style="green"
            if system_info["disk"]["percent_used"] < 90
            else "red",
        )

    console.print(resource_table)

    # Network and environment table
    env_table = Table(title="Network & Environment")
    env_table.add_column("Component", style="cyan")
    env_table.add_column("Status/Version", style="green")

    env_table.add_row(
        "Network",
        network_info["status"],
        style=None if network_info["status"] == "Connected" else "red",
    )
    env_table.add_row("Python Version", env_info["python_version"])
    env_table.add_row("Platform", env_info["platform"])
    env_table.add_row("System", env_info["system"])
    env_table.add_row("HTTPX Version", env_info["httpx_version"])
    env_table.add_row(
        "Psutil Version",
        env_info["psutil_version"],
        style="red" if env_info["psutil_version"] == "Not installed" else None,
    )

    console.print(env_table)


@health_app.callback(invoke_without_command=True)
def health_callback(ctx: typer.Context):
    """Health check callback."""
    if ctx.invoked_subcommand is None:
        # If no subcommand is provided, run the default health check
        check()


@health_app.command()
def check(
    api_only: bool = typer.Option(
        False, "--api-only", help="Check only API endpoints"
    ),
    system_only: bool = typer.Option(
        False, "--system-only", help="Check only system health"
    ),
    verbose: bool = typer.Option(
        False,
        "--verbose",
        "-v",
        help="Show detailed error information and API responses",
    ),
):
    """
    Run a comprehensive health check on API endpoints and system resources.

    This command checks:
    - API endpoints connectivity and response
    - Network connectivity
    - System resources (CPU, memory, disk)
    - Python environment

    Note: For full system resource checks, the 'psutil' package is required.
    Install with: pip install psutil
    """
    with console.status("[bold green]Running health checks...") as status:
        # Check API endpoints
        if not system_only:
            status.update("[bold green]Checking API endpoints...")
            api_results = asyncio.run(check_all_api_endpoints())
            display_api_health(api_results, verbose)

        # Check system health
        if not api_only:
            status.update("[bold green]Checking system resources...")
            system_info = check_system_resources()
            network_info = check_network_connectivity()
            env_info = check_python_environment()
            display_system_health(system_info, network_info, env_info)

    # Overall status
    if not api_only and not system_only:
        api_health = all(result["accessible"] for result in api_results)

        if "error" in system_info:
            system_health = False
        else:
            system_health = (
                network_info["status"] == "Connected"
                and system_info["memory"]["percent_used"] < 90
                and system_info["disk"]["percent_used"] < 90
            )

        if api_health and system_health:
            console.print(
                "\n[bold green]✓ All systems operational![/bold green]"
            )
        else:
            console.print(
                "\n[bold red]⚠ Some health checks failed. See details above.[/bold red]"
            )
            if verbose:
                console.print(
                    "[yellow]Run with --verbose flag to see detailed error information[/yellow]"
                )
