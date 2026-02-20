import asyncio

import pytest
from pytest_bdd import given, parsers, scenarios, then, when

from biomcp.articles.autocomplete import (
    Concept,
    Entity,
    EntityRequest,
    autocomplete,
)

scenarios("autocomplete.feature")

pytestmark = pytest.mark.usefixtures("mock_pubtator3_http")


@given(
    parsers.parse(
        'I have a valid concept "{concept}" and a valid query "{query}"',
    ),
    target_fixture="entity_request",
)
def entity_request(concept: Concept, query: str):
    return EntityRequest(concept=concept, query=query)


@given(
    parsers.parse(
        'I have a valid concept "{concept}" and an invalid query "{query}"',
    ),
    target_fixture="entity_request",
)
def invalid_query_request(concept: Concept, query: str):
    return EntityRequest(concept=concept, query=query)


@when(
    "I call the Pubtator Autocomplete API",
    target_fixture="entity",
)
def entity(entity_request) -> Entity | None:
    return asyncio.run(autocomplete(request=entity_request))


@then(parsers.parse('the response entity_id should be "{expected_id}"'))
def check_entity_id(entity, expected_id):
    assert entity.entity_id == expected_id


@then(parsers.parse('the response concept should be "{concept}"'))
def check_concept(entity, concept):
    assert entity.concept == concept


@then("the response should be empty")
def check_empty_response(entity):
    assert entity is None
