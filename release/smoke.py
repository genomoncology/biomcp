#!/usr/bin/env python3
"""Bounded no-provider release executable smoke."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


class SmokeError(ValueError):
    pass


def request(process: subprocess.Popen[str], value: dict) -> dict:
    assert process.stdin is not None and process.stdout is not None
    process.stdin.write(json.dumps(value) + "\n")
    process.stdin.flush()
    line = process.stdout.readline()
    if not line:
        raise SmokeError("MCP server closed before response")
    return json.loads(line)


def smoke(binary: Path, expected_version: str, expected_revision: str) -> None:
    for arguments, expected_success in (
        (["--version"], True),
        (["--help"], True),
        (["--json", "list"], True),
        (["--json", "not-a-command"], False),
    ):
        result = subprocess.run([binary, *arguments], text=True, capture_output=True, check=False)
        if (result.returncode == 0) != expected_success:
            raise SmokeError(f"unexpected exit for {' '.join(arguments)}")
        if arguments[0] == "--json":
            json.loads(result.stdout)
    version = subprocess.run(
        [binary, "--json", "version"], text=True, capture_output=True, check=True
    ).stdout
    if expected_version not in version or expected_revision not in version:
        raise SmokeError("release identity mismatch")
    process = subprocess.Popen(
        [binary, "serve"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        initialized = request(
            process,
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-03-26",
                    "capabilities": {},
                    "clientInfo": {"name": "release-smoke", "version": "1"},
                },
            },
        )
        if "result" not in initialized:
            raise SmokeError("MCP initialize failed")
        assert process.stdin is not None
        process.stdin.write(
            json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
            + "\n"
        )
        process.stdin.flush()
        tools = request(
            process, {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}
        )
        if len(tools.get("result", {}).get("tools", [])) != 7:
            raise SmokeError("release MCP catalog is not seven tools")
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bin", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    try:
        smoke(args.bin, args.version, args.revision)
        return 0
    except (OSError, SmokeError, json.JSONDecodeError, subprocess.SubprocessError) as error:
        print(f"release smoke: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
