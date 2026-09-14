#!/usr/bin/env python3
"""Run the bounded live clinical-trial MCP checklist against one candidate."""

from __future__ import annotations

import ipaddress
import json
import os
from pathlib import Path
import re
import shlex
import stat
import sys
from typing import Any
import urllib.error
import urllib.parse
import urllib.request

PROTOCOL_VERSION = "2026-07-28"
OPERATION_TIMEOUT_SECONDS = 90
MAX_RESPONSE_BYTES = 16 * 1024 * 1024
META = {
    "io.modelcontextprotocol/protocolVersion": PROTOCOL_VERSION,
    "io.modelcontextprotocol/clientCapabilities": {},
    "io.modelcontextprotocol/clientInfo": {
        "name": "clinical-trial-live-smoke",
        "version": "1",
    },
}
OUTPUT_FILES = [
    "01-raw-ctgov-detail.json",
    "02-raw-ctgov-search.json",
    "03-raw-nci-detail.json",
    "04-raw-nci-search.json",
    "05-typed-ctgov-detail.json",
    "06-typed-ctgov-search.json",
    "07-typed-nci-detail.json",
    "08-typed-nci-search.json",
    "09-raw-ctgov-documents.json",
    "10-raw-ctgov-document-rejection.json",
    "11-typed-get-schema.json",
    "12-typed-documents-rejection.json",
]
SAFE_ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:-]{0,127}\Z")


class SmokeError(RuntimeError):
    """A sanitized live-smoke contract failure."""


def _validate_mcp_url(value: str) -> str:
    parsed = urllib.parse.urlsplit(value)
    try:
        address = ipaddress.ip_address(parsed.hostname or "")
        loopback = address.is_loopback
    except ValueError:
        loopback = parsed.hostname == "localhost"
    if (
        parsed.scheme != "http"
        or not loopback
        or parsed.port is None
        or parsed.path != "/mcp"
        or parsed.username is not None
        or parsed.password is not None
        or parsed.query
        or parsed.fragment
    ):
        raise ValueError("MCP URL must be an exact loopback HTTP /mcp endpoint")
    return value


def validate_output_directory(path: Path) -> Path:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValueError("private output directory is unavailable") from error
    if (
        not stat.S_ISDIR(metadata.st_mode)
        or path.is_symlink()
        or metadata.st_uid != os.getuid()
        or stat.S_IMODE(metadata.st_mode) & 0o077
    ):
        raise ValueError("output directory must be a private owner-only directory")
    return path


def _validate_public_id(value: str, label: str) -> str:
    if not SAFE_ID.fullmatch(value):
        raise ValueError(f"{label} must be a bounded public trial identifier")
    return value


def _parse_response(body: bytes) -> dict[str, Any]:
    try:
        text = body.decode("utf-8")
    except UnicodeDecodeError as error:
        raise SmokeError("MCP response was not UTF-8") from error
    candidates = [
        line.removeprefix("data: ")
        for line in text.splitlines()
        if line.startswith("data: ")
    ]
    encoded = candidates[0] if candidates else text
    try:
        response = json.loads(encoded)
    except json.JSONDecodeError as error:
        raise SmokeError("MCP response was not JSON-RPC") from error
    if not isinstance(response, dict) or response.get("jsonrpc") != "2.0":
        raise SmokeError("MCP response was not a JSON-RPC object")
    return response


class McpClient:
    def __init__(self, url: str) -> None:
        self.url = _validate_mcp_url(url)
        self.next_id = 1

    def _request(
        self,
        method: str,
        params: dict[str, object],
        *,
        tool_name: str | None = None,
    ) -> dict[str, Any]:
        request_id = str(self.next_id)
        self.next_id += 1
        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": {"_meta": META, **params},
        }
        headers = {
            "Accept": "application/json, text/event-stream",
            "Content-Type": "application/json",
            "MCP-Protocol-Version": PROTOCOL_VERSION,
            "Mcp-Method": method,
        }
        if tool_name is not None:
            headers["Mcp-Name"] = tool_name
        request = urllib.request.Request(
            self.url,
            data=json.dumps(payload, separators=(",", ":")).encode(),
            headers=headers,
            method="POST",
        )
        try:
            with urllib.request.urlopen(
                request, timeout=OPERATION_TIMEOUT_SECONDS
            ) as response:
                body = response.read(MAX_RESPONSE_BYTES + 1)
                status_code = response.status
        except urllib.error.HTTPError as error:
            error.read(MAX_RESPONSE_BYTES + 1)
            raise SmokeError(
                f"MCP operation returned HTTP status {error.code}"
            ) from error
        except (urllib.error.URLError, TimeoutError) as error:
            raise SmokeError(
                "MCP operation did not complete within its network bound"
            ) from error
        if status_code != 200:
            raise SmokeError(f"MCP operation returned HTTP status {status_code}")
        if len(body) > MAX_RESPONSE_BYTES:
            raise SmokeError("MCP response exceeded the byte bound")
        result = _parse_response(body)
        if result.get("id") != request_id:
            raise SmokeError("MCP response identifier did not match its request")
        return result

    def call_tool(self, name: str, arguments: dict[str, object]) -> dict[str, Any]:
        return self._request(
            "tools/call",
            {"name": name, "arguments": arguments},
            tool_name=name,
        )

    def list_tools(self) -> dict[str, Any]:
        return self._request("tools/list", {})


def _result(response: dict[str, Any]) -> dict[str, Any] | None:
    result = response.get("result")
    return result if isinstance(result, dict) else None


def _expect_success(response: dict[str, Any], label: str) -> None:
    result = _result(response)
    if "error" in response or result is None or result.get("isError") is True:
        raise SmokeError(f"{label} did not return a successful MCP tool result")


def _expect_rejection(response: dict[str, Any], label: str) -> None:
    result = _result(response)
    if "error" not in response and (
        result is None or result.get("isError") is not True
    ):
        raise SmokeError(f"{label} did not return the required rejection")


def _tool_json(response: dict[str, Any], label: str) -> dict[str, Any]:
    result = _result(response)
    content = result.get("content") if result is not None else None
    if not isinstance(content, list):
        raise SmokeError(f"{label} did not contain MCP text content")
    for item in content:
        if not isinstance(item, dict) or item.get("type") != "text":
            continue
        text = item.get("text")
        if not isinstance(text, str):
            continue
        try:
            payload = json.loads(text)
        except json.JSONDecodeError:
            continue
        if isinstance(payload, dict):
            return payload
    raise SmokeError(f"{label} did not contain a JSON object")


def _advertised_filename(response: dict[str, Any]) -> str | None:
    payload = _tool_json(response, "document manifest")
    documents = payload.get("documents")
    if documents is None:
        return None
    if not isinstance(documents, list):
        raise SmokeError("document manifest did not contain a documents array")
    for document in documents:
        if not isinstance(document, dict):
            raise SmokeError("document manifest contained a malformed entry")
        filename = document.get("filename")
        if (
            isinstance(filename, str)
            and filename
            and not any(character in filename for character in "\x00\r\n")
        ):
            return filename
    return None


def _walk_dicts(value: object):
    if isinstance(value, dict):
        yield value
        for child in value.values():
            yield from _walk_dicts(child)
    elif isinstance(value, list):
        for child in value:
            yield from _walk_dicts(child)


def _validate_typed_get_schema(response: dict[str, Any]) -> None:
    result = _result(response)
    tools = result.get("tools") if result is not None else None
    if not isinstance(tools, list):
        raise SmokeError("tools/list did not return a tool array")
    get_tool = next(
        (
            tool
            for tool in tools
            if isinstance(tool, dict) and tool.get("name") == "get"
        ),
        None,
    )
    if get_tool is None:
        raise SmokeError("tools/list did not expose typed get")
    schema = get_tool.get("inputSchema")
    trial_branches = [
        node
        for node in _walk_dicts(schema)
        if isinstance(node.get("properties"), dict)
        and node["properties"].get("entity", {}).get("const") == "trial"
    ]
    if not trial_branches:
        raise SmokeError("typed get schema did not expose a trial branch")
    trial = trial_branches[0]
    sections = trial["properties"].get("sections", {})
    values = sections.get("items", {}).get("enum")
    if not isinstance(values, list) or {"document", "documents"} & set(values):
        raise SmokeError("typed get schema exposed a document operation")
    if any("filename" in node for node in _walk_dicts(trial)):
        raise SmokeError("typed get schema exposed a binary-document filename")


def _write_result(directory: Path, index: int, response: dict[str, Any]) -> None:
    path = directory / OUTPUT_FILES[index]
    descriptor = os.open(
        path,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
        0o600,
    )
    with os.fdopen(descriptor, "w", encoding="utf-8") as output:
        json.dump(response, output, ensure_ascii=False, indent=2, sort_keys=True)
        output.write("\n")


def run_smoke(
    client: McpClient,
    ctgov_trial_id: str,
    nci_trial_id: str,
    document_trial_id: str,
    output_directory: Path,
) -> None:
    output = validate_output_directory(output_directory)
    successful_calls = [
        ("biomcp", {"command": f"biomcp get trial {ctgov_trial_id} all", "json": True}),
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
                "command": f"biomcp get trial {nci_trial_id} --source nci all",
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
                "id": ctgov_trial_id,
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
                "id": nci_trial_id,
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
            {
                "command": f"biomcp get trial {document_trial_id} documents",
                "json": True,
            },
        ),
    ]
    responses: list[dict[str, Any]] = []
    for index, (name, arguments) in enumerate(successful_calls):
        response = client.call_tool(name, arguments)
        _expect_success(response, OUTPUT_FILES[index])
        _write_result(output, index, response)
        responses.append(response)

    filename = _advertised_filename(responses[-1]) or "not-advertised"
    document_rejection = client.call_tool(
        "biomcp",
        {
            "command": (
                f"biomcp get trial {document_trial_id} document {shlex.quote(filename)}"
            ),
            "json": True,
        },
    )
    _expect_rejection(document_rejection, "raw binary document call")
    _write_result(output, 9, document_rejection)

    tools = client.list_tools()
    _expect_success(tools, "tools/list")
    _validate_typed_get_schema(tools)
    _write_result(output, 10, tools)

    typed_rejection = client.call_tool(
        "get",
        {
            "entity": "trial",
            "id": document_trial_id,
            "source": "ctgov",
            "sections": ["documents"],
            "json": True,
        },
    )
    _expect_rejection(typed_rejection, "typed documents call")
    _write_result(output, 11, typed_rejection)


def main(argv: list[str]) -> int:
    if len(argv) != 5:
        print(
            "usage: clinical-trial-live-mcp-smoke.py MCP_URL CTGOV_ID NCI_ID DOCUMENT_TRIAL_ID OUTPUT_DIR",
            file=sys.stderr,
        )
        return 2
    try:
        client = McpClient(argv[0])
        ctgov_id = _validate_public_id(argv[1], "ClinicalTrials.gov ID")
        nci_id = _validate_public_id(argv[2], "NCI ID")
        document_id = _validate_public_id(argv[3], "document trial ID")
        output = validate_output_directory(Path(argv[4]))
        run_smoke(client, ctgov_id, nci_id, document_id, output)
    except (SmokeError, ValueError) as error:
        print(f"clinical-trial MCP smoke failed: {error}", file=sys.stderr)
        return 1
    except OSError:
        print(
            "clinical-trial MCP smoke failed: protected output write failed",
            file=sys.stderr,
        )
        return 1
    print("12 clinical-trial MCP operations completed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
