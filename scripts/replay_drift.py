"""Project recorded replies onto fields named by the provider's Rust parser.

This is a conservative source-derived approximation, not a Rust type checker.
It compares observed field paths and JSON types, never changing data values.
"""

from __future__ import annotations

import json
import re
from functools import lru_cache
from pathlib import Path

SOURCE_ALIASES = {"ctgov": "clinicaltrials", "s2": "semantic_scholar"}
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


@lru_cache(maxsize=None)
def source_text(root: Path, provider: str) -> str:
    name = SOURCE_ALIASES.get(provider, provider.replace("-", "_"))
    base = root / "src/sources"
    paths = [base / f"{name}.rs"]
    paths.extend(p for p in (base / name).rglob("*.rs") if "tests" not in p.parts)
    texts = []
    for path in paths:
        if path.is_file():
            texts.append(
                re.split(
                    r"#\[cfg\(test\)\]\s*mod tests\b", path.read_text(), maxsplit=1
                )[0]
            )
    if not texts:
        raise ValueError("no provider parser source")
    # Raw JSON values are also interpreted by provider-specific transforms.
    # Discover consumers through their source-module import, rather than
    # keeping a second field inventory here.
    for path in (root / "src/transform").rglob("*.rs"):
        if "tests" in path.parts:
            continue
        text = re.split(
            r"#\[cfg\(test\)\]\s*mod tests\b", path.read_text(), maxsplit=1
        )[0]
        if f"sources::{name}::" in text:
            texts.append(text)
    return "\n".join(texts)


def consumed_keys(text: str) -> set[str]:
    """Read model fields, serde wire names, and literal JSON accesses.

    Both Rust and camel-case field spellings form an overapproximation for
    rename_all models. Ancestors of selected fields remain in the projection.
    Dynamic accesses cannot generally be inferred; PubMed's numeric map is
    handled separately, and an empty projection is unsupported.
    """
    keys = set(re.findall(r'(?:\.get\(\s*|\[\s*)"([^"\n]+)"', text))
    keys.update(re.findall(r'(?:rename|alias)\s*=\s*"([^"\n]+)"', text))
    for body in re.findall(r"\bstruct\s+\w+[^;{]*\{([^{}]*)\}", text):
        for name in re.findall(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(\w+)\s*:", body, re.M):
            keys.add(name)
            first, *rest = name.split("_")
            keys.add(first + "".join(part.title() for part in rest))
    for pointer in re.findall(r'\.pointer\(\s*"([^"\n]+)"', text):
        keys.update(pointer.strip("/").split("/"))
    for array in re.findall(r"for\s+\w+\s+in\s+\[([^\]]*)\]", text):
        keys.update(re.findall(r'"([^"\n]+)"', array))
    return keys


def json_shape(value: object, keys: set[str], provider: str) -> dict[str, set[str]]:
    shape: dict[str, set[str]] = {}

    def walk(node: object, path: str, selected: bool = False) -> bool:
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
        child_selected = False
        if isinstance(node, dict):
            for name, child in node.items():
                # ESummary looks up entries by requested PMID, not a fixed key.
                dynamic = provider == "pubmed" and path == "$/result" and name.isdigit()
                segment = "*" if dynamic else name
                child_selected |= walk(
                    child, f"{path}/{segment}", name in keys or dynamic
                )
        elif isinstance(node, list):
            for child in node:
                child_selected |= walk(child, f"{path}/*", selected)
        if selected or child_selected or path == "$":
            shape.setdefault(path, set()).add(kind)
            return True
        return False

    walk(value, "$")
    if len(shape) <= 1:
        raise ValueError("no consumed fields in reply (unsupported or empty response)")
    return shape


def reply_shape(
    body: bytes, root: Path, provider: str, request_path: str
) -> dict[str, set[str]]:
    text = source_text(root, provider)
    if provider == "kegg":
        decoded = body.decode("utf-8")
        if "/find/" in request_path:
            lines = [line for line in decoded.splitlines() if line.strip()]
            if not lines:
                raise ValueError("empty KEGG search cannot certify response shape")
            if any("\t" not in line for line in lines):
                raise ValueError("KEGG search lacks tab-separated identifier/name")
            return {"$/rows": {"identifier/name"}}
        fields = set(re.findall(r"^([A-Z][A-Z_]+)\s+", decoded, re.M))
        consumed = set(re.findall(r'"([A-Z][A-Z_]+)"\s*(?:=>|if)', text))
        result = {f"$/{field}": {"text"} for field in fields & consumed}
        if not result:
            raise ValueError("unsupported KEGG response")
        return result
    return json_shape(json.loads(body), consumed_keys(text), provider)


def shape_drift(
    expected: dict[str, set[str]], actual: dict[str, set[str]]
) -> list[str]:
    problems = []
    for path, types in sorted(expected.items()):
        if path not in actual:
            problems.append(f"missing field {path}")
        elif "null" not in types and not (types & actual[path]):
            problems.append(f"changed type {path}")
    return problems
