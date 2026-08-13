from __future__ import annotations

import os
import shlex
import shutil
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = Path("scripts") / "pre-commit-reject-march-artifacts.sh"
ENTRYPOINT_PATH = Path("scripts") / "pre-commit"
INSTALLER_PATH = Path("scripts") / "install-pre-commit-hook"
ALLOWED_MARCH_PATHS = (".march/code-review-log.md",)
BAD_MARCH_PATHS = (
    ".march/verify-log.md",
    ".march/blueprint.md",
)


def _git(repo_root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=repo_root,
        check=True,
        capture_output=True,
        text=True,
    )


def _write(repo_root: Path, path: str, content: str = "tracked\n") -> None:
    target = repo_root / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def _copy_hook_fixture(tmp_path: Path) -> Path:
    fixture_root = tmp_path / "repo"
    (fixture_root / "scripts").mkdir(parents=True)
    source = REPO_ROOT / SCRIPT_PATH
    assert source.is_file(), f"missing hook helper: {SCRIPT_PATH}"
    shutil.copy2(source, fixture_root / SCRIPT_PATH)
    (fixture_root / ".gitignore").write_text(".march/\n", encoding="utf-8")
    _git(fixture_root, "init")
    _git(fixture_root, "config", "user.email", "tests@example.invalid")
    _git(fixture_root, "config", "user.name", "BioMCP Tests")
    return fixture_root


def _run_hook_script(repo_root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(repo_root / SCRIPT_PATH)],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=False,
    )


def _pre_commit_fixture(tmp_path: Path) -> tuple[Path, dict[str, str], Path, Path]:
    root = tmp_path / "repo with space"
    for directory in ("scripts", "tools", "fake-bin"):
        (root / directory).mkdir(parents=True, exist_ok=True)
    for relative in (
        ENTRYPOINT_PATH,
        INSTALLER_PATH,
        SCRIPT_PATH,
        Path("tools/check-tracked-text"),
        Path("tools/with-build-identity"),
    ):
        shutil.copy2(REPO_ROOT / relative, root / relative)
    quality_log = root / "quality.log"
    quality = root / "tools/check-quality-ratchet.sh"
    quality.write_text(
        f"#!/usr/bin/env bash\nprintf '%s\\n' \"${{QUALITY_RATCHET_AUDITS:-}}\" >>{shlex.quote(str(quality_log))}\n"
    )
    quality.chmod(0o755)
    cargo_log = root / "cargo.log"
    uv_log = root / "uv.log"
    for name, log in (("cargo", cargo_log), ("uv", uv_log)):
        command = root / "fake-bin" / name
        command.write_text(
            f"#!/usr/bin/env bash\nprintf '%s\\n' \"$*\" >>{shlex.quote(str(log))}\n"
        )
        command.chmod(0o755)
    _git(root, "init")
    _git(root, "config", "user.email", "tests@example.invalid")
    _git(root, "config", "user.name", "BioMCP Tests")
    env = os.environ | {
        "PATH": f"{root / 'fake-bin'}:{os.environ['PATH']}",
        "CARGO_PKG_VERSION": "0.0.0",
    }
    return root, env, cargo_log, uv_log


def _run_tracked_pre_commit(
    root: Path, env: dict[str, str]
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(root / ENTRYPOINT_PATH)],
        cwd=root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )


def test_pre_commit_reject_march_artifacts_script_is_executable() -> None:
    script = REPO_ROOT / SCRIPT_PATH

    assert script.is_file()
    assert os.access(script, os.X_OK)


def test_pre_commit_reject_march_artifacts_passes_with_no_staged_march_paths(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)

    result = _run_hook_script(repo_root)

    assert result.returncode == 0
    assert result.stdout == ""
    assert result.stderr == ""


def test_pre_commit_reject_march_artifacts_allows_allowlisted_paths(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    for path in ALLOWED_MARCH_PATHS:
        _write(repo_root, path)
    _git(repo_root, "add", "-f", *ALLOWED_MARCH_PATHS)

    result = _run_hook_script(repo_root)

    assert result.returncode == 0
    assert result.stdout == ""
    assert result.stderr == ""


def test_pre_commit_reject_march_artifacts_rejects_staged_bad_paths(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    for path in BAD_MARCH_PATHS:
        _write(repo_root, path)
    _git(repo_root, "add", "-f", *BAD_MARCH_PATHS)

    result = _run_hook_script(repo_root)

    assert result.returncode == 1
    assert result.stdout == ""
    assert "Error: staged non-allowlisted .march artifacts detected:" in result.stderr
    for path in BAD_MARCH_PATHS:
        assert path in result.stderr
    for path in ALLOWED_MARCH_PATHS:
        assert path in result.stderr
    assert "git restore --staged -- <path>" in result.stderr
    assert "git rm --cached -- <path>" in result.stderr


def test_pre_commit_reject_march_artifacts_allows_staged_bad_path_deletion(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    _write(repo_root, ".march/verify-log.md")
    _git(repo_root, "add", "-f", ".march/verify-log.md")
    _git(repo_root, "commit", "-m", "Track old March artifact")
    _git(repo_root, "rm", ".march/verify-log.md")

    result = _run_hook_script(repo_root)

    assert result.returncode == 0
    assert result.stdout == ""
    assert result.stderr == ""


def test_pre_commit_reject_march_artifacts_rejects_staged_bad_path_modification(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    _write(repo_root, ".march/verify-log.md", "old tracked artifact\n")
    _git(repo_root, "add", "-f", ".march/verify-log.md")
    _git(repo_root, "commit", "-m", "Track old March artifact")
    _write(repo_root, ".march/verify-log.md", "modified artifact\n")
    _git(repo_root, "add", "-f", ".march/verify-log.md")

    result = _run_hook_script(repo_root)

    assert result.returncode == 1
    assert result.stdout == ""
    assert ".march/verify-log.md" in result.stderr


def test_pre_commit_reject_march_artifacts_rejects_rename_into_bad_march_path(
    tmp_path: Path,
) -> None:
    repo_root = _copy_hook_fixture(tmp_path)
    _write(repo_root, "notes.md")
    _git(repo_root, "add", "notes.md")
    _git(repo_root, "commit", "-m", "Track note")
    (repo_root / ".march").mkdir()
    _git(repo_root, "mv", "-f", "notes.md", ".march/blueprint.md")

    result = _run_hook_script(repo_root)

    assert result.returncode == 1
    assert result.stdout == ""
    assert ".march/blueprint.md" in result.stderr


@pytest.mark.parametrize(
    "path",
    [
        "README.md",
        "sdlc/notes.md",
        "docs/guide.md",
        "architecture/decision.md",
        "spec/contract.md",
        "skills/guide.md",
        "docs/path with spaces.md",
    ],
)
def test_tracked_pre_commit_skips_cargo_for_allowed_markdown(
    tmp_path: Path, path: str
) -> None:
    root, env, cargo_log, uv_log = _pre_commit_fixture(tmp_path)
    _write(root, path, "# Safe documentation\n")
    _git(root, "add", path)

    result = _run_tracked_pre_commit(root, env)

    assert result.returncode == 0, result.stderr
    assert not cargo_log.exists()
    assert "run --no-sync mkdocs build --strict" in uv_log.read_text()
    quality_log = root / "quality.log"
    assert quality_log.exists() == path.startswith("spec/")
    if quality_log.exists():
        assert quality_log.read_text() == "spec_lint\n"


def test_tracked_pre_commit_uses_full_rust_checks_for_mixed_or_unknown_changes(
    tmp_path: Path,
) -> None:
    root, env, cargo_log, uv_log = _pre_commit_fixture(tmp_path)
    _write(root, "docs/guide.md", "# Guide\n")
    _write(root, "src/lib.rs", "pub fn value() {}\n")
    _git(root, "add", "docs/guide.md", "src/lib.rs")

    result = _run_tracked_pre_commit(root, env)

    assert result.returncode == 0, result.stderr
    cargo_calls = cargo_log.read_text().splitlines()
    assert cargo_calls == [
        "fmt --check",
        "clippy --no-default-features --lib --tests -- -D warnings",
    ]
    assert not uv_log.exists()


def test_tracked_pre_commit_classifies_deletes_renames_and_non_utf8_paths(
    tmp_path: Path,
) -> None:
    root, env, cargo_log, _ = _pre_commit_fixture(tmp_path)
    _write(root, "docs/old name.md", "# Old\n")
    _git(root, "add", "docs/old name.md")
    _git(root, "commit", "-m", "seed")
    _git(root, "mv", "docs/old name.md", "docs/new name.md")
    raw_path = os.path.join(os.fsencode(root), b"docs/non-utf8-\xff.md")
    descriptor = os.open(raw_path, os.O_WRONLY | os.O_CREAT, 0o644)
    os.write(descriptor, b"# Bytes-safe path\n")
    os.close(descriptor)
    _git(root, "add", os.fsdecode(b"docs/non-utf8-\xff.md"))

    result = _run_tracked_pre_commit(root, env)

    assert result.returncode == 0, result.stderr
    assert not cargo_log.exists()

    _git(root, "reset", "--hard", "-q", "HEAD")
    _git(root, "rm", "docs/old name.md")
    result = _run_tracked_pre_commit(root, env)
    assert result.returncode == 0, result.stderr
    assert not cargo_log.exists()


def test_tracked_pre_commit_keeps_credential_scan_for_markdown(tmp_path: Path) -> None:
    root, env, cargo_log, uv_log = _pre_commit_fixture(tmp_path)
    credential_fixture = "API_" + "KEY=real-looking-fixture-secret\n"
    _write(root, "docs/leak.md", credential_fixture)
    _git(root, "add", "docs/leak.md")

    result = _run_tracked_pre_commit(root, env)

    assert result.returncode != 0
    assert "Credential-like patterns found" in result.stdout
    assert not cargo_log.exists()
    assert not uv_log.exists()


def test_pre_commit_installer_writes_a_thin_handoff(tmp_path: Path) -> None:
    root, env, cargo_log, _ = _pre_commit_fixture(tmp_path)
    result = subprocess.run(
        [str(root / INSTALLER_PATH)],
        cwd=root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    hook = Path(
        _git(
            root,
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "hooks/pre-commit",
        ).stdout.strip()
    )
    lines = hook.read_text().splitlines()
    assert lines[0] == "#!/usr/bin/env bash"
    assert len(lines) == 2
    assert "scripts/pre-commit" in lines[1]

    _write(root, "change.bin", "unknown\n")
    _git(root, "add", "change.bin")
    invoked = subprocess.run([str(hook)], cwd=root, env=env, check=False)
    assert invoked.returncode == 0
    assert cargo_log.exists()
