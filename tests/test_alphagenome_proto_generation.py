from __future__ import annotations

import importlib.machinery
import importlib.util
from pathlib import Path
import subprocess


REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _load_generator():
    path = REPO_ROOT / "scripts" / "regenerate-alphagenome-proto"
    loader = importlib.machinery.SourceFileLoader("alphagenome_generator", str(path))
    spec = importlib.util.spec_from_loader(loader.name, loader)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module


def test_normal_build_consumes_only_committed_alphagenome_rust() -> None:
    build_script = _read("build.rs")
    manifest = _read("Cargo.toml")
    alphagenome = _read("src/sources/alphagenome.rs")

    assert "protoc" not in build_script
    assert "tonic_build" not in build_script
    assert "src/generated/google.gdm.gdmscience.alphagenome.v1main.rs" not in (
        build_script
    )
    assert 'alphagenome = ["dep:tonic", "dep:prost", "dep:zstd"]' in manifest
    assert "tonic-build" not in manifest
    assert "tonic::include_proto!" not in alphagenome
    assert 'env!("CARGO_MANIFEST_DIR")' in alphagenome
    assert "src/generated/google.gdm.gdmscience.alphagenome.v1main.rs" in alphagenome


def test_source_package_contains_generated_rust_not_generator_inputs() -> None:
    package_files = subprocess.run(
        ["cargo", "package", "--allow-dirty", "--list"],
        cwd=REPO_ROOT,
        check=True,
        text=True,
        capture_output=True,
    ).stdout.splitlines()

    assert "src/generated/google.gdm.gdmscience.alphagenome.v1main.rs" in package_files
    assert not any(path.startswith("protos/") for path in package_files)
    assert "scripts/regenerate-alphagenome-proto" not in package_files
    assert not any(
        path.startswith("tools/alphagenome-proto-generator/") for path in package_files
    )


def test_regeneration_is_explicit_pinned_atomic_and_checkable() -> None:
    generator = _read("scripts/regenerate-alphagenome-proto")
    workflow = _read(".github/workflows/ci.yml")

    assert 'PINNED_PROTOC_VERSION = "28.3"' in generator
    assert 'parser.add_argument("--check", action="store_true")' in generator
    assert "TemporaryDirectory" in generator
    assert "os.replace" in generator
    assert "unified_diff" in generator
    assert "generated provider client includes members unused by this runtime" in (
        generator
    )
    assert workflow.count("arduino/setup-protoc@v3") == 1
    assert 'version: "28.3"' in workflow
    assert "scripts/regenerate-alphagenome-proto --check" in workflow


def test_check_only_reports_candidate_diff_without_writing(
    tmp_path: Path, capsys
) -> None:
    tracked = tmp_path / "tracked.rs"
    candidate = tmp_path / "candidate.rs"
    tracked.write_text("committed\n", encoding="utf-8")
    candidate.write_text("regenerated\n", encoding="utf-8")

    result = _load_generator().install_or_check(tracked, candidate, check=True)

    assert result == 1
    assert tracked.read_text(encoding="utf-8") == "committed\n"
    assert candidate.read_text(encoding="utf-8") == "regenerated\n"
    error = capsys.readouterr().err
    assert "-committed" in error
    assert "+regenerated" in error


def test_regeneration_atomically_replaces_only_the_generated_file(
    tmp_path: Path,
) -> None:
    tracked = tmp_path / "tracked.rs"
    candidate = tmp_path / "candidate.rs"
    neighbor = tmp_path / "neighbor.rs"
    tracked.write_text("committed\n", encoding="utf-8")
    candidate.write_text("regenerated\n", encoding="utf-8")
    neighbor.write_text("untouched\n", encoding="utf-8")

    result = _load_generator().install_or_check(tracked, candidate, check=False)

    assert result == 0
    assert tracked.read_text(encoding="utf-8") == "regenerated\n"
    assert not candidate.exists()
    assert neighbor.read_text(encoding="utf-8") == "untouched\n"


def test_build_and_release_docs_agree_that_protoc_is_maintainer_only() -> None:
    docs = "\n".join(
        _read(path)
        for path in (
            "CONTRIBUTING.md",
            "RUN.md",
            "architecture/technical/overview.md",
            "docs/troubleshooting.md",
            "docs/getting-started/installation.md",
        )
    )

    assert "Normal builds do not run or require `protoc`" in docs
    assert "scripts/regenerate-alphagenome-proto --check" in docs
    assert "pinned `protoc` 28.3" in docs
