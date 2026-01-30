from biomcp.http_client import parse_response


def test_parse_response_csv_strips_bom_and_newlines():
    content = "\ufeff\nGeneric Name,Status\nFoo,Current\n"

    response, error = parse_response(200, content, response_model_type=None)

    assert error is None
    assert isinstance(response, list)
    assert response
    assert "Generic Name" in response[0]
    assert "\ufeffGeneric Name" not in response[0]
    assert response[0]["Generic Name"] == "Foo"
    assert response[0]["Status"] == "Current"
