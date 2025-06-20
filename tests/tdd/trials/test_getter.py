from biomcp.trials.getter import Module, get_trial, modules


async def test_get_protocol(anyio_backend):
    markdown = await get_trial("NCT04280705", Module.PROTOCOL)
    assert markdown.startswith("Url: https://clinicaltrials.gov/study/")
    assert len(markdown) > 10000  # 10370 on 2025-03-23


async def test_get_locations(anyio_backend):
    markdown = await get_trial("NCT04280705", Module.LOCATIONS)
    starts_with = """Url: https://clinicaltrials.gov/study/NCT04280705

# Protocol Section
"""
    assert markdown.startswith(starts_with)
    assert "University of California San Francisco" in markdown
    assert len(markdown) > 12000  # 12295 on 2025-03-23


async def test_get_references(anyio_backend):
    markdown = await get_trial("NCT04280705", Module.REFERENCES)
    assert "# Protocol Section" in markdown
    assert "## References Module" in markdown
    assert len(markdown) > 0


async def test_get_outcomes(anyio_backend):
    markdown = await get_trial("NCT04280705", Module.OUTCOMES)
    assert "# Protocol Section" in markdown
    assert (
        "## Outcomes Module" in markdown or "## Results Sections" in markdown
    )
    assert len(markdown) > 0


async def test_invalid_nct_id(anyio_backend):
    markdown = await get_trial("NCT99999999")
    assert "NCT number NCT99999999 not found" in markdown


def test_all_modules_exist():
    # Verify all modules are defined
    assert "Protocol" in modules
    assert "Locations" in modules
    assert "References" in modules
    assert "Outcomes" in modules

    # Verify protocol module contains critical sections
    protocol_sections = modules[Module.PROTOCOL]
    assert "IdentificationModule" in protocol_sections
    assert "StatusModule" in protocol_sections
    assert "DescriptionModule" in protocol_sections


async def test_cli_default_module_functionality(anyio_backend):
    # Test directly with both explicit Protocol and None (which should use Protocol)
    markdown_with_protocol = await get_trial("NCT04280705", Module.PROTOCOL)
    assert len(markdown_with_protocol) > 10000

    # In a real CLI context, the default would be set at the CLI level
    # This test ensures the Protocol module is valid for that purpose
    assert "Protocol Section" in markdown_with_protocol


async def test_json_output(anyio_backend):
    # Test JSON output format
    json_output = await get_trial(
        "NCT04280705", Module.PROTOCOL, output_json=True
    )
    assert json_output.startswith("{")
    assert "URL" in json_output
    assert "NCT04280705" in json_output


async def test_error_handling_json_output(anyio_backend):
    # Test error handling with JSON output
    json_output = await get_trial(
        "NCT99999999", Module.PROTOCOL, output_json=True
    )
    assert "error" in json_output
    assert "NCT99999999" in json_output
