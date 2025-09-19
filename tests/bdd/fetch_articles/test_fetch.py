import json
import shlex

from pytest_bdd import given, parsers, scenarios, then
from typer.testing import CliRunner

from biomcp.cli.main import app

scenarios("fetch.feature")

runner = CliRunner()


@given(parsers.parse('I run "{command}"'), target_fixture="cli_result")
def cli_result(command):
    """Run the given CLI command and return the parsed JSON output."""
    args = shlex.split(command)[1:]
    result = runner.invoke(app, args)
    return json.loads(result.stdout)


@then("the JSON output should be a non-empty list")
def check_non_empty_list(cli_result):
    """Check that the JSON output is a list with at least one article."""
    assert isinstance(cli_result, list), "Expected JSON output to be a list"
    assert len(cli_result) > 0, "Expected at least one article in the output"


@then("the first article's abstract should be populated")
def check_abstract_populated(cli_result):
    """Check that the first article has a non-empty abstract."""
    article = cli_result[0]
    abstract = article.get("abstract")
    assert abstract is not None, "Abstract field is missing"
    assert abstract.strip() != "", "Abstract field is empty"


@then("the application should return an error")
def step_impl(cli_result):
    assert cli_result == [
        {"error": 'Error 400: {"detail":"Could not retrieve publications"}'}
    ]


@then("the first article should have a DOI field")
def check_doi_field(cli_result):
    """Check that the first article has a DOI field."""
    article = cli_result[0]
    doi = article.get("doi")
    assert doi is not None, "DOI field is missing"
    assert doi.startswith("10."), f"Invalid DOI format: {doi}"


@then("the source should be Europe PMC")
def check_europe_pmc_source(cli_result):
    """Check that the article source is Europe PMC."""
    article = cli_result[0]
    source = article.get("source")
    assert (
        source == "Europe PMC"
    ), f"Expected source 'Europe PMC', got '{source}'"
