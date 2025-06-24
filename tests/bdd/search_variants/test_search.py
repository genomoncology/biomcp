import json
import shlex
from typing import Any

from assertpy import assert_that
from pytest_bdd import parsers, scenarios, then, when
from typer.testing import CliRunner

from biomcp.cli import app

scenarios("search.feature")

runner = CliRunner()

# Field mapping - Updated chromosome key
FIELD_MAP = {
    "chromosome": ["chrom"],
    "frequency": ["gnomad_exome", "af", "af"],
    "gene": ["dbnsfp", "genename"],
    "hgvsc": ["dbnsfp", "hgvsc"],
    "hgvsp": ["dbnsfp", "hgvsp"],
    "cadd": ["cadd", "phred"],
    "polyphen": ["dbnsfp", "polyphen2", "hdiv", "pred"],
    "position": ["vcf", "position"],
    "rsid": ["dbsnp", "rsid"],
    "sift": ["dbnsfp", "sift", "pred"],
    "significance": ["clinvar", "rcv", "clinical_significance"],
    "uniprot_id": ["mutdb", "uniprot_id"],
}


def get_value(data: dict, key: str) -> Any | None:
    """Extract value from nested dictionary using field mapping."""
    key_path = FIELD_MAP.get(key, [key])
    current_value = data.get("hits")
    for key in key_path:
        if isinstance(current_value, dict):
            current_value = current_value.get(key)
        elif isinstance(current_value, list):
            current_value = current_value[0].get(key)
    if current_value and isinstance(current_value, list):
        return current_value[0]
    return current_value


# --- @when Step ---
@when(
    parsers.re(r'I run "(?P<command>.*?)"(?: #.*)?$'),
    target_fixture="variants_data",
)
def variants_data(command) -> dict:
    """Run variant search command with --json and return parsed results."""
    args = shlex.split(command)[1:]  # trim 'biomcp'
    args += ["--json"]
    if "--size" not in args:
        args.extend(["--size", "10"])

    result = runner.invoke(app, args, catch_exceptions=False)
    assert result.exit_code == 0, "CLI command failed"
    data = json.loads(result.stdout)
    return data


def normalize(v):
    try:
        return float(v)
    except ValueError:
        try:
            return int(v)
        except ValueError:
            return v.lower()


@then(
    parsers.re(
        r"each variant should have (?P<field>\w+) that (?P<operator>(?:is|equal|to|contains|greater|less|than|or|\s)+)\s+(?P<expected>.+)$"
    )
)
def check_variant_field(it, variants_data, field, operator, expected):
    """
    For each variant, apply an assertpy operator against a given field.
    Supports operator names with spaces (e.g. "is equal to") or underscores (e.g. "is_equal_to").
    """
    # Normalize operator: lower case and replace spaces with underscores.
    operator = operator.strip().lower().replace(" ", "_")
    successes = set()
    failures = set()
    for v_num, value in it(FIELD_MAP, variants_data, field):
        value = normalize(value)
        expected = normalize(expected)
        f = getattr(assert_that(value), operator)
        try:
            f(expected)
            successes.add(v_num)
        except AssertionError:
            failures.add(v_num)

    failures -= successes
    assert len(failures) == 0, f"Failure: {field} {operator} {expected}"


@then(
    parsers.re(
        r"the number of variants (?P<operator>(?:is|equal|to|contains|greater|less|than|or|\s)+)\s+(?P<expected>\d+)$"
    )
)
def number_of_variants_check(variants_data, operator, expected):
    """Check the number of variants returned."""
    if (
        isinstance(variants_data, list)
        and len(variants_data) == 1
        and "error" in variants_data[0]
    ):
        count = 0  # If we have an error response, count as 0 variants
    elif isinstance(variants_data, dict) and "variants" in variants_data:
        # Handle new format with cBioPortal summary
        count = len(variants_data["variants"])
    elif isinstance(variants_data, dict) and "hits" in variants_data:
        # Handle myvariant.info response format
        count = len(variants_data["hits"])
    else:
        count = len(variants_data) if isinstance(variants_data, list) else 0
    operator = operator.strip().lower().replace(" ", "_")
    f = getattr(assert_that(count), operator)
    f(int(expected))
