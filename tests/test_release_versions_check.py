from __future__ import annotations

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / "scripts" / "check-release-versions.py"


def _run(
    tmp_path: Path,
    *,
    tag: str,
    cargo: str = "0.9.1",
    python: str = "0.9.1",
    sync_exit: int = 0,
) -> subprocess.CompletedProcess[str]:
    (tmp_path / "Cargo.toml").write_text(f'version = "{cargo}"\n', encoding="utf-8")
    (tmp_path / "pyproject.toml").write_text(
        f'version = "{python}"\n', encoding="utf-8"
    )
    sync = tmp_path / "check-version-sync.sh"
    sync.write_text(
        f"#!/usr/bin/env bash\necho invoked > {tmp_path / 'invoked'}\nexit {sync_exit}\n",
        encoding="utf-8",
    )
    sync.chmod(0o755)
    return subprocess.run(
        [
            sys.executable,
            str(CHECK),
            "--tag",
            tag,
            "--repo-root",
            str(tmp_path),
            "--sync-script",
            str(sync),
        ],
        capture_output=True,
        text=True,
    )


def test_matching_stable_versions_pass_and_invoke_sync(tmp_path: Path) -> None:
    result = _run(tmp_path, tag="v0.9.1")
    assert result.returncode == 0, result.stderr
    assert (tmp_path / "invoked").is_file()


def test_mismatched_version_fails_before_sync(tmp_path: Path) -> None:
    result = _run(tmp_path, tag="v0.9.1", python="0.9.2")
    assert result.returncode == 1
    assert "pyproject.toml=0.9.2" in result.stderr
    assert not (tmp_path / "invoked").exists()


def test_tag_without_v_prefix_fails(tmp_path: Path) -> None:
    result = _run(tmp_path, tag="0.9.1")
    assert result.returncode == 1
    assert "must start with v" in result.stderr


def test_prerelease_tag_fails_with_clear_message(tmp_path: Path) -> None:
    result = _run(tmp_path, tag="v0.9.1-rc.1", cargo="0.9.1-rc.1", python="0.9.1rc1")
    assert result.returncode == 1
    assert "pre-release tags are not supported" in result.stderr


def test_dev_tag_is_refused_before_any_publisher_starts(tmp_path: Path) -> None:
    # 2132: a tag whose version carries -dev names a development
    # build, never a release; the version-check job must stop the
    # workflow before any artifact is built.
    result = _run(tmp_path, tag="v0.9.3-dev", cargo="0.9.3-dev", python="0.9.3.dev0")
    assert result.returncode == 1
    assert "must start with v and contain a stable semantic version" in result.stderr
    assert "v0.9.3-dev" in result.stderr
    assert not (tmp_path / "invoked").exists()


def test_stable_tag_over_committed_dev_versions_is_refused(tmp_path: Path) -> None:
    # The release commit must drop the dev suffix before the tag is
    # pushed; a stable tag over a tree that still carries -dev fails
    # on tag equality before the sync script runs.
    result = _run(tmp_path, tag="v0.9.3", cargo="0.9.3-dev", python="0.9.3.dev0")
    assert result.returncode == 1
    assert "tag=0.9.3, Cargo.toml=0.9.3-dev" in result.stderr
    assert not (tmp_path / "invoked").exists()


def test_sync_failure_fails_the_release_check(tmp_path: Path) -> None:
    result = _run(tmp_path, tag="v0.9.1", sync_exit=7)
    assert result.returncode == 1
    assert "check-version-sync.sh failed" in result.stderr
