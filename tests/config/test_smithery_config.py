#!/usr/bin/env python
"""
Test script to validate Smithery configuration against actual function implementations.
This script checks that the schema definitions in smithery.yaml match the expected
function parameters in your codebase.
"""

import os
from typing import Any

import pytest
import yaml
from pydantic import BaseModel

from biomcp.articles.search import PubmedRequest

# Import the functions we want to test
from biomcp.trials.search import TrialQuery
from biomcp.variants.search import VariantQuery


@pytest.fixture
def smithery_config():
    """Load the Smithery configuration."""
    # Get the project root directory
    project_root = os.path.abspath(
        os.path.join(os.path.dirname(__file__), "../..")
    )
    config_path = os.path.join(project_root, "smithery.yaml")

    with open(config_path) as f:
        return yaml.safe_load(f)


def test_smithery_config(smithery_config):
    """Test that all tool schemas in smithery.yaml match the expected function parameters."""
    # Functions to test and their expected parameter types
    functions_to_test = {
        "trial_searcher": {"param_name": "query", "expected_type": TrialQuery},
        "variant_searcher": {
            "param_name": "query",
            "expected_type": VariantQuery,
        },
        "article_searcher": {
            "param_name": "query",
            "expected_type": PubmedRequest,
        },
        "trial_protocol": {"param_name": "nct_id", "expected_type": str},
        "trial_locations": {"param_name": "nct_id", "expected_type": str},
        "trial_outcomes": {"param_name": "nct_id", "expected_type": str},
        "trial_references": {"param_name": "nct_id", "expected_type": str},
        "article_details": {"param_name": "pmid", "expected_type": str},
        "variant_details": {"param_name": "variant_id", "expected_type": str},
    }

    for tool_name, param_info in functions_to_test.items():
        validate_tool_schema(smithery_config, tool_name, param_info)


def validate_tool_schema(
    smithery_config, tool_name: str, param_info: dict[str, Any]
):
    """Validate that the tool schema in smithery.yaml matches the expected function parameter."""
    param_name = param_info["param_name"]
    expected_type = param_info["expected_type"]

    # Check if the tool is defined in the smithery.yaml
    assert tool_name in smithery_config.get(
        "tools", {}
    ), f"Tool '{tool_name}' is not defined in smithery.yaml"

    tool_config = smithery_config["tools"][tool_name]

    # Check if the tool has an input schema
    assert (
        "input" in tool_config
    ), f"Tool '{tool_name}' does not have an input schema defined"

    input_schema = tool_config["input"].get("schema", {})

    # Check if the parameter is required
    if issubclass(expected_type, BaseModel):
        # For complex types like TrialQuery, check if 'query' is required
        assert (
            "required" in input_schema
        ), f"Tool '{tool_name}' does not have required parameters specified"
        assert (
            "query" in input_schema.get("required", [])
        ), f"Parameter 'query' for tool '{tool_name}' is not marked as required"
    else:
        assert (
            "required" in input_schema
        ), f"Tool '{tool_name}' does not have required parameters specified"
        assert (
            param_name in input_schema.get("required", [])
        ), f"Parameter '{param_name}' for tool '{tool_name}' is not marked as required"

    # For complex types (Pydantic models), check if the schema references the correct type
    if issubclass(expected_type, BaseModel):
        properties = input_schema.get("properties", {})
        assert (
            "query" in properties
        ), f"Tool '{tool_name}' does not have a 'query' property defined"

        query_prop = properties["query"]
        assert (
            "$ref" in query_prop
        ), f"Tool '{tool_name}' query property does not reference a schema"

        schema_ref = query_prop["$ref"]
        expected_schema_name = expected_type.__name__
        assert schema_ref.endswith(
            expected_schema_name
        ), f"Tool '{tool_name}' references incorrect schema: {schema_ref}, expected: {expected_schema_name}"
