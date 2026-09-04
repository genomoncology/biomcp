#!/usr/bin/env python3
"""Audit provenance receipts for committed source-test captures."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import parse_qsl, urlsplit

MANIFEST_NAME = "capture-receipts.json"
CLASSIFICATIONS = (
    "real_and_receipted",
    "authored",
    "synthetic_and_ineligible",
    "pending_verification",
)
REQUIRED_RECEIPT_FIELDS = (
    "provider",
    "request",
    "captured_at",
    "sha256",
    "minimization_or_redaction",
    "provider_origin_statement",
)
SHA256_RE = re.compile(r"[0-9a-f]{64}\Z")
RFC3339_UTC_RE = re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z\Z")
TRIAL_ENDPOINTS = {"ctgov", "nci"}
INLINE_CONVERTERS = {
    "from_ctgov_study": "ctgov",
    "from_nci_trial": "nci",
    "from_nci_hit": "nci",
}
LEGACY_EXCEPTION_POLICY = "ticket-1126-legacy-nci-aliases"
LEGACY_EXCEPTION_REASON = (
    "Synthetic unit input pins an accepted legacy alias and does not attest the NCI "
    "wire contract; ticket 1138 owns its code-side disposition."
)
LEGACY_EXCEPTION_KEYS = {
    (
        "src/transform/trial/tests.rs",
        "from_nci_trial_maps_supported_alias_fields:json:1",
        key,
    )
    for key in (
        "nctId",
        "briefTitle",
        "overallStatus",
        "phaseCode",
        "leadSponsor",
        "startDate",
        "completionDate",
        "briefSummary",
    )
} | {
    (
        "src/transform/trial/tests.rs",
        "trial_sections_maps_supported_nci_fields:json:1",
        "phase_code",
    ),
    *{
        (
            "src/transform/trial/tests.rs",
            f"trial_status_normalization_variants:json:{number}",
            key,
        )
        for number, keys in (
            (1, ("nctId", "briefTitle", "status")),
            (2, ("nctId", "briefTitle", "overallStatus")),
        )
        for key in keys
    },
}
UNSAFE_REQUEST_FIELDS = {
    "access_token",
    "api_key",
    "apikey",
    "auth",
    "authorization",
    "awsaccesskeyid",
    "client_secret",
    "credential",
    "key",
    "password",
    "secret",
    "signature",
    "sig",
    "token",
}


def invalid_request(request: object) -> bool:
    if not isinstance(request, str):
        return True
    parsed = urlsplit(request)
    if parsed.scheme not in {"http", "https"} or not parsed.netloc:
        return True
    if parsed.username is not None or parsed.password is not None or parsed.fragment:
        return True
    return any(
        key.lower() in UNSAFE_REQUEST_FIELDS
        or key.lower().startswith(("x-amz-", "x-goog-"))
        for component in (parsed.query, parsed.fragment)
        for key, _ in parse_qsl(component, keep_blank_values=True)
    )


def invalid_utc_timestamp(value: object) -> bool:
    if not isinstance(value, str) or not RFC3339_UTC_RE.fullmatch(value):
        return True
    try:
        parsed = dt.datetime.fromisoformat(value.removesuffix("Z") + "+00:00")
    except ValueError:
        return True
    return parsed.utcoffset() != dt.timedelta(0)


def _rust_literal_end(text: str, start: int) -> int | None:
    raw = re.match(r"(?:br|r)(?P<hashes>#{0,255})\"", text[start:])
    if raw:
        closing = '"' + raw.group("hashes")
        end = text.find(closing, start + raw.end())
        return len(text) if end == -1 else end + len(closing)
    if text[start] == '"':
        quote = '"'
    elif text[start] == "'" and re.match(r"'(?:\\.|[^'\\])'", text[start:]):
        quote = "'"
    else:
        return None
    index = start + 1
    escaped = False
    while index < len(text):
        if escaped:
            escaped = False
        elif text[index] == "\\":
            escaped = True
        elif text[index] == quote:
            return index + 1
        index += 1
    return len(text)


def _mask_rust_comments(text: str) -> str:
    masked = list(text)
    index = 0
    while index < len(text):
        literal_end = (
            _rust_literal_end(text, index)
            if text[index] in {"r", "b", '"', "'"}
            else None
        )
        if literal_end is not None:
            index = literal_end
            continue
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = len(text) if end == -1 else end
            masked[index:end] = " " * (end - index)
            index = end
            continue
        if text.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < len(text) and depth:
                if text.startswith("/*", end):
                    depth += 1
                    end += 2
                elif text.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            for position in range(index, end):
                if masked[position] != "\n":
                    masked[position] = " "
            index = end
            continue
        index += 1
    return "".join(masked)


def _matching_delimiter(text: str, start: int, opener: str, closer: str) -> int:
    depth = 0
    index = start
    while index < len(text):
        char = text[index]
        literal_end = (
            _rust_literal_end(text, index) if char in {"r", "b", '"', "'"} else None
        )
        if literal_end is not None:
            index = literal_end
            continue
        if char == opener:
            depth += 1
        elif char == closer:
            depth -= 1
            if depth == 0:
                return index
        index += 1
    raise ValueError(f"unclosed {opener} at byte {start}")


def _rust_functions(text: str) -> list[tuple[str, str]]:
    functions: list[tuple[str, str]] = []
    for match in re.finditer(
        r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)[^{]*\{", text
    ):
        start = match.end() - 1
        end = _matching_delimiter(text, start, "{", "}")
        functions.append((match.group(1), text[start + 1 : end]))
    return functions


def _json_macro(text: str, start: int) -> tuple[int, int, str] | None:
    match = re.search(r"\bjson!\s*\(", text[start:])
    if not match:
        return None
    macro_start = start + match.start()
    paren = start + match.end() - 1
    end = _matching_delimiter(text, paren, "(", ")")
    return macro_start, end + 1, text[paren + 1 : end]


def _skip_rust_value(text: str, start: int) -> int:
    stack: list[str] = []
    quote: str | None = None
    escaped = False
    pairs = {"(": ")", "[": "]", "{": "}"}
    index = start
    while index < len(text):
        char = text[index]
        if quote:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == quote:
                quote = None
        elif char in {'"', "'"}:
            quote = char
        elif char in pairs:
            stack.append(pairs[char])
        elif stack and char == stack[-1]:
            stack.pop()
        elif not stack and char in ",}]":
            return index
        index += 1
    return index


def _rust_json_paths(source: str) -> set[str]:
    """Read object-key paths from the JSON-shaped subset accepted by json!()."""
    paths: set[str] = set()

    def whitespace(index: int) -> int:
        while index < len(source) and source[index].isspace():
            index += 1
        return index

    def string(index: int) -> tuple[str, int]:
        end = index + 1
        escaped = False
        while end < len(source):
            if escaped:
                escaped = False
            elif source[end] == "\\":
                escaped = True
            elif source[end] == '"':
                return json.loads(source[index : end + 1]), end + 1
            end += 1
        raise ValueError("unclosed JSON key string in Rust fixture")

    def value(index: int, prefix: str) -> int:
        index = whitespace(index)
        if index < len(source) and source[index] == "{":
            return object_value(index, prefix)
        if index < len(source) and source[index] == "[":
            index += 1
            while True:
                index = whitespace(index)
                if index >= len(source) or source[index] == "]":
                    return index + 1
                index = value(index, f"{prefix}[]")
                index = whitespace(index)
                if index < len(source) and source[index] == ",":
                    index += 1
                    continue
                if index < len(source) and source[index] == "]":
                    return index + 1
                return _skip_rust_value(source, index)
        return _skip_rust_value(source, index)

    def object_value(index: int, prefix: str) -> int:
        index += 1
        while True:
            index = whitespace(index)
            if index >= len(source) or source[index] == "}":
                return index + 1
            if source[index] == '"':
                key, index = string(index)
            else:
                match = re.match(r"[A-Za-z_][A-Za-z0-9_]*", source[index:])
                if not match:
                    return _skip_rust_value(source, index)
                key = match.group(0)
                index += len(key)
            index = whitespace(index)
            if index >= len(source) or source[index] != ":":
                return _skip_rust_value(source, index)
            path = f"{prefix}.{key}" if prefix else key
            paths.add(path)
            index = value(index + 1, path)
            index = whitespace(index)
            if index < len(source) and source[index] == ",":
                index += 1

    value(0, "")
    return paths


def _inline_fixtures(repo_root: Path) -> tuple[list[dict[str, object]], list[str]]:
    trial_dir = repo_root / "src" / "transform" / "trial"
    files = [trial_dir / "tests.rs", *sorted((trial_dir / "tests").glob("*.rs"))]
    fixtures: list[dict[str, object]] = []
    errors: list[str] = []
    for path in files:
        if not path.is_file():
            continue
        text = _mask_rust_comments(path.read_text(encoding="utf-8"))
        for function, body in _rust_functions(text):
            candidates: dict[int, tuple[int, str]] = {}
            for converter, endpoint in INLINE_CONVERTERS.items():
                for call in re.finditer(rf"\b{converter}\s*\(", body):
                    paren = call.end() - 1
                    call_end = _matching_delimiter(body, paren, "(", ")")
                    argument = body[paren + 1 : call_end]
                    direct = _json_macro(argument, 0)
                    if (
                        direct
                        and re.fullmatch(r"\s*&?\s*", argument[: direct[0]])
                        and not argument[direct[1] :].strip()
                    ):
                        absolute = paren + 1 + direct[0]
                        candidates[absolute] = (paren + 1 + direct[1], endpoint)
                        continue
                    variable = re.fullmatch(
                        r"\s*&?\s*([A-Za-z_][A-Za-z0-9_]*)\s*", argument
                    )
                    if not variable:
                        errors.append(
                            f"{path.relative_to(repo_root)}:{function}: unsupported argument into {converter}"
                        )
                        continue
                    name = variable.group(1)
                    assignments = [
                        *re.finditer(
                            rf"\blet(?:\s+mut)?\s+{re.escape(name)}(?:\s*:[^=;]+)?\s*=(?![=>])",
                            body[: call.start()],
                        ),
                        *re.finditer(
                            rf"(?<![A-Za-z0-9_]){re.escape(name)}\s*=(?![=>])",
                            body[: call.start()],
                        ),
                    ]
                    if not assignments:
                        errors.append(
                            f"{path.relative_to(repo_root)}:{function}: unsupported assignment flow into {converter}"
                        )
                        continue
                    assignment = max(assignments, key=lambda item: item.start())
                    assignment_end = body.find(";", assignment.end())
                    macro = _json_macro(body, assignment.end())
                    if (
                        macro
                        and macro[0] < call.start()
                        and assignment_end != -1
                        and macro[0] < assignment_end
                    ):
                        candidates[macro[0]] = (macro[1], endpoint)
                    elif "testdata/sources/" not in body:
                        errors.append(
                            f"{path.relative_to(repo_root)}:{function}: unsupported assignment flow into {converter}"
                        )
            for number, (start, (end, endpoint)) in enumerate(
                sorted(candidates.items()), 1
            ):
                fixtures.append(
                    {
                        "path": path.relative_to(repo_root).as_posix(),
                        "selector": f"{function}:json:{number}",
                        "endpoint": endpoint,
                        "paths": set(),
                        "source": body[start:end],
                    }
                )
    for fixture in fixtures:
        macro = _json_macro(str(fixture.pop("source")), 0)
        if not macro:
            raise ValueError(
                f"cannot parse inline fixture {fixture['path']}:{fixture['selector']}"
            )
        fixture["paths"] = _rust_json_paths(macro[2])
    return fixtures, sorted(set(errors))


def _schema_paths(nodes: object, prefix: str = "") -> set[str]:
    paths: set[str] = set()
    if not isinstance(nodes, list):
        return paths
    for node in nodes:
        if not isinstance(node, dict) or not isinstance(node.get("name"), str):
            continue
        name = node["name"]
        path = f"{prefix}.{name}" if prefix else name
        paths.add(path)
        child_prefix = f"{path}[]" if str(node.get("type", "")).endswith("[]") else path
        paths.update(_schema_paths(node.get("children"), child_prefix))
    return paths


def _json_paths(value: object, prefix: str = "") -> set[str]:
    paths: set[str] = set()
    if isinstance(value, dict):
        for key, child in value.items():
            path = f"{prefix}.{key}" if prefix else key
            paths.add(path)
            paths.update(_json_paths(child, path))
    elif isinstance(value, list):
        for child in value:
            paths.update(_json_paths(child, f"{prefix}[]"))
    return paths


def _select_records(value: object, selector: str) -> list[object]:
    if selector == "/":
        return [value]
    values = [value]
    for raw in selector.strip("/").split("/"):
        token = raw.replace("~1", "/").replace("~0", "~")
        selected: list[object] = []
        for current in values:
            if token == "*" and isinstance(current, list):
                selected.extend(current)
            elif isinstance(current, dict) and token in current:
                selected.append(current[token])
        values = selected
    return values


def _rust_string_values(text: str) -> list[str]:
    values: list[str] = []
    pattern = re.compile(
        r'(?:br|r)(?P<hashes>#{0,255})"(?P<raw>.*?)"(?P=hashes)|"(?:\\.|[^"\\])*"',
        re.DOTALL,
    )
    for match in pattern.finditer(text):
        token = match.group(0)
        if token.startswith(("r", "br")):
            first_quote = token.find('"')
            hashes = token[:first_quote].removeprefix("br").removeprefix("r")
            values.append(token[first_quote + 1 : -1 - len(hashes)])
        else:
            try:
                values.append(json.loads(token))
            except json.JSONDecodeError:
                continue
    return values


def _consumed_trial_files(repo_root: Path) -> tuple[set[str], list[str]]:
    consumed: set[str] = set()
    errors: list[str] = []
    for path in (repo_root / "src").rglob("*.rs"):
        if any(part in {"target", ".git"} for part in path.parts):
            continue
        text = _mask_rust_comments(path.read_text(encoding="utf-8"))
        file_literals = "".join(_rust_string_values(text))
        for directory in ("clinicaltrials", "nci_cts"):
            marker = f"testdata/sources/{directory}/"
            for match in re.finditer(
                rf"testdata/sources/{directory}/([A-Za-z0-9_.-]+\.json)", text
            ):
                consumed.add(f"{directory}/{match.group(1)}")
            literal_fixture_macro = False
            if f"/{marker}" in text:
                fixture_calls = list(re.finditer(r"\bfixture!\s*\(([^)]*)\)", text))
                literal_fixture_macro = bool(fixture_calls)
                for match in fixture_calls:
                    literal = re.fullmatch(
                        r'\s*"([A-Za-z0-9_.-]+\.json)"\s*', match.group(1)
                    )
                    if literal:
                        consumed.add(f"{directory}/{literal.group(1)}")
                    else:
                        literal_fixture_macro = False
                        errors.append(
                            f"{path.relative_to(repo_root)}: dynamic fixture! reference is unsupported for {directory}"
                        )
            for match in re.finditer(r"\binclude_(?:str|bytes)!\s*\(", text):
                end = _matching_delimiter(text, match.end() - 1, "(", ")")
                argument = text[match.end() : end]
                assembled = "".join(_rust_string_values(argument))
                concrete = re.search(
                    rf"{re.escape(marker)}([A-Za-z0-9_.-]+\.json)", assembled
                )
                closed_macro_indirection = bool(
                    marker in assembled
                    and literal_fixture_macro
                    and re.search(r"\$[A-Za-z_][A-Za-z0-9_]*", argument)
                )
                if concrete:
                    consumed.add(f"{directory}/{concrete.group(1)}")
                elif marker in assembled and not closed_macro_indirection:
                    errors.append(
                        f"{path.relative_to(repo_root)}: dynamic include fixture reference is unsupported for {directory}"
                    )
                elif (
                    "concat!" in argument
                    and marker in file_literals
                    and not closed_macro_indirection
                ):
                    errors.append(
                        f"{path.relative_to(repo_root)}: dynamic include fixture reference is unsupported for {directory}"
                    )
    return consumed, errors


def _audit_fixture_keys(
    root: Path, manifest: dict[str, object]
) -> tuple[int, int, list[str]]:
    repo_root = root.parent.parent
    contract = manifest.get("fixture_key_contract")
    if not isinstance(contract, dict):
        return 0, 0, ["manifest requires fixture_key_contract object"]
    attestors = contract.get("attestors")
    on_disk = contract.get("on_disk")
    inline = contract.get("inline")
    exceptions = contract.get("exceptions")
    if not all(
        isinstance(item, list) for item in (attestors, on_disk, inline, exceptions)
    ):
        return (
            0,
            0,
            [
                "fixture_key_contract requires attestors, on_disk, inline, and exceptions arrays"
            ],
        )

    allowed: dict[str, set[str]] = {}
    endpoints: dict[str, str] = {}
    receipt_entries = {
        entry.get("path"): entry
        for entry in manifest.get("entries", [])
        if isinstance(entry, dict)
    }
    for attestor in attestors:
        if (
            not isinstance(attestor, dict)
            or attestor.get("endpoint") not in TRIAL_ENDPOINTS
        ):
            return 0, 0, ["invalid fixture-key attestor"]
        endpoint = str(attestor["endpoint"])
        if endpoint in allowed:
            return 0, 0, [f"duplicate fixture-key attestor for {endpoint}"]
        relative_path = str(attestor.get("path", ""))
        path = root / relative_path
        if (
            receipt_entries.get(relative_path, {}).get("classification")
            != "real_and_receipted"
        ):
            return (
                0,
                0,
                [f"{endpoint}: attestor {relative_path} must be real_and_receipted"],
            )
        try:
            body = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            return 0, 0, [f"{endpoint}: cannot read attestor {path}: {error}"]
        endpoints[endpoint] = str(attestor.get("label", endpoint))
        if attestor.get("kind") == "ctgov_schema" and endpoint == "ctgov":
            allowed[endpoint] = _schema_paths(body)
        elif attestor.get("kind") == "nci_top_level_capture" and endpoint == "nci":
            if (
                not isinstance(attestor.get("limitation"), str)
                or not attestor["limitation"]
            ):
                return 0, 0, ["nci: top-level attestor requires an explicit limitation"]
            records = _select_records(body, str(attestor.get("selector", "/data/*")))
            allowed[endpoint] = {
                key for record in records if isinstance(record, dict) for key in record
            }
        else:
            return 0, 0, [f"{endpoint}: unsupported fixture-key attestor kind"]
    if set(allowed) != TRIAL_ENDPOINTS:
        return 0, 0, ["fixture-key contract requires one CTGov and one NCI attestor"]

    errors: list[str] = []
    exception_map: dict[tuple[str, str, str], str] = {}
    for exception in exceptions:
        required = ("path", "selector", "checked_path", "reason")
        if not isinstance(exception, dict) or any(
            not isinstance(exception.get(field), str) or not exception[field]
            for field in required
        ):
            errors.append(
                "fixture-key exception requires path, selector, checked_path, and reason"
            )
            continue
        key = (
            str(exception.get("path")),
            str(exception.get("selector")),
            str(exception.get("checked_path")),
        )
        if key in exception_map:
            errors.append(f"duplicate fixture-key exception: {':'.join(key)}")
        exception_map[key] = exception["reason"]
    policy = contract.get("exception_policy")
    if exception_map:
        if policy != LEGACY_EXCEPTION_POLICY:
            errors.append(
                f"fixture-key exceptions require policy {LEGACY_EXCEPTION_POLICY}"
            )
        if set(exception_map) != LEGACY_EXCEPTION_KEYS:
            errors.append(
                "fixture-key exceptions differ from the authorized ticket 1126 set"
            )
        for key, reason in exception_map.items():
            if reason != LEGACY_EXCEPTION_REASON:
                errors.append(
                    f"fixture-key exception has unauthorized reason: {':'.join(key)}"
                )
    elif policy is not None:
        errors.append("fixture-key exception policy is present without exceptions")

    declared_disk: set[str] = set()
    seen_disk_declarations: set[tuple[str, str, str]] = set()
    fixtures: list[dict[str, object]] = []
    for declaration in on_disk:
        if not isinstance(declaration, dict):
            errors.append("invalid on-disk fixture declaration")
            continue
        relpath = str(declaration.get("path"))
        selector = str(declaration.get("selector"))
        endpoint = str(declaration.get("endpoint"))
        if endpoint not in TRIAL_ENDPOINTS or not selector.startswith("/"):
            errors.append(f"{relpath}: invalid on-disk endpoint or selector {selector}")
            continue
        declaration_key = (relpath, selector, endpoint)
        if declaration_key in seen_disk_declarations:
            errors.append(
                f"duplicate on-disk fixture declaration: {':'.join(declaration_key)}"
            )
            continue
        seen_disk_declarations.add(declaration_key)
        declared_disk.add(relpath)
        try:
            body = json.loads((root / relpath).read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            errors.append(f"{relpath}: cannot read declared fixture: {error}")
            continue
        records = _select_records(body, selector)
        if not records:
            errors.append(f"{relpath}: selector {selector} selected no records")
        for record in records:
            if not isinstance(record, dict):
                errors.append(
                    f"{relpath}: selector {selector} selected a non-object trial record"
                )
                continue
            paths = _json_paths(record)
            fixtures.append(
                {
                    "path": relpath,
                    "selector": selector,
                    "endpoint": endpoint,
                    "paths": paths,
                }
            )

    consumed, discovery_errors = _consumed_trial_files(repo_root)
    errors.extend(discovery_errors)
    for path in sorted(consumed - declared_disk):
        errors.append(f"{path}: consumed trial fixture is undeclared")
    for path in sorted(declared_disk - consumed):
        errors.append(f"{path}: declared trial fixture is not consumed")

    discovered_inline, inline_errors = _inline_fixtures(repo_root)
    errors.extend(inline_errors)
    declared_inline: set[tuple[str, str, str]] = set()
    for item in inline:
        if not isinstance(item, dict) or any(
            not isinstance(item.get(field), str) or not item[field]
            for field in ("path", "selector", "endpoint")
        ):
            errors.append("invalid inline fixture declaration")
            continue
        declaration = (item["path"], item["selector"], item["endpoint"])
        if declaration in declared_inline:
            errors.append(
                f"duplicate inline fixture declaration: {':'.join(declaration)}"
            )
        declared_inline.add(declaration)
    found_inline = {
        (str(item["path"]), str(item["selector"]), str(item["endpoint"]))
        for item in discovered_inline
    }
    for path, selector, endpoint in sorted(found_inline - declared_inline):
        errors.append(
            f"{path}:{selector}: inline fixture is undeclared for {endpoints.get(endpoint, endpoint)}"
        )
    for path, selector, endpoint in sorted(declared_inline - found_inline):
        errors.append(
            f"{path}:{selector}: declared inline fixture was not discovered for {endpoints.get(endpoint, endpoint)}"
        )
    fixtures.extend(discovered_inline)

    checked = 0
    used_exceptions: set[tuple[str, str, str]] = set()
    for fixture in fixtures:
        endpoint = str(fixture["endpoint"])
        fixture_allowed = allowed.get(endpoint)
        if fixture_allowed is None:
            errors.append(
                f"{fixture['path']}:{fixture['selector']}: unknown endpoint {endpoint}"
            )
            continue
        for checked_path in sorted(fixture["paths"]):
            checked += 1
            compared_path = (
                re.split(r"[.[]", checked_path, maxsplit=1)[0]
                if endpoint == "nci"
                else checked_path
            )
            if compared_path in fixture_allowed:
                continue
            exception_key = (
                str(fixture["path"]),
                str(fixture["selector"]),
                compared_path,
            )
            if exception_key in exception_map:
                used_exceptions.add(exception_key)
                continue
            errors.append(
                f"{fixture['path']}:{fixture['selector']}: unattested path {compared_path} for {endpoints.get(endpoint, endpoint)}"
            )
    for exception_key in sorted(set(exception_map) - used_exceptions):
        errors.append(f"unused fixture-key exception: {':'.join(exception_key)}")
    return checked, len(used_exceptions), errors


def audit(root: Path) -> dict[str, object]:
    manifest_path = root / MANIFEST_NAME
    errors: list[str] = []
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read {MANIFEST_NAME}: {error}") from error

    entries = manifest.get("entries") if isinstance(manifest, dict) else None
    if (
        not isinstance(manifest, dict)
        or manifest.get("schema_version") != 1
        or not isinstance(entries, list)
    ):
        raise ValueError("manifest requires schema_version 1 and entries array")

    files = sorted(
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path != manifest_path
    )
    discovered = set(files)
    by_path: dict[str, dict[str, object]] = {}
    classifications = dict.fromkeys(CLASSIFICATIONS, 0)
    for entry in entries:
        if not isinstance(entry, dict) or not isinstance(entry.get("path"), str):
            errors.append("entry requires path")
            continue
        path = entry["path"]
        if path in by_path:
            errors.append(f"duplicate entry: {path}")
            continue
        by_path[path] = entry

        classification = entry.get("classification")
        if classification not in classifications:
            errors.append(f"{path}: invalid classification")
            continue
        classifications[classification] += 1
        receipt = entry.get("receipt")
        if classification == "real_and_receipted":
            if not isinstance(receipt, dict):
                errors.append(f"{path}: receipt required")
                continue
            for field in REQUIRED_RECEIPT_FIELDS:
                if not isinstance(receipt.get(field), str) or not receipt[field]:
                    errors.append(f"{path}: receipt {field} required")
            if invalid_request(receipt.get("request")):
                errors.append(f"{path}: receipt request is unsafe")
            if invalid_utc_timestamp(receipt.get("captured_at")):
                errors.append(f"{path}: receipt captured_at must be RFC3339 UTC")
            digest = receipt.get("sha256")
            if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
                errors.append(f"{path}: receipt sha256 must be lowercase SHA-256")
            elif path in discovered:
                actual = hashlib.sha256((root / path).read_bytes()).hexdigest()
                if digest != actual:
                    errors.append(f"{path}: receipt sha256 does not match raw bytes")
        elif classification == "authored":
            if (
                not isinstance(entry.get("authored_reason"), str)
                or not entry["authored_reason"]
            ):
                errors.append(f"{path}: authored_reason required")
            if receipt is not None:
                errors.append(f"{path}: authored fixture cannot carry receipt")
        else:
            if (
                not isinstance(entry.get("ineligible_reason"), str)
                or not entry["ineligible_reason"]
            ):
                errors.append(f"{path}: ineligible_reason required")
            if receipt is not None:
                errors.append(f"{path}: non-real classification cannot carry receipt")

    missing = sorted(discovered - set(by_path))
    orphaned = sorted(set(by_path) - discovered)
    errors.extend(f"missing entry: {path}" for path in missing)
    errors.extend(f"orphan entry: {path}" for path in orphaned)
    if errors:
        raise ValueError("\n".join(errors))

    fixture_keys_checked, fixture_key_exceptions, fixture_errors = _audit_fixture_keys(
        root, manifest
    )
    if fixture_errors:
        raise ValueError("\n".join(fixture_errors))

    corrections = manifest.get("historical_corrections")
    if not isinstance(corrections, list):
        raise ValueError("historical_corrections must be an array")
    return {
        "audited_files": len(files),
        "classified_files": len(by_path),
        "classifications": classifications,
        "fixture_keys_checked": fixture_keys_checked,
        "fixture_key_exceptions": fixture_key_exceptions,
        "confirmed_byte_unfaithful": manifest.get("confirmed_byte_unfaithful"),
        "historical_corrections": corrections,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    try:
        report = audit(args.root)
    except ValueError as error:
        print(error, file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps(report, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
