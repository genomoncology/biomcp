from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def _module(name: str, relative: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


candidate = _module("candidate", "release/candidate.py")
package = _module("package", "release/package.py")
inspection = _module("release_inspection", "release/inspect.py")
targets = _module("release_targets", "release/build_target.py")


def test_target_registry_is_exactly_five_native_and_wheel_pairs() -> None:
    assert set(targets.TARGETS) == {
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
    }
    assert len(candidate.PLATFORM_ARTIFACTS) == 10
    for settings in targets.TARGETS.values():
        assert f"native-{settings['slug']}" in candidate.PLATFORM_ARTIFACTS
        assert f"wheel-{settings['slug']}" in candidate.PLATFORM_ARTIFACTS


def test_linux_tags_and_archive_names_preserve_installer_contract() -> None:
    assert (
        targets.TARGETS["x86_64-unknown-linux-gnu"]["archive"]
        == "biomcp-linux-x86_64.tar.gz"
    )
    assert (
        targets.TARGETS["aarch64-unknown-linux-gnu"]["wheel"]
        == "manylinux_2_28_aarch64"
    )
    assert (
        targets.TARGETS["x86_64-pc-windows-msvc"]["archive"]
        == "biomcp-windows-x86_64.zip"
    )


def test_glibc_inspection_accepts_floor_and_rejects_newer_import() -> None:
    assert inspection.max_glibc_version("Name: GLIBC_2.17 Name: GLIBC_2.28") == (2, 28)
    assert inspection.max_glibc_version("Name: GLIBC_2.9 Name: GLIBC_2.3") == (2, 9)


def test_sbom_is_deterministic_and_binds_source(tmp_path: Path) -> None:
    lock = tmp_path / "Cargo.lock"
    lock.write_text(
        '[[package]]\nname = "z"\nversion = "2.0.0"\n\n'
        '[[package]]\nname = "a"\nversion = "1.0.0"\nsource = "registry+x"\n'
    )
    first = tmp_path / "one.json"
    second = tmp_path / "two.json"
    first_hash = targets.sbom(lock, first, "a" * 40, "1.2.3")
    second_hash = targets.sbom(lock, second, "a" * 40, "1.2.3")
    assert first_hash == second_hash
    value = json.loads(first.read_text())
    assert value["metadata"]["source_sha"] == "a" * 40
    assert [item["name"] for item in value["components"]] == ["a", "z"]


def test_release_target_build_uses_the_canonical_identity_wrapper() -> None:
    source = (ROOT / "release/build_target.py").read_text(encoding="utf-8")
    assert 'str(args.repo / "tools/with-build-identity")' in source


def test_release_target_build_threads_python_package_identity() -> None:
    source = (ROOT / "release/build_target.py").read_text(encoding="utf-8")
    assert 'parser.add_argument("--python-version", required=True)' in source
    assert 'f"biomcp_cli-{args.python_version}-py3-none-' in source
    assert '"--python-version",\n            args.python_version' in source
