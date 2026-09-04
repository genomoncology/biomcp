from __future__ import annotations

import importlib.util
from pathlib import Path
import re

import pytest
import yaml


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github" / "workflows" / "docs-edge.yml"
VERIFY = ROOT / "scripts" / "verify-docs-publication.py"
SHA = "0123456789abcdef0123456789abcdef01234567"


def _workflow() -> dict:
    return yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))


def _load_verifier():
    spec = importlib.util.spec_from_file_location("docs_publication_verifier", VERIFY)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_docs_publication_is_one_main_push_only_writer() -> None:
    workflow = _workflow()
    triggers = workflow.get("on", workflow.get(True))
    assert triggers == {"push": {"branches": ["main"]}}
    assert workflow["concurrency"] == {
        "group": "publish-biomcp-docs",
        "cancel-in-progress": True,
    }
    assert list(workflow["jobs"]) == ["publish-docs"]
    job = workflow["jobs"]["publish-docs"]
    assert job["if"] == (
        "github.event_name == 'push' && github.ref == 'refs/heads/main'"
    )
    assert job["permissions"] == {"contents": "write", "pages": "write"}
    assert "permissions" not in workflow

    publishers = []
    for path in (ROOT / ".github" / "workflows").glob("*.yml"):
        candidate = yaml.safe_load(path.read_text(encoding="utf-8"))
        for name, candidate_job in candidate.get("jobs", {}).items():
            serialized = yaml.safe_dump(candidate_job).lower()
            if any(
                marker in serialized
                for marker in ("mkdocs gh-deploy", "/pages/builds", "deploy-pages@")
            ):
                publishers.append((path.name, name))
    assert publishers == [("docs-edge.yml", "publish-docs")]


def test_docs_publication_preserves_history_and_orders_authorized_writes() -> None:
    workflow = _workflow()
    steps = workflow["jobs"]["publish-docs"]["steps"]
    checkout = steps[0]
    assert checkout["uses"].startswith("actions/checkout@")
    assert re.fullmatch(r"actions/checkout@[0-9a-f]{40}", checkout["uses"])
    assert checkout["with"] == {"ref": "${{ github.sha }}", "fetch-depth": 0}

    commands = [step.get("run", "") for step in steps]
    joined = "\n".join(commands)
    assert "git fetch origin gh-pages:refs/remotes/origin/gh-pages" in joined
    deploy = next(
        i for i, command in enumerate(commands) if "mkdocs gh-deploy" in command
    )
    guard = commands[deploy - 1]
    assert "git fetch --no-tags origin main:refs/remotes/origin/main" in guard
    assert "git rev-parse refs/remotes/origin/main" in guard
    assert '"$GITHUB_SHA"' in guard
    assert "uv run --no-sync mkdocs gh-deploy --strict --force" in commands[deploy]
    assert "--no-history" not in commands[deploy]

    pages = next(i for i, command in enumerate(commands) if "/pages/builds" in command)
    verify = next(
        i
        for i, command in enumerate(commands)
        if "verify-docs-publication.py" in command
    )
    assert deploy < pages < verify
    assert "--request POST" in commands[pages]
    assert "Authorization: Bearer $GITHUB_TOKEN" in commands[pages]
    assert "--timeout-seconds 600" in commands[verify]
    assert "continue-on-error" not in WORKFLOW.read_text(encoding="utf-8")


def test_docs_publication_uses_locked_build_and_validated_revision() -> None:
    workflow = _workflow()
    steps = workflow["jobs"]["publish-docs"]["steps"]
    actions = [step["uses"] for step in steps if "uses" in step]
    assert all(re.fullmatch(r"[^@]+@[0-9a-f]{40}", action) for action in actions)
    commands = "\n".join(step.get("run", "") for step in steps)
    assert "uv sync --extra dev --no-install-project --locked" in commands
    assert "mkdocs build --strict" in commands
    build = next(
        step for step in steps if "mkdocs build --strict" in step.get("run", "")
    )
    deploy = next(step for step in steps if "mkdocs gh-deploy" in step.get("run", ""))
    assert build["env"]["BIOMCP_DOCS_REVISION"] == "${{ github.sha }}"
    assert deploy["env"]["BIOMCP_DOCS_REVISION"] == "${{ github.sha }}"


def test_revision_witness_is_absent_or_exact_and_stale_files_are_removed(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    hook_spec = importlib.util.spec_from_file_location(
        "copy_markdown_twins", ROOT / "scripts" / "copy-markdown-twins.py"
    )
    assert hook_spec is not None and hook_spec.loader is not None
    hook = importlib.util.module_from_spec(hook_spec)
    hook_spec.loader.exec_module(hook)
    docs = tmp_path / "docs"
    site = tmp_path / "site"
    docs.mkdir()
    site.mkdir()
    (docs / "page.md").write_text("page\n", encoding="utf-8")
    stale = site / "__biomcp_revision__"
    stale.mkdir()
    (stale / f"{'f' * 40}.txt").write_text("stale\n", encoding="utf-8")
    config = type("Config", (), {"docs_dir": str(docs), "site_dir": str(site)})()

    monkeypatch.delenv("BIOMCP_DOCS_REVISION", raising=False)
    hook.on_post_build(config)
    assert not stale.exists()

    monkeypatch.setenv("BIOMCP_DOCS_REVISION", SHA)
    hook.on_post_build(config)
    assert (stale / f"{SHA}.txt").read_bytes() == f"{SHA}\n".encode()
    assert list(stale.iterdir()) == [stale / f"{SHA}.txt"]

    monkeypatch.setenv("BIOMCP_DOCS_REVISION", "not-a-sha")
    with pytest.raises(ValueError, match="40-character lowercase"):
        hook.on_post_build(config)


def test_live_verifier_cache_busts_and_fails_closed_on_wrong_paths(
    tmp_path: Path,
) -> None:
    verifier = _load_verifier()
    site = tmp_path / "site"
    site.mkdir()
    (site / "llms.txt").write_text(
        "# BioMCP\n\n[Trial](https://biomcp.org/user-guide/trial.md)\n",
        encoding="utf-8",
    )
    (site / "llms-full.txt").write_text("full\n", encoding="utf-8")
    page = site / "user-guide"
    page.mkdir()
    (page / "trial.md").write_text("trial\n", encoding="utf-8")

    seen: list[object] = []
    timeouts: list[float] = []

    class Response:
        status = 200

        def __init__(self, request, body: bytes, final_path: str | None = None):
            self.request = request
            self.body = body
            self.final_path = final_path

        def __enter__(self):
            return self

        def __exit__(self, *args):
            return None

        def read(self) -> bytes:
            return self.body

        def geturl(self) -> str:
            if self.final_path:
                return "https://biomcp.org" + self.final_path
            return self.request.full_url

    bodies = {
        f"/__biomcp_revision__/{SHA}.txt": f"{SHA}\n".encode(),
        "/llms.txt": (site / "llms.txt").read_bytes(),
        "/llms-full.txt": b"full\n",
        "/user-guide/trial.md": b"trial\n",
    }

    def opener(request, timeout):
        seen.append(request)
        timeouts.append(timeout)
        path = verifier.urlsplit(request.full_url).path
        return Response(request, bodies[path])

    verifier.verify_publication(
        revision=SHA,
        site_dir=site,
        base_url="https://biomcp.org",
        timeout_seconds=1,
        opener=opener,
        sleep=lambda _: None,
    )
    urls = [request.full_url for request in seen]
    assert len(urls) == len(set(urls))
    assert all("biomcp_verify=" in url for url in urls)
    assert all(request.get_header("Cache-control") == "no-cache" for request in seen)
    assert all(request.get_header("Pragma") == "no-cache" for request in seen)
    assert all(0 < timeout <= verifier.REQUEST_TIMEOUT_SECONDS for timeout in timeouts)

    def redirected(request, timeout):
        del timeout
        return Response(request, f"{SHA}\n".encode(), "/wrong.txt")

    with pytest.raises(verifier.VerificationError, match="unexpected path"):
        verifier.verify_publication(
            revision=SHA,
            site_dir=site,
            base_url="https://biomcp.org",
            timeout_seconds=0.01,
            opener=redirected,
            sleep=lambda _: None,
        )

    def cross_origin(request, timeout):
        del timeout
        response = Response(request, f"{SHA}\n".encode())
        response.geturl = lambda: (
            "https://attacker.example"
            f"/__biomcp_revision__/{SHA}.txt?biomcp_verify=redirected"
        )
        return response

    with pytest.raises(verifier.VerificationError, match="unexpected origin"):
        verifier.verify_publication(
            revision=SHA,
            site_dir=site,
            base_url="https://biomcp.org",
            timeout_seconds=0.01,
            opener=cross_origin,
            sleep=lambda _: None,
        )
    assert verifier._origin("HTTPS://BIOMCP.ORG") == verifier._origin(
        "https://biomcp.org:443"
    )

    def mismatched_index(request, timeout):
        del timeout
        path = verifier.urlsplit(request.full_url).path
        body = bodies[path]
        if path == "/llms.txt":
            body = b"[Outside](https://biomcp.org/../../outside-secret)\n"
        return Response(request, body)

    with pytest.raises(
        verifier.VerificationError, match="live bytes do not match.*llms.txt"
    ):
        verifier.verify_publication(
            revision=SHA,
            site_dir=site,
            base_url="https://biomcp.org",
            timeout_seconds=1,
            opener=mismatched_index,
            sleep=lambda _: None,
        )

    outside = tmp_path / "outside-secret"
    outside.write_text("secret\n", encoding="utf-8")
    with pytest.raises(verifier.VerificationError, match="escapes the built site"):
        verifier._expected_publication(
            site,
            b"[Outside](https://biomcp.org/../../outside-secret)\n",
        )

    clock = iter([0.0, 0.1, 0.2, 1.1])
    with pytest.raises(verifier.VerificationError, match="exceeded its deadline"):
        verifier.verify_publication(
            revision=SHA,
            site_dir=site,
            base_url="https://biomcp.org",
            timeout_seconds=1,
            opener=opener,
            sleep=lambda _: None,
            monotonic=lambda: next(clock),
        )

    with pytest.raises(verifier.VerificationError, match="at most 600"):
        verifier.verify_publication(
            revision=SHA,
            site_dir=site,
            base_url="https://biomcp.org",
            timeout_seconds=601,
        )


def test_removed_worker_delivery_has_no_active_residue() -> None:
    assert not (ROOT / "docs-edge-worker.mjs").exists()
    assert not (ROOT / "wrangler.toml").exists()
    active = [ROOT / ".github", ROOT / "scripts", ROOT / "tests"]
    this_test = Path(__file__).resolve()
    text = "\n".join(
        path.read_text(encoding="utf-8", errors="ignore")
        for directory in active
        for path in directory.rglob("*")
        if path.is_file()
        and path.resolve() != this_test
        and "__pycache__" not in path.parts
    )
    assert "wrangler deploy" not in text.lower()
    assert "cloudflare_api_token" not in text.lower()
    assert "cloudflare_account_id" not in text.lower()
