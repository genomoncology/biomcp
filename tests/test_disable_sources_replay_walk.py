"""The disable-sources walk: every source id, one command, no request.

Ticket 2124's outcome says a disabled source is never called by any
command. This proof walks every id in `docs/reference/sources.json`,
disables that one id, runs a command that uses the source against the
golden suite's replay server, and asserts the provider's request log
stays quiet for it. An enabled baseline run of the same command proves
the command really uses the source, so a renamed route or a source that
never reaches the wire cannot pass silently.

Each entry names the provider prefixes the replay server derives from
`src/sources` (the first path segment of a request). Entries marked
`local` cover sources with no HTTP client of their own — local datasets,
or fields other providers pass along — so no request can exist for them
by construction; the entry still runs their command disabled and asserts
it completes.

On a build without the alphagenome feature the `alphagenome` entry is
`local` by necessity: the routine gate compiles `--no-default-features`,
and `make full-feature-check` owns that client's behavior tests.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

import pytest

_HERE = Path(__file__).resolve().parent
_GOLDEN = _HERE / "golden"
if str(_GOLDEN) not in sys.path:
    sys.path.insert(0, str(_GOLDEN))

from replay_env import REPO_ROOT, replay_environment  # noqa: E402

pytestmark = [pytest.mark.needs_binary]

SOURCES_JSON = REPO_ROOT / "docs" / "reference" / "sources.json"
RUN_TIMEOUT_SECONDS = 120.0


@dataclass(frozen=True)
class WalkEntry:
    """One source id, the command that uses it, and its wire prefixes."""

    id: str
    argv: tuple[str, ...]
    prefixes: tuple[str, ...] = ()
    kind: str = "routed"
    extra_env: tuple[tuple[str, str], ...] = field(default=())


WALK: tuple[WalkEntry, ...] = (
    # Variant fan-out and helpers.
    WalkEntry("alphagenome", ("--json", "get", "variant", "7-140453136-A-T", "predict"), kind="local"),
    WalkEntry("cancer-genome-interpreter", ("--json", "get", "variant", "BRAF", "V600E", "cgi"), kind="local"),
    WalkEntry("cancerhotspots", ("--json", "variant", "structure", "BRAF", "V600E"), ("cancerhotspots",)),
    WalkEntry("cbioportal", ("--json", "get", "variant", "BRAF", "V600E", "cbioportal"), ("cbioportal", "cbioportal-datahub")),
    WalkEntry("civic", ("--json", "get", "variant", "BRAF", "V600E", "civic"), ("civic",)),
    WalkEntry("clinvar", ("--json", "get", "variant", "BRAF", "V600E", "clinvar"), ("clinvar",)),
    WalkEntry("cosmic", ("--json", "get", "variant", "BRAF", "V600E", "cosmic"), kind="local"),
    WalkEntry("dbsnp", ("--json", "get", "variant", "BRAF", "V600E", "population"), ("dbsnp",)),
    WalkEntry("gnomad", ("--json", "get", "variant", "BRAF", "V600E", "population-details"), ("gnomad",)),
    WalkEntry("gwas-catalog", ("--json", "get", "variant", "BRAF", "V600E", "gwas"), ("gwas",)),
    WalkEntry("myvariant-info", ("--json", "get", "variant", "BRAF", "V600E", "predictions"), ("myvariant",)),
    WalkEntry("oncokb", ("--json", "variant", "oncokb", "BRAF V600E"), ("oncokb",), extra_env=(("ONCOKB_TOKEN", "replay-walk-token"),)),
    WalkEntry("mutalyzer", ("--json", "variant", "normalize", "mutalyzer", "NM_004448.2:c.829G>A"), ("mutalyzer",)),
    WalkEntry("variantvalidator", ("--json", "variant", "normalize", "variantvalidator", "NM_004448.2:c.829G>T"), ("variantvalidator",)),
    WalkEntry("clingen_car", ("--json", "variant", "normalize", "car", "NM_000546.6:c.215C>G"), ("clingen-car",)),
    WalkEntry("clingen_erepo", ("--json", "variant", "erepo", "CA015543"), ("clingen-erepo",)),
    WalkEntry("clingen_cspec", ("--json", "gene", "cspec", "BRAF"), ("cspec",)),
    WalkEntry("clingen_ldh", ("--json", "variant", "articles", "BRAF", "V600E", "--verify-identity", "--limit", "1"), ("ldh",)),
    # Gene card and enrichment.
    WalkEntry("mygene-info", ("--json", "get", "gene", "BRAF"), ("mygene",)),
    WalkEntry("clingen", ("--json", "get", "gene", "BRAF", "clingen"), ("clingen",)),
    WalkEntry("gencc", ("--json", "gencc", "sync"), ("gencc",)),
    WalkEntry("disgenet", ("--json", "get", "gene", "BRAF", "disgenet"), ("disgenet",), extra_env=(("DISGENET_API_KEY", "replay-walk-key"),)),
    WalkEntry("nih-reporter", ("--json", "get", "gene", "BRAF", "funding"), ("nih-reporter",)),
    WalkEntry("quickgo", ("--json", "get", "gene", "BRAF", "go"), ("quickgo",)),
    WalkEntry("gtex", ("--json", "get", "gene", "BRAF", "expression"), ("gtex",)),
    WalkEntry("human-protein-atlas", ("--json", "get", "gene", "BRAF", "hpa"), ("hpa",)),
    WalkEntry("string", ("--json", "get", "protein", "P15056", "interactions"), ("string",)),
    WalkEntry("dgidb", ("--json", "get", "gene", "BRAF", "druggability"), ("dgidb",)),
    WalkEntry("opentargets", ("--json", "get", "gene", "BRAF", "druggability"), ("opentargets",)),
    WalkEntry("enrichr", ("--json", "get", "gene", "BRAF", "ontology"), ("enrichr",)),
    WalkEntry("gprofiler", ("--json", "enrich", "BRAF,KRAS", "--limit", "3"), ("gprofiler",)),
    WalkEntry("kegg", ("--json", "search", "pathway", "MAPK signaling pathway", "--limit", "2"), ("kegg",)),
    WalkEntry("reactome", ("--json", "search", "pathway", "MAPK signaling pathway", "--limit", "2"), ("reactome",)),
    WalkEntry("wikipathways", ("--json", "search", "pathway", "MAPK signaling pathway", "--limit", "2"), ("wikipathways",)),
    # Protein card.
    WalkEntry("uniprot", ("--json", "get", "protein", "P15056"), ("uniprot",)),
    WalkEntry("interpro", ("--json", "get", "protein", "P15056", "domains"), ("interpro",)),
    WalkEntry("complexportal", ("--json", "get", "protein", "P15056", "complexes"), ("complexportal",)),
    WalkEntry("alphafold-db", ("--json", "get", "protein", "P15056", "structures"), kind="local"),
    WalkEntry("pdb", ("--json", "variant", "structure", "BRAF", "V600E"), kind="local"),
    # Disease and phenotype.
    WalkEntry("mydisease-info", ("--json", "get", "disease", "MONDO:0007947"), ("mydisease",)),
    WalkEntry("monarch-initiative", ("--json", "search", "disease", "Rett syndrome", "--limit", "1"), ("monarch",)),
    WalkEntry("mondo", ("--json", "get", "disease", "MONDO:0007947"), kind="local"),
    WalkEntry("disease-ontology", ("--json", "search", "disease", "lung cancer", "--limit", "1"), kind="local"),
    WalkEntry("hpo-jax-api", ("--json", "search", "phenotype", "HP:0001250,HP:0001263"), ("hpo",)),
    WalkEntry("seer-explorer", ("--json", "get", "disease", "MONDO:0007947", "survival"), ("seer",)),
    WalkEntry("ols4", ("--json", "discover", "BRAF"), ("ols4",)),
    WalkEntry("umls", ("--json", "discover", "chest pain"), ("umls",), extra_env=(("UMLS_API_KEY", "replay-walk-key"),)),
    WalkEntry("medlineplus", ("--json", "discover", "chest pain"), ("medlineplus",)),
    # Drugs and pharmacogenomics.
    WalkEntry("mychem-info", ("--json", "search", "drug", "imatinib", "--limit", "1"), ("mychem",)),
    WalkEntry("chembl", ("--json", "get", "drug", "imatinib", "targets"), ("chembl",)),
    WalkEntry("openfda", ("--json", "get", "drug", "pembrolizumab", "safety"), ("openfda",)),
    WalkEntry("fda-orphan-designations", ("--json", "get", "drug", "pembrolizumab", "regulatory"), ("fda-orphan",)),
    WalkEntry("ema", ("--json", "ema", "sync"), ("ema-report",)),
    WalkEntry("pharmgkb", ("--json", "search", "pgx", "-g", "CYP2D6"), ("pharmgkb",)),
    WalkEntry("cpic", ("--json", "search", "pgx", "-g", "CYP2D6"), ("cpic",)),
    WalkEntry("ddinter", ("--json", "drug", "interactions", "warfarin"), kind="local"),
    WalkEntry("drugbank", ("--json", "get", "drug", "pembrolizumab"), kind="local"),
    WalkEntry("drugs-at-fda", ("--json", "get", "drug", "pembrolizumab", "regulatory"), kind="local"),
    WalkEntry("vaers", ("--json", "search", "adverse-event", "--drug", "pembrolizumab", "--source", "vaers", "--limit", "1"), ("vaers",)),
    # Cell lines.
    WalkEntry("cellosaurus", ("--json", "get", "cell-line", "CVCL_2119"), ("cellosaurus",)),
    WalkEntry("pharmacodb", ("--json", "cell-line", "drug-response", "CVCL_2119", "--dataset", "GDSC1"), ("pharmacodb",)),
    WalkEntry("cell-model-passports", ("--json", "get", "cell-line", "CVCL_2119"), kind="local"),
    WalkEntry("depmap", ("--json", "cell-line", "drug-response", "CVCL_2119", "--dataset", "GDSC1"), kind="local"),
    # Trials and articles.
    WalkEntry("clinicaltrials-gov", ("--json", "search", "trial", "melanoma", "--limit", "1"), ("ctgov",)),
    WalkEntry("nci-cts", ("--json", "search", "trial", "melanoma", "--source", "nci", "--limit", "1"), ("nci-cts",), extra_env=(("NCI_API_KEY", "replay-walk-key"),)),
    WalkEntry("pubmed", ("--json", "search", "article", "BRAF", "--source", "pubmed", "--limit", "1"), ("pubmed",)),
    WalkEntry("ncbi-e-utilities", ("--json", "get", "article", "22663011", "fulltext"), ("pubmed",)),
    WalkEntry("europe-pmc", ("--json", "search", "article", "BRAF", "--source", "europepmc", "--limit", "1"), ("europepmc",)),
    WalkEntry("semantic-scholar", ("--json", "search", "article", "BRAF", "--source", "semanticscholar", "--limit", "1"), ("s2",)),
    WalkEntry("litsense2", ("--json", "search", "article", "BRAF V600E melanoma", "--source", "litsense2", "--limit", "1"), ("litsense2",)),
    WalkEntry("pubtator3", ("--json", "article", "entities", "22663011"), ("pubtator",)),
    WalkEntry("opencitations", ("--json", "article", "citation-evidence", "22663011", "22663012"), ("opencitations",)),
    WalkEntry("ncbi-id-converter", ("--json", "get", "article", "22663011", "fulltext"), ("ncbi-idconv",)),
    WalkEntry("orcid", ("--json", "author", "papers", "orcid:0000-0002-1825-0097"), ("orcid",)),
    WalkEntry("figshare", ("--json", "get", "article", "22663011", "assets"), ("figshare",)),
    WalkEntry("pmc-oa", ("--json", "get", "article", "22663011", "assets"), ("pmc-oa", "pmc-html")),
    # Local dataset syncs.
    WalkEntry("cdc-cvx", ("--json", "cvx", "sync"), ("cvx", "cvx-tradename", "mvx")),
    WalkEntry("gtr", ("--json", "gtr", "sync"), ("gtr-condition-gene", "gtr-test-version")),
    WalkEntry("who-ivd", ("--json", "who-ivd", "sync"), ("who-ivd",)),
    WalkEntry("who-prequalification", ("--json", "who", "sync"), ("who-pq", "who-pq-api", "who-vaccines")),
)


def _provider_prefix(entry_key: str) -> str | None:
    """The replay request key is `METHOD /prefix/path?query`."""
    path = entry_key.split(" ", 1)[-1] if " " in entry_key else entry_key
    segments = path.lstrip("/").split("/", 1)
    return segments[0] or None


def _prefix_requests(lines: list[dict], prefixes: tuple[str, ...]) -> list[str]:
    return [
        line.get("key", "?")
        for line in lines
        if _provider_prefix(str(line.get("key", ""))) in prefixes
    ]


def test_walk_covers_every_source_id_in_the_registry() -> None:
    """A new source id cannot join the registry without a walk entry."""
    recorded = {record["id"] for record in json.loads(SOURCES_JSON.read_text(encoding="utf-8"))}
    walked = {entry.id for entry in WALK}
    assert walked == recorded, (
        f"walk entries and sources.json ids disagree: "
        f"missing entries={sorted(recorded - walked)} "
        f"unknown entries={sorted(walked - recorded)}"
    )


def test_every_disabled_source_id_stops_its_own_requests() -> None:
    """Disable each id alone; its provider stays off the wire."""
    import os

    binary = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target" / "debug" / "biomcp"))
    assert binary.is_file(), f"built biomcp binary not found at {binary}"

    failures: list[str] = []
    binary = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target" / "debug" / "biomcp"))
    assert binary.is_file(), f"built biomcp binary not found at {binary}"

    with replay_environment(record=False) as replay:
        def run(argv: tuple[str, ...], extra_env: tuple[tuple[str, str], ...]) -> tuple[int, str, list[dict]]:
            run_dir = replay._next_run_dir()
            env = replay.command_env(run_dir)
            env.update(extra_env)
            before = replay.request_log_line_count()
            process = subprocess.run(
                [str(binary), *argv],
                env=env,
                capture_output=True,
                text=True,
                timeout=RUN_TIMEOUT_SECONDS,
                cwd=str(REPO_ROOT),
            )
            lines = replay.request_log_entries(since=before)
            combined = process.stdout + process.stderr
            return process.returncode, combined, lines

        # Baselines first: each routed command, run enabled, must put its
        # own provider on the wire, so a wrong entry fails loudly here
        # instead of passing vacuously.
        baseline_keys: set[tuple] = set()
        for entry in WALK:
            if entry.kind != "routed":
                continue
            key = (entry.argv, entry.extra_env)
            if key in baseline_keys:
                continue
            baseline_keys.add(key)
            code, combined, lines = run(entry.argv, entry.extra_env)
            seen = {prefix for line in lines for prefix in (_provider_prefix(str(line.get("key", ""))),) if prefix}
            missing = [prefix for prefix in entry.prefixes if prefix not in seen]
            if missing:
                failures.append(
                    f"baseline for {entry.id} never reached {missing} "
                    f"(exit {code}); output follows:\n{combined[-2000:]}"
                )

        # The walk: disable one id, run its command, and require the
        # provider's request log to stay silent for it.
        for entry in WALK:
            argv = ("--disable-sources", entry.id, *entry.argv)
            code, combined, lines = run(argv, entry.extra_env)
            if entry.kind == "routed":
                leaked = _prefix_requests(lines, entry.prefixes)
                if leaked:
                    failures.append(
                        f"disabled {entry.id} still made requests: {leaked}; "
                        f"output follows:\n{combined[-2000:]}"
                    )
            else:
                if code != 0:
                    failures.append(
                        f"local-source command for {entry.id} failed (exit {code}); "
                        f"output follows:\n{combined[-2000:]}"
                    )

    assert not failures, "\n\n".join(failures)
