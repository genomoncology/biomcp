from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import textwrap
import tomllib

import pytest

ROOT = Path(
    os.environ.get("BIOMCP_TEST_ROOT", Path(__file__).resolve().parents[1])
).resolve()
DOCS = ROOT / "docs"


@pytest.fixture(scope="module")
def built_site(tmp_path_factory: pytest.TempPathFactory) -> Path:
    site = tmp_path_factory.mktemp("markdown-twins-site")
    completed = subprocess.run(
        [
            "uv",
            "run",
            "--project",
            str(ROOT),
            "--no-sync",
            "mkdocs",
            "build",
            "--strict",
            "--site-dir",
            str(site),
        ],
        cwd=ROOT,
        env=os.environ | {"NO_MKDOCS_2_WARNING": "1"},
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode == 0, completed.stdout + completed.stderr
    return site


def _page_routes(source: Path) -> tuple[str, str]:
    relative = source.relative_to(DOCS)
    markdown_route = "/" + relative.as_posix()
    if relative == Path("index.md"):
        return "/", markdown_route
    return "/" + relative.with_suffix("").as_posix() + "/", markdown_route


def test_docs_build_publishes_an_exact_markdown_twin_for_every_page(
    built_site: Path,
) -> None:
    missing_or_changed = []
    for source in sorted(DOCS.rglob("*.md")):
        twin = built_site / source.relative_to(DOCS)
        if not twin.is_file():
            missing_or_changed.append(f"missing {twin.relative_to(built_site)}")
        elif twin.read_bytes() != source.read_bytes():
            missing_or_changed.append(f"changed {twin.relative_to(built_site)}")

    assert not missing_or_changed, "\n".join(missing_or_changed)


def test_docs_edge_announces_and_serves_every_markdown_twin(
    built_site: Path, tmp_path: Path
) -> None:
    config_path = ROOT / "wrangler.toml"
    assert config_path.is_file(), "docs edge deployment has no wrangler.toml"
    config = tomllib.loads(config_path.read_text(encoding="utf-8"))
    worker = ROOT / config["main"]
    assert worker.is_file(), f"docs edge worker does not exist: {worker}"
    assert config["assets"]["binding"] == "ASSETS"
    assert Path(config["assets"]["directory"]).name == "site"
    assert any(
        route.get("custom_domain") is True and route.get("pattern") == "biomcp.org"
        for route in config["routes"]
    ), "docs edge worker is not bound to biomcp.org"

    requests = []
    expected = {}
    for source in sorted(DOCS.rglob("*.md")):
        html_route, markdown_route = _page_routes(source)
        requests.extend(
            [
                {"url": f"https://biomcp.org{html_route}"},
                {
                    "url": f"https://biomcp.org{html_route}",
                    "accept": "text/markdown",
                },
                {"url": f"https://biomcp.org{markdown_route}"},
            ]
        )
        expected[html_route] = (markdown_route, source.read_text(encoding="utf-8"))

    harness = tmp_path / "exercise-worker.mjs"
    harness.write_text(
        textwrap.dedent(
            """
            import fs from "node:fs/promises";
            import path from "node:path";
            import { pathToFileURL } from "node:url";

            const [workerPath, sitePath] = process.argv.slice(2);
            const handler = (await import(pathToFileURL(workerPath))).default;
            const requests = JSON.parse(await new Promise((resolve) => {
              let input = "";
              process.stdin.setEncoding("utf8");
              process.stdin.on("data", (chunk) => input += chunk);
              process.stdin.on("end", () => resolve(input));
            }));
            const contentType = (file) => file.endsWith(".md")
              ? "text/markdown; charset=utf-8"
              : "text/html; charset=utf-8";
            const env = { ASSETS: { fetch: async (request) => {
              const url = new URL(request.url);
              let relative = decodeURIComponent(url.pathname).replace(/^\\//, "");
              if (relative === "" || relative.endsWith("/")) relative += "index.html";
              try {
                return new Response(await fs.readFile(path.join(sitePath, relative)), {
                  headers: { "content-type": contentType(relative) },
                });
              } catch (error) {
                if (error.code !== "ENOENT") throw error;
                return new Response("not found", { status: 404 });
              }
            } } };
            const results = [];
            for (const item of requests) {
              const headers = item.accept ? { Accept: item.accept } : {};
              const response = await handler.fetch(new Request(item.url, { headers }), env, {});
              results.push({
                status: response.status,
                headers: Object.fromEntries(response.headers),
                body: await response.text(),
              });
            }
            process.stdout.write(JSON.stringify(results));
            """
        ),
        encoding="utf-8",
    )
    completed = subprocess.run(
        ["node", str(harness), str(worker), str(built_site)],
        cwd=ROOT,
        input=json.dumps(requests),
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode == 0, completed.stderr
    results = iter(json.loads(completed.stdout))

    for html_route, (markdown_route, source) in expected.items():
        html = next(results)
        negotiated = next(results)
        direct = next(results)
        markdown_url = f"https://biomcp.org{markdown_route}"

        assert html["status"] == 200, html_route
        assert html["headers"].get("x-markdown-url") == markdown_url, html_route
        assert negotiated["status"] == 200, html_route
        assert negotiated["body"] == source, html_route
        assert negotiated["headers"]["content-type"].startswith("text/markdown"), (
            html_route
        )
        assert "accept" in negotiated["headers"].get("vary", "").lower(), html_route
        assert direct["status"] == 200, markdown_route
        assert direct["body"] == source, markdown_route
        assert direct["headers"]["content-type"].startswith("text/markdown"), (
            markdown_route
        )


def test_llms_txt_documents_markdown_content_negotiation() -> None:
    agent_index = (DOCS / "llms.txt").read_text(encoding="utf-8").lower()

    assert "accept: text/markdown" in agent_index
    assert ".md" in agent_index


def test_docs_deployment_builds_strictly_and_deploys_the_edge() -> None:
    workflow_path = ROOT / ".github/workflows/docs.yml"
    assert workflow_path.is_file(), "the docs edge has no deployment workflow"
    workflow = workflow_path.read_text(encoding="utf-8")

    assert "mkdocs build --strict" in workflow
    assert "wrangler deploy" in workflow
