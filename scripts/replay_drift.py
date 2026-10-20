"""Compare observed JSON key/type unions or XML element paths, ignoring values."""

from __future__ import annotations

import json
import tomllib
from pathlib import Path
from urllib.parse import parse_qsl, urlsplit
from xml.etree import ElementTree

ID_ALIASES = {
    "ctgov": "clinicaltrials-gov",
    "s2": "semantic-scholar",
    "pubtator": "pubtator3",
    "mygene": "mygene-info",
    "mychem": "mychem-info",
    "myvariant": "myvariant-info",
    "mydisease": "mydisease-info",
    "gwas": "gwas-catalog",
    "europepmc": "europe-pmc",
}


def recording_provider(key: str, entry: dict, routes: dict[str, str]) -> str:
    """Resolve exact origin-path recordings from their declared capture provider."""
    method, raw_path = key.split(" ", 1)
    path = urlsplit(raw_path)
    prefix = path.path.lstrip("/").split("/", 1)[0]
    if prefix in routes:
        return prefix
    provider = entry.get("recording_provenance", {}).get("provider")
    if method not in {"GET", "HEAD"} or provider not in routes:
        return prefix
    upstream = urlsplit(entry.get("upstream", ""))
    base = urlsplit(routes[provider])
    if (
        (upstream.scheme, upstream.netloc) == (base.scheme, base.netloc)
        and (upstream.path == base.path or upstream.path.startswith(base.path.rstrip("/") + "/"))
        and upstream.path == path.path
        and not path.fragment
        and sorted(parse_qsl(upstream.query, keep_blank_values=True))
        == sorted(parse_qsl(path.query, keep_blank_values=True))
    ):
        return provider
    return prefix


def product_headers(root: Path) -> dict[str, str]:
    """Shared client's uncached application profile; urllib transport stays distinct."""
    version = tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"]
    return {
        "user-agent": f"biomcp-cli/{version}",
        "accept": "*/*",
        "cache-control": "no-store",
        # Keep bounded replies directly comparable without a decompressor.
        "accept-encoding": "identity",
    }


def segment(name: str) -> str:
    # Keep literal keys distinct from separators and array wildcards.
    return name.replace("~", "~0").replace("/", "~1").replace("*", "~2")


def json_shape(value: object) -> dict[str, set[str]]:
    shape: dict[str, set[str]] = {}

    def walk(node: object, path: str) -> None:
        kind = (
            "null"
            if node is None
            else "boolean"
            if isinstance(node, bool)
            else "object"
            if isinstance(node, dict)
            else "array"
            if isinstance(node, list)
            else "number"
            if isinstance(node, (int, float))
            else "string"
        )
        shape.setdefault(path, set()).add(kind)
        if isinstance(node, dict):
            for name, child in node.items():
                walk(child, f"{path}/{segment(name)}")
        elif isinstance(node, list):
            for child in node:
                walk(child, f"{path}/*")

    walk(value, "$")
    return shape


def reply_shape(body: bytes, content_type: str) -> dict[str, set[str]]:
    media_type = content_type.split(";", 1)[0].strip().lower()
    if media_type == "application/json" or media_type.endswith("+json"):
        try:
            return json_shape(json.loads(body))
        except (ValueError, UnicodeError, RecursionError) as error:
            raise ValueError("invalid JSON response structure") from error
    if media_type in {"application/xml", "text/xml"} or media_type.endswith("+xml"):
        # No external entities or document types belong in a structure check.
        if b"<!DOCTYPE" in body.upper() or b"<!ENTITY" in body.upper():
            raise ValueError("XML document types and entities are unsupported")
        try:
            root = ElementTree.fromstring(body)
            shape: dict[str, set[str]] = {}

            def walk(element: ElementTree.Element, path: str) -> None:
                path += "/" + segment(element.tag)
                shape[path] = {"element"}
                for child in element:
                    walk(child, path)

            walk(root, "$")
            return shape
        except (ElementTree.ParseError, LookupError, RecursionError) as error:
            raise ValueError("invalid XML response structure") from error
    raise ValueError(
        "unsupported response format; structure comparison supports JSON and XML only"
    )


def shape_drift(
    expected: dict[str, set[str]], actual: dict[str, set[str]]
) -> list[str]:
    problems = []
    for path, types in sorted(expected.items()):
        if path not in actual:
            problems.append(f"missing field {path}")
        elif types != actual[path]:
            problems.append(f"changed type {path}")
    return problems
