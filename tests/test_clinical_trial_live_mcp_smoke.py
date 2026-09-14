from __future__ import annotations

import copy
import importlib.util
import io
import json
from pathlib import Path
import sys
from typing import Any

import pytest

ROOT = Path(__file__).resolve().parents[1]
CLIENT_PATH = ROOT / "tools/clinical-trial-live-mcp-smoke.py"


def _module():
    spec = importlib.util.spec_from_file_location(
        "clinical_trial_live_mcp_smoke", CLIENT_PATH
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _tool_response(payload: object, *, is_error: bool = False) -> dict[str, Any]:
    return {
        "jsonrpc": "2.0",
        "id": "fixture",
        "result": {
            "content": [{"type": "text", "text": json.dumps(payload)}],
            "isError": is_error,
        },
    }


def _detail(identity: str, authority: str) -> dict[str, Any]:
    return {
        "identities": [{"authority": authority, "identifier": identity}],
        "title": f"{authority} trial",
        "capture": {
            "source_authority": authority,
            "provider_record_identity": identity,
            "digest": "sha256:" + "a" * 64,
        },
        "conversion_report": [],
        "section_states": {"eligibility": "present"},
    }


def _search(identity: str, authority: str) -> dict[str, Any]:
    return {
        "count": 1,
        "results": [
            {
                "nct_id": identity,
                "title": f"{authority} search result",
                "capture": {
                    "source_authority": authority,
                    "provider_record_identity": identity,
                    "digest": "sha256:" + "b" * 64,
                },
                "conversion_report": [],
            }
        ],
        "pagination": {
            "offset": 0,
            "limit": 5,
            "returned": 1,
            "total": 1,
            "total_precision": "exact",
            "continuation_status": "terminal",
            "next_page_token": None,
            "has_more": False,
        },
    }


def _rpc_error(code: int) -> dict[str, Any]:
    return {
        "jsonrpc": "2.0",
        "id": "fixture",
        "error": {"code": code, "message": "sanitized fixture error"},
    }


def _payload_for_call(call: int) -> dict[str, Any]:
    payloads = (
        _detail("NCT03361748", "clinicaltrials.gov"),
        _search("NCT03361748", "clinicaltrials.gov"),
        _detail("NCI-2020-00001", "nci"),
        _search("NCT05929768", "nci"),
    )
    return copy.deepcopy(payloads[(call - 1) % 4])


def _capture_for_payload(payload: dict[str, Any]) -> dict[str, Any]:
    if "results" in payload:
        return payload["results"][0]["capture"]
    return payload["capture"]


def _mutated_success(call: int, mutation: str) -> dict[str, Any]:
    payload = _payload_for_call(call)
    capture = _capture_for_payload(payload)
    if mutation == "authority":
        capture["source_authority"] = "wrong"
    elif mutation == "identity":
        capture["provider_record_identity"] = "NCT00000000"
    elif mutation == "digest":
        capture["digest"] = "sha256:" + "A" * 64
    else:
        pagination = payload["pagination"]
        if mutation == "invalid-precision":
            pagination["total_precision"] = "estimated"
        elif mutation == "exact-reason":
            pagination["total_reason"] = "before_local_filtering"
        elif mutation == "exact-null-reason":
            pagination["total_reason"] = None
        elif mutation == "approximate-reason":
            pagination["total_precision"] = "approximate"
            pagination["total_reason"] = "provider_omitted_total"
        elif mutation == "unknown-number":
            pagination["total_precision"] = "unknown"
            pagination["total_reason"] = "provider_omitted_total"
        elif mutation == "unknown-reason":
            pagination["total"] = None
            pagination["total_precision"] = "unknown"
            pagination["total_reason"] = "unrecognized"
        elif mutation == "returned-mismatch":
            pagination["returned"] = 2
        elif mutation == "over-limit":
            pagination["limit"] = 0
        elif mutation == "total-too-small":
            pagination["offset"] = 1
        elif mutation == "terminal-more":
            pagination["has_more"] = True
        elif mutation == "terminal-null-offset":
            pagination["next_offset"] = None
        elif mutation == "terminal-null-reason":
            pagination["continuation_reason"] = None
        elif mutation == "cursor-missing":
            pagination["continuation_status"] = "cursor"
            pagination["has_more"] = True
        elif mutation == "offset-mismatch":
            pagination["continuation_status"] = "offset"
            pagination["next_offset"] = 9
            pagination["has_more"] = True
        elif mutation == "invalid-status":
            pagination["continuation_status"] = "more"
        else:
            raise AssertionError(f"unknown fixture mutation: {mutation}")
    return _tool_response(payload)


class FakeClient:
    def __init__(
        self,
        *,
        documents: list[dict[str, object]],
        mutations: dict[int, dict[str, Any]] | None = None,
    ) -> None:
        self.documents = documents
        self.mutations = mutations or {}
        self.calls: list[tuple[str, dict[str, object]]] = []

    def call_tool(self, name: str, arguments: dict[str, object]) -> dict[str, Any]:
        self.calls.append((name, arguments))
        if len(self.calls) in self.mutations:
            return self.mutations[len(self.calls)]
        if len(self.calls) <= 8:
            return _tool_response(_payload_for_call(len(self.calls)))
        if len(self.calls) == 9:
            return _tool_response({"documents": self.documents})
        if len(self.calls) == 10:
            return _tool_response(
                {"error": {"code": "invalid_argument", "message": "unsupported"}},
                is_error=True,
            )
        return _rpc_error(-32602)

    def list_tools(self) -> dict[str, Any]:
        self.calls.append(("tools/list", {}))
        trial = {
            "properties": {
                "entity": {"const": "trial"},
                "sections": {"items": {"enum": ["all", "eligibility"]}},
            }
        }
        return {
            "jsonrpc": "2.0",
            "id": "fixture",
            "result": {"tools": [{"name": "get", "inputSchema": {"oneOf": [trial]}}]},
        }


def _run_with_mutation(tmp_path: Path, call: int, response: dict[str, Any]) -> None:
    module = _module()
    tmp_path.chmod(0o700)
    client = FakeClient(documents=[], mutations={call: response})
    with pytest.raises(module.SmokeError):
        module.run_smoke(
            client, "NCT03361748", "NCI-2020-00001", "NCT02576665", tmp_path
        )


def test_client_runs_the_exact_twelve_operations_and_writes_private_results(
    tmp_path: Path,
) -> None:
    module = _module()
    tmp_path.chmod(0o700)
    client = FakeClient(documents=[{"filename": "Protocol final.pdf"}])

    module.run_smoke(client, "NCT03361748", "NCI-2020-00001", "NCT02576665", tmp_path)

    assert client.calls == [
        ("biomcp", {"command": "biomcp get trial NCT03361748 all", "json": True}),
        (
            "biomcp",
            {
                "command": "biomcp search trial -c melanoma --source ctgov --limit 5",
                "json": True,
            },
        ),
        (
            "biomcp",
            {
                "command": "biomcp get trial NCI-2020-00001 --source nci all",
                "json": True,
            },
        ),
        (
            "biomcp",
            {
                "command": "biomcp search trial -c melanoma --source nci --limit 5",
                "json": True,
            },
        ),
        (
            "get",
            {
                "entity": "trial",
                "id": "NCT03361748",
                "source": "ctgov",
                "sections": ["all"],
                "json": True,
            },
        ),
        (
            "search",
            {
                "entity": "trial",
                "condition": "melanoma",
                "source": "ctgov",
                "limit": 5,
                "json": True,
            },
        ),
        (
            "get",
            {
                "entity": "trial",
                "id": "NCI-2020-00001",
                "source": "nci",
                "sections": ["all"],
                "json": True,
            },
        ),
        (
            "search",
            {
                "entity": "trial",
                "condition": "melanoma",
                "source": "nci",
                "limit": 5,
                "json": True,
            },
        ),
        (
            "biomcp",
            {"command": "biomcp get trial NCT02576665 documents", "json": True},
        ),
        (
            "biomcp",
            {
                "command": "biomcp get trial NCT02576665 document 'Protocol final.pdf'",
                "json": True,
            },
        ),
        ("tools/list", {}),
        (
            "get",
            {
                "entity": "trial",
                "id": "NCT02576665",
                "source": "ctgov",
                "sections": ["documents"],
                "json": True,
            },
        ),
    ]
    assert [
        path.name for path in sorted(tmp_path.glob("*.json"))
    ] == module.OUTPUT_FILES
    assert all((path.stat().st_mode & 0o077) == 0 for path in tmp_path.glob("*.json"))


def test_client_bounds_each_http_operation_and_accepts_sse(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    module = _module()
    observed: dict[str, object] = {}

    class Response:
        status = 200
        headers = {"Content-Type": "text/event-stream"}

        def __enter__(self):
            return self

        def __exit__(self, *_args: object) -> None:
            return None

        def read(self, size: int) -> bytes:
            observed["size"] = size
            return b'data: {"jsonrpc":"2.0","id":"1","result":{"tools":[]}}\n\n'

    def open_request(request: object, *, timeout: int):
        observed["request"] = request
        observed["timeout"] = timeout
        return Response()

    monkeypatch.setattr(module.urllib.request, "urlopen", open_request)
    client = module.McpClient("http://127.0.0.1:8765/mcp")
    response = client.list_tools()

    assert response["result"] == {"tools": []}
    assert observed["timeout"] == 90
    assert observed["size"] == module.MAX_RESPONSE_BYTES + 1
    request = observed["request"]
    assert request.get_header("Mcp-method") == "tools/list"
    assert request.get_header("Mcp-protocol-version") == "2026-07-28"


@pytest.mark.parametrize(
    ("status", "code"),
    [
        (404, -32601),
        (400, -32020),
        (400, -32022),
        (400, -32600),
        (400, -32602),
    ],
)
def test_client_accepts_bounded_http_error_json_rpc_envelope(
    monkeypatch: pytest.MonkeyPatch, status: int, code: int
) -> None:
    module = _module()
    observed: dict[str, object] = {}

    class ErrorBody(io.BytesIO):
        def read(self, size: int = -1) -> bytes:
            observed["size"] = size
            return super().read(size)

    def open_request(request: object, *, timeout: int):
        observed["timeout"] = timeout
        body = ErrorBody(json.dumps(_rpc_error(code) | {"id": "1"}).encode())
        raise module.urllib.error.HTTPError(
            request.full_url, status, "fixture error", {}, body
        )

    monkeypatch.setattr(module.urllib.request, "urlopen", open_request)
    response = module.McpClient("http://127.0.0.1:8765/mcp").list_tools()

    assert response["error"]["code"] == code
    assert observed == {
        "size": module.MAX_RESPONSE_BYTES + 1,
        "timeout": module.OPERATION_TIMEOUT_SECONDS,
    }


@pytest.mark.parametrize(
    ("status", "body", "message"),
    [
        (400, b"x" * (16 * 1024 * 1024 + 1), "byte bound"),
        (400, b"not JSON", "not JSON-RPC"),
        (400, b'{"id":"1","error":{}}', "not a JSON-RPC object"),
        (400, b'{"jsonrpc":"2.0","id":"1","error":{}}', "error was malformed"),
        (
            400,
            b'{"jsonrpc":"2.0","id":"1","result":{}}',
            "status did not match",
        ),
        (
            400,
            b'{"jsonrpc":"2.0","id":"wrong","error":{"code":-32602,"message":"bad"}}',
            "identifier",
        ),
        (
            500,
            b'{"jsonrpc":"2.0","id":"1","error":{"code":-32602,"message":"bad"}}',
            "status did not match",
        ),
        (
            404,
            b'{"jsonrpc":"2.0","id":"1","error":{"code":-32602,"message":"bad"}}',
            "status did not match",
        ),
        (
            400,
            b'{"jsonrpc":"2.0","id":"1","error":{"code":-32601,"message":"bad"}}',
            "status did not match",
        ),
        (
            400,
            b'{"jsonrpc":"2.0","id":"1","error":{"code":-32603,"message":"bad"}}',
            "status did not match",
        ),
    ],
    ids=(
        "oversized",
        "malformed",
        "non-json-rpc",
        "malformed-error",
        "success-envelope",
        "wrong-id",
        "server-error-for-invalid-params",
        "not-found-for-invalid-params",
        "bad-request-for-method-not-found",
        "bad-request-for-unmapped-error",
    ),
)
def test_client_rejects_invalid_http_error_bodies(
    monkeypatch: pytest.MonkeyPatch, status: int, body: bytes, message: str
) -> None:
    module = _module()

    def open_request(request: object, *, timeout: int):
        raise module.urllib.error.HTTPError(
            request.full_url, status, "fixture error", {}, io.BytesIO(body)
        )

    monkeypatch.setattr(module.urllib.request, "urlopen", open_request)
    with pytest.raises(module.SmokeError, match=message):
        module.McpClient("http://127.0.0.1:8765/mcp").list_tools()


def test_client_rejects_non_loopback_urls_and_public_output_permissions(
    tmp_path: Path,
) -> None:
    module = _module()
    with pytest.raises(ValueError, match="loopback"):
        module.McpClient("https://example.org/mcp")
    tmp_path.chmod(0o755)
    with pytest.raises(ValueError, match="private"):
        module.validate_output_directory(tmp_path)


@pytest.mark.parametrize(
    ("call", "response"),
    [
        (1, _tool_response({})),
        (
            1,
            {
                "jsonrpc": "2.0",
                "id": "fixture",
                "result": {"content": [{"type": "text", "text": "not JSON"}]},
            },
        ),
        (1, _tool_response(_detail("NCT00000000", "clinicaltrials.gov"))),
        (
            1,
            _tool_response(
                {
                    key: value
                    for key, value in _detail(
                        "NCT03361748", "clinicaltrials.gov"
                    ).items()
                    if key != "capture"
                }
            ),
        ),
        (
            2,
            _tool_response(
                {
                    key: value
                    for key, value in _search(
                        "NCT03361748", "clinicaltrials.gov"
                    ).items()
                    if key != "pagination"
                }
            ),
        ),
        (
            2,
            _tool_response(
                {
                    **_search("NCT03361748", "clinicaltrials.gov"),
                    "results": [
                        {
                            key: value
                            for key, value in _search(
                                "NCT03361748", "clinicaltrials.gov"
                            )["results"][0].items()
                            if key != "nct_id"
                        }
                    ],
                }
            ),
        ),
        (5, _tool_response(_detail("NCT03361748", "different-authority"))),
        (
            1,
            _tool_response(_detail("NCT03361748", "clinicaltrials.gov"))
            | {"error": {"code": -32603, "message": "internal"}},
        ),
        (
            10,
            _tool_response(
                {"error": {"code": "internal", "message": "wrong failure"}},
                is_error=True,
            ),
        ),
        (12, _rpc_error(-32603)),
    ],
    ids=(
        "empty-success",
        "malformed-success",
        "wrong-identity",
        "missing-contract-field",
        "missing-search-pagination",
        "missing-search-identity",
        "raw-typed-disagreement",
        "mixed-envelope",
        "wrong-raw-rejection",
        "wrong-typed-error-code",
    ),
)
def test_client_rejects_non_evidence_envelopes(
    tmp_path: Path, call: int, response: dict[str, Any]
) -> None:
    _run_with_mutation(tmp_path, call, response)


@pytest.mark.parametrize("call", range(1, 9))
@pytest.mark.parametrize("mutation", ("authority", "identity", "digest"))
def test_raw_and_typed_capture_mutations_fail_symmetrically(
    tmp_path: Path, call: int, mutation: str
) -> None:
    _run_with_mutation(tmp_path, call, _mutated_success(call, mutation))


@pytest.mark.parametrize("call", (2, 4, 6, 8))
@pytest.mark.parametrize(
    "mutation",
    (
        "invalid-precision",
        "exact-reason",
        "exact-null-reason",
        "approximate-reason",
        "unknown-number",
        "unknown-reason",
        "returned-mismatch",
        "over-limit",
        "total-too-small",
        "terminal-more",
        "terminal-null-offset",
        "terminal-null-reason",
        "cursor-missing",
        "offset-mismatch",
        "invalid-status",
    ),
)
def test_raw_and_typed_pagination_contradictions_fail_symmetrically(
    tmp_path: Path, call: int, mutation: str
) -> None:
    _run_with_mutation(tmp_path, call, _mutated_success(call, mutation))
