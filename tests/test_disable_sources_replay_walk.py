"""The disable-sources walk: every source id, one command, no request.

Ticket 2124's outcome says a disabled source is never called by any
command. This proof walks every id in `docs/reference/sources.json`,
disables that one id, runs a command that uses the source against the
golden suite's replay server, and asserts the provider's request log
stays quiet for it. An enabled baseline run of the same command proves
the command really uses the source, so a renamed route or an entry whose
chain never reaches the wire cannot pass silently.

Each routed entry names the provider prefixes the replay server derives
from `src/sources` (the first path segment of a request). Entries marked
`carried` cover sources the recorded replies cannot put on the wire in
this environment — fields another provider passes along, a local dataset
with no routable download URL, or a chain that stops short of the
source — and each names why. A carried entry still runs its command with
the source disabled, asserts the run answers with the same exit status
as the enabled baseline (the disable must not break a command whose
source has no reachable client), and, where the source owns an exclusive
wire prefix, asserts that prefix stays silent. The replay server answers
every request from disk and refuses to reach a live provider, so a
branch without the gate cannot leak onto the network from this test
either; it fails on the logged requests instead.
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

# Keys a test run sets only when the command needs them; the replay
# server ignores authentication, so placeholder values suffice.
ONCOKB_ENV = (("ONCOKB_TOKEN", "replay-walk-token"),)
NCI_ENV = (("NCI_API_KEY", "replay-walk-key"),)
DISGENET_ENV = (("DISGENET_API_KEY", "replay-walk-key"),)
UMLS_ENV = (("UMLS_API_KEY", "replay-walk-key"),)
ORCID_ENV = (("ORCID_ACCESS_TOKEN", "replay-walk-token"),)


@dataclass(frozen=True)
class WalkEntry:
    """One source id, the command that uses it, and its wire prefixes."""

    id: str
    argv: tuple[str, ...]
    prefixes: tuple[str, ...] = ()
    kind: str = "routed"
    extra_env: tuple[tuple[str, str], ...] = field(default=())


WALK: tuple[WalkEntry, ...] = (
    # Variant card sections and helpers.
    WalkEntry("civic", ("--json", "get", "gene", "BRAF", "civic"), ("civic",)),
    WalkEntry("clinvar", ("--json", "get", "variant", "BRAF V600E", "clinvar"), ("clinvar",)),
    WalkEntry("dbsnp", ("--json", "get", "variant", "BRAF V600E", "population"), ("dbsnp",)),
    WalkEntry("gnomad", ("--json", "get", "variant", "BRAF V600E", "population"), ("gnomad",)),
    WalkEntry("gwas-catalog", ("--json", "get", "variant", "BRAF V600E", "gwas"), ("gwas",)),
    WalkEntry("myvariant-info", ("--json", "get", "variant", "BRAF V600E", "predictions"), ("myvariant",)),
    # The variant card queries the cbioPortal API; its data-hub study
    # download endpoint shares the id and stays unqueried here.
    WalkEntry("cbioportal", ("--json", "get", "variant", "BRAF V600E", "cbioportal"), ("cbioportal",)),
    WalkEntry("oncokb", ("--json", "variant", "oncokb", "BRAF V600E"), ("oncokb",), extra_env=ONCOKB_ENV),
    WalkEntry("mutalyzer", ("--json", "variant", "normalize", "mutalyzer", "NM_004448.2:c.829G>A"), ("mutalyzer",)),
    WalkEntry("variantvalidator", ("--json", "variant", "normalize", "variantvalidator", "NM_004448.2:c.829G>T"), ("variantvalidator",)),
    WalkEntry("clingen_car", ("--json", "variant", "normalize", "car", "NM_000546.6:c.215C>G"), ("clingen-car",)),
    WalkEntry("clingen_erepo", ("--json", "variant", "erepo", "CA015543"), ("clingen-erepo",)),
    WalkEntry("clingen_cspec", ("--json", "gene", "cspec", "BRAF"), ("cspec",)),
    # Carried in the MyVariant payload; the section reads them without a
    # request of their own.
    WalkEntry("cosmic", ("--json", "get", "variant", "BRAF V600E", "cosmic"), kind="carried"),
    WalkEntry("cancer-genome-interpreter", ("--json", "get", "variant", "BRAF V600E", "cgi"), kind="carried"),
    # The predict section needs the alphagenome feature; the routine gate
    # builds without it, and `make full-feature-check` owns that client.
    WalkEntry("alphagenome", ("--json", "get", "variant", "chr7:g.140453136A>T", "predict"), kind="carried"),
    # Reached only through the article full-text and asset chains, which
    # stop at article resolution on the recorded replies; the LDH client
    # sits behind identity verification there, and OpenCitations behind
    # the citation-evidence chain.
    WalkEntry("clingen_ldh", ("--json", "variant", "articles", "BRAF V600E", "--verify-identity", "--limit", "1"), ("ldh",), kind="carried"),
    WalkEntry("ncbi-e-utilities", ("--json", "get", "article", "42875213", "fulltext"), kind="carried"),
    WalkEntry("ncbi-id-converter", ("--json", "get", "article", "42875213", "fulltext"), ("ncbi-idconv",), kind="carried"),
    WalkEntry("figshare", ("--json", "get", "article", "42875213", "assets"), ("figshare",), kind="carried"),
    WalkEntry("pmc-oa", ("--json", "get", "article", "42875213", "assets"), ("pmc-oa", "pmc-html"), kind="carried"),
    WalkEntry("opencitations", ("--json", "article", "citation-evidence", "42875213", "42870784"), ("opencitations",), kind="carried"),
    # Structure helper and protein card.
    WalkEntry("cancerhotspots", ("--json", "variant", "structure", "BRAF V600E"), ("cancerhotspots",)),
    WalkEntry("interpro", ("--json", "variant", "structure", "BRAF V600E"), ("interpro",)),
    WalkEntry("uniprot", ("--json", "get", "protein", "P15056"), ("uniprot",)),
    WalkEntry("complexportal", ("--json", "get", "protein", "P15056", "complexes"), ("complexportal",)),
    # PDB and AlphaFold identifiers ride the UniProt record; the
    # structures section that names them stops on the recorded reply.
    WalkEntry("pdb", ("--json", "get", "protein", "P15056", "domains"), kind="carried"),
    WalkEntry("alphafold-db", ("--json", "get", "protein", "P15056", "domains"), kind="carried"),
    # Gene card and enrichment.
    WalkEntry("mygene-info", ("--json", "get", "gene", "BRAF"), ("mygene",)),
    WalkEntry("string", ("--json", "get", "gene", "BRAF", "interactions"), ("string",)),
    WalkEntry("clingen", ("--json", "get", "gene", "BRAF", "clingen"), ("clingen",)),
    WalkEntry("disgenet", ("--json", "get", "gene", "BRAF", "disgenet"), ("disgenet",), extra_env=DISGENET_ENV),
    WalkEntry("nih-reporter", ("--json", "get", "gene", "BRAF", "funding"), ("nih-reporter",)),
    WalkEntry("quickgo", ("--json", "get", "gene", "BRAF", "go"), ("quickgo",)),
    WalkEntry("gtex", ("--json", "get", "gene", "BRAF", "expression"), ("gtex",)),
    WalkEntry("human-protein-atlas", ("--json", "get", "gene", "BRAF", "hpa"), ("hpa",)),
    WalkEntry("enrichr", ("--json", "get", "gene", "BRAF", "ontology"), ("enrichr",)),
    WalkEntry("dgidb", ("--json", "get", "gene", "BRAF", "druggability"), ("dgidb",)),
    WalkEntry("opentargets", ("--json", "get", "gene", "BRAF", "druggability"), ("opentargets",)),
    WalkEntry("gprofiler", ("--json", "enrich", "BRAF,KRAS", "--limit", "3"), ("gprofiler",)),
    WalkEntry("kegg", ("--json", "search", "pathway", "MAPK signaling pathway", "--limit", "2"), ("kegg",)),
    WalkEntry("reactome", ("--json", "search", "pathway", "MAPK signaling pathway", "--limit", "2"), ("reactome",)),
    WalkEntry("wikipathways", ("--json", "search", "pathway", "MAPK signaling pathway", "--limit", "2"), ("wikipathways",)),
    # Disease and phenotype.
    WalkEntry("mydisease-info", ("--json", "get", "disease", "MONDO:0007947"), ("mydisease",)),
    WalkEntry("monarch-initiative", ("--json", "get", "disease", "MONDO:0007947", "all"), ("monarch",)),
    WalkEntry("seer-explorer", ("--json", "get", "disease", "MONDO:0007947", "survival"), ("seer",)),
    WalkEntry("hpo-jax-api", ("--json", "search", "phenotype", "HP:0001250,HP:0001263"), ("hpo",)),
    WalkEntry("ols4", ("--json", "discover", "BRAF"), ("ols4",)),
    WalkEntry("medlineplus", ("--json", "discover", "BRAF"), ("medlineplus",)),
    WalkEntry("umls", ("--json", "discover", "chest pain"), ("umls",), extra_env=UMLS_ENV),
    # MONDO and Disease Ontology names ride the MyDisease record.
    WalkEntry("mondo", ("--json", "get", "disease", "MONDO:0007947"), kind="carried"),
    WalkEntry("disease-ontology", ("--json", "get", "disease", "MONDO:0007947"), kind="carried"),
    # Drugs and pharmacogenomics.
    WalkEntry("mychem-info", ("--json", "search", "drug", "imatinib", "--limit", "1"), ("mychem",)),
    WalkEntry("openfda", ("--json", "search", "adverse-event", "--drug", "pembrolizumab", "--source", "faers", "--limit", "1"), ("openfda",)),
    WalkEntry("cpic", ("--json", "search", "pgx", "-g", "CYP2D6"), ("cpic",)),
    # The drug card's targets, regulatory rows, DDInter interactions,
    # VAERS lookups, and PharmacoDB responses sit behind the MyChem name
    # search, the CVX dataset, or the Cellosaurus lookup, none of which
    # the recorded replies carry; DrugBank fields ride the MyChem record.
    WalkEntry("chembl", ("--json", "get", "drug", "pembrolizumab", "targets"), ("chembl",), kind="carried"),
    WalkEntry("fda-orphan-designations", ("--json", "get", "drug", "pembrolizumab", "regulatory"), ("fda-orphan",), kind="carried"),
    WalkEntry("ddinter", ("--json", "drug", "interactions", "warfarin"), kind="carried"),
    WalkEntry("vaers", ("--json", "search", "adverse-event", "--drug", "pembrolizumab", "--source", "vaers", "--limit", "1"), ("vaers",), kind="carried"),
    WalkEntry("drugbank", ("--json", "get", "drug", "pembrolizumab"), kind="carried"),
    WalkEntry("drugs-at-fda", ("--json", "get", "drug", "pembrolizumab", "regulatory"), kind="carried"),
    WalkEntry("pharmgkb", ("--json", "search", "pgx", "-g", "CYP2D6"), ("pharmgkb",)),
    # The GenCC store refuses an empty per-run dataset directory before
    # any download; the gene card's GenCC section reads that store.
    WalkEntry("gencc", ("--json", "gencc", "sync"), ("gencc",), kind="carried"),
    # Cell lines.
    WalkEntry("cellosaurus", ("--json", "get", "cell-line", "CVCL_2119"), ("cellosaurus",)),
    WalkEntry("pharmacodb", ("--json", "cell-line", "drug-response", "CVCL_2119", "--dataset", "GDSC1"), ("pharmacodb",), kind="carried"),
    WalkEntry("depmap", ("--json", "cell-line", "drug-response", "CVCL_2119", "--dataset", "GDSC1"), kind="carried"),
    WalkEntry("cell-model-passports", ("--json", "get", "cell-line", "CVCL_2119"), kind="carried"),
    # Trials and articles.
    WalkEntry("clinicaltrials-gov", ("--json", "search", "trial", "melanoma", "--limit", "1"), ("ctgov",)),
    WalkEntry("nci-cts", ("--json", "search", "trial", "melanoma", "--source", "nci", "--limit", "1"), ("nci-cts",), extra_env=NCI_ENV),
    WalkEntry("pubmed", ("--json", "search", "article", "BRAF", "--source", "pubmed", "--limit", "1"), ("pubmed",)),
    WalkEntry("europe-pmc", ("--json", "search", "article", "BRAF", "--source", "europepmc", "--limit", "1"), ("europepmc",)),
    WalkEntry("semantic-scholar", ("--json", "search", "article", "BRAF", "--source", "semanticscholar", "--limit", "1"), ("s2",)),
    WalkEntry("litsense2", ("--json", "search", "article", "BRAF V600E melanoma", "--source", "litsense2", "--limit", "1"), ("litsense2",)),
    WalkEntry("pubtator3", ("--json", "article", "entities", "22663011"), ("pubtator",)),
    WalkEntry("orcid", ("--json", "author", "papers", "orcid:0000-0002-1825-0097"), ("orcid",), extra_env=ORCID_ENV),
    # Local dataset syncs.
    WalkEntry("cdc-cvx", ("--json", "cvx", "sync"), ("cvx", "cvx-tradename", "mvx")),
    # The sync's version probe is the request the replay store sees; the
    # condition-gene download follows only when the version differs.
    WalkEntry("gtr", ("--json", "gtr", "sync"), ("gtr-test-version",)),
    WalkEntry("who-ivd", ("--json", "who-ivd", "sync"), ("who-ivd",)),
    WalkEntry("who-prequalification", ("--json", "who", "sync"), ("who-pq", "who-pq-api", "who-vaccines")),
    WalkEntry("ema", ("--json", "ema", "sync"), ("ema-report",)),
)


def _provider_prefix(entry_key: str) -> str | None:
    """The replay request key is `METHOD /prefix/path?query`."""
    path = entry_key.split(" ", 1)[-1] if " " in entry_key else entry_key
    segments = path.lstrip("/").split("/", 1)
    return segments[0] or None


def _prefix_requests(lines: list[dict], prefixes: tuple[str, ...]) -> list[str]:
    return [
        str(line.get("key", "?"))
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
    binary = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target" / "debug" / "biomcp"))
    assert binary.is_file(), f"built biomcp binary not found at {binary}"

    failures: list[str] = []
    with replay_environment(record=False) as replay:

        def run(argv: tuple[str, ...], extra_env: tuple[tuple[str, str], ...]):
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

        # Baselines: each distinct command runs once, enabled. A routed
        # entry's provider must appear on the wire, and a carried entry
        # records the exit status its disabled run must reproduce.
        expected_exit: dict[tuple, int] = {}
        baseline_keys: set[tuple] = set()
        for entry in WALK:
            key = (entry.argv, entry.extra_env)
            if key in baseline_keys:
                continue
            baseline_keys.add(key)
            code, combined, lines = run(entry.argv, entry.extra_env)
            expected_exit[key] = code
            if entry.kind != "routed":
                continue
            seen = {
                prefix
                for line in lines
                for prefix in (_provider_prefix(str(line.get("key", ""))),)
                if prefix
            }
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
                if entry.prefixes:
                    leaked = _prefix_requests(lines, entry.prefixes)
                    if leaked:
                        failures.append(
                            f"carried {entry.id} unexpectedly reached the wire: {leaked}; "
                            f"output follows:\n{combined[-2000:]}"
                        )
                baseline = expected_exit[(entry.argv, entry.extra_env)]
                if code != baseline:
                    failures.append(
                        f"carried {entry.id} changed the command's outcome "
                        f"(exit {code} with the source disabled, {baseline} enabled); "
                        f"output follows:\n{combined[-2000:]}"
                    )

    assert not failures, "\n\n".join(failures)
