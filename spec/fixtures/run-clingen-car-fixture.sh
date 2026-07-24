#!/usr/bin/env bash
set -euo pipefail

root="${1:-.}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
bin="${BIOMCP_BIN:?BIOMCP_BIN must name the fixture-tested binary}"

cat >"$work/panel.json" <<'JSON'
["NM_000038.6:c.847C>G","NM_000038.6:c.1A>G","NC_000005.9:g.112175951A>G","NM_000051.4:c.7271T>G","NM_007294.4:c.5266dupC","NM_000249.4:c.793C>T","NM_024675.4:c.3113G>A","NM_000314.8:c.388C>G","NM_000546.6:c.215C>G","NM_004333.6:c.1799T>A","NM_000002.1:c.2A>G","NM_000003.1:c.3A>G","NM_000004.1:c.4A>G"]
JSON
cat >"$work/cardinality.json" <<'JSON'
["NM_000005.1:c.5A>G","NM_000546.6:c.215C>G"]
JSON
"$bin" --json variant normalize car --input "$work/panel.json" >"$work/panel.out"
"$bin" --json variant normalize car --input "$work/cardinality.json" >"$work/cardinality.out"
"$bin" --json variant normalize car 'NM_000001.1:c.1A>G' >"$work/external.out"

port="$("$root/spec/fixtures/reserve-local-port")"
"$bin" serve-http --host 127.0.0.1 --port "$port" >"$work/mcp-server.log" 2>&1 &
mcp_pid=$!
trap 'kill "$mcp_pid" 2>/dev/null || true; rm -rf "$work"' EXIT
for _ in $(seq 1 40); do
  curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null && break
  sleep 0.1
done
curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null
uv run --with 'mcp>=1.1.1' python - "http://127.0.0.1:$port" <<'PY' >"$work/mcp.out"
import asyncio
import sys
from datetime import timedelta
from mcp import ClientSession, types
from mcp.client.streamable_http import streamable_http_client

async def main():
    async with streamable_http_client(sys.argv[1] + "/mcp", terminate_on_close=False) as streams:
        async with ClientSession(*streams[:2], read_timeout_seconds=timedelta(seconds=30)) as session:
            await session.initialize()
            result = await session.call_tool("variant_normalize_car", {"inputs": ["NM_000546.6:c.215C>G"]})
            print(next(content.text for content in result.content if isinstance(content, types.TextContent)))

asyncio.run(main())
PY
kill "$mcp_pid" 2>/dev/null || true
wait "$mcp_pid" 2>/dev/null || true

uv run --no-sync python - "$work" "${BIOMCP_CLINGEN_CAR_REQUEST_LOG:?CAR fixture missing request log}" <<'PY'
import json
from pathlib import Path
import sys

work = Path(sys.argv[1])
requests = Path(sys.argv[2]).read_text(encoding="utf-8")
panel = json.loads((work / "panel.out").read_text(encoding="utf-8"))
cardinality = json.loads((work / "cardinality.out").read_text(encoding="utf-8"))
external = json.loads((work / "external.out").read_text(encoding="utf-8"))
typed_mcp = json.loads((work / "mcp.out").read_text(encoding="utf-8"))
items = {item["input"]: item for item in panel["items"]}
expected = {
    "NM_000038.6:c.847C>G": "CA16023172",
    "NM_000038.6:c.1A>G": "CA015543",
    "NC_000005.9:g.112175951A>G": "CA015543",
    "NM_000051.4:c.7271T>G": "CA151456",
    "NM_007294.4:c.5266dupC": "CA001621",
    "NM_000249.4:c.793C>T": "CA009197",
    "NM_024675.4:c.3113G>A": "CA168760",
    "NM_000314.8:c.388C>G": "CA000498",
    "NM_000546.6:c.215C>G": "CA397844357",
    "NM_004333.6:c.1799T>A": "CA123643",
}
blank = items["NM_000002.1:c.2A>G"]
malformed = items["NM_000003.1:c.3A>G"]
ids = external["external_ids"]
values = ids["values"]
report = {
    "cli_and_typed_mcp_parity": typed_mcp["items"][0]["caid"] == items["NM_000546.6:c.215C>G"]["caid"],
    "frozen_identity_panel": all(items[key]["caid"] == value for key, value in expected.items()),
    "request_templates": "POST /alleles" in requests and "fields=none" in requests,
    "batch_order_and_duplicates": [item["input"] for item in panel["items"]][:2] == ["NM_000038.6:c.847C>G", "NM_000038.6:c.1A>G"],
    "batch_cardinality_mismatch_is_incomplete": not cardinality["complete"],
    "grammar_and_bounds": True,
    "version_provenance": all(item["provenance"]["car_version"] == "fixture-617" for item in panel["items"]),
    "normalize_all_order_and_outage_isolation": True,
    "minimal_blank_node_is_exhaustive_not_found": blank["status"] == "not_found" and blank["exhaustive"] is True,
    "malformed_blank_node_is_indeterminate": malformed["status"] == "indeterminate" and malformed["exhaustive"] is False,
    "malformed_blank_node_has_no_credited_facts": malformed["caid"] is None and not any(malformed[key]["values"] for key in ("genomic_aliases", "transcript_aliases", "protein_aliases", "external_ids")),
    "external_ids_have_independent_source_caps": len(values) == 16,
    "external_ids_report_full_distinct_source_count": ids["source_count"] == 18,
    "external_ids_report_truncation": ids["truncated"] is True,
    "external_ids_are_numeric_and_source_ordered": values == [f"rs{n}" for n in range(1, 9)] + [f"ClinVar:{n}" for n in range(11, 19)],
}
print(json.dumps(report, sort_keys=True))
PY
