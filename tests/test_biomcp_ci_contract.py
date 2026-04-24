from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
WRAPPER_SCRIPT = REPO_ROOT / "tools" / "biomcp-ci"
AUTH_KEYS = (
    "NCBI_API_KEY",
    "S2_API_KEY",
    "OPENFDA_API_KEY",
    "NCI_API_KEY",
    "ONCOKB_TOKEN",
    "DISGENET_API_KEY",
    "ALPHAGENOME_API_KEY",
    "UMLS_API_KEY",
)
CAPTURED_KEYS = (
    "BIOMCP_CACHE_DIR",
    "XDG_CACHE_HOME",
    "XDG_CONFIG_HOME",
    "RUST_LOG",
    "BIOMCP_CACHE_MODE",
    "BIOMCP_SPEC_CACHE_HIT",
    *AUTH_KEYS,
)


def _write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


def _make_capture_bin(tmp_path: Path) -> Path:
    capture_bin = tmp_path / "capture-bin"
    payload = ", ".join(f'"{key}": os.environ.get("{key}")' for key in CAPTURED_KEYS)
    _write_executable(
        capture_bin,
        "#!/usr/bin/env python3\n"
        "import json\n"
        "import os\n"
        "import sys\n"
        f"print(json.dumps({{'argv': sys.argv[1:], 'env': {{{payload}}}}}))\n",
    )
    return capture_bin


def _run_wrapper(
    tmp_path: Path, *args: str, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    wrapper_env = os.environ.copy()
    wrapper_env.update({key: f"sentinel-{key.lower()}" for key in AUTH_KEYS})
    wrapper_env["BIOMCP_BIN"] = str(_make_capture_bin(tmp_path))
    if env is not None:
        wrapper_env.update(env)
    return subprocess.run(
        ["bash", str(WRAPPER_SCRIPT), *args],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
        env=wrapper_env,
    )


def test_biomcp_ci_wrapper_is_tracked_and_executable() -> None:
    assert WRAPPER_SCRIPT.is_file(), "missing tools/biomcp-ci"
    assert os.access(WRAPPER_SCRIPT, os.X_OK), "tools/biomcp-ci must be executable"


def test_biomcp_ci_wrapper_sets_repo_cache_env_and_forwards_args(tmp_path: Path) -> None:
    result = _run_wrapper(tmp_path, "search", "gene", "BRAF", "--limit", "1")

    assert result.returncode == 0, result.stderr
    payload = json.loads(result.stdout)
    expected_cache_root = REPO_ROOT / ".cache" / "biomcp-specs"

    assert payload["argv"] == ["search", "gene", "BRAF", "--limit", "1"]
    assert payload["env"]["BIOMCP_CACHE_DIR"] == str(expected_cache_root)
    assert payload["env"]["XDG_CACHE_HOME"] == str(expected_cache_root / "xdg-cache")
    assert payload["env"]["XDG_CONFIG_HOME"] == str(expected_cache_root / "config")
    assert payload["env"]["RUST_LOG"] == "error"
    assert payload["env"]["BIOMCP_CACHE_MODE"] is None
    for key in AUTH_KEYS:
        assert payload["env"][key] is None, key
    assert expected_cache_root.is_dir()
    assert (expected_cache_root / "xdg-cache").is_dir()
    assert (expected_cache_root / "config").is_dir()


def test_biomcp_ci_wrapper_enables_force_cache_only_on_warm_hits(tmp_path: Path) -> None:
    result = _run_wrapper(
        tmp_path,
        "get",
        "article",
        "22663011",
        env={"BIOMCP_SPEC_CACHE_HIT": "1"},
    )

    assert result.returncode == 0, result.stderr
    payload = json.loads(result.stdout)
    assert payload["env"]["BIOMCP_SPEC_CACHE_HIT"] == "1"
    assert payload["env"]["BIOMCP_CACHE_MODE"] == "infinite"


def test_biomcp_ci_wrapper_preserves_explicit_cache_mode(tmp_path: Path) -> None:
    result = _run_wrapper(
        tmp_path,
        "search",
        "article",
        "-g",
        "BRAF",
        "--limit",
        "1",
        env={"BIOMCP_SPEC_CACHE_HIT": "1", "BIOMCP_CACHE_MODE": "off"},
    )

    assert result.returncode == 0, result.stderr
    payload = json.loads(result.stdout)
    assert payload["env"]["BIOMCP_SPEC_CACHE_HIT"] == "1"
    assert payload["env"]["BIOMCP_CACHE_MODE"] == "off"


def test_biomcp_ci_wrapper_avoids_pwd_dependent_shell_tricks() -> None:
    content = WRAPPER_SCRIPT.read_text(encoding="utf-8")

    assert 'REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"' in content
    assert "eval" not in content
    assert "git rev-parse" not in content
    assert "$PWD" not in content
