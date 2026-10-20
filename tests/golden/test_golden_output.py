"""Golden output cases: each command's JSON and Markdown against committed files.

Every case runs the real binary against the replay server. The golden file
pins the command's standard output byte for byte; a deliberate format change
is `pytest tests/golden --update`, read the diff, commit. A case also asserts
that its requests replayed from disk: any miss names the recording to make.
"""

from __future__ import annotations

import difflib
import json
import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

import pytest

from replay_env import REPO_ROOT, ReplayEnvironment

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module

CASES_DIR = Path(__file__).resolve().parent / "cases"
BINARY = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
OVERRIDE_NAME_RE = re.compile(r'"(BIOMCP_[A-Z0-9_]+)"')
OVERRIDE_SUFFIXES = ("_BASE", "_BASE_URL", "_FIXTURE_ORIGIN", "_URL")
NON_PROVIDER_OVERRIDES = frozenset(
    {"BIOMCP_HEALTH_PROBE_BASE", "BIOMCP_S2_PDF_FIXTURE_ORIGIN"}
)
RECORD_PASSES = 3
RECORD_TIMEOUT_SECONDS = 600.0
# search-all JSON embeds the command's own wall time. A golden file pins the
# output format, not the stopwatch, so the measured value is blanked on both
# sides of the comparison.
WALL_TIME_RE = re.compile(r'("wall_time_ms": )\d+')


@dataclass(frozen=True)
class Case:
    id: str
    argv: tuple[str, ...]
    formats: tuple[str, ...] = ("json", "markdown")


CASES = (
    Case(
        "get-disease-mondo0005105",
        ("get", "disease", "MONDO:0005105"),
    ),
    Case(
        "get-pgx-cyp2d6-frequencies",
        ("get", "pgx", "CYP2D6", "frequencies"),
    ),
    Case(
        "get-pgx-cyp2d6-guidelines",
        ("get", "pgx", "CYP2D6", "guidelines"),
    ),
    Case(
        "get-article-23056577-fulltext",
        ("--disable-sources", "semantic-scholar,figshare", "get", "article", "23056577", "fulltext"),
    ),
    Case(
        "get-article-23056577-assets",
        ("--disable-sources", "semantic-scholar,figshare", "get", "article", "23056577", "assets"),
        formats=("json",),
    ),
    Case(
        'get-gene-braf-go-interactions-hpa',
        (
            'get',
            'gene',
            'BRAF',
            'go',
            'interactions',
            'hpa',
        ),
    ),
    Case(
        'get-gene-egfr-druggability',
        (
            'get',
            'gene',
            'EGFR',
            'druggability',
        ),
    ),
    Case(
        'get-gene-erbb2-funding',
        (
            'get',
            'gene',
            'ERBB2',
            'funding',
        ),
    ),
    Case(
        'get-cell-line-cvcl0005-xrefs',
        (
            'get',
            'cell-line',
            'CVCL_0005',
            'xrefs',
        ),
    ),
    Case(
        'get-cell-line-cvcl0064-variants-xrefs',
        (
            'get',
            'cell-line',
            'CVCL_0064',
            'variants',
            'xrefs',
        ),
    ),
    Case(
        'get-cell-line-cvcl1844-variants',
        (
            'get',
            'cell-line',
            'CVCL_1844',
            'variants',
        ),
    ),
    Case(
        'search-cell-line-kg1-limit3',
        (
            'search',
            'cell-line',
            'KG1',
            '--limit',
            '3',
        ),
    ),
    Case(
        'get-article-20516115',
        (
            '--disable-sources',
            'semantic-scholar',
            'get',
            'article',
            '20516115',
        ),
    ),
    Case(
        'get-article-30738221-annotations',
        (
            '--disable-sources',
            'semantic-scholar',
            'get',
            'article',
            '30738221',
            'annotations',
        ),
    ),
    Case(
        'article-entities-30738221',
        (
            '--disable-sources',
            'semantic-scholar',
            'article',
            'entities',
            '30738221',
        ),
    ),
    Case(
        'search-disease-melanoma-limit1',
        ('search', 'disease', 'melanoma', '--limit', '1'),
    ),
    Case(
        'search-variant-braf-melanoma-limit3',
        (
            'search',
            'variant',
            '--gene',
            'BRAF',
            '--condition',
            'melanoma',
            '--limit',
            '3',
        ),
    ),
    Case(
        'search-gwas-braf-limit49',
        ('search', 'gwas', '--gene', 'BRAF', '--limit', '49'),
    ),
    Case(
        'search-gwas-melanoma-limit49',
        ('search', 'gwas', '--trait', 'melanoma', '--limit', '49'),
    ),
    Case(
        'search-pgx-cyp2d6-limit20',
        ('search', 'pgx', '--gene', 'CYP2D6', '--limit', '20'),
    ),
    Case(
        'search-pgx-clopidogrel-limit20',
        ('search', 'pgx', '--drug', 'clopidogrel', '--limit', '20'),
    ),
    Case(
        'get-author-orcid-4299-3018',
        ('get', 'author', 'orcid:0000-0002-4299-3018'),
    ),
    Case(
        'author-papers-orcid-4299-3018-limit3',
        ('author', 'papers', 'orcid:0000-0002-4299-3018', '--limit', '3'),
    ),
    Case(
        'search-protein-braf-limit3',
        ('search', 'protein', 'BRAF', '--limit', '3'),
    ),
    Case(
        'search-phenotype-hp1250-hp1263-limit1',
        ('search', 'phenotype', 'HP:0001250 HP:0001263', '--limit', '1'),
    ),
    Case(
        'get-pgx-cyp2d6-interactions',
        ('get', 'pgx', 'CYP2D6', 'interactions'),
    ),
    Case(
        'get-pgx-cyp2d6-recommendations',
        ('get', 'pgx', 'CYP2D6', 'recommendations'),
    ),
    Case("get-gene-braf", ("get", "gene", "BRAF")),
    Case("get-protein-p15056", ("get", "protein", "P15056")),
    Case(
        "get-protein-p15056-structures",
        ("get", "protein", "P15056", "structures"),
    ),
    Case(
        "protein-structures-p15056-limit1-offset1",
        ("protein", "structures", "P15056", "--limit", "1", "--offset", "1"),
    ),
    Case("search-gene-braf", ("search", "gene", "BRAF", "--limit", "3")),
    Case("get-variant-braf-v600e", ("get", "variant", "BRAF V600E")),
    Case(
        "search-pathway-mapk-long-form-limit3",
        (
            "search",
            "pathway",
            "mitogen activated protein kinase signaling pathway",
            "--limit",
            "3",
        ),
    ),
    Case(
        "search-pathway-mapk-exact-limit3",
        ("search", "pathway", "MAPK signaling pathway", "--limit", "3"),
    ),
    Case(
        "search-pathway-mapk-exact-limit5",
        ("search", "pathway", "MAPK signaling pathway", "--limit", "5"),
    ),
    Case("get-pathway-hsa05200", ("get", "pathway", "hsa05200")),
    Case(
        "get-pathway-r-hsa-5673001-genes-events",
        ("get", "pathway", "R-HSA-5673001", "genes", "events"),
    ),
    Case(
        "get-trial-nct02576665-references",
        ("get", "trial", "NCT02576665", "--source", "ctgov", "references"),
    ),
    Case(
        "get-trial-nct06131398-references",
        ("get", "trial", "NCT06131398", "--source", "ctgov", "references"),
    ),
    Case(
        "search-trial-melanoma-recruiting",
        ("search", "trial", "-c", "melanoma", "-s", "recruiting", "--limit", "3"),
    ),
    Case(
        "search-all-braf-melanoma",
        ("search", "all", "--gene", "BRAF", "--disease", "melanoma"),
    ),
    Case(
        "search-all-braf-melanoma-counts-only",
        ("search", "all", "--gene", "BRAF", "--disease", "melanoma", "--counts-only"),
    ),
)
FORMAT_SUFFIX = {"json": ".json", "markdown": ".md"}


def _normalize(text: str) -> str:
    return WALL_TIME_RE.sub(r"\1<WALL_TIME_MS>", text)


def _golden_path(case: Case, output_format: str) -> Path:
    return CASES_DIR / f"{case.id}{FORMAT_SUFFIX[output_format]}"


def _failed_requests(run) -> list[dict]:
    return [
        entry
        for entry in run.request_log_lines
        if entry.get("result") in ("miss", "record-error")
    ]


def _run_case(
    replay: ReplayEnvironment, case: Case, output_format: str
) -> subprocess.CompletedProcess:
    """Run the case until its requests all replay, then return that run.

    In a recording run the first pass captures live replies; the last pass
    must be miss-free so a golden file never pins a live call. In a replay
    run one pass is enough and a miss fails the case.
    """
    argv = [*case.argv]
    if output_format == "json":
        argv.append("--json")
    timeout = RECORD_TIMEOUT_SECONDS if replay.record else 120.0
    run = replay.run(BINARY, tuple(argv), timeout=timeout)
    passes = 1
    while replay.record and _failed_requests(run) and passes < RECORD_PASSES:
        run = replay.run(BINARY, tuple(argv), timeout=timeout)
        passes += 1
    failures = _failed_requests(run)
    assert not failures, (
        f"{case.id} ({output_format}) made requests with no recorded reply:\n"
        + "\n".join(json.dumps(entry, sort_keys=True) for entry in failures)
    )
    assert run.process.returncode == 0, (
        f"{case.id} ({output_format}) exited {run.process.returncode}\n"
        f"stderr:\n{run.process.stderr}"
    )
    # The CLI reports its managed download path. Only this run's private
    # cache prefix varies; keep the basename and all output around it pinned.
    cache = run.run_dir / "cache"
    # macOS may expose the same temporary root through /var and /private/var.
    roots = {str(cache), str(cache.resolve())}
    for root in sorted(roots, key=len, reverse=True):
        run.process.stdout = run.process.stdout.replace(
            json.dumps(root)[1:-1], "<CACHE_ROOT>"
        ).replace(root, "<CACHE_ROOT>")
    return run.process


def _diff(expected: str, actual: str, name: str) -> str:
    return "".join(
        difflib.unified_diff(
            expected.splitlines(keepends=True),
            actual.splitlines(keepends=True),
            fromfile=f"{name} (committed)",
            tofile=f"{name} (actual)",
        )
    )


@pytest.mark.parametrize(
    "case, output_format",
    [(case, fmt) for case in CASES for fmt in case.formats],
    ids=[f"{case.id}-{fmt}" for case in CASES for fmt in case.formats],
)
def test_golden_output(
    case: Case,
    output_format: str,
    replay: ReplayEnvironment,
    golden_update: bool,
) -> None:
    process = _run_case(replay, case, output_format)
    output = _normalize(process.stdout)
    if output_format == "json":
        json.loads(process.stdout)
    golden = _golden_path(case, output_format)
    if golden_update:
        golden.parent.mkdir(parents=True, exist_ok=True)
        golden.write_text(output, encoding="utf-8")
        return
    assert golden.is_file(), f"{golden} is missing; run: pytest tests/golden --update"
    expected = golden.read_text(encoding="utf-8")
    assert expected == output, _diff(expected, output, golden.name)


def test_replay_env_covers_every_provider_override(
    replay: ReplayEnvironment,
) -> None:
    """No provider override in src may bypass the replay server."""
    names = set()
    for path in sorted((REPO_ROOT / "src").rglob("*.rs")):
        for name in OVERRIDE_NAME_RE.findall(path.read_text(encoding="utf-8")):
            if name.endswith(OVERRIDE_SUFFIXES):
                names.add(name)
    uncovered = sorted(names - NON_PROVIDER_OVERRIDES - set(replay.provider_env))
    assert not uncovered, f"provider overrides missing from the replay env: {uncovered}"


def test_article_supplement_stdout_preserves_recorded_bytes(
    replay: ReplayEnvironment,
) -> None:
    run = replay.run(
        BINARY,
        (
            "--disable-sources",
            "semantic-scholar,figshare",
            "get",
            "article",
            "23056577",
            "asset",
            "pone.0047054.s002.doc",
        ),
        text=False,
    )
    assert not _failed_requests(run), run.request_log_lines
    assert run.process.returncode == 0, run.process.stderr
    recorded = REPO_ROOT / "testdata/replay/responses/pmc-oa/pmc3467229-s002.doc.bin"
    assert run.process.stdout == recorded.read_bytes()
