from __future__ import annotations

import base64
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile

import pytest


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools" / "check-zero-coupling.py"
TOKEN = "bio" + "data"


def _handoff(*words: str) -> str:
    return "clinical " + "trial " + " ".join(words)


def _module():
    spec = importlib.util.spec_from_file_location("zero_coupling", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _inventory(path: Path, entries: dict[str, bytes]) -> Path:
    content = {
        base64.b64encode(name.encode()).decode(): hashlib.sha256(data).hexdigest()
        for name, data in entries.items()
    }
    path.write_text(json.dumps(content), encoding="utf-8")
    return path


def _inventory_values(path: Path, entries: dict[str, object]) -> Path:
    content = {
        base64.b64encode(name.encode()).decode(): value
        for name, value in entries.items()
    }
    path.write_text(json.dumps(content), encoding="utf-8")
    return path


def _segmented_record(checker, block: bytes) -> bytes:
    return (
        ("historical " + TOKEN + "\n").encode()
        + checker.SEGMENT_BEGIN
        + block
        + checker.SEGMENT_END
        + b"\n"
    )


def _segmented_entry(checker, data: bytes) -> dict[str, object]:
    parts = checker._segment_parts(data)
    assert parts is not None
    prefix, _, suffix = parts
    return {
        "version": 1,
        "mode": "hosted-evidence-v1",
        "prefix_sha256": hashlib.sha256(prefix).hexdigest(),
        "suffix_sha256": hashlib.sha256(suffix).hexdigest(),
    }


def _closed_evidence(*, sha: str = "a" * 40) -> bytes:
    jobs = (
        "canonical-gates",
        "full-features",
        "windows-contracts",
        "repository-contracts",
    )
    lines = ["", "status: closed", f"reviewed-sha: {sha}"]
    lines.extend(
        f"{job}: success https://github.com/genomoncology/biomcp/actions/runs/{index}/job/{index + 10}"
        for index, job in enumerate(jobs, start=1)
    )
    return ("\n".join(lines) + "\n").encode()


def _drop_evidence_job(block: bytes, job: bytes) -> bytes:
    return b"".join(
        line
        for line in block.splitlines(keepends=True)
        if not line.startswith(job + b":")
    )


def _swap_first_two_evidence_jobs(block: bytes) -> bytes:
    lines = block.splitlines(keepends=True)
    lines[3], lines[4] = lines[4], lines[3]
    return b"".join(lines)


def test_tracked_text_scan_is_default_deny_and_case_insensitive(tmp_path: Path) -> None:
    checker = _module()
    inventory = _inventory(tmp_path / "inventory.json", {})
    cases = {
        "Cargo.toml": f'[dependencies]\n{TOKEN}={{path="../private"}}',
        "src/lib.rs": f"use {TOKEN}::Trial;",
        "tests/value.rs": f"fn {TOKEN}_handoff() {{}}",
        "docs/claim.md": f"owned by {TOKEN}",
        "sdlc/tickets/active.md": f"waits on {TOKEN}",
        "Cargo.lock": f"source = 'git+https://example.test/{TOKEN}'",
        "unknown.xyzzy": TOKEN.swapcase(),
        "NO_EXTENSION": TOKEN,
        "sdlc/records/new.md": TOKEN,
        "generated/output.rs": TOKEN,
        "revision.txt": "cfafc69d27c9a2fc" + "74909f21692a418a8b17db83",
    }
    for name, text in cases.items():
        path = tmp_path / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    assert checker.scan_files(tmp_path, list(cases), inventory) == sorted(cases)


def test_tracked_path_scan_rejects_forbidden_names_with_clean_contents(
    tmp_path: Path,
) -> None:
    checker = _module()
    inventory = _inventory(tmp_path / "inventory.json", {})
    name = "src/" + TOKEN.swapcase() + "_adapter.rs"
    path = tmp_path / name
    path.parent.mkdir(parents=True)
    path.write_text("pub struct LocalTrial;", encoding="utf-8")
    assert checker.scan_files(tmp_path, [name], inventory) == [name]


def test_realistic_external_trial_handoffs_are_rejected(tmp_path: Path) -> None:
    checker = _module()
    inventory = _inventory(tmp_path / "inventory.json", {})
    owner_a = "trial" + "-core"
    owner_b = "shared" + "-trial-types"
    owner_c = "trial" + "-contract"
    owner_d = "clinical" + "-trial-contract"
    owner_env_a = "TRIAL" + "_CORE_DIR"
    owner_env_b = "SHARED" + "_TRIAL_SCHEMA_ROOT"
    owner_upper = "TRIAL" + "-CORE"
    cases = {
        "Cargo.toml": (f'[dependencies]\n{owner_a} = {{ path = "../{owner_a}" }}\n'),
        "renamed.toml": (
            f'[dependencies]\ntransport = {{ package = "{owner_b}", '
            'path = "../contracts" }\n'
        ),
        "table.toml": (f'[dependencies.{owner_c}]\npath = "../{owner_c}"\n'),
        "patch.toml": (
            f'[patch.crates-io]\n{owner_a} = {{ git = "https://example.test/core", '
            'rev = "abc" }\n'
        ),
        "generated.rs": (
            f'include!(concat!(env!("{owner_env_a}"), "/generated.rs"));\n'
        ),
        "generated-renamed.rs": (
            f'include!(concat!(env!("{owner_env_b}"), "/bindings/generated.rs"));\n'
        ),
        "checkout.sh": f"git clone https://example.test/contracts {owner_a}\n",
        "worktree.sh": f"git worktree add ../{owner_d} feature\n",
        "case.toml": (
            f'[DEPENDENCIES]\n{owner_upper} = {{ PATH = "../{owner_upper}" }}\n'
        ),
    }
    for name, content in cases.items():
        (tmp_path / name).write_text(content, encoding="utf-8")
    assert checker.scan_files(tmp_path, list(cases), inventory) == sorted(cases)


def test_unrelated_local_build_mechanisms_remain_allowed(tmp_path: Path) -> None:
    checker = _module()
    inventory = _inventory(tmp_path / "inventory.json", {})
    cases = {
        "Cargo.toml": '[dependencies]\nhelper-core = { path = "../helper-core" }\n',
        "src/generated.rs": 'include!(concat!(env!("OUT_DIR"), "/generated.rs"));\n',
        "scripts/setup.sh": "git clone https://example.test/tools billing-core\n",
        "docs/source.md": "The clinical trials source returns generated identifiers.\n",
    }
    for name, content in cases.items():
        path = tmp_path / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
    assert checker.scan_files(tmp_path, list(cases), inventory) == []


def test_cargo_table_target_replace_and_lock_handoffs_are_rejected(
    tmp_path: Path,
) -> None:
    checker = _module()
    inventory = _inventory(tmp_path / "inventory.json", {})
    owner_a = "trial" + "-core"
    owner_b = "shared" + "-trial-types"
    owner_c = "trial" + "-contract"
    cases = {
        "dependency-git.toml": (
            f'[dependencies.{owner_a}]\ngit = "https://example.test/core"\n'
            'rev = "abc"\n'
        ),
        "dependency-alias-git.toml": (
            "[dependencies.transport]\n"
            f'package = "{owner_b}"\ngit = "https://example.test/types"\n'
        ),
        "target-path.toml": (
            f"[target.'cfg(windows)'.dependencies.{owner_a}]\npath = \"../{owner_a}\"\n"
        ),
        "Cargo.lock": (
            f'[[package]]\nname = "{owner_a}"\nversion = "0.1.0"\n'
            'source = "git+https://example.test/core#abc"\n'
        ),
        "target-build-alias.toml": (
            "[target.x86_64-pc-windows-msvc.build-dependencies.transport]\n"
            f'package = "{owner_b}"\npath = "../types"\n'
        ),
        "patch-subtable.toml": (
            f'[patch.crates-io.{owner_c}]\ngit = "https://example.test/contract"\n'
        ),
        "patch-registry.toml": (
            f'[patch.crates-io]\n{owner_a} = {{ registry = "internal" }}\n'
        ),
        "replace.toml": (
            f'[replace]\n"{owner_a}:0.1.0" = {{ path = "../replacement" }}\n'
        ),
    }
    for name, content in cases.items():
        (tmp_path / name).write_text(content, encoding="utf-8")
    assert checker.scan_files(tmp_path, list(cases), inventory) == sorted(cases)


def test_unrelated_cargo_tables_targets_and_lock_sources_remain_allowed(
    tmp_path: Path,
) -> None:
    checker = _module()
    inventory = _inventory(tmp_path / "inventory.json", {})
    owner = "trial" + "-core"
    cases = {
        "dependency.toml": (
            '[dependencies.helper-core]\ngit = "https://example.test/helper"\n'
        ),
        "target.toml": (
            "[target.'cfg(windows)'.dependencies.windows-sys]\n"
            'path = "../windows-sys"\n'
        ),
        "Cargo.lock": (
            f'[[package]]\nname = "{owner}"\nversion = "0.1.0"\n'
            'source = "registry+https://github.com/rust-lang/crates.io-index"\n'
        ),
        "docs.md": "Clinical trial results can mention a Git source in prose.\n",
    }
    for name, content in cases.items():
        (tmp_path / name).write_text(content, encoding="utf-8")
    assert checker.scan_files(tmp_path, list(cases), inventory) == []


def test_exact_historical_digest_passes_but_mutation_and_rename_fail(
    tmp_path: Path,
) -> None:
    checker = _module()
    name = "sdlc/records/known.md"
    data = f"historical {TOKEN}".encode()
    path = tmp_path / name
    path.parent.mkdir(parents=True)
    path.write_bytes(data)
    inventory = _inventory(tmp_path / "inventory.json", {name: data})
    assert checker.scan_files(tmp_path, [name], inventory) == []
    path.write_bytes(data + b" extra")
    assert checker.scan_files(tmp_path, [name], inventory)
    renamed = "sdlc/records/renamed.md"
    path.rename(tmp_path / renamed)
    assert checker.scan_files(tmp_path, [renamed], inventory)


def test_1183_segmented_record_accepts_pending_and_fixed_closed_grammar(
    tmp_path: Path,
) -> None:
    checker = _module()
    name = checker.SEGMENTED_RECORD
    pending = _segmented_record(checker, checker.PENDING_EVIDENCE)
    path = tmp_path / name
    path.parent.mkdir(parents=True)
    path.write_bytes(pending)
    entry = _segmented_entry(checker, pending)
    inventory = _inventory_values(tmp_path / "inventory.json", {name: entry})
    assert checker.scan_files(tmp_path, [name], inventory) == []

    closed = _segmented_record(checker, _closed_evidence())
    path.write_bytes(closed)
    assert checker.scan_files(tmp_path, [name], inventory) == []
    assert checker.closure_diff_is_block_only(pending, closed)


@pytest.mark.parametrize(
    "mutation",
    [
        "missing-begin",
        "missing-end",
        "duplicate-begin",
        "duplicate-end",
        "reversed",
        "prefix",
        "suffix",
    ],
)
def test_1183_segmented_record_rejects_marker_and_immutable_region_changes(
    tmp_path: Path, mutation: str
) -> None:
    checker = _module()
    name = checker.SEGMENTED_RECORD
    original = _segmented_record(checker, checker.PENDING_EVIDENCE)
    entry = _segmented_entry(checker, original)
    changed = {
        "missing-begin": original.replace(checker.SEGMENT_BEGIN, b""),
        "missing-end": original.replace(checker.SEGMENT_END, b""),
        "duplicate-begin": original.replace(
            checker.SEGMENT_BEGIN, checker.SEGMENT_BEGIN * 2
        ),
        "duplicate-end": original.replace(checker.SEGMENT_END, checker.SEGMENT_END * 2),
        "reversed": (
            ("historical " + TOKEN + "\n").encode()
            + checker.SEGMENT_END
            + checker.PENDING_EVIDENCE
            + checker.SEGMENT_BEGIN
            + b"\n"
        ),
        "prefix": b"changed " + original,
        "suffix": original + b"changed\n",
    }[mutation]
    path = tmp_path / name
    path.parent.mkdir(parents=True)
    path.write_bytes(changed)
    inventory = _inventory_values(tmp_path / "inventory.json", {name: entry})
    assert checker.scan_files(tmp_path, [name], inventory)


def test_segmented_inventory_mode_is_legal_only_for_exact_1183_path(
    tmp_path: Path,
) -> None:
    checker = _module()
    data = _segmented_record(checker, checker.PENDING_EVIDENCE)
    entry = _segmented_entry(checker, data)
    wrong = "sdlc/records/1183-copy.md"
    path = tmp_path / wrong
    path.parent.mkdir(parents=True)
    path.write_bytes(data)
    inventory = _inventory_values(tmp_path / "inventory.json", {wrong: entry})
    assert checker.scan_files(tmp_path, [wrong], inventory)

    for key, value in {
        "version": 2,
        "mode": "hosted-evidence-v2",
        "prefix_sha256": "A" * 64,
        "suffix_sha256": "0" * 63,
        "extra": "not-allowed",
    }.items():
        invalid = dict(entry)
        invalid[key] = value
        exact = tmp_path / checker.SEGMENTED_RECORD
        exact.parent.mkdir(parents=True, exist_ok=True)
        exact.write_bytes(data)
        bad_inventory = _inventory_values(
            tmp_path / f"inventory-{key}.json",
            {checker.SEGMENTED_RECORD: invalid},
        )
        assert checker.scan_files(tmp_path, [checker.SEGMENTED_RECORD], bad_inventory)


@pytest.mark.parametrize(
    "block",
    [
        b"\nstatus: pending\nextra: value\n",
        _closed_evidence() + b"extra: value\n",
        _closed_evidence(sha="A" * 40),
        _closed_evidence(sha="a" * 39),
        _drop_evidence_job(_closed_evidence(), b"full-features"),
        _swap_first_two_evidence_jobs(_closed_evidence()),
        _closed_evidence().replace(
            b"canonical-gates: success", b"canonical-gates: failure"
        ),
        _closed_evidence().replace(b"https://", b"http://", 1),
        _closed_evidence().replace(
            b"github.com/genomoncology/biomcp", b"example.test/biomcp", 1
        ),
        _closed_evidence().replace(b"/runs/1/job/11", b"/runs/run-1/job/11", 1),
        _closed_evidence().replace(b"/runs/1/job/11", b"/runs/1/job/job-11", 1),
        _closed_evidence().replace(b"full-features", b"extra-gate", 1),
        _closed_evidence().replace(
            b"canonical-gates: success", b"windows-contracts: success", 1
        ),
        _closed_evidence() + ("forbidden " + TOKEN + "\n").encode(),
        _closed_evidence() + _handoff("path", "depend" + "ency").encode() + b"\n",
    ],
)
def test_1183_segmented_record_rejects_noncanonical_hosted_blocks(
    tmp_path: Path, block: bytes
) -> None:
    checker = _module()
    name = checker.SEGMENTED_RECORD
    original = _segmented_record(checker, checker.PENDING_EVIDENCE)
    entry = _segmented_entry(checker, original)
    path = tmp_path / name
    path.parent.mkdir(parents=True)
    path.write_bytes(_segmented_record(checker, block))
    inventory = _inventory_values(tmp_path / "inventory.json", {name: entry})
    assert checker.scan_files(tmp_path, [name], inventory)


def test_closure_diff_helper_rejects_changes_outside_the_hosted_block() -> None:
    checker = _module()
    pending = _segmented_record(checker, checker.PENDING_EVIDENCE)
    closed = _segmented_record(checker, _closed_evidence())
    assert checker.closure_diff_is_block_only(pending, closed)
    assert not checker.closure_diff_is_block_only(pending, b"prefix " + closed)
    assert not checker.closure_diff_is_block_only(pending, closed + b"suffix")
    assert not checker.closure_diff_is_block_only(pending, pending)
    assert not checker.closure_diff_is_block_only(
        pending, _segmented_record(checker, b"\nstatus: closed\n")
    )


def test_archive_scan_rejects_packaged_only_text_and_skips_binary(
    tmp_path: Path,
) -> None:
    checker = _module()
    archive_path = tmp_path / "package.crate"
    owner = "trial" + "-core"
    with tarfile.open(archive_path, "w:gz") as archive:
        for name, data in {
            "pkg/only-in-package.txt": TOKEN.encode(),
            "pkg/Cargo.toml": (
                f'[dependencies]\n{owner} = {{ path = "../{owner}" }}\n'.encode()
            ),
            "pkg/binary.bin": b"\0\xff" + TOKEN.encode(),
        }.items():
            info = tarfile.TarInfo(name)
            info.size = len(data)
            archive.addfile(info, io.BytesIO(data))
    assert checker.scan_archive(archive_path) == [
        "pkg/Cargo.toml",
        "pkg/only-in-package.txt",
    ]


def test_archive_scan_rejects_forbidden_member_name_with_clean_contents(
    tmp_path: Path,
) -> None:
    checker = _module()
    archive_path = tmp_path / "package.crate"
    name = "pkg/generated-" + TOKEN.swapcase() + ".rs"
    data = b"pub struct LocalTrial;"
    with tarfile.open(archive_path, "w:gz") as archive:
        info = tarfile.TarInfo(name)
        info.size = len(data)
        archive.addfile(info, io.BytesIO(data))
    assert checker.scan_archive(archive_path) == [name]


def test_repository_historical_inventory_is_exact_and_current() -> None:
    checker = _module()
    assert checker.scan_files(ROOT, checker.tracked(ROOT)) == []
