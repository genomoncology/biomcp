from __future__ import annotations

import os
import shutil
import stat
import subprocess
import textwrap
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _write_executable(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def _fake_biomcp_script(git_sha: str, build_date: str = "2026-06-24T00:00:00Z") -> str:
    return textwrap.dedent(
        f"""\
        #!/usr/bin/env python3
        import json
        import sys
        import time

        GIT_SHA = {git_sha!r}
        BUILD_DATE = {build_date!r}
        args = sys.argv[1:]

        def version():
            print(f"biomcp 0.8.24 (git {{GIT_SHA}}, build {{BUILD_DATE}})")

        if args == ["--version"]:
            print("biomcp 0.8.24")
            raise SystemExit(0)

        if args == ["version"]:
            version()
            raise SystemExit(0)

        if args and args[0] == "serve-http":
            time.sleep(30)
            raise SystemExit(0)

        if args == ["serve"]:
            for line in sys.stdin:
                msg = json.loads(line)
                method = msg.get("method")
                if method == "initialize":
                    print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"result":{{"capabilities":{{}}}}}}), flush=True)
                elif method == "tools/list":
                    schema = {{"type":"object","properties":{{"entity":{{"enum":["gene"]}}}}}}
                    tools = [
                        {{"name":"search","inputSchema":schema}},
                        {{"name":"get","inputSchema":schema}},
                        {{"name":"biomcp","inputSchema":{{"type":"object"}}}},
                    ]
                    print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"result":{{"tools":tools}}}}), flush=True)
                elif method == "tools/call":
                    text = "# BRAF\\n\\n## Sources\\nIdentity: NCBI Gene"
                    print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"result":{{"content":[{{"type":"text","text":text}}]}}}}), flush=True)
            raise SystemExit(0)

        joined = " ".join(args)
        if joined == "search gene BRAF --limit 51":
            print("--limit must be between 1 and 50", file=sys.stderr)
            raise SystemExit(2)
        if joined == "get drug foo bar":
            print('Try searching: biomcp search drug -q "foo bar"', file=sys.stderr)
            raise SystemExit(1)
        if joined == "get disease foo bar":
            print('Try searching: biomcp search disease -q "foo bar"', file=sys.stderr)
            raise SystemExit(1)
        if joined == "get pathway P15056":
            print("did you mean `biomcp get protein P15056`", file=sys.stderr)
            raise SystemExit(2)
        if joined == "get gene BRAF":
            print("# BRAF ")
            raise SystemExit(0)
        if joined == "gene trials BRAF --limit 1":
            print("Results: 1")
            raise SystemExit(0)
        if joined == "disease trials melanoma --limit 1":
            print("Results: 1")
            raise SystemExit(0)
        if joined == "get gene PD-L1":
            print("# CD274 ")
            raise SystemExit(0)
        if joined == "get gene HER2":
            print("# ERBB2 ")
            raise SystemExit(0)
        if joined == "get gene P53":
            print("# TP53 ")
            raise SystemExit(0)
        if joined == "get variant NM_004333.6:c.1799T>A":
            print("BRAF p.V600E rs113488022")
            raise SystemExit(0)
        if joined == "--json get gene NOT_A_GENE_445":
            print('{{"error":"not found"}}')
            raise SystemExit(1)
        if joined == "--json get variant bogusvar445":
            print('{{"error":"bad variant"}}')
            raise SystemExit(2)
        if joined == "get pathway ENSG00000157764":
            print("did you mean `biomcp get gene ENSG00000157764`", file=sys.stderr)
            raise SystemExit(2)
        if joined == "get pathway BRAF":
            print("did you mean `biomcp get gene BRAF`", file=sys.stderr)
            raise SystemExit(2)
        if joined == "get pathway rs113488022":
            print("did you mean `biomcp get variant rs113488022`", file=sys.stderr)
            raise SystemExit(2)

        print(f"unexpected fake biomcp args: {{joined}}", file=sys.stderr)
        raise SystemExit(99)
        """
    )


def _copy_release_smoke_fixture(tmp_path: Path) -> Path:
    fixture_root = tmp_path / "repo"
    (fixture_root / "scripts").mkdir(parents=True)
    shutil.copy2(REPO_ROOT / "scripts" / "release-smoke.sh", fixture_root / "scripts" / "release-smoke.sh")
    (fixture_root / ".claude-plugin").mkdir()
    (fixture_root / ".claude-plugin" / "marketplace.json").write_text(
        '{"plugins":[{"mcpServers":{"biomcp":{"command":"biomcp","args":["serve"]}}}]}\n',
        encoding="utf-8",
    )
    subprocess.run(["git", "init"], cwd=fixture_root, check=True, capture_output=True)
    subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=fixture_root, check=True)
    subprocess.run(["git", "config", "user.name", "Test User"], cwd=fixture_root, check=True)
    subprocess.run(["git", "add", "."], cwd=fixture_root, check=True)
    subprocess.run(["git", "commit", "-m", "fixture"], cwd=fixture_root, check=True, capture_output=True)
    return fixture_root


def _install_fake_tools(repo_root: Path, head_sha: str) -> Path:
    fake_bin = repo_root / "fake-bin"
    fake_bin.mkdir()
    _write_executable(
        fake_bin / "curl",
        """#!/usr/bin/env bash
case "$*" in
  *"/mcp"*) printf '403' ;;
  *" -w "*) printf '200' ;;
esac
exit 0
""",
    )
    _write_executable(fake_bin / "jq", "#!/usr/bin/env bash\nexit 0\n")
    _write_executable(
        fake_bin / "cargo",
        "#!/usr/bin/env bash\n"
        "printf '%s\\n' \"$*\" >> cargo-called.log\n"
        "mkdir -p target/release\n"
        f"cat > target/release/biomcp <<'PY'\n{_fake_biomcp_script(head_sha)}PY\n"
        "chmod +x target/release/biomcp\n",
    )
    return fake_bin


def _run_release_smoke(repo_root: Path, fake_bin: Path, *args: str) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["PATH"] = f"{fake_bin}{os.pathsep}{env['PATH']}"
    return subprocess.run(
        ["bash", "scripts/release-smoke.sh", *args],
        cwd=repo_root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
        timeout=30,
    )


def test_default_release_smoke_rebuilds_stale_target_binary(tmp_path: Path) -> None:
    repo_root = _copy_release_smoke_fixture(tmp_path)
    head_sha = subprocess.check_output(
        ["git", "rev-parse", "--short=8", "HEAD"], cwd=repo_root, text=True
    ).strip()
    fake_bin = _install_fake_tools(repo_root, head_sha)
    stale = repo_root / "target" / "release" / "biomcp"
    stale.parent.mkdir(parents=True)
    _write_executable(stale, _fake_biomcp_script("deadbeef"))

    result = _run_release_smoke(repo_root, fake_bin)

    assert result.returncode == 0, result.stderr + result.stdout
    assert "stamped git deadbeef, not HEAD" in result.stderr
    assert (repo_root / "cargo-called.log").read_text(encoding="utf-8").strip() == "build --release --locked"
    assert f"Binary git SHA: {head_sha}" in result.stdout
    assert "Binary build date: 2026-06-24T00:00:00Z" in result.stdout
    assert f"Current HEAD: {head_sha}" in result.stdout
    assert "FAIL: 0" in result.stdout


def test_explicit_bin_mismatch_is_visible_and_not_rebuilt(tmp_path: Path) -> None:
    repo_root = _copy_release_smoke_fixture(tmp_path)
    head_sha = subprocess.check_output(
        ["git", "rev-parse", "--short=8", "HEAD"], cwd=repo_root, text=True
    ).strip()
    fake_bin = _install_fake_tools(repo_root, head_sha)
    explicit_bin = repo_root / "artifacts" / "biomcp-old"
    explicit_bin.parent.mkdir()
    _write_executable(explicit_bin, _fake_biomcp_script("deadbeef"))

    result = _run_release_smoke(repo_root, fake_bin, "--bin", str(explicit_bin))

    assert result.returncode == 1
    assert not (repo_root / "cargo-called.log").exists()
    assert "Binary git SHA: deadbeef" in result.stdout
    assert f"Current HEAD: {head_sha}" in result.stdout
    assert "WARNING: --bin binary is stamped git deadbeef" in result.stdout
    assert "FAIL 444 version command reports HEAD git SHA" in result.stdout
