from __future__ import annotations

import importlib.util
import json
import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
WRAPPER_SCRIPT = REPO_ROOT / "tools" / "check-quality-ratchet.sh"
EXCEPTION_REGISTRY = REPO_ROOT / "tools" / "cli-surface-contract-exceptions.json"
QUALITY_RATCHET_TOOL = REPO_ROOT / "tools" / "check-quality-ratchet.py"


def _load_quality_ratchet_module():
    spec = importlib.util.spec_from_file_location("check_quality_ratchet", QUALITY_RATCHET_TOOL)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def test_quality_ratchet_runs_whole_surface_cli_contract(tmp_path: Path) -> None:
    output_dir = tmp_path / "ratchet-output"
    env = os.environ.copy()
    env["QUALITY_RATCHET_OUTPUT_DIR"] = str(output_dir)
    env["QUALITY_RATCHET_AUDITS"] = "cli_surface_contract"

    result = subprocess.run(
        ["bash", str(WRAPPER_SCRIPT)],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    summary_path = output_dir / "quality-ratchet-summary.json"
    assert summary_path.exists(), "quality ratchet did not write summary artifact"
    summary = json.loads(summary_path.read_text(encoding="utf-8"))

    assert "cli_surface_contract" in summary, (
        "quality ratchet must include the whole-surface CLI contract lane so "
        "make lint fails on help/list/docs/spec/JSON drift"
    )
    assert summary["cli_surface_contract"]["status"] == "pass"

    detail_path = output_dir / "quality-ratchet-cli-surface-contract.json"
    assert detail_path.exists(), "whole-surface CLI contract artifact is missing"
    detail = json.loads(detail_path.read_text(encoding="utf-8"))
    assert detail["status"] == "pass"
    assert detail["exception_registry"] == "tools/cli-surface-contract-exceptions.json"
    assert detail["checks"] == [
        "public_flags_and_value_aliases_documented",
        "list_and_reference_docs_cover_public_commands",
        "runnable_helpers_are_discoverable_in_list_pages",
        "json_entity_surfaces_include_next_commands_or_exception",
        "copy_paste_examples_are_shell_safe",
        "entities_do_not_depend_on_markdown_shell_quoting",
        "trial_status_vocabulary_documented",
        "author_entity_present_in_entity_tables",
        "release_process_versions_match_package_metadata",
    ]
    checked_surfaces = set(detail["checked_surfaces"])
    assert "docs/user-guide/variant.md" in checked_surfaces
    assert "spec/surface/cli-contract-ratchet.md" in checked_surfaces


def test_cli_surface_contract_flags_must_be_documented_outside_source(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    cli_dir = root / "src" / "cli"
    cli_dir.mkdir(parents=True)
    (cli_dir / "demo.rs").write_text(
        '#[arg(long = "date-to", visible_alias = "until")]\nfield: Option<String>,\n',
        encoding="utf-8",
    )

    module = _load_quality_ratchet_module()
    result = module.check_public_flags_and_value_aliases_documented(root, {"docs.md": ""})

    assert result["status"] == "fail"
    assert result["findings"] == [
        {
            "token": "until",
            "message": "public visible/value alias is accepted by clap but absent from help/list/docs/spec evidence",
        }
    ]


def test_cli_surface_contract_catches_runnable_helper_missing_from_list_page(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    for relative in [
        "src/cli/drug",
        "src/cli/disease",
        "src/cli/variant",
        "src/cli/list",
    ]:
        (root / relative).mkdir(parents=True, exist_ok=True)
    (root / "src/cli/drug/mod.rs").write_text(
        "pub enum DrugCommand {\n    Trials { name: String },\n    Interactions { name: String },\n    External(Vec<String>),\n}\n",
        encoding="utf-8",
    )
    (root / "src/cli/disease/mod.rs").write_text(
        "pub enum DiseaseCommand {\n    Trials { name: String },\n}\n",
        encoding="utf-8",
    )
    (root / "src/cli/variant/mod.rs").write_text(
        "pub enum VariantCommand {\n    Structure { id: String },\n}\n",
        encoding="utf-8",
    )
    (root / "src/cli/list/clinical.rs").write_text(
        "- `drug trials <name>`\n- `disease trials <name>`\n",
        encoding="utf-8",
    )
    (root / "src/cli/list/molecular.rs").write_text(
        "- `variant structure <variant>`\n",
        encoding="utf-8",
    )

    module = _load_quality_ratchet_module()
    result = module.check_runnable_helpers_are_discoverable_in_list_pages(root)

    assert result["status"] == "fail"
    assert result["findings"] == [
        {
            "command": "biomcp drug interactions",
            "surface": "biomcp list drug",
            "message": "runnable helper command is missing from matching list page discovery text",
        }
    ]


def test_cli_surface_contract_rejects_unquoted_hgvs_redirect_examples(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    root.mkdir()

    module = _load_quality_ratchet_module()
    result = module.check_copy_paste_examples_are_shell_safe(
        root,
        {
            "docs.md": "\n".join(
                [
                    "biomcp variant normalize all 'NM_004448.2:c.829G>T'",
                    "biomcp variant normalize all NM_004448.2:c.829G>T",
                    "biomcp variant normalize all --genome <assembly> NM_004448.2:c.829G>T",
                ]
            ),
        },
    )

    assert result["status"] == "fail"
    assert result["findings"]
    assert any(
        finding["path"] == "docs.md"
        and "NM_004448.2:c.829G>T" in finding["text"]
        and "unquoted shell redirection" in finding["message"]
        for finding in result["findings"]
    )


def test_cli_surface_contract_rejects_entity_markdown_quoting_imports(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    entity_dir = root / "src" / "entities"
    entity_dir.mkdir(parents=True)
    (entity_dir / "brace.rs").write_text(
        "use crate::render::markdown::{quote_arg, shell_quote_arg};\n"
        "fn demo(value: &str) -> String { crate::render::markdown::quote_arg(value) }\n",
        encoding="utf-8",
    )
    (entity_dir / "direct.rs").write_text(
        "use crate::render::markdown::quote_arg;\n"
        "use crate::render::markdown::shell_quote_arg;\n",
        encoding="utf-8",
    )

    module = _load_quality_ratchet_module()
    result = module.check_entity_markdown_quoting_dependencies(root)

    assert result["status"] == "fail"
    patterns = {finding["pattern"] for finding in result["findings"]}
    assert patterns == {
        "crate::render::markdown::quote_arg",
        "crate::render::markdown::shell_quote_arg",
    }
    paths = {finding["path"] for finding in result["findings"]}
    assert paths == {"src/entities/brace.rs", "src/entities/direct.rs"}


def test_cli_surface_contract_pins_trial_status_vocabulary_and_active_refusal(
    tmp_path: Path,
) -> None:
    root = tmp_path / "repo"
    trial_dir = root / "src" / "cli" / "trial"
    trial_dir.mkdir(parents=True)
    (trial_dir / "mod.rs").write_text(
        "    /// Filter by trial status [values: recruiting, active_not_recruiting]\n",
        encoding="utf-8",
    )
    docs = root / "docs"
    (docs / "user-guide").mkdir(parents=True)
    (docs / "reference").mkdir(parents=True)
    contract = (
        "`recruiting` and `active_not_recruiting` are accepted. A bare "
        '`--status active` is refused as ambiguous because NCI uses "active" '
        "for a trial that is open and accruing, while ClinicalTrials.gov uses it "
        "for one that has stopped accruing. Use `--status recruiting` for open "
        "and accruing trials, or `--status active_not_recruiting` for enrolled "
        "and no longer accruing trials. The comma form `active, not recruiting` "
        "is still accepted."
    )
    (docs / "user-guide" / "trial.md").write_text(contract, encoding="utf-8")
    quick_reference = docs / "reference" / "quick-reference.md"
    # Reflowing the contract across lines must not trip the guard.
    quick_reference.write_text(contract.replace(" ", "\n"), encoding="utf-8")

    module = _load_quality_ratchet_module()
    result = module.check_trial_status_vocabulary_documented(root)

    assert result["status"] == "pass"
    assert result["status_values"] == ["recruiting", "active_not_recruiting"]
    assert result["findings"] == []

    quick_reference.write_text(
        contract.replace(
            "A bare `--status active` is refused as ambiguous",
            "`--status active` is accepted",
        ),
        encoding="utf-8",
    )
    result = module.check_trial_status_vocabulary_documented(root)

    assert result["status"] == "fail"
    assert any(
        finding.get("fragment") == "bare `--status active`"
        for finding in result["findings"]
    )


def test_cli_surface_contract_requires_author_in_entity_tables(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    (root / "docs").mkdir(parents=True)
    table = (
        "## Entities and sources\n\n### Gettable entities\n\n"
        "| Entity | Upstream providers used by BioMCP | Example |\n"
        "|--------|-----------------------------------|---------|\n"
        "| gene | MyGene.info | `biomcp get gene BRAF` |\n"
        "| author | Semantic Scholar, ORCID | `biomcp get author semanticscholar:1716151` |\n\n"
        "### Search-only entities\n\n"
        "| Entity | Upstream providers used by BioMCP | Example |\n"
        "|--------|-----------------------------------|---------|\n"
        "| gwas | GWAS Catalog | `biomcp search gwas --trait \"type 2 diabetes\"` |\n"
    )
    (root / "README.md").write_text(table, encoding="utf-8")
    index = root / "docs" / "index.md"
    index.write_text(table, encoding="utf-8")

    module = _load_quality_ratchet_module()
    assert module.check_author_entity_present_in_entity_tables(root)["status"] == "pass"

    index.write_text(table.replace("| author |", "| authors |"), encoding="utf-8")
    result = module.check_author_entity_present_in_entity_tables(root)

    assert result["status"] == "fail"
    assert result["findings"] == [
        {
            "path": "docs/index.md",
            "entity": "author",
            "message": "gettable entity is missing from the entity table",
        }
    ]


def test_cli_surface_contract_compares_release_process_versions_to_metadata(
    tmp_path: Path,
) -> None:
    root = tmp_path / "repo"
    (root / "docs" / "reference").mkdir(parents=True)
    cargo = root / "Cargo.toml"
    cargo.write_text(
        '[package]\nname = "biomcp-cli"\nversion = "0.9.1-dev.1"\n',
        encoding="utf-8",
    )
    (root / "pyproject.toml").write_text(
        '[project]\nname = "biomcp-cli"\nversion = "0.9.1.dev1"\n',
        encoding="utf-8",
    )
    release_process = root / "docs" / "reference" / "release-process.md"
    release_process.write_text(
        "The private development candidate is Cargo `0.9.1-dev.1` and Python\n"
        "`0.9.1.dev1`; public metadata stays on the latest published release.\n",
        encoding="utf-8",
    )

    module = _load_quality_ratchet_module()
    assert (
        module.check_release_process_versions_match_package_metadata(root)["status"]
        == "pass"
    )

    release_process.write_text(
        "The private development candidate is Cargo `0.8.0` and Python `0.9.1.dev1`.\n",
        encoding="utf-8",
    )
    result = module.check_release_process_versions_match_package_metadata(root)

    assert result["status"] == "fail"
    assert any(
        finding.get("fragment") == "Cargo `0.9.1-dev.1`"
        for finding in result["findings"]
    )

    release_process.write_text(
        "The private development candidate is Cargo `0.9.1-dev.1` and Python `0.9.1.dev1`.\n",
        encoding="utf-8",
    )
    cargo.write_text(
        '[package]\nname = "biomcp-cli"\nversion = "0.9.2-dev.1"\n',
        encoding="utf-8",
    )
    result = module.check_release_process_versions_match_package_metadata(root)

    assert result["status"] == "fail"
    assert any(
        finding.get("fragment") == "Cargo `0.9.2-dev.1`"
        for finding in result["findings"]
    )


def test_cli_surface_contract_exception_registry_names_initial_exceptions() -> None:
    assert EXCEPTION_REGISTRY.exists(), (
        "whole-surface CLI contract exceptions must be source-controlled, "
        "not hard-coded as inline skips"
    )
    registry = json.loads(EXCEPTION_REGISTRY.read_text(encoding="utf-8"))

    assert registry["schema"] == "biomcp-cli-surface-contract-exceptions-v1"
    entries = registry["entries"]
    by_command = {entry["command"]: entry for entry in entries}

    for command in ["biomcp --json list", "biomcp --json version"]:
        entry = by_command[command]
        assert entry["reason"].strip(), command
        assert entry["owner_test"].startswith("tests/test_cli_surface_contract_ratchet.py::"), command

    assert "biomcp cache path" not in by_command
    assert by_command["biomcp --json list"]["exception"] == "command_reference_payload"
    assert by_command["biomcp --json version"]["exception"] == "release_identity_payload"
    assert "biomcp --json search all --counts-only" not in by_command
