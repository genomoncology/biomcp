from __future__ import annotations

import json
import subprocess
from pathlib import Path
from urllib.error import HTTPError
from urllib.parse import urlencode
from urllib.request import Request, urlopen

import pytest


REPO_ROOT = Path(__file__).resolve().parents[1]
SETUP = REPO_ROOT / "spec/fixtures/setup-provider-contract-spec-fixture.sh"
CLEANUP = REPO_ROOT / "spec/fixtures/cleanup-provider-contract-spec-fixture.sh"


def _workspace(tmp_path: Path) -> Path:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    (workspace / "spec").symlink_to(REPO_ROOT / "spec", target_is_directory=True)
    (workspace / "testdata").symlink_to(
        REPO_ROOT / "testdata", target_is_directory=True
    )
    return workspace


def _exports(path: Path) -> dict[str, str]:
    command = f"source {path!s}; env"
    output = subprocess.run(
        ["bash", "-c", command], check=True, capture_output=True, text=True
    ).stdout
    return dict(line.split("=", 1) for line in output.splitlines() if "=" in line)


@pytest.mark.skipif(not Path("/proc").is_dir(), reason="fixture supervision needs procfs")
def test_provider_fixture_serves_receipted_routes_and_fails_closed(
    tmp_path: Path,
) -> None:
    workspace = _workspace(tmp_path)
    try:
        subprocess.run(["bash", str(SETUP), str(workspace)], check=True)
        values = _exports(workspace / ".cache/spec-provider-contract-env")
        base = values["BIOMCP_MYCHEM_BASE"]

        with urlopen(f"{base}/query?q=Keytruda", timeout=2) as response:
            body = json.load(response)
        assert body["hits"][0]["_id"] == "C3855203"

        mygene = values["BIOMCP_MYGENE_BASE"]
        query = urlencode({"q": 'symbol:"BRAF"'})
        with urlopen(f"{mygene}/query?{query}", timeout=2) as response:
            gene = json.load(response)
        assert gene["hits"][0]["symbol"] == "BRAF"

        chembl = values["BIOMCP_CHEMBL_BASE"]
        with urlopen(
            f"{chembl}/mechanism.json?molecule_chembl_id=CHEMBL3137343&limit=15",
            timeout=2,
        ) as response:
            mechanisms = json.load(response)
        assert mechanisms["mechanisms"][0]["target_chembl_id"] == "CHEMBL3307223"

        opentargets = values["BIOMCP_OPENTARGETS_BASE"]
        request = Request(
            f"{opentargets}/graphql",
            data=json.dumps(
                {"query": "query fixture", "variables": {"chemblId": "CHEMBL3137343"}}
            ).encode(),
            headers={"Content-Type": "application/json"},
        )
        with urlopen(request, timeout=2) as response:
            target = json.load(response)
        assert target["data"]["drug"]["id"] == "CHEMBL3137343"

        with pytest.raises(HTTPError) as error:
            urlopen(f"{base}/unknown", timeout=2)
        assert error.value.code == 404

        request_log = Path(values["BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG"])
        assert "GET /mychem/v1/query?q=Keytruda" in request_log.read_text()
        assert "GET /mygene/v3/query?q=symbol%3A%22BRAF%22" in request_log.read_text()
        assert "POST /opentargets/api/v4/graphql" in request_log.read_text()
        assert values["BIOMCP_CACHE_MODE"] == "off"
        assert Path(values["BIOMCP_EMA_DIR"]).is_dir()
        assert Path(values["BIOMCP_WHO_DIR"]).is_dir()
    finally:
        subprocess.run(["bash", str(CLEANUP), str(workspace)], check=False)

    assert not (workspace / ".cache/spec-provider-contract-env").exists()
    assert not (workspace / ".cache/spec-provider-contract-ownership").exists()
