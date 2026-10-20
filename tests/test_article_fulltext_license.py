"""Fulltext reports license metadata fetched while discovering article assets."""
from __future__ import annotations

import json
import os
import re
from pathlib import Path

import pytest

from golden.replay_env import REPO_ROOT, replay_environment

pytestmark = [pytest.mark.needs_binary]


@pytest.mark.parametrize("license_code,jats_license,known_license,expected,source", [
    ("CC BY", None, None, "CC BY", "PMC OA"),
    ("CC BY-NC-ND", None, None, "CC BY-NC-ND", "PMC OA"),
    (None, None, None, None, None),
    (None, "observed", None, "http://creativecommons.org/licenses/by/4.0/", "Europe PMC"),
    (None, "ali-only", None, "https://creativecommons.org/licenses/by/4.0/", "Europe PMC"),
    ("CC BY", "observed", "CC BY-NC-ND", "CC BY-NC-ND", "Europe PMC"),
])
@pytest.mark.parametrize("package_available", [True, False])
def test_fulltext_keeps_fetched_license_even_without_asset_package(
    tmp_path: Path, license_code: str | None, jats_license: str | None,
    known_license: str | None, expected: str | None, source: str | None,
    package_available: bool,
):
    # Captures stay unchanged on disk. These authored source combinations
    # isolate which fetched metadata supplies the license; body text is kept.
    root = tmp_path / "replay"
    replay_dir = root / "testdata/replay"
    replay_dir.mkdir(parents=True)
    (root / "src").symlink_to(REPO_ROOT / "src", target_is_directory=True)
    (replay_dir / "responses").symlink_to(
        REPO_ROOT / "testdata/replay/responses", target_is_directory=True,
    )
    manifest = json.loads((REPO_ROOT / "testdata/replay/manifest.json").read_text())
    route = manifest["routes"]["GET /PMC3467229.1/PMC3467229.1.json"]
    metadata = json.loads((REPO_ROOT / "testdata/replay" / route["file"]).read_text())
    metadata["license_code"] = license_code
    (replay_dir / "metadata.json").write_text(json.dumps(metadata))
    route["file"] = "metadata.json"
    xml_route = manifest["routes"]["GET /europepmc/PMC3467229/fullTextXML"]
    xml = (REPO_ROOT / "testdata/replay" / xml_route["file"]).read_text()
    if jats_license is None:
        xml = re.sub(r"<license\b.*?</license>", "", xml, flags=re.DOTALL)
    elif jats_license == "ali-only":
        xml = xml.replace(' xlink:href="http://creativecommons.org/licenses/by/4.0/"', "")
    (replay_dir / "article.xml").write_text(xml)
    xml_route["file"] = "article.xml"
    if jats_license is None:
        for key, entry in manifest["routes"].items():
            if key.startswith("GET /PMC3467229.1/") and key.endswith(".xml"):
                package_xml = (REPO_ROOT / "testdata/replay" / entry["file"]).read_text()
                package_xml = re.sub(r"<license\b.*?</license>", "", package_xml, flags=re.DOTALL)
                (replay_dir / "package.xml").write_text(package_xml)
                entry["file"] = "package.xml"
    if known_license:
        identity_route = next(entry for key, entry in manifest["routes"].items()
                              if key.startswith("GET /europepmc/search?") and "23056577" in key)
        identity = json.loads((REPO_ROOT / "testdata/replay" / identity_route["file"]).read_text())
        identity["resultList"]["result"][0]["license"] = known_license
        (replay_dir / "identity.json").write_text(json.dumps(identity))
        identity_route["file"] = "identity.json"
    unavailable = replay_dir / "authored-unavailable.json"
    unavailable.write_text('{"authored":true,"error":"Unavailable"}\n')
    manifest["routes"]["GET /europepmc/PMC3467229/supplementaryFiles"] = {
        "status": 503, "file": unavailable.name,
        "headers": {"content-type": "application/json"},
    }
    for key, entry in manifest["routes"].items():
        if key == "GET /europepmc/PMC3467229/supplementaryFiles" or (
            not package_available and key.startswith("GET /PMC3467229.1/")
            and not key.endswith(".json")
        ):
            entry.update(status=503, file=unavailable.name)
    (replay_dir / "manifest.json").write_text(json.dumps(manifest))
    with replay_environment(root=root) as replay:
        run = replay.run(Path(os.environ["BIOMCP_BIN"]), (
            "--json", "--disable-sources", "semantic-scholar,figshare",
            "get", "article", "23056577", "fulltext",
        ))
        assert run.process.returncode == 0, run.process.stderr
        assert not replay.misses(), replay.misses()
        article = json.loads(run.process.stdout)
    reuse = article["full_text_manifest"]["reuse"]
    assert reuse["license_present"] is (expected is not None), reuse
    if expected:
        assert reuse["license"] == expected
        assert reuse["license_source"]["source"] == source
        assert "reuse_warning" not in reuse
    else:
        assert "license" not in reuse
        assert "unknown" in reuse["reuse_warning"]
    assert article["full_text_manifest"]["provider"]["source"] == "Europe PMC"
