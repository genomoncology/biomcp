"""Local cache failures preserve recorded provider answers through the CLI."""
from __future__ import annotations

import base64
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
from urllib.request import urlopen

import pytest

sys.path.insert(0, str(Path(__file__).parent / "golden"))
from replay_env import REPO_ROOT, replay_environment  # noqa: E402

pytestmark = [pytest.mark.needs_binary]
BINARY = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
ARTICLE = ("search", "article", "--gene", "BRAF", "--disease", "melanoma", "--source", "europepmc", "--limit", "1")
GENE = ("search", "gene", "BRAF", "--limit", "1")


@pytest.fixture
def provider(tmp_path: Path):
    with replay_environment(record=False) as replay:
        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                # Reuse the recorded article answer: its search-all route anchors
                # the same gene, while this CLI case only exercises cache behavior.
                path = self.path.replace("query=BRAF+AND+melanoma", "query=GENE_PROTEIN%3ABRAF+AND+melanoma")
                with urlopen(replay.origin + path, timeout=10) as reply:
                    body = reply.read()
                    status = reply.status
                if path.startswith("/europepmc/"):
                    payload = json.loads(body)
                    payload["hitCount"] = len(payload["resultList"]["result"])
                    payload.pop("nextCursorMark", None)
                    payload.pop("nextPageUrl", None)
                    body = json.dumps(payload).encode()
                before_reply[0]()
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Cache-Control", "public, max-age=3600")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *_args):
                pass

        before_reply = [lambda: None]
        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        run_dir = replay._next_run_dir()
        env = replay.command_env(run_dir)
        env.pop("BIOMCP_CACHE_MODE", None)
        origin = f"http://127.0.0.1:{server.server_port}"
        for name, value in replay.provider_env.items():
            env[name] = value.replace(replay.origin, origin)
        env.update(BIOMCP_TEST_UNPACED_ORIGIN=origin, RUST_LOG="warn", BIOMCP_CACHE_MIN_DISK_FREE="1B",
                   BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS="3000")
        cache = Path(env["BIOMCP_CACHE_DIR"])

        def run(argv):
            return subprocess.run([str(BINARY), "--json", *argv], cwd=REPO_ROOT, env=env,
                                  capture_output=True, text=True, timeout=20)

        try:
            yield run, cache, before_reply, replay
        finally:
            server.shutdown()
            server.server_close()
            thread.join()
            assert not replay.misses(), replay.misses()


def assert_answer(result, cause):
    assert result.returncode == 0, (result.stdout, result.stderr)
    assert json.loads(result.stdout)["results"]
    warnings = [line for line in result.stderr.splitlines() if "local HTTP cache" in line]
    assert len(warnings) == 1, result.stderr
    assert cause in warnings[0], warnings
    assert "http://" not in warnings[0]
    assert "https://" not in warnings[0]
    assert "io_kind" in warnings[0] and "os_code" in warnings[0]


@pytest.mark.parametrize("command", [ARTICLE, GENE])
def test_invalid_cache_tree_uses_provider_without_trusting_tree(provider, tmp_path, command):
    run, cache, _, _ = provider
    outside = tmp_path / "outside"
    outside.mkdir()
    sentinel = outside / "keep"
    sentinel.write_bytes(b"unchanged")
    cache.rmdir()
    cache.symlink_to(outside, target_is_directory=True)
    assert_answer(run(command), "initialization")
    assert sentinel.read_bytes() == b"unchanged"
    assert list(outside.iterdir()) == [sentinel]


def test_undecodable_cache_entry_refetches_then_keeps_cache_hits(provider):
    run, cache, _, replay = provider
    first = run(GENE)
    assert first.returncode == 0, first.stderr
    buckets = list((cache / "http/index-v5").rglob("*"))
    buckets = [path for path in buckets if path.is_file()]
    assert len(buckets) == 1
    bucket = buckets[0]
    entry = json.loads(bucket.read_text().strip().split("\t")[-1])
    invalid = b"\x00"
    digest = hashlib.sha512(invalid).digest()
    hexadecimal = digest.hex()
    blob = cache / "http/content-v2/sha512" / hexadecimal[:2] / hexadecimal[2:4] / hexadecimal[4:]
    blob.parent.mkdir(parents=True, exist_ok=True)
    blob.write_bytes(invalid)
    entry.update(integrity="sha512-" + base64.b64encode(digest).decode(), size=len(invalid))
    encoded = json.dumps(entry, separators=(",", ":"))
    bucket.write_text("\n" + hashlib.sha256(encoded.encode()).hexdigest() + "\t" + encoded)
    before = replay.request_log_line_count()
    assert_answer(run(GENE), "read")
    assert replay.request_log_line_count() == before + 1
    hit = run(GENE)
    assert hit.returncode == 0, hit.stderr
    assert json.loads(hit.stdout)["results"] == json.loads(first.stdout)["results"]
    assert replay.request_log_line_count() == before + 1


def test_failed_cache_write_keeps_the_provider_answer(provider):
    run, cache, before_reply, _ = provider

    def obstruct_write():
        (cache / ".biomcp-key-locks").write_bytes(b"blocked lock directory")

    before_reply[0] = obstruct_write
    assert_answer(run(GENE), "write")


def test_no_cache_ignores_an_invalid_cache_tree_without_warning(provider):
    run, cache, _, _ = provider
    (cache / "http").write_bytes(b"not a directory")
    result = run(("--no-cache", *GENE))
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout)["results"]
    assert "local HTTP cache" not in result.stderr
    assert (cache / "http").read_bytes() == b"not a directory"


@pytest.mark.skipif(os.name != "posix", reason="uses a POSIX lock holder")
@pytest.mark.parametrize("lock_name", [".body-limit-cache-v1.lock", ".biomcp-operation.lock"])
def test_cache_lock_contention_leaves_time_for_article_provider(provider, lock_name):
    import fcntl

    run, cache, _, replay = provider
    with (cache / lock_name).open("wb") as held:
        fcntl.flock(held, fcntl.LOCK_EX)
        before = replay.request_log_line_count()
        assert_answer(run(ARTICLE), "initialization")
        assert replay.request_log_line_count() > before


def test_unreadable_cache_configuration_uses_provider(provider):
    run, cache, _, _ = provider
    config_home = "Library/Application Support" if sys.platform == "darwin" else "xdg-config"
    config = cache.parent / config_home / "biomcp/cache.toml"
    config.mkdir(parents=True)
    assert_answer(run(GENE), "initialization")
