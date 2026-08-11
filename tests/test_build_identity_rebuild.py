from __future__ import annotations

import json
from pathlib import Path
import shutil
import subprocess


REPO_ROOT = Path(__file__).resolve().parents[1]


def _run(*args: str | Path, cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(arg) for arg in args],
        cwd=cwd,
        check=True,
        capture_output=True,
        text=True,
    )


def _write_fixture(root: Path) -> None:
    (root / "src").mkdir(parents=True)
    wrapper = root / "with-build-identity"
    shutil.copy2(REPO_ROOT / "tools" / "with-build-identity", wrapper)
    wrapper.chmod(0o755)
    (root / "Cargo.toml").write_text(
        '[package]\nname = "identity-fixture"\nversion = "1.2.3"\nedition = "2024"\n',
        encoding="utf-8",
    )
    (root / "src" / "lib.rs").write_text(
        'pub fn product_value() -> u8 { 1 }\n', encoding="utf-8"
    )
    (root / "src" / "main.rs").write_text(
        'const VERSION: &str = match option_env!("BIOMCP_BUILD_VERSION") { Some(value) => value, None => env!("CARGO_PKG_VERSION") };\n'
        'const SHA: &str = match option_env!("BIOMCP_BUILD_GIT_SHA") { Some(value) => value, None => "unknown" };\n'
        'const DATE: &str = match option_env!("BIOMCP_BUILD_DATE") { Some(value) => value, None => "unknown" };\n'
        "fn main() { println!(\"{}|{}|{}\", VERSION, SHA, DATE); }\n",
        encoding="utf-8",
    )


def _init_git(root: Path) -> None:
    _run("git", "init", "-q", cwd=root)
    _run("git", "config", "user.email", "identity-test@example.invalid", cwd=root)
    _run("git", "config", "user.name", "Identity Test", cwd=root)
    _run("git", "add", ".", cwd=root)
    _run("git", "commit", "-qm", "initial", cwd=root)


def _build(root: Path) -> list[dict[str, object]]:
    result = _run(
        root / "with-build-identity",
        "cargo",
        "build",
        "--message-format=json",
        cwd=root,
    )
    return [json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]


def _target_fresh(messages: list[dict[str, object]], kind: str) -> bool:
    target_name = "identity_fixture" if kind == "lib" else "identity-fixture"
    artifacts = [
        message
        for message in messages
        if message.get("reason") == "compiler-artifact"
        and kind in message["target"]["kind"]
        and message["target"]["name"] == target_name
    ]
    assert len(artifacts) == 1
    return bool(artifacts[0]["fresh"])


def _identity(root: Path) -> tuple[str, str, str]:
    output = _run(root / "target" / "debug" / "identity-fixture", cwd=root).stdout
    version, sha, date = output.strip().split("|")
    return version, sha, date


def test_head_only_identity_rebuild_keeps_library_fresh(tmp_path: Path) -> None:
    root = tmp_path / "fixture"
    _write_fixture(root)
    _init_git(root)
    _build(root)

    (root / "record.md").write_text("metadata only\n", encoding="utf-8")
    _run("git", "add", "record.md", cwd=root)
    _run("git", "commit", "-qm", "metadata", cwd=root)
    expected_sha = _run("git", "rev-parse", "--short=8", "HEAD", cwd=root).stdout.strip()

    messages = _build(root)

    assert _target_fresh(messages, "lib") is True
    assert _target_fresh(messages, "bin") is False
    assert _identity(root)[1] == expected_sha


def test_identity_consumer_and_product_changes_rebuild_their_owner(
    tmp_path: Path,
) -> None:
    root = tmp_path / "fixture"
    _write_fixture(root)
    _init_git(root)
    _build(root)

    main = root / "src" / "main.rs"
    main.write_text(main.read_text() + "// identity consumer change\n", encoding="utf-8")
    consumer_messages = _build(root)
    assert _target_fresh(consumer_messages, "lib") is True
    assert _target_fresh(consumer_messages, "bin") is False

    library = root / "src" / "lib.rs"
    library.write_text("pub fn product_value() -> u8 { 2 }\n", encoding="utf-8")
    product_messages = _build(root)
    assert _target_fresh(product_messages, "lib") is False


def test_identity_handles_exact_tags_dirty_source_and_gitless_archives(
    tmp_path: Path,
) -> None:
    root = tmp_path / "fixture"
    _write_fixture(root)
    _init_git(root)
    _run("git", "tag", "v1.2.3", cwd=root)
    _build(root)
    assert _identity(root)[0] == "1.2.3"

    library = root / "src" / "lib.rs"
    library.write_text("pub fn product_value() -> u8 { 2 }\n", encoding="utf-8")
    _build(root)
    version, sha, _ = _identity(root)
    assert version.endswith(".dirty")
    assert sha.endswith("-dirty")

    parent = tmp_path / "surrounding-repository"
    _run("git", "init", "-q", str(parent), cwd=tmp_path)
    archive = parent / "archive"
    _write_fixture(archive)
    _build(archive)
    assert _identity(archive) == ("1.2.3", "unknown", "unknown")
