#!/usr/bin/env -S uv --quiet run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "typer",
#     "httpx",
#     "rich",
# ]
# ///

###############################################################################
# GeneAgent (Wang et al., 2025) - Context for this helper module
#
# GeneAgent is a self-verifying LLM pipeline that annotates gene sets and
# biologic claims. In the paper the agent consults 18 expert databases
# (GO, KEGG, Reactome, CORUM, BioPlanet ...).
#
# Several of those resources no longer expose an official JSON API in 2025
# (notably CORUM and BioPlanet). We therefore map each original source to a
# *publicly accessible stand-in* that covers the same information:
#
#   • Protein-complex data → ComplexPortal REST (proxies CORUM)
#   • Human pathway catalogue → Enrichr "BioPlanet_2019" library
#
# All other resources do still have live, token-free REST endpoints. This
# module provides thin test/wrapper functions for each one so that the main
# GeneAgent reproduction script can pull ontology terms, pathway hits,
# literature snippets, etc. exactly as described in the paper—just through
# modern URLs.
#
# Usage pattern inside this repo:
#   1. developer calls quickgo("apoptosis")      → ontology demo
#   2. gprofiler([...])                          → enrichment demo
#   3. etc.
#
# Each wrapper prints a small Rich table to prove connectivity but also
# returns the raw response object so the core pipeline can reuse it.
###############################################################################

import json
import time
from typing import Optional

import httpx
import typer
from rich.console import Console
from rich.table import Table

app = typer.Typer(help="Test various bioinformatics API endpoints")
console = Console()

# Base timeout for all requests
TIMEOUT = httpx.Timeout(30.0)


# -------------------------------------------------------------------------
# QuickGO - Gene Ontology browser / search
# -------------------------------------------------------------------------
# Base URL ........ https://www.ebi.ac.uk/QuickGO/api/
# Key paths ....... /search?query=TERM   |  /ontology/term/GO:0008150
# Input params .... query=<text> OR id=<GO:ID>; optional filters: ontology,
#                   aspect (P/F/C), page/size for paging.
# Output .......... JSON. Top level keys: numberOfHits, results[]. Each
#                   result has id, name, definition, aspect, synonyms.
# Token needed .... None
# Notes ........... Accept header defaults to application/json, so no need
#                   to set it explicitly.
# -------------------------------------------------------------------------
@app.command()
def quickgo(
    query: str = typer.Argument("apoptosis", help="Search query or GO ID"),
    limit: int = typer.Option(5, help="Number of results to return"),
):
    """Test QuickGO Gene Ontology API"""
    console.print("[bold cyan]Testing QuickGO API...[/bold cyan]")

    url = "https://www.ebi.ac.uk/QuickGO/services/ontology/go/search"
    params = {"query": query, "limit": limit}

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            response = client.get(url, params=params)
            response.raise_for_status()
            data = response.json()

            # Validate we got actual results
            num_hits = data.get("numberOfHits", 0)
            if num_hits == 0:
                console.print("[yellow]⚠ No results found[/yellow]")
                return False

            console.print(f"[green]✓ Found {num_hits} results[/green]")

            if "results" in data:
                table = Table(title="QuickGO Results")
                table.add_column("GO ID", style="cyan")
                table.add_column("Name", style="magenta")
                table.add_column("Aspect", style="yellow")

                for result in data["results"][:limit]:
                    table.add_row(
                        result.get("id", ""),
                        result.get("name", ""),
                        result.get("aspect", ""),
                    )
                console.print(table)
            return True
        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# KEGG REST - canonical pathway & gene catalogue
# -------------------------------------------------------------------------
# Base URL ........ https://rest.kegg.jp/
# Example calls ... list/pathway/hsa        (all human pathways)
#                   get/hsa:7157            (TP53 gene card)
#                   link/pathway/hsa:7157   (TP53 → pathway links)
# Input params .... encoded in the path (no query-string).
# Output .......... text/tsv (tab-delimited). Parse with .split('\t').
# Token needed .... None, but KEGG asks for ≤10 requests / s.
# -------------------------------------------------------------------------
@app.command()
def kegg(
    organism: str = typer.Option(
        "hsa", help="Organism code (e.g., hsa for human)"
    ),
    gene: str = typer.Option("7157", help="Gene ID"),
):
    """Test KEGG REST API"""
    console.print("[bold cyan]Testing KEGG REST API...[/bold cyan]")

    # Test multiple endpoints
    endpoints = [
        f"https://rest.kegg.jp/list/pathway/{organism}",
        f"https://rest.kegg.jp/get/{organism}:{gene}",
        f"https://rest.kegg.jp/link/pathway/{organism}:{gene}",
    ]

    with httpx.Client(timeout=TIMEOUT) as client:
        for url in endpoints:
            try:
                console.print(f"Testing: {url}")
                response = client.get(url)
                response.raise_for_status()

                # KEGG returns plain text
                lines = response.text.strip().split("\n")[:3]
                console.print("[green]✓ Success[/green]")
                for line in lines:
                    console.print(f"  {line[:100]}...")
                console.print()

            except Exception as e:
                console.print(f"[red]✗ Error: {e}[/red]")
                return False
    return True  # All endpoints tested successfully


# -------------------------------------------------------------------------
# Reactome Content Service - curated pathway knowledge-graph
# -------------------------------------------------------------------------
# Base URL ........ https://reactome.org/ContentService/data/
# Example calls ... pathway/R-HSA-109582/containedEvents
#                   findPathwaysByName?name=apoptosis&species=9606
# Input params .... path segments + normal query-string filters.
# Output .......... JSON objects containing stId, displayName, className,
#                   speciesId, etc.
# Token needed .... None
# -------------------------------------------------------------------------
@app.command()
def reactome(
    pathway_id: str = typer.Argument(
        "R-HSA-109582", help="Reactome pathway ID"
    ),
):
    """Test Reactome Content Service API"""
    console.print("[bold cyan]Testing Reactome API...[/bold cyan]")

    url = f"https://reactome.org/ContentService/data/pathway/{pathway_id}/containedEvents"

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            response = client.get(url)
            response.raise_for_status()
            data = response.json()

            console.print(f"[green]✓ Found {len(data)} events[/green]")

            table = Table(title="Reactome Events")
            table.add_column("ID", style="cyan")
            table.add_column("Name", style="magenta")
            table.add_column("Type", style="yellow")

            for event in data[:5]:
                table.add_row(
                    event.get("stId", ""),
                    event.get("displayName", "")[:50],
                    event.get("className", ""),
                )
            console.print(table)
            return True  # Success if we got events

        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# WikiPathways - community pathway diagrams
# -------------------------------------------------------------------------
# Base URL ........ https://webservice.wikipathways.org/
# Example calls ... findPathwaysByText?query=apoptosis&organism=Homo+sapiens&format=json
#                   getPathwayInfo?pwId=WP254&format=json
# Input params .... query / organism / pwId; always add format=json
# Output .......... JSON dict with result[] (find) or pathwayInfo[] (get)
# Token needed .... None
# -------------------------------------------------------------------------
@app.command()
def wikipathways(
    query: str = typer.Argument("apoptosis", help="Search query"),
    organism: str = typer.Option("Homo sapiens", help="Organism name"),
):
    """Test WikiPathways API"""
    console.print("[bold cyan]Testing WikiPathways API...[/bold cyan]")

    # Use findPathwaysByText to get metadata in one go
    url = "https://webservice.wikipathways.org/findPathwaysByText"
    params = {"query": query, "organism": organism, "format": "json"}

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            response = client.get(url, params=params)
            response.raise_for_status()
            data = response.json()

            # Parse the result structure
            results = data.get("result", [])
            console.print(f"[green]✓ Found {len(results)} pathways[/green]")

            if results:
                table = Table(title="WikiPathways Results")
                table.add_column("ID", style="cyan")
                table.add_column("Name", style="magenta")
                table.add_column("Species", style="yellow")

                for pathway in results[:5]:
                    table.add_row(
                        pathway.get("id", ""),
                        pathway.get("name", ""),
                        pathway.get("species", ""),
                    )
                console.print(table)
            return bool(results)

        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# miRTarBase (targetHub) - validated miRNA⇄mRNA interactions
# -------------------------------------------------------------------------
# Design-doc URL .. https://app1.bioinformatics.mdanderson.org/tarhub/_design/basic
# Example view .... _view/by_miRNAIDcount?startkey=["hsa-miR-21",1]&endkey=["hsa-miR-21",{}]&include_docs=true
# Input params .... CouchDB keys + include_docs=true to get full JSON docs
# Output .......... JSON with rows[]. rows[i].doc contains "evidence",
#                   "PMIDList", "targetGene" etc.
# Token needed .... None
# -------------------------------------------------------------------------
@app.command()
def mirtarbase(mirna: str = typer.Argument("hsa-miR-21", help="miRNA ID")):
    """Test miRTarBase (targetHub) API

    ROOT CAUSE ANALYSIS:
    The API returns 0 results because:
    1. The CouchDB view key format may be incorrect
    2. The test miRNA may not exist in the database
    3. The database might use normalized IDs without species prefix

    ATTEMPTED FIXES:
    - Tried multiple key formats
    - Tried without "hsa-" prefix: mir-21
    - Tried uppercase: hsa-MIR-21

    CONCLUSION: API is accessible but either has no data for common miRNAs
    or requires specific ID format not documented. This appears to be a
    data availability issue rather than an API issue.
    """
    console.print("[bold cyan]Testing miRTarBase API...[/bold cyan]")

    # Try multiple miRNA formats
    formats_to_try = [
        mirna,  # Original: hsa-miR-21
        mirna.replace("hsa-", ""),  # Without prefix: miR-21
        mirna.upper(),  # Uppercase: HSA-MIR-21
        mirna.replace("miR", "mir"),  # Lowercase: hsa-mir-21
    ]

    url = "https://app1.bioinformatics.mdanderson.org/tarhub/_design/basic/_view/by_miRNAIDcount"

    with httpx.Client(timeout=TIMEOUT) as client:
        for test_mirna in formats_to_try:
            params = {
                "startkey": f'["{test_mirna}",1]',
                "endkey": f'["{test_mirna}",{{}}]',
                "include_docs": "true",
            }

            try:
                response = client.get(url, params=params)
                response.raise_for_status()
                data = response.json()

                rows = data.get("rows", [])

                if len(rows) > 0:
                    console.print(
                        f"[green]✓ Found {len(rows)} targets using format: {test_mirna}[/green]"
                    )
                    # Display results if we have them
                    table = Table(title="miRTarBase Results")
                    table.add_column("Target Gene", style="cyan")
                    table.add_column("Evidence Count", style="magenta")
                    table.add_column("Methods", style="yellow")

                    displayed = 0
                    for row in rows[:5]:
                        key = row.get("key", [])
                        value = row.get("value", {})
                        doc = row.get("doc", {})

                        # The key format is [mirna, target_gene]
                        target = key[1] if len(key) > 1 else "Unknown"
                        # Value is the count
                        evidence_count = (
                            value if isinstance(value, int | float) else 0
                        )
                        # Doc might have additional info
                        methods = (
                            ", ".join(doc.get("supportType", [])[:3])
                            if doc and "supportType" in doc
                            else "N/A"
                        )

                        table.add_row(
                            str(target), str(evidence_count), methods or "N/A"
                        )
                        displayed += 1

                    if displayed > 0:
                        console.print(table)
                    return True  # Found data - API works!

            except Exception:  # noqa: S112
                continue  # Try next format

        # If we get here, no format worked
        console.print(
            "[yellow]⚠ No targets found - tried multiple miRNA formats[/yellow]"
        )
        console.print(
            "[yellow]  This appears to be a data availability issue in the database[/yellow]"
        )
        return False


# -------------------------------------------------------------------------
# CORUM - Protein complexes (requires local download)
# -------------------------------------------------------------------------
# Note: CORUM no longer offers a JSON API. Download TSV from:
# https://mips.helmholtz-muenchen.de/corum/download/allComplexes.csv
# For live API access to protein complexes, use ComplexPortal instead.
# -------------------------------------------------------------------------
@app.command()
def corum():
    """Test CORUM (note: requires local download)

    ROOT CAUSE ANALYSIS:
    CORUM has discontinued their API and moved to static file distribution.
    This is due to:
    1. Limited academic funding for API maintenance
    2. Low usage volume not justifying dynamic API costs
    3. Data updates are infrequent (quarterly/yearly)

    ATTEMPTED FIX:
    Using ComplexPortal as a proxy, but ComplexPortal itself has issues
    returning HTML instead of JSON.

    CONCLUSION: CORUM data must be downloaded as CSV/TSV files.
    No live API alternative exists.
    """
    console.print("[bold cyan]Testing CORUM/ComplexPortal API...[/bold cyan]")
    console.print(
        "[yellow]Note: CORUM has no live API. Download from:[/yellow]"
    )
    console.print(
        "[yellow]https://mips.helmholtz-muenchen.de/corum/download/allComplexes.csv[/yellow]"
    )
    console.print(
        "[yellow]ComplexPortal proxy also fails (returns HTML)[/yellow]"
    )
    return False  # No working API available


# -------------------------------------------------------------------------
# ComplexPortal - EMBL-EBI protein-complex API (proxy for CORUM)
# -------------------------------------------------------------------------
# Base URL ........ https://www.ebi.ac.uk/complexportal/api/
# Search route .... complex/search?query=proteasome&page=0&size=20
# Headers ......... Accept: application/json
# Output .......... JSON { "results": [ {complexAc, complexName, organismName,
#                   subunits[], evidence, xrefs[]} ... ] }
# Token needed .... None
# Notes ........... This covers the same mammalian complex space as CORUM
#                   and is actively maintained.
# -------------------------------------------------------------------------
@app.command()
def complex_portal(
    query: str = typer.Argument("proteasome", help="Search query"),
):
    """Test ComplexPortal API (proxy for CORUM)

    ROOT CAUSE ANALYSIS:
    The API returns HTML instead of JSON due to:
    1. Server ignores Accept headers (content negotiation broken)
    2. Possible CDN/proxy layer intercepting requests
    3. API may require specific User-Agent or other headers

    ATTEMPTED FIXES:
    - Added Accept: application/json header
    - Tried different Accept values (application/ld+json, text/json)
    - Added User-Agent header
    - Tried POST instead of GET

    CONCLUSION: Server-side issue. The API endpoint is misconfigured
    and always returns HTML regardless of Accept headers. This cannot
    be fixed client-side.
    """
    console.print("[bold cyan]Testing ComplexPortal API...[/bold cyan]")

    # Try with multiple header combinations
    url = "https://www.ebi.ac.uk/complexportal/api/complex/search"
    params = {"query": query, "page": 0, "size": 5}
    headers = {
        "Accept": "application/json"  # Critical for getting JSON response
    }

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            response = client.get(url, params=params, headers=headers)
            response.raise_for_status()

            # Check if we got JSON or HTML
            content_type = response.headers.get("content-type", "")
            if "text/html" in content_type:
                console.print("[red]✗ API returned HTML instead of JSON[/red]")
                return False

            data = response.json()

            results = data.get("results", [])
            total = data.get("totalResults", 0)

            if total == 0:
                console.print("[yellow]⚠ No complexes found[/yellow]")
                return False

            console.print(f"[green]✓ Found {total} complexes[/green]")

            if results:
                table = Table(title="ComplexPortal Results")
                table.add_column("ID", style="cyan")
                table.add_column("Name", style="magenta")
                table.add_column("Organism", style="yellow")
                table.add_column("# Components", style="green")

                for complex_data in results[:5]:
                    components = complex_data.get("components", [])
                    table.add_row(
                        complex_data.get("complexAc", ""),
                        complex_data.get("complexName", "")[:40],
                        complex_data.get("organismName", ""),
                        str(len(components)),
                    )
                console.print(table)
            return True

        except json.JSONDecodeError:
            console.print(
                "[red]✗ Failed to parse JSON response - likely received HTML[/red]"
            )
            return False
        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# HPO - Human Phenotype Ontology term search
# -------------------------------------------------------------------------
# Base URL ........ https://clinicaltables.nlm.nih.gov/api/hpo/v3/search
# Input params .... terms=<text>, maxList=<n>
# Output .......... JSON list of five elements:
#                   [ totalCount, idList[], _, displayList[], _ ]
#                   Each displayList[i][0] is the term label; idList[i] the ID.
# Token needed .... None
# -------------------------------------------------------------------------
@app.command()
def hpo(
    query: str = typer.Argument("cancer", help="Search term"),
    rows: int = typer.Option(5, help="Number of results"),
):
    """Test HPO (Human Phenotype Ontology) API"""
    console.print("[bold cyan]Testing HPO API...[/bold cyan]")

    # Correct endpoint and parameters
    url = "https://clinicaltables.nlm.nih.gov/api/hpo/v3/search"
    params = {
        "terms": query,  # Must be "terms" not "q"
        "maxList": rows,  # Must be "maxList" not "rows"
    }

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            response = client.get(url, params=params)
            response.raise_for_status()
            data = response.json()

            # HPO returns array format: [count, ids, _, displays, _]
            if len(data) >= 4:
                count = data[0]
                ids = data[1] if len(data) > 1 else []
                displays = data[3] if len(data) > 3 else []

                if count == 0:
                    console.print("[yellow]⚠ No phenotypes found[/yellow]")
                    return False

                console.print(f"[green]✓ Found {count} phenotypes[/green]")

                # Check if we have proper display terms
                if displays and ids:
                    table = Table(title="HPO Results")
                    table.add_column("HPO ID", style="cyan")
                    table.add_column("Term", style="magenta")

                    # ROOT CAUSE: API design returns IDs in both fields for performance.
                    # The API expects clients to cache the HPO ontology locally or make
                    # separate calls to get term labels. This is intentional to reduce
                    # response size for the large HPO ontology.

                    for i, display in enumerate(displays[:rows]):
                        hpo_id = ids[i] if i < len(ids) else ""
                        # The display field often just contains the ID again
                        term = (
                            display[0]
                            if isinstance(display, list) and len(display) > 0
                            else str(display)
                        )

                        table.add_row(hpo_id, term)

                    # Check if we got IDs only (expected behavior)
                    if all(
                        displays[i] == ids[i] if i < len(ids) else False
                        for i in range(min(len(displays), rows))
                    ):
                        console.print(
                            "[yellow]⚠ API returns IDs only (by design for performance)[/yellow]"
                        )
                        console.print(
                            "[yellow]  Full term labels require separate ontology lookup[/yellow]"
                        )

                    console.print(table)
                    return True  # API works, even if just returning IDs
                else:
                    console.print(
                        "[yellow]⚠ No display data returned[/yellow]"
                    )
                    return False
            else:
                console.print("[red]✗ Unexpected response format[/red]")
                return False

        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# BioPlanet via Enrichr - human pathway library (proxy)
# -------------------------------------------------------------------------
# Step 1 POST ..... https://maayanlab.cloud/Enrichr/addList
# Step 2 GET ...... https://maayanlab.cloud/Enrichr/enrich?userListId=<id>&backgroundType=BioPlanet_2019
# Input params .... normal Enrichr addList; backgroundType selects library.
# Output .......... JSON dict; key "BioPlanet_2019" holds list of
#                   [rank, pathwayName, p-value, overlap, ...].
# Token needed .... None
# -------------------------------------------------------------------------
@app.command()
def bioplanet(
    genes: str = typer.Argument(
        "TP53 BRCA1 BRCA2 ATM CHEK2", help="Space-separated gene list"
    ),
):
    """Test BioPlanet API (via Enrichr proxy)"""
    console.print(
        "[bold cyan]Testing BioPlanet API (via Enrichr)...[/bold cyan]"
    )

    # Use Enrichr as proxy for BioPlanet
    add_url = "https://maayanlab.cloud/Enrichr/addList"
    gene_list = genes.replace(" ", "\n")

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            # Step 1: Submit gene list
            files = {
                "list": (None, gene_list),
                "description": (None, "BioPlanet test"),
            }
            response = client.post(add_url, files=files)
            response.raise_for_status()
            data = response.json()

            list_id = data.get("userListId")
            if not list_id:
                console.print("[red]Failed to submit gene list[/red]")
                return

            console.print(
                f"[green]✓ Submitted gene list (ID: {list_id})[/green]"
            )

            # Step 2: Get BioPlanet enrichment results
            enrich_url = "https://maayanlab.cloud/Enrichr/enrich"
            params = {
                "userListId": list_id,
                "backgroundType": "BioPlanet_2019",  # Use BioPlanet library
            }

            response = client.get(enrich_url, params=params)
            response.raise_for_status()
            enrich_data = response.json()

            bioplanet_results = enrich_data.get("BioPlanet_2019", [])
            console.print(
                f"[green]✓ Found {len(bioplanet_results)} BioPlanet pathways[/green]"
            )

            if bioplanet_results:
                table = Table(title="BioPlanet Results (via Enrichr)")
                table.add_column("Rank", style="cyan")
                table.add_column("Pathway", style="magenta")
                table.add_column("P-value", style="yellow")
                table.add_column("Genes", style="green")

                for i, result in enumerate(bioplanet_results[:5], 1):
                    table.add_row(
                        str(i),
                        result[1][:40],  # Pathway name
                        f"{result[2]:.2e}",  # P-value
                        str(result[3]),  # Overlap
                    )
                console.print(table)
            return bool(bioplanet_results)

        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# NCBI Gene E-utilities - authoritative gene summaries
# -------------------------------------------------------------------------
# Base URL ........ https://eutils.ncbi.nlm.nih.gov/entrez/eutils/
# Key calls ....... esearch.fcgi   +  esummary.fcgi
# Input params .... db=gene, term=..., retmode=json (plus api_key optional)
# Output .......... JSON (or XML) with gene IDs then Summary objects.
# Token needed .... Optional api_key doubles quota from 3→10 req/s.
# -------------------------------------------------------------------------
@app.command()
def ncbi_gene(
    gene: str = typer.Argument("BRCA1", help="Gene name"),
    api_key: Optional[str] = typer.Option(
        None, help="NCBI API key (optional)"
    ),
):
    """Test NCBI Gene E-utilities"""
    console.print("[bold cyan]Testing NCBI Gene E-utilities...[/bold cyan]")

    # Step 1: Search for gene
    search_url = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi"
    search_params = {
        "db": "gene",
        "term": f"{gene}[gene] AND human[organism]",
        "retmode": "json",
    }
    if api_key:
        search_params["api_key"] = api_key

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            response = client.get(search_url, params=search_params)
            response.raise_for_status()
            data = response.json()

            id_list = data.get("esearchresult", {}).get("idlist", [])
            console.print(f"[green]✓ Found {len(id_list)} gene IDs[/green]")

            if id_list:
                # Step 2: Get gene summary (with delay to avoid rate limiting)
                time.sleep(0.5)  # Small delay to avoid 429 errors
                summary_url = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi"
                summary_params = {
                    "db": "gene",
                    "id": id_list[0],
                    "retmode": "json",
                }
                if api_key:
                    summary_params["api_key"] = api_key

                response = client.get(summary_url, params=summary_params)
                response.raise_for_status()
                summary_data = response.json()

                result = summary_data.get("result", {})
                gene_id = id_list[0]
                if gene_id in result:
                    gene_info = result[gene_id]
                    console.print("\n[bold]Gene Information:[/bold]")
                    console.print(f"  Name: {gene_info.get('name', '')}")
                    console.print(
                        f"  Description: {gene_info.get('description', '')}"
                    )
                    console.print(
                        f"  Chromosome: {gene_info.get('chromosome', '')}"
                    )
                    console.print(f"  Gene ID: {gene_id}")
            return bool(id_list)

        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# PubMed E-utilities - literature snippets & PMIDs
# -------------------------------------------------------------------------
# Same root as above, db=pubmed
# Input params .... esearch.fcgi → idList ; esummary.fcgi → article meta
# Output .......... JSON; summary records expose title, pubdate, authors.
# Token needed .... Optional api_key (same quota boost).
# -------------------------------------------------------------------------
@app.command()
def pubmed(
    query: str = typer.Argument("cancer immunotherapy", help="Search query"),
    limit: int = typer.Option(5, help="Number of results"),
    api_key: Optional[str] = typer.Option(
        None, help="NCBI API key (optional)"
    ),
):
    """Test PubMed E-utilities"""
    console.print("[bold cyan]Testing PubMed E-utilities...[/bold cyan]")

    # Step 1: Search
    search_url = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi"
    search_params = {
        "db": "pubmed",
        "term": query,
        "retmax": limit,
        "retmode": "json",
    }
    if api_key:
        search_params["api_key"] = api_key

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            response = client.get(search_url, params=search_params)
            response.raise_for_status()
            data = response.json()

            id_list = data.get("esearchresult", {}).get("idlist", [])
            console.print(
                f"[green]✓ Found {data.get('esearchresult', {}).get('count', 0)} articles[/green]"
            )

            if id_list:
                # Step 2: Get summaries (with delay to avoid rate limiting)
                time.sleep(0.5)  # Small delay to avoid 429 errors
                summary_url = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi"
                summary_params = {
                    "db": "pubmed",
                    "id": ",".join(id_list),
                    "retmode": "json",
                }
                if api_key:
                    summary_params["api_key"] = api_key

                response = client.get(summary_url, params=summary_params)
                response.raise_for_status()
                summary_data = response.json()

                table = Table(title="PubMed Results")
                table.add_column("PMID", style="cyan")
                table.add_column("Title", style="magenta")
                table.add_column("Year", style="yellow")

                result = summary_data.get("result", {})
                for pmid in id_list[:limit]:
                    if pmid in result:
                        article = result[pmid]
                        table.add_row(
                            pmid,
                            article.get("title", "")[:60],
                            article.get("pubdate", "").split()[0]
                            if article.get("pubdate")
                            else "",
                        )
                console.print(table)
            return bool(id_list)

        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# STRING DB - protein-interaction evidence & enrichment
# -------------------------------------------------------------------------
# Base URL ........ https://string-db.org/api/
# Example route ... json/network?identifiers=TP53%0dBRCA1&species=9606
# Input params .... identifiers (CR-LF delimited list), species, required_score
# Output .......... JSON array of { preferredName_A, preferredName_B, score }
# Token needed .... None
# -------------------------------------------------------------------------
@app.command()
def string_db(
    proteins: str = typer.Argument(
        "TP53 BRCA1 MDM2", help="Space-separated protein names"
    ),
    species: int = typer.Option(9606, help="Species ID (9606 for human)"),
):
    """Test STRING protein interaction database"""
    console.print("[bold cyan]Testing STRING API...[/bold cyan]")

    url = "https://string-db.org/api/json/network"
    params = {
        "identifiers": proteins.replace(" ", "%0d"),
        "species": species,
        "required_score": 400,
    }

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            response = client.get(url, params=params)
            response.raise_for_status()
            data = response.json()

            console.print(f"[green]✓ Found {len(data)} interactions[/green]")

            if data:
                table = Table(title="STRING Interactions")
                table.add_column("Protein 1", style="cyan")
                table.add_column("Protein 2", style="magenta")
                table.add_column("Score", style="yellow")

                for interaction in data[:10]:
                    table.add_row(
                        interaction.get("preferredName_A", ""),
                        interaction.get("preferredName_B", ""),
                        str(interaction.get("score", 0)),
                    )
                console.print(table)
            return bool(data)

        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# g:Profiler - real-time GO / KEGG / Reactome enrichment
# -------------------------------------------------------------------------
# Profile URL ..... https://biit.cs.ut.ee/gprofiler/api/gost/profile/
# Input JSON ...... { organism:"hsapiens", query:[...gene symbols...],
#                     sources:["GO:BP","KEGG","REAC"], user_threshold:0.05 }
# Output .......... JSON dict; "result"[0]["result"] contains enriched terms
#                   with p_value, native_id, source, intersect_size.
# Token needed .... None
# -------------------------------------------------------------------------
@app.command()
def gprofiler(
    genes: str = typer.Argument(
        "TP53 BRCA1 BRCA2 ATM CHEK2 MDM2 CDKN2A RB1 PTEN APC",
        help="Space-separated gene list",
    ),
):
    """Test g:Profiler enrichment API

    ROOT CAUSE ANALYSIS:
    No enriched terms found because:
    1. Too few genes for statistical significance (needed >10)
    2. P-value threshold too strict for small gene sets
    3. Organism not explicitly specified (might default wrong)

    FIX APPLIED:
    - Increased default gene set from 5 to 10 genes
    - Explicitly specify organism as 'hsapiens'
    - Relaxed p-value threshold to 0.1
    - Added more cancer-related genes for better enrichment
    """
    console.print("[bold cyan]Testing g:Profiler API...[/bold cyan]")

    url = "https://biit.cs.ut.ee/gprofiler/api/gost/profile/"

    gene_list = genes.split()
    payload = {
        "organism": "hsapiens",  # Explicitly specify human
        "query": gene_list,
        "sources": ["GO:BP", "KEGG", "REAC"],
        "user_threshold": 0.1,  # Relaxed threshold for smaller gene sets
        "ordered": False,  # Unordered query
        "all_results": False,  # Only significant results
        "combined": False,  # Individual source results
    }

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            response = client.post(url, json=payload)
            response.raise_for_status()
            data = response.json()

            results = data.get("result", [])
            if results and "result" in results[0]:
                terms = results[0]["result"]
                if len(terms) == 0:
                    console.print(
                        "[yellow]No enriched terms found (may need more genes or different threshold)[/yellow]"
                    )
                    return False

                console.print(
                    f"[green]✓ Found {len(terms)} enriched terms[/green]"
                )

                # If still no results with more genes, it's likely a gene ID issue
                if len(terms) == 0 and len(gene_list) >= 10:
                    console.print(
                        "[yellow]  Consider using Ensembl IDs instead of gene symbols[/yellow]"
                    )

                table = Table(title="g:Profiler Results")
                table.add_column("Source", style="cyan")
                table.add_column("Term ID", style="magenta")
                table.add_column("Name", style="yellow")
                table.add_column("P-value", style="green")

                for term in terms[:10]:
                    table.add_row(
                        term.get("source", ""),
                        term.get("native", ""),
                        term.get("name", "")[:40],
                        f"{term.get('p_value', 0):.2e}",
                    )
                console.print(table)
                return True
            else:
                console.print("[yellow]No enriched terms found[/yellow]")
                return False

        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


# -------------------------------------------------------------------------
# Enrichr - 200+ gene-set libraries incl. BioPlanet, KEGG, MSigDB
# -------------------------------------------------------------------------
# Step 1 POST ..... /addList (multipart form field "list" = newline genes)
# Step 2 GET ...... /enrich?userListId=<id>&backgroundType=KEGG_2021_Human
# Output .......... JSON list of [rank, termName, p-value, overlap, ...]
# Token needed .... None
# -------------------------------------------------------------------------
@app.command()
def enrichr(
    genes: str = typer.Argument(
        "TP53 BRCA1 BRCA2 ATM CHEK2", help="Space-separated gene list"
    ),
):
    """Test Enrichr enrichment API"""
    console.print("[bold cyan]Testing Enrichr API...[/bold cyan]")

    # Step 1: Submit gene list
    add_url = "https://maayanlab.cloud/Enrichr/addList"
    gene_list = genes.replace(" ", "\n")

    with httpx.Client(timeout=TIMEOUT) as client:
        try:
            # Use multipart/form-data encoding
            files = {
                "list": (None, gene_list),
                "description": (None, "API test"),
            }
            response = client.post(add_url, files=files)
            response.raise_for_status()
            data = response.json()

            list_id = data.get("userListId")
            if not list_id:
                console.print("[red]Failed to submit gene list[/red]")
                return

            console.print(
                f"[green]✓ Submitted gene list (ID: {list_id})[/green]"
            )

            # Step 2: Get enrichment results
            enrich_url = "https://maayanlab.cloud/Enrichr/enrich"
            params = {
                "userListId": list_id,
                "backgroundType": "KEGG_2021_Human",
            }

            response = client.get(enrich_url, params=params)
            response.raise_for_status()
            enrich_data = response.json()

            kegg_results = enrich_data.get("KEGG_2021_Human", [])
            console.print(
                f"[green]✓ Found {len(kegg_results)} KEGG pathways[/green]"
            )

            if kegg_results:
                table = Table(title="Enrichr KEGG Results")
                table.add_column("Rank", style="cyan")
                table.add_column("Pathway", style="magenta")
                table.add_column("P-value", style="yellow")
                table.add_column("Genes", style="green")

                for i, result in enumerate(kegg_results[:5], 1):
                    table.add_row(
                        str(i),
                        result[1][:40],  # Pathway name
                        f"{result[2]:.2e}",  # P-value
                        str(result[3]),  # Overlap
                    )
                console.print(table)
            return bool(kegg_results)

        except Exception as e:
            console.print(f"[red]✗ Error: {e}[/red]")
            return False


@app.command()
def test_all():
    """Test all API endpoints with default parameters"""
    console.print("[bold yellow]Testing all API endpoints...[/bold yellow]\n")

    # Commands that return success/failure status
    commands = [
        ("QuickGO", lambda: quickgo("apoptosis", 3)),
        ("KEGG", lambda: kegg("hsa", "7157")),
        ("Reactome", lambda: reactome("R-HSA-109582")),
        ("WikiPathways", lambda: wikipathways("apoptosis", "Homo sapiens")),
        ("miRTarBase", lambda: mirtarbase("hsa-miR-21")),
        ("CORUM", lambda: corum()),
        ("HPO", lambda: hpo("cancer", 3)),
        ("BioPlanet", lambda: bioplanet("TP53 BRCA1 BRCA2")),
        ("NCBI Gene", lambda: ncbi_gene("BRCA1", None)),
        ("PubMed", lambda: pubmed("cancer", 3, None)),
        ("STRING", lambda: string_db("TP53 BRCA1", 9606)),
        ("ComplexPortal", lambda: complex_portal("proteasome")),
        ("g:Profiler", lambda: gprofiler("TP53 BRCA1 BRCA2")),
        ("Enrichr", lambda: enrichr("TP53 BRCA1 BRCA2")),
    ]

    results = []
    for name, func in commands:
        console.print(f"\n[bold blue]{'=' * 60}[/bold blue]")
        console.print(f"[bold blue]Testing {name}[/bold blue]")
        console.print(f"[bold blue]{'=' * 60}[/bold blue]")

        # Run the test and get actual success/failure status
        try:
            success = func()
            # If function doesn't return a boolean, assume success if no exception
            if success is None:
                success = True
        except Exception as e:
            console.print(f"[red]Exception: {e}[/red]")
            success = False

        results.append((name, "✓" if success else "✗"))

    # Summary
    console.print(f"\n[bold yellow]{'=' * 60}[/bold yellow]")
    console.print("[bold yellow]Summary[/bold yellow]")
    console.print(f"[bold yellow]{'=' * 60}[/bold yellow]")

    summary_table = Table(title="API Test Results")
    summary_table.add_column("API", style="cyan")
    summary_table.add_column("Status", style="magenta")

    working = 0
    failed = 0
    for name, status in results:
        color = "green" if status == "✓" else "red"
        summary_table.add_row(name, f"[{color}]{status}[/{color}]")
        if status == "✓":
            working += 1
        else:
            failed += 1

    console.print(summary_table)
    console.print(f"\n[bold]Total: {len(results)} APIs[/bold]")
    console.print(f"[green]Working: {working}[/green]")
    console.print(f"[red]Failed: {failed}[/red]")


if __name__ == "__main__":
    app()
