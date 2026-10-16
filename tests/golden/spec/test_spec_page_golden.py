"""Spec-page golden cases: JSON output and request-log arithmetic.

The spec pages stay reader-facing specifications (`like` and `expect=`
assertions); their JSON field checks and the request-log counting they used
to do in bash live here, beside 2079's golden cases, run against the same
replay server. Each case runs the real command, requires every request to
replay from disk, and pins what the client asked for: the provider routes a
command consults and the query shapes the pages asserted by hand.
"""

from __future__ import annotations

import pytest

from replay_env import ReplayEnvironment
from test_golden_output import BINARY, CASES, _failed_requests

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module

CASES_BY_ID = {case.id: case for case in CASES}


def _run(replay: ReplayEnvironment, case_id: str, *extra: str):
    case = CASES_BY_ID[case_id]
    run = replay.run(BINARY, (*case.argv, *extra))
    failures = _failed_requests(run)
    assert not failures, (
        f"{case_id} made requests with no recorded reply:\n"
        + "\n".join(str(entry) for entry in failures)
    )
    assert run.process.returncode == 0, (
        f"{case_id} exited {run.process.returncode}\nstderr:\n{run.process.stderr}"
    )
    return run


def _routes(run) -> list[str]:
    """The provider route (first path segment) of every request, in order."""
    routes: list[str] = []
    for entry in run.request_log_lines:
        key = str(entry["key"])
        # The log key is "METHOD /path?query"; strip the method before the path.
        path = key.split(" ", 1)[1] if " " in key else key
        routes.append(path.split("?", 1)[0].strip("/").split("/", 1)[0])
    return routes


def _keys_matching(run, fragment: str) -> list[str]:
    return [
        str(entry["key"])
        for entry in run.request_log_lines
        if fragment in str(entry["key"])
    ]


def test_get_gene_queries_mygene_once_by_symbol(replay: ReplayEnvironment) -> None:
    run = _run(replay, "get-gene-braf", "--json")
    # The acquisition arithmetic the gene page counted by hand: one
    # MyGene query for the spelled symbol, no alias scan, no second fetch.
    queries = _keys_matching(run, "GET /mygene/query?")
    assert len(queries) == 1, run.request_log_lines
    assert 'q=symbol%3A%22BRAF%22' in queries[0]
    assert "species=human" in queries[0]


def test_get_variant_queries_myvariant_once_by_protein_change(
    replay: ReplayEnvironment,
) -> None:
    run = _run(replay, "get-variant-braf-v600e", "--json")
    queries = _keys_matching(run, "GET /myvariant/query?")
    assert len(queries) == 1, run.request_log_lines
    assert "dbnsfp.genename%3ABRAF" in queries[0]
    assert "dbnsfp.hgvsp%3A%22p.V600E%22" in queries[0]


def test_search_all_consults_the_recorded_provider_matrix(
    replay: ReplayEnvironment,
) -> None:
    run = _run(replay, "search-all-braf-melanoma", "--json")
    assert sorted(set(_routes(run))) == [
        "cpic",
        "ctgov",
        "europepmc",
        "gwas",
        "kegg",
        "mychem",
        "mydisease",
        "mygene",
        "myvariant",
        "pubmed",
        "pubtator",
        "reactome",
        "s2",
        "wikipathways",
    ]
    # The trial leg asks ClinicalTrials.gov for the typed condition with the
    # gene as term, exactly the query the trial page asserted by hand.
    trial_queries = _keys_matching(run, "GET /ctgov/studies?")
    assert len(trial_queries) == 1, run.request_log_lines
    assert "query.cond=melanoma" in trial_queries[0]
    assert "BRAF%22" in trial_queries[0]


def test_search_all_counts_only_keeps_the_same_provider_matrix(
    replay: ReplayEnvironment,
) -> None:
    full = _run(replay, "search-all-braf-melanoma", "--json")
    counts = _run(replay, "search-all-braf-melanoma-counts-only", "--json")
    assert sorted(set(_routes(counts))) == sorted(set(_routes(full)))
