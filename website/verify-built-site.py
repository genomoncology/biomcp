#!/usr/bin/env python3
"""Prove the built ClinicalTrial discovery routes without network access."""

from html.parser import HTMLParser
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

root = Path(__file__).resolve().parent
dist = root / "dist"

MODELS = {
    "clinical-trial": {
        "root": "model:ClinicalTrial",
        "phrases": ("ClinicalTrial", "Direct relationship", "Reproduce the recorded ClinicalTrials.gov conversion", "FHIR", "USDM", "unsupported"),
        "downloads": (
            "clinical-trial.schema.json", "clinical-trial-projection.schema.json",
            "clinical-trial-relationships.svg", "nct02576665-provider-types.json",
            "ctgov-clinical-trial-projection.json", "clinical-trial-v1.bundle.json",
        ),
    },
    "scientific-publication": {
        "root": "model:ScientificPublication",
        "phrases": ("ScientificPublication", "Direct relationship", "PubTator", "Europe PMC", "unsupported"),
        "downloads": (
            "scientific-publication.schema.json", "pubtator3-scientific-publication.json",
            "scientific-publication-relationships.svg", "scientific-publication-v1.bundle.json",
        ),
    },
}

required = ["llms.txt", "llms-full.txt"]
for slug, model in MODELS.items():
    required += [
        f"biodata/models/{slug}/index.html",
        f"biodata/models/{slug}.md",
        f"biodata/discovery/{slug}.json",
    ]
    required += [f"downloads/biodata/{name}" for name in model["downloads"]]


class Links(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.targets: list[str] = []
        self.words: list[str] = []

    def handle_starttag(self, _tag: str, attrs: list[tuple[str, str | None]]) -> None:
        for name, value in attrs:
            if name in {"href", "src"} and value:
                self.targets.append(value)

    def handle_data(self, data: str) -> None:
        self.words.append(data)


def local_target(url: str) -> Path | None:
    prefix = "https://biomcp.org/"
    if url.startswith(prefix):
        relative = url.removeprefix(prefix)
    elif url.startswith("/"):
        relative = url.removeprefix("/")
    else:
        return None
    target = dist / relative
    if url.endswith("/"):
        target /= "index.html"
    return target


for relative in required:
    if not (dist / relative).is_file():
        raise SystemExit(f"missing built route: /{relative}")

all_urls: set[str] = set()
indexes = [(dist / name).read_text() for name in ("llms.txt", "llms-full.txt")]

for slug, model in MODELS.items():
    html_parser = Links()
    html_parser.feed((dist / f"biodata/models/{slug}/index.html").read_text())
    html_text = " ".join(html_parser.words)
    raw_text = (dist / f"biodata/models/{slug}.md").read_text()
    discovery = json.loads((dist / f"biodata/discovery/{slug}.json").read_text())

    for phrase in model["phrases"]:
        if phrase not in html_text or phrase not in raw_text:
            raise SystemExit(
                f"discovery answer missing from {slug} HTML or raw Markdown: {phrase}"
            )

    public_urls = set(discovery["routes"].values())
    for index in indexes:
        missing = sorted(url for url in public_urls if url not in index)
        if missing:
            raise SystemExit(f"agent index omits discovery routes: {', '.join(missing)}")

    all_urls.update(
        url
        for url in html_parser.targets
        if url.startswith("/biodata/") or url.startswith("/downloads/biodata/")
    )
    all_urls.update(public_urls)
    all_urls.update(
        re.findall(r"\]\((?:https://biomcp\.org)?(/[^\s)>]+)", raw_text)
    )

    bundle = json.loads(
        (dist / f"downloads/biodata/{slug}-v1.bundle.json").read_text()
    )
    direct = [
        item
        for item in bundle["relationships"]
        if item["source"].startswith(f"{model['root']}/field:")
    ]
    svg = ET.parse(
        dist / f"downloads/biodata/{slug}-relationships.svg"
    ).getroot()
    diagram_ids = [
        node.attrib["data-relationship-id"]
        for node in svg.findall(".//{http://www.w3.org/2000/svg}g")
    ]
    if diagram_ids != [item["id"] for item in direct]:
        raise SystemExit(f"{slug} diagram relationships differ from the adopted catalog")
    if discovery["diagram_relationships"] != direct:
        raise SystemExit(
            f"{slug} discovery diagram relationships differ from the adopted catalog"
        )
    if discovery["support"] != bundle["support"]:
        raise SystemExit(f"{slug} discovery support differs from the adopted catalog")
    if discovery["crosswalks"] != bundle["crosswalks"]:
        raise SystemExit(
            f"{slug} discovery crosswalks differ from the adopted catalog"
        )
    for group, item in zip(
        svg.findall(".//{http://www.w3.org/2000/svg}g"), direct, strict=True
    ):
        if group.attrib != {
            "data-relationship-id": item["id"],
            "data-source": item["source"],
            "data-target": item["target"],
            "data-kind": item["kind"],
        }:
            raise SystemExit(f"diagram relationship attributes differ: {item['id']}")
        text = {
            node.attrib["data-role"]: node.text
            for node in group.findall("{http://www.w3.org/2000/svg}text")
        }
        expected = {
            "id": item["id"],
            "source": item["source"],
            "target": item["target"],
            "kind": item["kind"],
            "explanation": item["explanation"],
        }
        if text != expected:
            raise SystemExit(f"diagram relationship text differs: {item['id']}")
    adjacent = raw_text.split("## Direct relationship text", 1)[1].split(
        "## Component relationships", 1
    )[0]
    adjacent_rows = [line for line in adjacent.splitlines() if line.startswith("- `")]
    if len(adjacent_rows) != len(direct):
        raise SystemExit(
            f"{slug} adjacent relationship row count differs from the diagram"
        )
    for line, item in zip(adjacent_rows, direct, strict=True):
        for value in (item["id"], item["source"], item["target"], item["kind"], item["explanation"]):
            escaped = value
            for character in "*_{}[]()#+-.!":
                escaped = escaped.replace(character, f"\\{character}")
            if escaped not in line:
                raise SystemExit(f"adjacent relationship text omits: {item['id']} {value}")

    for task in discovery["tasks"]:
        if not task.get("question") or not task.get("answer"):
            raise SystemExit("discovery task lacks a question or checked answer")
        if local_target(task["route"]) is None:
            raise SystemExit("discovery task does not use a stable BioMCP route")

for index in indexes:
    all_urls.update(
        re.findall(r"\]\((?:https://biomcp\.org)?(/[^\s)>]+)", index)
    )
for url in sorted(all_urls):
    target = local_target(url)
    if target is not None and not target.is_file():
        raise SystemExit(f"unresolved public URL: {url}")
