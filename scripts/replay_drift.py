"""Compare JSON/XML structure or explicitly limited response signatures."""

from __future__ import annotations

import json
import re
from pathlib import Path
from urllib.parse import parse_qsl, urlsplit
from xml.etree import ElementTree
from xml.parsers import expat

OPAQUE_SIGNATURE_BYTES = 16

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
    provenance = entry.get("recording_provenance", {})
    provider = provenance.get("provider") if isinstance(provenance, dict) else None
    if method not in {"GET", "HEAD"} or provider not in routes:
        return prefix
    upstream = urlsplit(entry.get("upstream", ""))
    base = urlsplit(routes[provider])
    if (
        (upstream.scheme, upstream.netloc) == (base.scheme, base.netloc)
        and (
            upstream.path == base.path
            or upstream.path.startswith(base.path.rstrip("/") + "/")
        )
        and upstream.path == path.path
        and not path.fragment
        and sorted(parse_qsl(upstream.query, keep_blank_values=True))
        == sorted(parse_qsl(path.query, keep_blank_values=True))
    ):
        return provider
    return prefix


def product_headers(root: Path) -> dict[str, str]:
    """Shared client's uncached application profile; urllib transport stays distinct."""
    import tomllib

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


def reply_qualification(content_type: str) -> str:
    media_type = content_type.split(";", 1)[0].strip().lower()
    if media_type in {
        "application/json", "application/xml", "text/xml"
    } or media_type.endswith(("+json", "+xml")):
        return "structure"
    return "status/type/signature only"


def file_signature(body: bytes, media_type: str) -> str:
    signatures = {
        "image/jpeg": (b"\xff\xd8\xff",),
        "application/msword": (b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1",),
        "application/zip": (b"PK\x03\x04", b"PK\x05\x06", b"PK\x07\x08"),
    }
    if media_type in signatures:
        for signature in signatures[media_type]:
            if body.startswith(signature):
                return signature.hex()
        raise ValueError("response does not match its file signature")
    if media_type == "text/html":
        prefix = body.removeprefix(b"\xef\xbb\xbf").lstrip()
        if re.match(br"(?:<!doctype\s+html(?:\s|>)|<html(?:\s|>))", prefix, re.I):
            return "html"
        raise ValueError("response does not match its file signature")
    if media_type in {"application/octet-stream", "binary/octet-stream"}:
        if len(body) < OPAQUE_SIGNATURE_BYTES:
            raise ValueError("response is too short for its recorded file signature")
        # Literal bytes as served: do not infer JSON or another media type.
        return body[:OPAQUE_SIGNATURE_BYTES].hex()
    raise ValueError(
        "unsupported response format; structure comparison supports JSON and XML only"
    )


def reject_xml_entities(body: bytes) -> None:
    # Expat never loads an external subset. Refuse declarations before a second
    # parse can expand internal entities, including parameter entities.
    parser = expat.ParserCreate()

    def refuse(*_args):
        raise ValueError("XML entities and external resources are unsupported")

    parser.EntityDeclHandler = refuse
    parser.ExternalEntityRefHandler = refuse
    parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
    parser.Parse(body, True)


def reply_shape(body: bytes, content_type: str) -> dict[str, set[str]]:
    media_type = content_type.split(";", 1)[0].strip().lower()
    if media_type == "application/json" or media_type.endswith("+json"):
        try:
            return json_shape(json.loads(body))
        except (ValueError, UnicodeError, RecursionError) as error:
            raise ValueError("invalid JSON response structure") from error
    if media_type in {"application/xml", "text/xml"} or media_type.endswith("+xml"):
        try:
            reject_xml_entities(body)
            root = ElementTree.fromstring(body)
            shape: dict[str, set[str]] = {}

            def walk(element: ElementTree.Element, path: str) -> None:
                path += "/" + segment(element.tag)
                shape[path] = {"element"}
                for child in element:
                    walk(child, path)

            walk(root, "$")
            return shape
        except (
            ElementTree.ParseError, expat.ExpatError, LookupError, RecursionError
        ) as error:
            raise ValueError("invalid XML response structure") from error
    return {"signature": {file_signature(body, media_type)}}


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
