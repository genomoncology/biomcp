"""Fulltext reports license metadata fetched while discovering article assets."""
from __future__ import annotations

import json
import os
from pathlib import Path

import pytest

from golden.replay_env import REPO_ROOT, replay_environment

pytestmark = [pytest.mark.needs_binary]


@pytest.mark.parametrize("license_code", ["CC BY", "CC BY-NC-ND", None])
@pytest.mark.parametrize("package_available", [True, False])
def test_fulltext_keeps_fetched_license_even_without_asset_package(
    tmp_path: Path, license_code: str | None, package_available: bool,
):
    # Only the unchanged CC BY row is an observed response. Other rows are
    # authored controls for absent/restricted metadata and failed downloads.
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
    assert reuse["license_present"] is (license_code is not None), reuse
    if license_code:
        assert reuse["license"] == license_code
        assert reuse["license_source"] == {"label": "PMC OA Archive", "source": "PMC OA"}
        assert "reuse_warning" not in reuse
    else:
        assert "license" not in reuse
        assert "unknown" in reuse["reuse_warning"]
    assert article["full_text_manifest"]["provider"]["source"] == "Europe PMC"
