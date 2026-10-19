"""CGI output identifies the carried MyVariant subset in either format.

The populated reply is the committed BRAF V600E recording. The empty case
is an authored projection of that reply with only each hit's CGI removed;
it is not a captured empty response or a KRAS fixture.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
import replay_env  # noqa: E402

SOURCE_NOTE = "MyVariant's variant-level CGI subset, loaded 2023-02-27."
EMPTY_NOTE = "No record in this subset."


@pytest.mark.needs_binary
@pytest.mark.parametrize("empty", [False, True], ids=["recorded-braf", "synthetic-cgi-removed"])
@pytest.mark.parametrize("json_output", [False, True], ids=["markdown", "json"])
def test_variant_cgi_names_subset_and_load_date(tmp_path, monkeypatch, empty, json_output):
    """A caller can distinguish a subset absence from missing gene evidence."""
    root = replay_env.REPO_ROOT
    replay = root / "testdata/replay"
    fixture = tmp_path / "fixture"
    fixture.mkdir()
    (fixture / "src").symlink_to(root / "src", target_is_directory=True)
    projected = fixture / "testdata/replay"
    projected.mkdir(parents=True)
    manifest = json.loads((replay / "manifest.json").read_text())
    for key, route in manifest["routes"].items():
        target = projected / route["file"]
        target.parent.mkdir(parents=True, exist_ok=True)
        original = replay / route["file"]
        if empty and "/myvariant/query?" in key and "p.V600E" in key:
            reply = json.loads(original.read_text())
            for hit in reply["hits"]:
                hit.pop("cgi", None)
            if target.is_symlink():
                target.unlink()
            target.write_text(json.dumps(reply))
        elif not target.exists():
            target.symlink_to(original)
    (projected / "manifest.json").write_text(json.dumps(manifest))
    monkeypatch.setattr(replay_env, "REPO_ROOT", fixture)
    binary = Path(os.environ.get("BIOMCP_BIN", root / "target/debug/biomcp"))
    argv = ("get", "variant", "BRAF V600E", "cgi")
    if json_output:
        argv += ("--json",)
    with replay_env.replay_environment() as env:
        run = env.run(binary, argv)
        assert run.process.returncode == 0, run.process.stderr
        assert not env.misses(), env.misses()
    output = run.process.stdout
    if json_output:
        payload = json.loads(output)
        assert len(payload.get("cgi_associations", [])) == (0 if empty else 10)
        notes = payload["_meta"].get("notes", [])
        assert SOURCE_NOTE in notes
        assert (EMPTY_NOTE in notes) == empty
    else:
        assert SOURCE_NOTE in output
        assert (EMPTY_NOTE in output) == empty
        assert "No CGI drug associations reported." not in output
        if not empty:
            assert "| Drug | Association | Tumor Type | Evidence |" in output
            assert "Vemurafenib" in output
