from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

import jsonschema

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = REPO_ROOT / "target" / "release" / "biomcp"
LADDER_WORKFLOW_SLUGS = [
    "treatment-lookup",
    "article-follow-up",
    "variant-pathogenicity",
    "trial-recruitment",
    "mechanism-pathway",
    "pharmacogene-cumulative",
    "mutation-catalog",
]
EXPECTED_SLUGS = [
    "treatment-lookup",
    "symptom-phenotype",
    "gene-disease-orientation",
    "article-follow-up",
    "variant-pathogenicity",
    "drug-regulatory",
    "gene-function-localization",
    "mechanism-pathway",
    "trial-recruitment",
    "pharmacogene-cumulative",
    "disease-locus-mapping",
    "cellular-process-regulation",
    "mutation-catalog",
    "syndrome-disambiguation",
    "negative-evidence",
]
NEW_PLAYBOOK_SLUGS = EXPECTED_SLUGS[4:]
EXPECTED_PLAYBOOK_MARKERS = {
    "variant-pathogenicity": [
        "# Pattern: Variant pathogenicity evidence",
        'biomcp get variant "BRAF V600E" clinvar predictions population',
        'biomcp get variant "BRAF V600E" civic cgi',
        'biomcp variant trials "BRAF V600E" --limit 5',
        'biomcp variant articles "BRAF V600E" --limit 5',
    ],
    "drug-regulatory": [
        "# Pattern: Drug regulatory and approval evidence",
        'biomcp get drug "Gocovri" regulatory --region us',
        'biomcp get drug "zanubrutinib" approvals',
        'biomcp get drug "zanubrutinib" targets',
        'biomcp search article --drug "zanubrutinib" -k "second-generation BTK inhibitor" --limit 5',
    ],
    "gene-function-localization": [
        "# Pattern: Gene function and localization",
        "biomcp get gene OPA1 protein hpa",
        "biomcp get gene OPA1 ontology",
        "biomcp gene pathways OPA1 --limit 5",
        'biomcp search article -g OPA1 -k "mitochondrial intermembrane space localization" --type review --limit 5',
    ],
    "mechanism-pathway": [
        "# Pattern: Mechanism and pathway orientation",
        "biomcp search drug imatinib --limit 5",
        "biomcp get drug imatinib targets regulatory",
        "biomcp get gene ABL1 pathways protein",
        'biomcp search article --drug imatinib -g ABL1 -d "chronic myeloid leukemia" --type review --limit 5',
    ],
    "trial-recruitment": [
        "# Pattern: Trial recruitment check",
        'biomcp search disease "tick-borne encephalitis" --limit 5',
        "biomcp get disease MONDO:0017572",
        'biomcp search trial -c "tick-borne encephalitis" --status recruiting --limit 5',
        'biomcp search article -d "tick-borne encephalitis" --type review --limit 5',
    ],
    "pharmacogene-cumulative": [
        "# Pattern: Pharmacogene cumulative evidence",
        "biomcp search pgx -d warfarin --limit 10",
        "biomcp get pgx warfarin recommendations annotations",
        'biomcp search article --drug warfarin -k "CYP2C9 VKORC1 dose response" --limit 10',
        "biomcp article batch 17048007 19794411 19958090",
    ],
    "disease-locus-mapping": [
        "# Pattern: Disease locus and chromosome mapping",
        'biomcp search article -k "Arnold Chiari syndrome chromosome" --type review --limit 10',
        "biomcp article batch 39309470 17103432 12210325",
        'biomcp search article -k "\\"Arnold Chiari\\" deletion duplication trisomy chromosome" --limit 10',
        "biomcp article batch 12522795 15742475 29410707",
    ],
    "cellular-process-regulation": [
        "# Pattern: Cellular process regulation",
        "biomcp get gene NANOG",
        "biomcp get gene NANOG ontology",
        "biomcp gene pathways NANOG --limit 5",
        'biomcp search article -g NANOG -k "cell cycle G1 S transition" --limit 5',
    ],
    "mutation-catalog": [
        "# Pattern: Mutation catalog for one gene and disease",
        "biomcp get gene PLN",
        "biomcp search variant -g PLN --limit 10",
        "biomcp search variant -g PLN --hgvsp L39X --limit 5",
        "biomcp search article -g PLN -d cardiomyopathy --type review --limit 10",
    ],
    "syndrome-disambiguation": [
        "# Pattern: Syndrome name disambiguation",
        'biomcp search disease "Goldberg-Shprintzen syndrome" --limit 5',
        "biomcp get disease MONDO:0012280 phenotypes",
        'biomcp search disease "Shprintzen-Goldberg syndrome" --limit 5',
        'biomcp search article -k "\\"Goldberg-Shprintzen\\" \\"Shprintzen-Goldberg\\"" --type review --limit 5',
    ],
    "negative-evidence": [
        "# Pattern: Negative evidence and no-association checks",
        'biomcp search article -k "\\"Borna disease virus\\" \\"brain tumor\\"" --type review --limit 5',
        'biomcp search disease "Borna disease" --limit 5',
        'biomcp search article -k "\\"Borna disease virus\\" glioma association" --limit 5',
        'biomcp search article -k "\\"Notch\\" CADASIL Pick prion neurodegenerative" --type review --limit 5',
    ],
}
REMOVED_ACTIVE_SLUGS = [
    "variant-to-treatment",
    "drug-investigation",
    "gene-function-lookup",
    "trial-searching",
    "literature-synthesis",
]


def _require_release_binary() -> Path:
    assert RELEASE_BIN.exists(), f"missing release binary: {RELEASE_BIN}"
    return RELEASE_BIN


def _run_bytes(*args: str) -> bytes:
    binary = _require_release_binary()
    result = subprocess.run(
        [str(binary), *args],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    )
    return result.stdout


def _run_text(*args: str) -> str:
    return _run_bytes(*args).decode("utf-8")


def _listed_slugs(*args: str) -> list[str]:
    listing = _run_text(*args)
    return re.findall(r"^\d{2} ([a-z0-9-]+) -", listing, flags=re.MULTILINE)


def _use_case_path(slug: str) -> Path:
    matches = sorted((REPO_ROOT / "skills" / "use-cases").glob(f"[0-9][0-9]-{slug}.md"))
    assert len(matches) == 1, f"expected one use-case file for {slug}, found {matches}"
    return matches[0]


def _read_use_case(slug: str) -> str:
    return _use_case_path(slug).read_text(encoding="utf-8")


def _bash_block(markdown: str) -> str:
    assert markdown.count("```bash") == 1
    assert markdown.count("```") == 2
    return markdown.split("```bash\n", 1)[1].split("\n```", 1)[0]


def _bash_commands(markdown: str) -> list[str]:
    return [line.strip() for line in _bash_block(markdown).splitlines() if line.strip()]


def test_skill_prompt_render_install_and_slug_surfaces_match(tmp_path: Path) -> None:
    overview_stdout = _run_bytes("skill")
    render_stdout = _run_bytes("skill", "render")

    assert overview_stdout == render_stdout
    assert render_stdout.endswith(b"\n")
    assert not render_stdout.endswith(b"\n\n")

    prompt = render_stdout.decode("utf-8")
    assert 'biomcp suggest "<question>"' in prompt
    assert prompt.index('biomcp suggest "<question>"') < prompt.index("## Routing rules")
    for marker in (
        "## Routing rules",
        "## Section reference",
        "## Cross-entity pivot rules",
        "## How-to reference",
        "## Anti-patterns",
        "## Output and evidence rules",
        "## Answer commitment",
    ):
        assert marker in prompt
    assert "../docs/" not in prompt
    assert ".md)" not in prompt

    agent_root = tmp_path / "agent"
    _run_text("skill", "install", str(agent_root), "--force")
    installed_root = agent_root / "skills" / "biomcp"
    assert (installed_root / "SKILL.md").read_bytes() == render_stdout

    slugs = _listed_slugs("skill", "list")
    assert slugs == EXPECTED_SLUGS
    assert _listed_slugs("list", "skill") == slugs
    for slug in slugs:
        expected = _read_use_case(slug) + "\n"
        body = _run_text("skill", slug)
        assert body == expected

    installed_use_case_slugs = [
        path.stem[3:] for path in sorted((installed_root / "use-cases").glob("[0-9][0-9]-*.md"))
    ]
    assert installed_use_case_slugs == slugs

    assert _run_text("skill", "05") == _read_use_case("variant-pathogenicity") + "\n"
    assert _run_text("skill", "mutation", "catalog") == _read_use_case("mutation-catalog") + "\n"

    for slug, markers in EXPECTED_PLAYBOOK_MARKERS.items():
        body = _read_use_case(slug)
        for marker in markers:
            assert marker in body

    for slug in NEW_PLAYBOOK_SLUGS:
        body = _read_use_case(slug)
        physical_lines = body.splitlines()
        assert 15 <= len(physical_lines) <= 30

        lines_after_h1 = body.splitlines()[1:]
        first_description = next(line.strip() for line in lines_after_h1 if line.strip())
        assert first_description.startswith("Use this when")

        commands = [line.strip() for line in _bash_block(body).splitlines() if line.strip()]
        assert 3 <= len(commands) <= 4
        for command in commands:
            assert command.startswith("biomcp ")

        interpretation_bullets = re.findall(r"^- ", body, flags=re.MULTILINE)
        assert 3 <= len(interpretation_bullets) <= 5

    examples_readme = (REPO_ROOT / "examples" / "README.md").read_text(encoding="utf-8")
    listing = _run_text("skill", "list")
    list_skill_listing = _run_text("list", "skill")
    for removed in REMOVED_ACTIVE_SLUGS:
        assert removed not in listing
        assert removed not in list_skill_listing
        assert removed not in prompt
        assert removed not in examples_readme


def test_workflow_ladder_sidecars_match_schema_and_playbooks() -> None:
    schema_path = REPO_ROOT / "skills" / "schemas" / "workflow-ladder.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    sidecars = sorted((REPO_ROOT / "skills" / "use-cases").glob("*.ladder.json"))

    assert [path.stem.removesuffix(".ladder") for path in sidecars] == sorted(
        LADDER_WORKFLOW_SLUGS
    )

    for path in sidecars:
        slug = path.stem.removesuffix(".ladder")
        sidecar = json.loads(path.read_text(encoding="utf-8"))
        jsonschema.validate(sidecar, schema)

        assert sidecar["workflow"] == slug
        assert path.name == f"{sidecar['workflow']}.ladder.json"
        assert sidecar["playbook"] == f"biomcp skill {slug}"

        playbook = _use_case_path(slug)
        assert playbook.name.endswith(f"-{slug}.md")
        playbook_commands = _bash_commands(playbook.read_text(encoding="utf-8"))
        ladder = sidecar["ladder"]
        assert isinstance(ladder, list)
        assert [step["step"] for step in ladder] == list(range(1, len(ladder) + 1))
        assert [step["command"] for step in ladder] == playbook_commands

        for step in ladder:
            assert step["what_it_gives"].strip()
            assert not re.search(r"<[^>]+>", step["command"])


def test_installed_output_schemas_allow_workflow_ladder_meta() -> None:
    for schema_path in sorted((REPO_ROOT / "skills" / "schemas").glob("*.json")):
        if schema_path.name == "workflow-ladder.schema.json":
            continue
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        meta_properties = schema["properties"]["_meta"]["properties"]
        assert "workflow" in meta_properties
        assert "ladder" in meta_properties
        assert meta_properties["ladder"]["items"]["required"] == [
            "step",
            "command",
            "what_it_gives",
        ]


def test_rust_sources_do_not_embed_workflow_ladder_commands() -> None:
    commands: list[str] = []
    for sidecar_path in sorted((REPO_ROOT / "skills" / "use-cases").glob("*.ladder.json")):
        sidecar = json.loads(sidecar_path.read_text(encoding="utf-8"))
        commands.extend(step["command"] for step in sidecar["ladder"])

    offenders: list[str] = []
    for rust_path in sorted((REPO_ROOT / "src").rglob("*.rs")):
        relative = rust_path.relative_to(REPO_ROOT)
        if rust_path.name == "tests.rs" or "tests" in rust_path.parts:
            continue
        if str(relative) in {"src/cli/article/mod.rs", "src/cli/commands.rs"}:
            continue
        text = rust_path.read_text(encoding="utf-8")
        for command in commands:
            if command in text:
                offenders.append(f"{relative} embeds {command!r}")

    assert offenders == []
