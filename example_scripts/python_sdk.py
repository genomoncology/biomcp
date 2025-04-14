#!/usr/bin/env -S uv --quiet run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "biomcp-python",
# ]
# ///

# Scripts to reproduce this page:
# https://biomcp.org/python_sdk/

import asyncio
import json

from biomcp.trials.search import (
    RecruitingStatus,
    TrialPhase,
    TrialQuery,
    search_trials,
)
from biomcp.variants.getter import get_variant
from biomcp.variants.search import VariantQuery, search_variants


async def find_pathogenic_tp53():
    # noinspection PyTypeChecker
    query = VariantQuery(gene="TP53", significance="pathogenic", size=5)
    # Get results as Markdown (default)
    json_output_str = await search_variants(query, output_json=True)
    data = json.loads(json_output_str)
    assert len(data) == 5
    for item in data:
        clinvar = item.get("clinvar")
        for rcv in clinvar.get("rcv", []):
            assert "pathogenic" in rcv["clinical_significance"].lower()


async def get_braf_v600e_details():
    variant_id = "chr7:g.140453136A>T"  # BRAF V600E variant

    # Get results as JSON string
    json_output_str = await get_variant(variant_id, output_json=True)
    data = json.loads(json_output_str)

    # Process the variant data
    assert data, "No data returned for BRAF V600E variant"
    variant = data[0]
    clinvar = variant.get("clinvar", {})
    cosmic = variant.get("cosmic", {})
    docm = variant.get("docm", {})

    # Verify key variant details
    assert clinvar.get("gene", {}).get("symbol") == "BRAF"
    assert clinvar.get("chrom") == "7"
    assert clinvar.get("cytogenic") == "7q34"
    assert cosmic.get("cosmic_id") == "COSM476"
    assert docm.get("aa_change") == "p.V600E"

    # Verify HGVS coding variants
    hgvs_coding = clinvar.get("hgvs", {}).get("coding", [])
    assert len(hgvs_coding) >= 13
    assert "NM_004333.6:c.1799T>A" in hgvs_coding


async def find_melanoma_trials():
    query = TrialQuery(
        conditions=["Melanoma"],
        interventions=["Pembrolizumab"],
        recruiting_status=RecruitingStatus.OPEN,
        phase=TrialPhase.PHASE3,
    )

    # Get results as JSON string
    json_output_str = await search_trials(query, output_json=True)
    data = json.loads(json_output_str)

    # Verify we got results
    assert data, "No trials found"
    assert len(data) >= 2, "Expected at least 2 melanoma trials"

    # Verify first trial details (NCT05727904)
    trial1 = data[0]
    assert trial1["NCT Number"] == "NCT05727904"
    assert "lifileucel" in trial1["Study Title"].lower()
    assert trial1["Study Status"] == "RECRUITING"
    assert trial1["Phases"] == "PHASE3"
    assert int(trial1["Enrollment"]) == 670
    assert "Melanoma" in trial1["Conditions"]
    assert "Pembrolizumab" in trial1["Interventions"]

    # Verify second trial details (NCT06697301)
    trial2 = data[1]
    assert trial2["NCT Number"] == "NCT06697301"
    assert "EIK1001" in trial2["Study Title"]
    assert trial2["Study Status"] == "RECRUITING"
    assert "PHASE3" in trial2["Phases"]
    assert int(trial2["Enrollment"]) == 740
    assert trial2["Conditions"] == "Advanced Melanoma"


def run():
    asyncio.run(find_pathogenic_tp53())
    asyncio.run(get_braf_v600e_details())
    asyncio.run(find_melanoma_trials())


if __name__ == "__main__":
    run()
