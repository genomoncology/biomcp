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
    # Remove the initial token ("biomcp") if present.
    args = shlex.split(command)[1:]
    result = runner.invoke(app, args)
    assert result.exit_code == 0, f"CLI command failed: {result.stderr}"
    return json.loads(result.stdout)


def get_field_value(data, field_path):
    """
    Access a nested dictionary value using a dot-notation path.
    Supports array notation like "locations[0]".
    """
    parts = field_path.split(".")
    value = data
    for part in parts:
        if "[" in part and part.endswith("]"):
            # e.g. "locations[0]"
            base, index_str = part[:-1].split("[")
            index = int(index_str)
            value = value[base][index]
        else:
            value = value[part]
    return value


@then(parsers.parse('the field "{field}" should equal "{expected}"'))
def field_should_equal(cli_result, field, expected):
    """
    Verify that the value at the specified dot-notation field equals the expected value.
    """
    actual = get_field_value(cli_result, field)
    # Compare as strings for simplicity.
    assert (
        str(actual) == expected
    ), f"Expected field '{field}' to equal '{expected}', but got '{actual}'"
