import json
import shlex

from pytest_bdd import given, parsers, scenarios, then
from typer.testing import CliRunner

from biomcp.cli.main import app

# Link to the feature file
scenarios("get.feature")

runner = CliRunner()


@given(parsers.parse('I run "{command}"'), target_fixture="cli_result")
def cli_result(command):
    """
    Run the given CLI command and return the parsed JSON output.
    The command is expected to include the '--json' flag.
    """
    args = shlex.split(command)[1:]  # remove the leading "biomcp" token
    result = runner.invoke(app, args)
    assert result.exit_code == 0, f"CLI command failed: {result.stderr}"
    return json.loads(result.stdout)


def get_field_value_from_variant(variant, field_path):
    """
    Retrieve a value from a variant dictionary using a simple dot-notation path.
    (This version does not support array indexing.)
    """
    parts = field_path.split(".")
    value = variant
    for part in parts:
        value = value.get(part)
        if value is None:
            break
    return value


@then(
    parsers.parse(
        'at least one variant should have field "{field}" equal to "{expected}"'
    )
)
def variant_field_should_equal(cli_result, field, expected):
    """
    Verify that at least one variant in the returned list has the specified field equal to the expected value.
    """
    # cli_result is already a list of variant dicts.
    matching = [
        v
        for v in cli_result
        if str(get_field_value_from_variant(v, field)) == expected
    ]
    assert (
        matching
    ), f"No variant found with field '{field}' equal to '{expected}'"
