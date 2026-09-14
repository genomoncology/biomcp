from __future__ import annotations

import importlib.util
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


class FakeClient:
    def __init__(self, *, documents: list[dict[str, object]]) -> None:
        self.documents = documents
        self.calls: list[tuple[str, dict[str, object]]] = []

    def call_tool(self, name: str, arguments: dict[str, object]) -> dict[str, Any]:
        self.calls.append((name, arguments))
        if len(self.calls) == 9:
            return _tool_response({"documents": self.documents})
        if len(self.calls) in {10, 12}:
            return _tool_response({"code": "invalid_argument"}, is_error=True)
        return _tool_response({"ok": True})

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


def test_client_rejects_non_loopback_urls_and_public_output_permissions(
    tmp_path: Path,
) -> None:
    module = _module()
    with pytest.raises(ValueError, match="loopback"):
        module.McpClient("https://example.org/mcp")
    tmp_path.chmod(0o755)
    with pytest.raises(ValueError, match="private"):
        module.validate_output_directory(tmp_path)
